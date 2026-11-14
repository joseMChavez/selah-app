// Las Biblias de la app: las incluidas en la instalación (solo lectura, una carpeta por idioma) y las que importa
// el usuario, copiadas a la carpeta de datos. Se cargan en memoria una vez.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::domain::{Bible, BibleInfo, SearchHit, Verse};
use super::xml::parse;

/// Carpetas donde buscar Biblias.
pub struct BibleDirs {
    /// Biblias incluidas en la instalación, organizadas en subcarpetas por idioma.
    pub builtin: Option<PathBuf>,
    /// Biblias importadas por el usuario.
    pub user: PathBuf,
}

#[derive(Default)]
pub struct Library {
    loaded: bool,
    bibles: HashMap<String, Bible>,
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
pub fn folder_language(path: &Path) -> Option<String> {
    let folder = path.parent()?.file_name()?.to_str()?;
    (folder.len() == 2 && folder.chars().all(|c| c.is_ascii_lowercase())).then(|| folder.to_string())
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
        Some(self.bibles.get(id)?.search(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Usa los XML de resources/bibles/ (fuera del repositorio): `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn parses_sample_bibles() {
        let dir = BibleDirs {
            builtin: Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/bibles")),
            user: std::env::temp_dir().join("emetpro-test-sin-biblias"),
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
