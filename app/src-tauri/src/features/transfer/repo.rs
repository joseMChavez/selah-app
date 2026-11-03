// Guarda una importación entera en una sola transacción: o entran todas las letras (y la lista), o ninguna.
use crate::core::db::Store;
use crate::features::songs::domain::Song;
use crate::features::songs::repo::upsert_song;

impl Store {
    /// Guarda las letras y añade `to_setlist` al final de la lista del culto (sin repetir las que ya están).
    pub fn import_songs(&mut self, songs: &[Song], to_setlist: &[String]) -> rusqlite::Result<usize> {
        let tx = self.conn.transaction()?;
        for song in songs {
            upsert_song(&tx, song)?;
        }
        let mut added = 0;
        for id in to_setlist {
            added += tx.execute(
                "INSERT INTO setlist (position, song_id)
                 SELECT COALESCE((SELECT MAX(position) FROM setlist), -1) + 1, id FROM songs
                 WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM setlist WHERE song_id = ?1)",
                [id],
            )?;
        }
        tx.commit()?;
        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::db::Store;
    use crate::features::songs::domain::Song;

    fn song(id: &str) -> Song {
        Song { id: id.into(), title: id.into(), singer: String::new(), writer: String::new(), body: "x".into(), background: None }
    }

    #[test]
    fn importa_y_amplia_la_lista_sin_repetir() {
        let dir = std::env::temp_dir().join(format!("selah-transfer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = Store::open(&dir);
        store.save_song(&song("a")).unwrap();
        store.set_setlist(&["a".into()]).unwrap();
        let added = store.import_songs(&[song("b"), song("c")], &["b".into(), "a".into(), "c".into(), "no".into()]).unwrap();
        assert_eq!(added, 2, "a ya estaba y «no» no existe");
        assert_eq!(store.setlist().unwrap(), ["a", "b", "c"]);
        assert_eq!(store.songs().unwrap().len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
