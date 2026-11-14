// Planes del culto en la base.
use rusqlite::{params, Connection, OptionalExtension};

use super::domain::{Plan, PlanItem};
use crate::core::db::{now_secs, Store};

fn to_sql_error(e: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(e))
}

/// Crea o actualiza un plan entero. También la usan otras funcionalidades dentro de su propia transacción
/// (features/transfer/repo.rs).
pub fn upsert_plan(conn: &Connection, plan: &Plan) -> rusqlite::Result<()> {
    let items = serde_json::to_string(&plan.items).map_err(to_sql_error)?;
    conn.execute(
        "INSERT INTO plans (id, name, date, notes, items, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, date = excluded.date, notes = excluded.notes,
             items = excluded.items, updated_at = excluded.updated_at",
        params![plan.id, plan.name, plan.date, plan.notes, items, now_secs()],
    )?;
    Ok(())
}

fn read_plan(row: &rusqlite::Row) -> rusqlite::Result<Plan> {
    let items: String = row.get(4)?;
    let items: Vec<PlanItem> = serde_json::from_str(&items)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
    Ok(Plan { id: row.get(0)?, name: row.get(1)?, date: row.get(2)?, notes: row.get(3)?, items })
}

impl Store {
    /// Los planes con fecha primero, del más próximo al más lejano; después los que no tienen fecha.
    pub fn plans(&self) -> rusqlite::Result<Vec<Plan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, date, notes, items FROM plans
             ORDER BY date IS NULL, date, name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], read_plan)?;
        rows.collect()
    }

    pub fn plan(&self, id: &str) -> rusqlite::Result<Option<Plan>> {
        self.conn
            .query_row("SELECT id, name, date, notes, items FROM plans WHERE id = ?1", [id], read_plan)
            .optional()
    }

    pub fn save_plan(&self, plan: &Plan) -> rusqlite::Result<()> {
        upsert_plan(&self.conn, plan)
    }

    pub fn delete_plan(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM plans WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Borra todos los planes.
    pub fn reset_plans(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM plans", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plans::domain::ItemContent;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emetpro-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn plan(id: &str, name: &str, date: Option<&str>) -> Plan {
        Plan {
            id: id.into(),
            name: name.into(),
            date: date.map(Into::into),
            notes: String::new(),
            items: vec![PlanItem { id: "s".into(), content: ItemContent::Section { title: "Alabanza".into() } }],
        }
    }

    #[test]
    fn planes_ordenados_por_fecha() {
        let dir = temp_dir("planes");
        let store = Store::open(&dir);
        store.save_plan(&plan("a", "Sin fecha", None)).unwrap();
        store.save_plan(&plan("b", "Domingo", Some("2026-11-15"))).unwrap();
        store.save_plan(&plan("c", "Jueves", Some("2026-11-12"))).unwrap();
        let names: Vec<_> = store.plans().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Jueves", "Domingo", "Sin fecha"]);

        let mut changed = plan("b", "Domingo de Santa Cena", Some("2026-11-15"));
        changed.items.clear();
        store.save_plan(&changed).unwrap();
        assert_eq!(store.plan("b").unwrap(), Some(changed));

        store.delete_plan("a").unwrap();
        assert_eq!(store.plans().unwrap().len(), 2);
        store.reset_plans().unwrap();
        assert!(store.plans().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Una base de la versión 0.6 con lista del culto: la lista pasa a ser el plan «Lista», en el mismo orden.
    #[test]
    fn la_lista_del_culto_pasa_a_ser_un_plan() {
        let dir = temp_dir("lista-a-plan");
        std::fs::create_dir_all(&dir).unwrap();
        {
            let conn = Connection::open(dir.join(crate::core::db::FILE)).unwrap();
            conn.execute_batch(
                "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE songs (id TEXT PRIMARY KEY, title TEXT NOT NULL, singer TEXT NOT NULL DEFAULT '',
                     writer TEXT NOT NULL DEFAULT '', body TEXT NOT NULL, background TEXT, updated_at INTEGER NOT NULL);
                 CREATE TABLE setlist (position INTEGER PRIMARY KEY, song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE);
                 INSERT INTO songs VALUES ('x', 'X', '', '', 'a', NULL, 0), ('y', 'Y', '', '', 'b', NULL, 0);
                 INSERT INTO setlist VALUES (0, 'y'), (1, 'x');
                 PRAGMA user_version = 1;",
            )
            .unwrap();
        }
        let store = Store::open(&dir);
        let plans = store.plans().unwrap();
        assert_eq!(plans.len(), 1);
        let songs: Vec<_> = plans[0]
            .items
            .iter()
            .map(|i| match &i.content {
                ItemContent::Song { song_id } => song_id.as_str(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(songs, ["y", "x"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sin_lista_no_se_crea_ningun_plan() {
        let dir = temp_dir("sin-lista");
        let store = Store::open(&dir);
        assert!(store.plans().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
