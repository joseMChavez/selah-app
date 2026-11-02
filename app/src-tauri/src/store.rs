// Base de datos local (SQLite): ajustes, biblioteca de letras y lista del culto.
// Los ajustes se guardan como JSON por clave: la interfaz conoce sus valores por defecto y los valida al leer,
// así que borrar una clave es volver al estado inicial.
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FILE: &str = "selah.db";

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
"];

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    id: String,
    title: String,
    singer: String,
    writer: String,
    body: String,
    background: Option<String>,
}

pub struct Store {
    conn: Connection,
    /// None si se trabaja en memoria porque no se pudo abrir el archivo.
    path: Option<PathBuf>,
}

fn now_secs() -> i64 {
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

    pub fn settings(&self) -> rusqlite::Result<serde_json::Map<String, Value>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        let mut map = serde_json::Map::new();
        for row in rows {
            let (key, value) = row?;
            // Un valor ilegible se ignora: la interfaz usará el valor por defecto.
            if let Ok(value) = serde_json::from_str(&value) {
                map.insert(key, value);
            }
        }
        Ok(map)
    }

    pub fn set_setting(&self, key: &str, value: &Value) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value.to_string()],
        )?;
        Ok(())
    }

    /// Vuelve al valor inicial esas claves; sin claves, todos los ajustes.
    pub fn reset_settings(&mut self, keys: &[String]) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        if keys.is_empty() {
            tx.execute("DELETE FROM settings", [])?;
        } else {
            for key in keys {
                tx.execute("DELETE FROM settings WHERE key = ?1", [key])?;
            }
        }
        tx.commit()
    }

    pub fn songs(&self) -> rusqlite::Result<Vec<Song>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, singer, writer, body, background FROM songs ORDER BY title COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Song {
                id: row.get(0)?,
                title: row.get(1)?,
                singer: row.get(2)?,
                writer: row.get(3)?,
                body: row.get(4)?,
                background: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn save_song(&self, song: &Song) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO songs (id, title, singer, writer, body, background, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET title = excluded.title, singer = excluded.singer,
                 writer = excluded.writer, body = excluded.body, background = excluded.background,
                 updated_at = excluded.updated_at",
            params![song.id, song.title, song.singer, song.writer, song.body, song.background, now_secs()],
        )?;
        Ok(())
    }

    pub fn delete_song(&mut self, id: &str) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM setlist WHERE song_id = ?1", [id])?;
        tx.execute("DELETE FROM songs WHERE id = ?1", [id])?;
        tx.commit()
    }

    pub fn setlist(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT song_id FROM setlist ORDER BY position")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    /// Sustituye la lista entera. Los ids que no están en la biblioteca se descartan.
    pub fn set_setlist(&mut self, ids: &[String]) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM setlist", [])?;
        for (position, id) in ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO setlist (position, song_id) SELECT ?1, id FROM songs WHERE id = ?2",
                params![position as i64, id],
            )?;
        }
        tx.commit()
    }

    /// Borra la biblioteca de letras y la lista.
    pub fn reset_songs(&mut self) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM setlist", [])?;
        tx.execute("DELETE FROM songs", [])?;
        tx.commit()
    }
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    let applied: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    // Una base de una versión más nueva de Selah se usa tal cual: no se toca su esquema.
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(applied.max(0) as usize) {
        conn.execute_batch(&format!("BEGIN; {sql} PRAGMA user_version = {}; COMMIT;", i + 1))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("selah-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn song(id: &str, title: &str) -> Song {
        Song {
            id: id.into(),
            title: title.into(),
            singer: String::new(),
            writer: String::new(),
            body: "a\n****\nb".into(),
            background: None,
        }
    }

    #[test]
    fn guarda_y_restablece_ajustes() {
        let dir = temp_dir("ajustes");
        let store = Store::open(&dir);
        store.set_setting("theme", &Value::from("dark")).unwrap();
        store.set_setting("bible", &Value::from("rvr")).unwrap();
        drop(store);

        let mut store = Store::open(&dir);
        assert_eq!(store.settings().unwrap()["theme"], "dark");
        store.reset_settings(&["theme".into()]).unwrap();
        let settings = store.settings().unwrap();
        assert!(!settings.contains_key("theme"));
        assert_eq!(settings["bible"], "rvr");
        store.reset_settings(&[]).unwrap();
        assert!(store.settings().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn letras_y_lista() {
        let dir = temp_dir("letras");
        let mut store = Store::open(&dir);
        store.save_song(&song("b", "Bendito")).unwrap();
        store.save_song(&song("a", "Alabanza")).unwrap();
        store.set_setlist(&["b".into(), "no-existe".into(), "a".into()]).unwrap();
        assert_eq!(store.setlist().unwrap(), vec!["b", "a"]);
        assert_eq!(store.songs().unwrap()[0].title, "Alabanza");

        store.delete_song("b").unwrap();
        assert_eq!(store.setlist().unwrap(), vec!["a"]);
        store.reset_songs().unwrap();
        assert!(store.songs().unwrap().is_empty() && store.setlist().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
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
