// Paso de Selah a EmetPro (0.9.0): la app cambió de nombre y de identificador (org.selah.app → org.emetpro.app),
// así que su carpeta de datos también cambió. En el primer arranque se copia lo de la carpeta vieja a la nueva
// para que quien actualiza no pierda Biblias, letras, planes, medios, ajustes ni la clave del control remoto.
//
// Se copia, no se mueve: la carpeta vieja queda intacta como respaldo. La base es lo último que llega y llega de
// golpe (un rename): mientras no está, la migración no ha terminado y el siguiente arranque la repite.
// Va como primer plugin para que ninguna funcionalidad lea la carpeta nueva antes de tiempo.
use std::fs;
use std::io;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use rusqlite::Connection;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Wry};

use crate::core::db::FILE;

/// Identificador de Selah: su carpeta de datos es hermana de la de EmetPro (la plataforma pone las dos juntas).
pub const OLD_IDENTIFIER: &str = "org.selah.app";
const OLD_FILE: &str = "selah.db";

/// Lo que es de la app dentro de su carpeta de datos. El resto (localstorage, cachés…) lo crea el webview y no se copia.
const ITEMS: &[&str] = &["bibles", "fondos", "medios", "libreoffice", "remote-key"];

/// Testigo de una migración empezada y no terminada.
const PENDING: &str = "migracion-selah.pendiente";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new("legacy")
        .setup(|app, _api| {
            let Ok(new) = app.path().app_data_dir() else { return Ok(()) };
            let Some(parent) = new.parent() else { return Ok(()) };
            let old = parent.join(OLD_IDENTIFIER);
            match migrate(&old, &new) {
                Ok(true) => eprintln!("[selah → emetpro] datos copiados de {} a {}", old.display(), new.display()),
                Ok(false) => {}
                // Sin datos copiados la app arranca vacía, pero la carpeta vieja sigue ahí y se reintenta al volver a abrir.
                Err(e) => eprintln!("[selah → emetpro] no se pudieron copiar los datos de {}: {e}", old.display()),
            }
            Ok(())
        })
        .build()
}

/// Copia los datos de Selah (`old`) a la carpeta de EmetPro (`new`). Devuelve si copió algo.
pub fn migrate(old: &Path, new: &Path) -> Result<bool, String> {
    if !old.join(OLD_FILE).is_file() || !needed(new) {
        return Ok(false);
    }
    fs::create_dir_all(new).map_err(|e| e.to_string())?;
    fs::write(new.join(PENDING), b"").map_err(|e| e.to_string())?;

    for item in ITEMS {
        let from = old.join(item);
        if from.symlink_metadata().is_ok() {
            copy_tree(&from, &new.join(item)).map_err(|e| format!("{item}: {e}"))?;
        }
    }

    let staging = new.join(format!("{FILE}.migrando"));
    remove_db(&staging);
    copy_db(old, &staging, new).map_err(|e| format!("base de datos: {e}"))?;
    let target = new.join(FILE);
    remove_db(&target);
    fs::rename(&staging, &target).map_err(|e| e.to_string())?;

    let _ = fs::remove_file(new.join(PENDING));
    Ok(true)
}

/// Hace falta migrar si EmetPro aún no tiene base, o si un intento anterior se cortó y desde entonces no se ha
/// guardado nada propio (ni letras ni planes): así un reintento nunca pisa trabajo hecho ya en EmetPro.
fn needed(new: &Path) -> bool {
    let db = new.join(FILE);
    if !db.exists() {
        return true;
    }
    if !new.join(PENDING).exists() {
        return false;
    }
    let Ok(conn) = Connection::open(&db) else { return false };
    let count = |table: &str| -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0)).unwrap_or(0)
    };
    count("songs") == 0 && count("plans") == 0
}

/// La base se copia con su WAL (ahí están los últimos cambios) y se abre la copia, que los incorpora.
/// En esa copia se corrigen las rutas que apuntaban a la carpeta vieja y se deja en un solo archivo.
fn copy_db(old: &Path, staging: &Path, new: &Path) -> Result<(), String> {
    fs::copy(old.join(OLD_FILE), staging).map_err(|e| e.to_string())?;
    let wal = old.join(format!("{OLD_FILE}-wal"));
    if wal.is_file() {
        fs::copy(&wal, sidecar(staging, "-wal")).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(staging).map_err(|e| e.to_string())?;
    let check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if check != "ok" {
        return Err(format!("la copia está dañada: {check}"));
    }
    rewrite_paths(&conn, old, new).map_err(|e| e.to_string())?;
    // Un solo archivo: el rename final se lleva todo. Store::open vuelve a poner WAL al abrirla.
    conn.pragma_update(None, "journal_mode", "DELETE").map_err(|e| e.to_string())?;
    conn.close().map_err(|(_, e)| e.to_string())?;
    Ok(())
}

/// Las letras (fondo), los planes (medios copiados) y los ajustes guardan rutas absolutas dentro de la carpeta
/// de datos: se cambian a la carpeta nueva, donde ya están las copias. En JSON la ruta va escapada (Windows: `\\`).
fn rewrite_paths(conn: &Connection, old: &Path, new: &Path) -> rusqlite::Result<()> {
    let raw = |dir: &Path| format!("{}{MAIN_SEPARATOR}", dir.to_string_lossy());
    let escaped = |text: &str| {
        let quoted = serde_json::to_string(text).unwrap_or_default();
        quoted[1..quoted.len() - 1].to_string()
    };
    let (from, to) = (raw(old), raw(new));
    let mut pairs = vec![(from.clone(), to.clone())];
    if escaped(&from) != from {
        pairs.push((escaped(&from), escaped(&to)));
    }
    for (table, column) in [("songs", "background"), ("plans", "items"), ("settings", "value")] {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )?;
        if !exists {
            continue;
        }
        for (from, to) in &pairs {
            conn.execute(
                &format!("UPDATE {table} SET {column} = replace({column}, ?1, ?2) WHERE instr({column}, ?1) > 0"),
                [from, to],
            )?;
        }
    }
    // La fuente de texto elegida para OBS: sus escenas y fuentes también cambian de nombre (obs/client.rs).
    conn.execute(
        "UPDATE settings SET value = replace(value, '\"Selah · ', '\"EmetPro · ') WHERE key = 'obsTextSource'",
        [],
    )?;
    Ok(())
}

fn sidecar(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn remove_db(db: &Path) {
    for path in [db.to_path_buf(), sidecar(db, "-wal"), sidecar(db, "-shm")] {
        let _ = fs::remove_file(path);
    }
}

/// Copia un archivo o una carpeta entera. Los enlaces simbólicos se recrean (la copia privada de LibreOffice
/// los usa); en Windows se copia aquello a lo que apuntan.
fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(unix)]
    if fs::symlink_metadata(from)?.file_type().is_symlink() {
        let _ = fs::remove_file(to);
        return std::os::unix::fs::symlink(fs::read_link(from)?, to);
    }
    if fs::metadata(from)?.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        fs::copy(from, to).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::Store;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emetpro-legacy-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    /// Una carpeta de Selah como la deja la app: base con su esquema, cambios aún en el WAL (Selah abierta),
    /// Biblias, medios, fondos, clave y un enlace simbólico en LibreOffice.
    fn selah_folder(root: &Path) -> (PathBuf, Connection) {
        let old = root.join(OLD_IDENTIFIER);
        drop(Store::open(&old));
        fs::rename(old.join(FILE), old.join(OLD_FILE)).unwrap();
        let conn = Connection::open(old.join(OLD_FILE)).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
        let background = old.join("fondos").join("f.png").to_string_lossy().into_owned();
        let media = old.join("medios").join("abc").join("icon.png").to_string_lossy().into_owned();
        conn.execute(
            "INSERT INTO songs (id, title, body, background, updated_at) VALUES ('s1', 'Príncipe de paz', 'Letra', ?1, 0)",
            [&background],
        )
        .unwrap();
        let items = serde_json::json!([{ "id": "m1", "kind": "media", "path": media }]).to_string();
        conn.execute("INSERT INTO plans (id, name, items, updated_at) VALUES ('p1', 'Domingo', ?1, 0)", [&items]).unwrap();
        conn.execute("INSERT INTO settings (key, value) VALUES ('theme', '\"dark\"')", []).unwrap();
        conn.execute("INSERT INTO settings (key, value) VALUES ('obsTextSource', '\"Selah · Texto\"')", []).unwrap();

        fs::create_dir_all(old.join("bibles")).unwrap();
        fs::write(old.join("bibles").join("rvr.xml"), "<bible/>").unwrap();
        fs::create_dir_all(old.join("medios").join("abc")).unwrap();
        fs::write(old.join("medios").join("abc").join("icon.png"), "png").unwrap();
        fs::create_dir_all(old.join("fondos")).unwrap();
        fs::write(old.join("fondos").join("f.png"), "png").unwrap();
        fs::write(old.join("remote-key"), "clave").unwrap();
        fs::create_dir_all(old.join("libreoffice").join("program")).unwrap();
        fs::write(old.join("libreoffice").join("program").join("soffice.bin"), "bin").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("program/soffice.bin", old.join("libreoffice").join("soffice")).unwrap();
        fs::create_dir_all(old.join("localstorage")).unwrap();
        (old, conn)
    }

    fn listing(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if fs::symlink_metadata(&path).unwrap().is_dir() {
                out.extend(listing(&path));
            } else {
                out.push((path.clone(), fs::read(&path).unwrap_or_default()));
            }
        }
        out.sort();
        out
    }

    #[test]
    fn copia_todo_corrige_rutas_y_deja_la_carpeta_vieja_igual() {
        let root = temp_dir("completa");
        let (old, selah) = selah_folder(&root);
        let before = listing(&old);
        let new = root.join("org.emetpro.app");

        assert_eq!(migrate(&old, &new), Ok(true));
        assert_eq!(listing(&old), before, "la carpeta de Selah no se toca");
        drop(selah);

        let store = Store::open(&new);
        let (title, background): (String, String) =
            store.conn.query_row("SELECT title, background FROM songs", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(title, "Príncipe de paz");
        assert_eq!(PathBuf::from(&background), new.join("fondos").join("f.png"));
        assert!(PathBuf::from(&background).is_file());
        let items: String = store.conn.query_row("SELECT items FROM plans", [], |r| r.get(0)).unwrap();
        let items: serde_json::Value = serde_json::from_str(&items).unwrap();
        let media = PathBuf::from(items[0]["path"].as_str().unwrap());
        assert_eq!(media, new.join("medios").join("abc").join("icon.png"));
        assert!(media.is_file());
        let theme: String = store.conn.query_row("SELECT value FROM settings WHERE key = 'theme'", [], |r| r.get(0)).unwrap();
        assert_eq!(theme, "\"dark\"");
        let source: String =
            store.conn.query_row("SELECT value FROM settings WHERE key = 'obsTextSource'", [], |r| r.get(0)).unwrap();
        assert_eq!(source, "\"EmetPro · Texto\"");

        assert_eq!(fs::read_to_string(new.join("remote-key")).unwrap(), "clave");
        assert!(new.join("bibles").join("rvr.xml").is_file());
        #[cfg(unix)]
        assert_eq!(fs::read_link(new.join("libreoffice").join("soffice")).unwrap(), PathBuf::from("program/soffice.bin"));
        assert!(!new.join("localstorage").exists(), "lo del webview no se copia");
        assert!(!new.join(PENDING).exists());
        assert!(!new.join(format!("{FILE}.migrando")).exists());
        drop(store);

        // Segundo arranque: ya hay base, no se vuelve a copiar.
        assert_eq!(migrate(&old, &new), Ok(false));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sin_selah_no_hace_nada() {
        let root = temp_dir("sin-selah");
        let new = root.join("org.emetpro.app");
        assert_eq!(migrate(&root.join(OLD_IDENTIFIER), &new), Ok(false));
        assert!(!new.exists());
    }

    #[test]
    fn un_intento_cortado_se_repite_sin_pisar_lo_nuevo() {
        let root = temp_dir("cortado");
        let (old, selah) = selah_folder(&root);
        drop(selah);
        let new = root.join("org.emetpro.app");

        // Intento cortado: testigo puesto y base vacía (la que la app creó al arrancar sin datos).
        drop(Store::open(&new));
        fs::write(new.join(PENDING), b"").unwrap();
        assert_eq!(migrate(&old, &new), Ok(true));
        let store = Store::open(&new);
        let songs: i64 = store.conn.query_row("SELECT COUNT(*) FROM songs", [], |r| r.get(0)).unwrap();
        assert_eq!(songs, 1);

        // Si ya se guardó algo en EmetPro, el testigo no basta para volver a copiar encima.
        fs::write(new.join(PENDING), b"").unwrap();
        drop(store);
        assert_eq!(migrate(&old, &new), Ok(false));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn otra_carpeta_con_el_mismo_principio_no_se_toca() {
        let root = temp_dir("prefijo");
        let (old, selah) = selah_folder(&root);
        selah
            .execute("UPDATE songs SET background = ?1", [format!("{}.respaldo{MAIN_SEPARATOR}f.png", old.to_string_lossy())])
            .unwrap();
        let new = root.join("org.emetpro.app");
        assert_eq!(migrate(&old, &new), Ok(true));
        drop(selah);
        let store = Store::open(&new);
        let background: String = store.conn.query_row("SELECT background FROM songs", [], |r| r.get(0)).unwrap();
        assert!(background.contains("org.selah.app.respaldo"));
        let _ = fs::remove_dir_all(&root);
    }
}
