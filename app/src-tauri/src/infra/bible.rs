// Biblias en el formato XML de Beblia (D-012):
// <bible translation status|info link> > testament > book[number] > chapter[number] > verse[number].
// Hay dos orígenes: las Biblias incluidas en la instalación (solo lectura, una carpeta por idioma)
// y las que importa el usuario, que se copian a la carpeta de datos de la app. Se cargan en memoria una vez.
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Carpetas donde buscar Biblias.
pub struct BibleDirs {
    /// Biblias incluidas en la instalación, organizadas en subcarpetas por idioma.
    pub builtin: Option<PathBuf>,
    /// Biblias importadas por el usuario.
    pub user: PathBuf,
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

struct Bible {
    info: BibleInfo,
    chapters: BTreeMap<(u32, u32), Vec<Verse>>,
    /// Texto normalizado (sin acentos ni signos) para buscar por palabras, en el mismo orden que `chapters`.
    search_index: Vec<(u32, u32, u32, String)>,
}

#[derive(Default)]
pub struct Library {
    loaded: bool,
    bibles: HashMap<String, Bible>,
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
fn abbreviation_and_language(stem: &str) -> (String, Option<String>) {
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
fn clean_name(name: &str) -> String {
    match name.find(" (Más información") {
        Some(i) => name[..i].trim().to_string(),
        None => name.trim().to_string(),
    }
}

fn elements<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    tag: &'static str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'input>> {
    node.children()
        .filter(move |n| n.is_element() && n.tag_name().name() == tag)
}

fn number(node: roxmltree::Node<'_, '_>) -> Option<u32> {
    node.attribute("number").and_then(|n| n.trim().parse().ok())
}

/// Archivos .xml de la carpeta y de sus subcarpetas directas (una por idioma).
fn xml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
    let mut files = Vec::new();
    for path in entries.flatten().map(|e| e.path()) {
        if path.is_dir() {
            if let Ok(sub) = fs::read_dir(&path) {
                files.extend(sub.flatten().map(|e| e.path()).filter(|p| is_xml(p)));
            }
        } else if is_xml(&path) {
            files.push(path);
        }
    }
    files
}

fn is_xml(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml"))
}

/// "…/bibles/es/archivo.xml" -> Some("es").
fn folder_language(path: &Path) -> Option<String> {
    let folder = path.parent()?.file_name()?.to_str()?;
    (folder.len() == 2 && folder.chars().all(|c| c.is_ascii_lowercase())).then(|| folder.to_string())
}

fn parse(path: &Path, builtin: bool) -> Result<Bible, String> {
    let xml = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = roxmltree::Document::parse(xml.trim_start_matches('\u{feff}'))
        .map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.tag_name().name() != "bible" {
        return Err("falta el elemento <bible>".into());
    }

    let mut chapters = BTreeMap::new();
    for book in root
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "book")
    {
        let Some(book_number) = number(book) else {
            continue;
        };
        for chapter in elements(book, "chapter") {
            let Some(chapter_number) = number(chapter) else {
                continue;
            };
            let verses: Vec<Verse> = elements(chapter, "verse")
                .filter_map(|v| {
                    let text: String = v
                        .descendants()
                        .filter(|n| n.is_text())
                        .filter_map(|n| n.text())
                        .collect();
                    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    // Algunas traducciones unen versículos y dejan los demás vacíos.
                    (!text.is_empty()).then_some(Verse {
                        number: number(v)?,
                        text,
                    })
                })
                .collect();
            if !verses.is_empty() {
                chapters.insert((book_number, chapter_number), verses);
            }
        }
    }
    if chapters.is_empty() {
        return Err("no contiene versículos".into());
    }

    let mut books: BTreeMap<u32, u32> = BTreeMap::new();
    for &(book, chapter) in chapters.keys() {
        let max = books.entry(book).or_default();
        *max = (*max).max(chapter);
    }
    let search_index = chapters
        .iter()
        .flat_map(|(&(b, c), verses)| {
            verses
                .iter()
                .map(move |v| (b, c, v.number, normalize(&v.text)))
        })
        .collect();

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("biblia")
        .to_string();
    let (abbreviation, name_language) = abbreviation_and_language(&stem);
    let language = folder_language(path).or(name_language);
    let copyright = root
        .attribute("status")
        .or(root.attribute("info"))
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(Bible {
        info: BibleInfo {
            id: stem.clone(),
            abbreviation,
            name: clean_name(root.attribute("translation").unwrap_or(&stem)),
            copyright,
            link: root.attribute("link").map(str::to_string),
            language,
            builtin,
            books: books
                .into_iter()
                .map(|(number, chapters)| BookInfo { number, chapters })
                .collect(),
        },
        chapters,
        search_index,
    })
}

impl Library {
    /// Carga una sola vez todas las Biblias. Un archivo dañado no impide cargar los demás.
    /// Si una Biblia importada coincide con una incluida, se usa la incluida.
    pub fn ensure_loaded(&mut self, dirs: &BibleDirs) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        let mut paths: Vec<(PathBuf, bool)> = Vec::new();
        if let Some(builtin) = &dirs.builtin {
            paths.extend(xml_files(builtin).into_iter().map(|p| (p, true)));
        }
        paths.extend(xml_files(&dirs.user).into_iter().map(|p| (p, false)));

        // Cada archivo se lee en su propio hilo: con varias Biblias la espera es la del archivo más grande.
        let parsed: Vec<(PathBuf, Result<Bible, String>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = paths
                .into_iter()
                .map(|(path, builtin)| scope.spawn(move || {
                    let result = parse(&path, builtin);
                    (path, result)
                }))
                .collect();
            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });
        for (path, result) in parsed {
            match result {
                Ok(bible) => {
                    let keep_existing = self.bibles.get(&bible.info.id).is_some_and(|b| b.info.builtin);
                    if !keep_existing {
                        self.bibles.insert(bible.info.id.clone(), bible);
                    }
                }
                Err(e) => eprintln!("Biblia ignorada {}: {e}", path.display()),
            }
        }
    }

    pub fn list(&mut self, dirs: &BibleDirs) -> Vec<BibleInfo> {
        self.ensure_loaded(dirs);
        let mut list: Vec<BibleInfo> = self.bibles.values().map(|b| b.info.clone()).collect();
        list.sort_by(|a, b| a.language.cmp(&b.language).then_with(|| a.abbreviation.cmp(&b.abbreviation)));
        list
    }

    /// Valida el archivo antes de copiarlo: si no se puede leer, no se guarda nada.
    /// Si ya viene incluida en la instalación, no se copia otra vez.
    pub fn import(&mut self, dirs: &BibleDirs, source: &Path) -> Result<BibleInfo, String> {
        self.ensure_loaded(dirs);
        let bible = parse(source, false)?;
        if let Some(existing) = self.bibles.get(&bible.info.id).filter(|b| b.info.builtin) {
            return Ok(existing.info.clone());
        }
        fs::create_dir_all(&dirs.user).map_err(|e| e.to_string())?;
        let target = dirs.user.join(format!("{}.xml", bible.info.id));
        if source != target {
            fs::copy(source, &target).map_err(|e| e.to_string())?;
        }
        let info = bible.info.clone();
        self.bibles.insert(info.id.clone(), bible);
        Ok(info)
    }

    pub fn remove(&mut self, dirs: &BibleDirs, id: &str) -> Result<(), String> {
        self.ensure_loaded(dirs);
        if self.bibles.get(id).is_some_and(|b| b.info.builtin) {
            return Err("las Biblias incluidas en la instalación no se pueden quitar".into());
        }
        if self.bibles.remove(id).is_some() {
            fs::remove_file(dirs.user.join(format!("{id}.xml"))).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn chapter(&mut self, dirs: &BibleDirs, id: &str, book: u32, chapter: u32) -> Option<Vec<Verse>> {
        self.ensure_loaded(dirs);
        self.bibles.get(id)?.chapters.get(&(book, chapter)).cloned()
    }

    /// Versículos que contienen todas las palabras buscadas, sin distinguir acentos ni mayúsculas.
    pub fn search(&mut self, dirs: &BibleDirs, id: &str, query: &str) -> Option<Vec<SearchHit>> {
        self.ensure_loaded(dirs);
        let bible = self.bibles.get(id)?;
        let words: Vec<String> = normalize(query)
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if words.is_empty() {
            return Some(Vec::new());
        }
        let hits = bible
            .search_index
            .iter()
            .filter(|(.., text)| words.iter().all(|w| text.contains(w.as_str())))
            .take(MAX_SEARCH_HITS)
            .filter_map(|&(book, chapter, verse, _)| {
                let text = bible
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
            .collect();
        Some(hits)
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

    /// Usa los XML de resources/bibles/ (fuera del repositorio): `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn parses_sample_bibles() {
        let dir = BibleDirs {
            builtin: Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/bibles")),
            user: std::env::temp_dir().join("selah-test-sin-biblias"),
        };
        let mut library = Library::default();
        let start = std::time::Instant::now();
        let list = library.list(&dir);
        println!("{} Biblias en {:?}", list.len(), start.elapsed());
        assert!(!list.is_empty());
        for info in &list {
            let john = library.chapter(&dir, &info.id, 43, 3).expect("Juan 3");
            let v16 = john.iter().find(|v| v.number == 16).expect("Juan 3:16");
            println!(
                "{} ({} libros) | {}\n  Juan 3:16 {}\n  © {}",
                info.abbreviation,
                info.books.len(),
                info.name,
                v16.text,
                info.copyright
            );
            let hits = library.search(&dir, &info.id, "unigenito").unwrap();
            println!("  búsqueda «unigenito»: {} resultados", hits.len());
        }
    }
}
