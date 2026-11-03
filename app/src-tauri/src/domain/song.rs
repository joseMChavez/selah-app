// Una letra de la biblioteca. El cuerpo lleva las diapositivas separadas por una línea con «****».
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub singer: String,
    pub writer: String,
    pub body: String,
    pub background: Option<String>,
}
