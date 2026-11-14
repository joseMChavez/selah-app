// Guarda una importación entera en una sola transacción: o entran todas las letras (y su plan), o ninguna.
use crate::core::db::Store;
use crate::features::plans::domain::Plan;
use crate::features::plans::repo::upsert_plan;
use crate::features::songs::domain::Song;
use crate::features::songs::repo::upsert_song;

impl Store {
    /// Guarda las letras y, si viene, el plan con la lista del archivo.
    pub fn import_songs(&mut self, songs: &[Song], plan: Option<&Plan>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        for song in songs {
            upsert_song(&tx, song)?;
        }
        if let Some(plan) = plan {
            upsert_plan(&tx, plan)?;
        }
        tx.commit()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::db::Store;
    use crate::features::plans::domain::{ItemContent, Plan, PlanItem};
    use crate::features::songs::domain::Song;

    fn song(id: &str) -> Song {
        Song { id: id.into(), title: id.into(), singer: String::new(), writer: String::new(), body: "x".into(), background: None }
    }

    #[test]
    fn importa_las_letras_y_su_plan_juntos() {
        let dir = std::env::temp_dir().join(format!("emetpro-transfer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = Store::open(&dir);
        store.save_song(&song("a")).unwrap();
        let plan = Plan {
            id: "p".into(),
            name: "Domingo".into(),
            date: None,
            notes: String::new(),
            items: ["b", "a"]
                .iter()
                .map(|id| PlanItem { id: id.to_string(), content: ItemContent::Song { song_id: id.to_string() } })
                .collect(),
        };
        store.import_songs(&[song("b"), song("c")], Some(&plan)).unwrap();
        assert_eq!(store.songs().unwrap().len(), 3);
        assert_eq!(store.plans().unwrap(), [plan]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
