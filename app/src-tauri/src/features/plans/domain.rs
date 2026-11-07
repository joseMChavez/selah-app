// Un plan del culto: lo que se va a presentar, en orden (D-019). Tipos puros: sin Tauri.
use serde::{Deserialize, Serialize};

use crate::features::media::domain::MediaKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub name: String,
    /// Día del culto, «AAAA-MM-DD»; None = sin fecha.
    pub date: Option<String>,
    pub notes: String,
    pub items: Vec<PlanItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    /// Único dentro del plan; la interfaz lo usa para seguir cada elemento al reordenar.
    pub id: String,
    #[serde(flatten)]
    pub content: ItemContent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ItemContent {
    /// Un título que ordena el plan («Alabanza», «Predicación»); no se muestra en pantalla.
    Section { title: String },
    /// Una letra de la biblioteca. Si se borra de la biblioteca, el plan la marca como borrada.
    Song { song_id: String },
    /// Imagen, video, PDF o presentación, por su ruta en este equipo.
    Media { path: String, media: MediaKind },
    /// Versículos que el predicador quiere tener a un clic: un capítulo entero o un rango.
    Scripture {
        book: u32,
        chapter: u32,
        /// None = el capítulo entero.
        verse_start: Option<u32>,
        verse_end: Option<u32>,
        /// Versión fija (id de la Biblia); None = la elegida en la consola al mostrarlo.
        #[serde(default)]
        bible: Option<String>,
    },
    /// Un tiempo con cuenta atrás en pantalla (receso, conferencia).
    Pause {
        title: String,
        duration_ms: u64,
        #[serde(default)]
        background: Option<String>,
    },
}

impl ItemContent {
    /// El archivo que este elemento necesita mostrar, si lo hay.
    pub fn file(&self) -> Option<&str> {
        match self {
            Self::Media { path, .. } => Some(path),
            Self::Pause { background, .. } => background.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elementos_en_json() {
        let json = r#"[
            {"id":"1","kind":"section","title":"Alabanza"},
            {"id":"2","kind":"song","songId":"abc"},
            {"id":"3","kind":"media","path":"/x/a.mp4","media":"video"},
            {"id":"4","kind":"pause","title":"Receso","durationMs":600000},
            {"id":"5","kind":"scripture","book":43,"chapter":3,"verseStart":16,"verseEnd":18}
        ]"#;
        let items: Vec<PlanItem> = serde_json::from_str(json).unwrap();
        assert_eq!(items[1].content, ItemContent::Song { song_id: "abc".into() });
        assert_eq!(items[2].content.file(), Some("/x/a.mp4"));
        assert_eq!(items[3].content, ItemContent::Pause { title: "Receso".into(), duration_ms: 600_000, background: None });
        assert_eq!(
            items[4].content,
            ItemContent::Scripture { book: 43, chapter: 3, verse_start: Some(16), verse_end: Some(18), bible: None }
        );
        let back = serde_json::to_string(&items[1]).unwrap();
        assert_eq!(back, r#"{"id":"2","kind":"song","songId":"abc"}"#);
    }
}
