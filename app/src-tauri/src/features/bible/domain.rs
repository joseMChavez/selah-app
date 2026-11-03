// Biblias en memoria: tipos que viajan a la interfaz, normalización de texto y búsqueda. Sin archivos ni Tauri.
use std::collections::BTreeMap;

use serde::Serialize;

const MAX_SEARCH_HITS: usize = 200;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookInfo {
    pub number: u32,
    pub chapters: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleInfo {
    pub id: String,
    pub abbreviation: String,
    pub name: String,
    /// Texto de derechos tal como lo declara el archivo; se muestra siempre junto al pasaje.
    pub copyright: String,
    pub link: Option<String>,
    /// Idioma del texto (no de la interfaz): de la carpeta (es/, en/…) o del nombre del archivo.
    pub language: Option<String>,
    /// Incluida en la instalación: no se puede quitar desde la app.
    pub builtin: bool,
    pub books: Vec<BookInfo>,
}

#[derive(Clone, Serialize)]
pub struct Verse {
    pub number: u32,
    pub text: String,
}

#[derive(Serialize)]
pub struct SearchHit {
    pub book: u32,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

pub struct Bible {
    pub info: BibleInfo,
    pub chapters: BTreeMap<(u32, u32), Vec<Verse>>,
    /// Texto normalizado (sin acentos ni signos) para buscar por palabras, en el mismo orden que `chapters`.
    pub search_index: Vec<(u32, u32, u32, String)>,
}

impl Bible {
    /// Versículos que contienen todas las palabras buscadas, sin distinguir acentos ni mayúsculas.
    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        let words: Vec<String> = normalize(query)
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if words.is_empty() {
            return Vec::new();
        }
        self.search_index
            .iter()
            .filter(|(.., text)| words.iter().all(|w| text.contains(w.as_str())))
            .take(MAX_SEARCH_HITS)
            .filter_map(|&(book, chapter, verse, _)| {
                let text = self
                    .chapters
                    .get(&(book, chapter))?
                    .iter()
                    .find(|v| v.number == verse)?
                    .text
                    .clone();
                Some(SearchHit {
                    book,
                    chapter,
                    verse,
                    text,
                })
            })
            .collect()
    }
}

/// Minúsculas, sin acentos (se conserva la ñ) y sin signos de puntuación.
pub fn normalize(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect()
}

/// "SpanishRVR1960Bible" -> ("RVR1960", Some("es")). Para otros nombres se usa el nombre del archivo.
pub fn abbreviation_and_language(stem: &str) -> (String, Option<String>) {
    let base = stem.strip_suffix("Bible").unwrap_or(stem);
    let split = base
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase() || c.is_ascii_digit())
        .map(|(i, _)| i);
    let (prefix, rest) = match split {
        Some(i) => base.split_at(i),
        None => (base, ""),
    };
    let language = match prefix {
        "Spanish" => Some("es"),
        "English" => Some("en"),
        "Italian" => Some("it"),
        _ => None,
    };
    let abbreviation = if language.is_some() && !rest.is_empty() {
        rest
    } else {
        base
    };
    (abbreviation.to_string(), language.map(str::to_string))
}

/// Quita coletillas como " (Más información sobre Biblia Reina Valera 1960)".
pub fn clean_name(name: &str) -> String {
    match name.find(" (Más información") {
        Some(i) => name[..i].trim().to_string(),
        None => name.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviation_from_beblia_file_names() {
        assert_eq!(
            abbreviation_and_language("SpanishRVR1960Bible"),
            ("RVR1960".into(), Some("es".into()))
        );
        assert_eq!(
            abbreviation_and_language("SpanishVBL2022Bible"),
            ("VBL2022".into(), Some("es".into()))
        );
        assert_eq!(
            abbreviation_and_language("SpanishBible"),
            ("Spanish".into(), Some("es".into()))
        );
        assert_eq!(
            abbreviation_and_language("mi_biblia"),
            ("mi_biblia".into(), None)
        );
    }

    #[test]
    fn normalize_ignores_accents_and_punctuation() {
        assert_eq!(normalize("¡Señor, AMÓ!"), " señor  amo ");
    }
}
