// Recorrido de carpetas en busca de medios.
use std::path::PathBuf;

use super::domain::media_kind;

/// Archivos compatibles de la carpeta y sus subcarpetas (sin las ocultas), ordenados por ruta y como mucho `limit`.
pub fn media_in_folder(root: PathBuf, limit: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for path in entries.flatten().map(|e| e.path()) {
            let hidden = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.'));
            if hidden {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else if media_kind(&path).is_some() {
                found.push(path);
            }
        }
        if found.len() >= limit {
            break;
        }
    }
    found.sort();
    found.truncate(limit);
    found
}
