// Base de datos local (SQLite) compartida por las funcionalidades: cada una añade sus consultas con su propio
// `impl Store` (features/settings/repo.rs, features/songs/repo.rs, features/plans/repo.rs). Aquí solo se abre el archivo y se migra.
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::core::error::CommandError;

pub const FILE: &str = "emetpro.db";

/// Cada entrada lleva la base de una versión a la siguiente; PRAGMA user_version guarda cuántas se aplicaron.
/// Nunca se edita una entrada publicada: los cambios van en una entrada nueva.
const MIGRATIONS: &[&str] = &["
    CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE songs (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        singer TEXT NOT NULL DEFAULT '',
        writer TEXT NOT NULL DEFAULT '',
        body TEXT NOT NULL,
        background TEXT,
        updated_at INTEGER NOT NULL
    );
    CREATE TABLE setlist (
        position INTEGER PRIMARY KEY,
        song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE
    );
", "
    -- Planes del culto (D-019). Los elementos van juntos como JSON (features/plans/domain.rs): el plan se guarda entero.
    CREATE TABLE plans (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        date TEXT,
        notes TEXT NOT NULL DEFAULT '',
        items TEXT NOT NULL DEFAULT '[]',
        updated_at INTEGER NOT NULL
    );
    -- La lista del culto pasa a ser un plan, con sus letras en el mismo orden.
    INSERT INTO plans (id, name, date, notes, items, updated_at)
    SELECT 'lista', 'Lista', NULL, '',
        json_group_array(json_object('id', song_id, 'kind', 'song', 'songId', song_id)),
        CAST(strftime('%s', 'now') AS INTEGER)
    FROM (SELECT song_id FROM setlist ORDER BY position)
    HAVING COUNT(*) > 0;
    DROP TABLE setlist;
"];

pub struct Store {
    pub(crate) conn: Connection,
    /// None si se trabaja en memoria porque no se pudo abrir el archivo.
    path: Option<PathBuf>,
}

pub(crate) fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl Store {
    /// Abre (o crea) la base en esa carpeta. Siempre devuelve un Store utilizable:
    /// si el archivo está dañado se aparta con otro nombre y se empieza de cero;
    /// si ni así se puede, se trabaja en memoria y los cambios no se guardan.
    pub fn open(dir: &Path) -> Self {
        let path = dir.join(FILE);
        let opened = std::fs::create_dir_all(dir)
            .map_err(|e| e.to_string())
            .and_then(|_| Self::open_file(&path));
        match opened {
            Ok(store) => store,
            Err(reason) => {
                eprintln!("[base de datos] no se pudo abrir {}: {reason}", path.display());
                if path.exists() {
                    // Se aparta en vez de borrarla: así se puede intentar recuperar a mano.
                    let aside = path.with_extension(format!("db.dañada-{}", now_secs()));
                    if std::fs::rename(&path, &aside).is_ok() {
                        eprintln!("[base de datos] la anterior quedó en {}", aside.display());
                        for ext in ["db-wal", "db-shm"] {
                            let _ = std::fs::remove_file(path.with_extension(ext));
                        }
                        if let Ok(store) = Self::open_file(&path) {
                            return store;
                        }
                    }
                }
                eprintln!("[base de datos] se trabaja en memoria: los cambios no se guardarán");
                let conn = Connection::open_in_memory().expect("SQLite en memoria");
                migrate(&conn).expect("migraciones en memoria");
                Self { conn, path: None }
            }
        }
    }

    fn open_file(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(3)).map_err(|e| e.to_string())?;
        let check: String = conn
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if check != "ok" {
            return Err(format!("archivo dañado: {check}"));
        }
        // WAL: un corte de luz a mitad de una escritura no deja la base a medias.
        conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
        migrate(&conn).map_err(|e| e.to_string())?;
        Ok(Self { conn, path: Some(path.to_path_buf()) })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    let applied: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    // Una base de una versión más nueva de EmetPro se usa tal cual: no se toca su esquema.
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(applied.max(0) as usize) {
        conn.execute_batch(&format!("BEGIN; {sql} PRAGMA user_version = {}; COMMIT;", i + 1))?;
    }
    Ok(())
}

/// La base de la app, gestionada por Tauri. None solo antes de abrirla al arrancar.
#[derive(Default)]
pub struct Db(pub Mutex<Option<Store>>);

pub fn with_store<T>(
    db: &Db,
    action: impl FnOnce(&mut Store) -> rusqlite::Result<T>,
) -> Result<T, CommandError> {
    let mut guard = db.0.lock().unwrap();
    let store = guard.as_mut().ok_or_else(|| CommandError::Storage {
        message: "la base de datos aún no está lista".into(),
    })?;
    action(store).map_err(|e| CommandError::Storage { message: e.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emetpro-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn base_dañada_se_aparta_y_se_empieza_de_cero() {
        let dir = temp_dir("dañada");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), b"esto no es una base de datos SQLite, solo basura de prueba").unwrap();
        let store = Store::open(&dir);
        assert!(store.path().is_some());
        store.set_setting("theme", &Value::from("light")).unwrap();
        let aside = std::fs::read_dir(&dir).unwrap().flatten().any(|e| e.file_name().to_string_lossy().contains("dañada-"));
        assert!(aside);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
