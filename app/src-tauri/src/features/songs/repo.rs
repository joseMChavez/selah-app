// Letras y lista del culto en la base.
use rusqlite::{params, Connection};

use super::domain::Song;
use crate::core::db::{now_secs, Store};

/// Crea o actualiza una letra. También la usan otras funcionalidades dentro de su propia transacción
/// (features/transfer/repo.rs).
pub fn upsert_song(conn: &Connection, song: &Song) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO songs (id, title, singer, writer, body, background, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET title = excluded.title, singer = excluded.singer,
             writer = excluded.writer, body = excluded.body, background = excluded.background,
             updated_at = excluded.updated_at",
        params![song.id, song.title, song.singer, song.writer, song.body, song.background, now_secs()],
    )?;
    Ok(())
}

impl Store {
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
        upsert_song(&self.conn, song)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}
