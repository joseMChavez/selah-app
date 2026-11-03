// Ajustes en la base: JSON por clave. La interfaz conoce sus valores por defecto y los valida al leer,
// así que borrar una clave es volver al estado inicial.
use rusqlite::params;
use serde_json::Value;

use crate::core::db::Store;

impl Store {
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
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use crate::core::db::Store;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("selah-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
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
}
