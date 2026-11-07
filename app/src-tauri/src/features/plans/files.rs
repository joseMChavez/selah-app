// Copias de los medios de los planes en la carpeta de la app (`app_data_dir/medios`), para que un plan siga
// funcionando aunque el archivo original se mueva o se borre (D-019). La base guarda la ruta de la copia.
// Cada copia va en una carpeta con el principio del SHA-256 de su contenido y conserva su nombre:
// añadir dos veces el mismo archivo no lo duplica, y el plan sigue mostrando el nombre de siempre.
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub const DIR: &str = "medios";

/// Copia `source` a `dir` (si no estaba ya) y devuelve la ruta de la copia.
pub fn store_copy(dir: &Path, source: &Path) -> io::Result<PathBuf> {
    let mut hasher = Sha256::new();
    io::copy(&mut fs::File::open(source)?, &mut hasher)?;
    let hash = format!("{:x}", hasher.finalize());
    let name = source
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "sin nombre de archivo"))?;
    let folder = dir.join(&hash[..16]);
    let target = folder.join(name);
    if target.is_file() {
        return Ok(target);
    }
    fs::create_dir_all(&folder)?;
    // Se copia aparte y se renombra al final: un corte a mitad no deja una copia rota con el nombre bueno.
    let partial = folder.join(".copiando");
    fs::copy(source, &partial)?;
    fs::rename(&partial, &target)?;
    Ok(target)
}

/// Borra las copias que ya no usa ningún plan y devuelve lo que ocupan las que quedan, en bytes.
pub fn clean(dir: &Path, used: &HashSet<PathBuf>) -> u64 {
    let mut kept = 0;
    let Ok(folders) = fs::read_dir(dir) else { return 0 };
    for folder in folders.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
        let Ok(files) = fs::read_dir(&folder) else { continue };
        for file in files.flatten().map(|e| e.path()) {
            if used.contains(&file) {
                kept += file.metadata().map(|m| m.len()).unwrap_or(0);
            } else {
                let _ = fs::remove_file(&file);
            }
        }
        // Solo se borra si quedó vacía.
        let _ = fs::remove_dir(&folder);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("selah-medios-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copia_sin_duplicar_y_limpia_lo_que_no_se_usa() {
        let base = temp_dir("copias");
        let dir = base.join(DIR);
        let a = base.join("anuncio.png");
        let b = base.join("otro.png");
        fs::write(&a, b"imagen a").unwrap();
        fs::write(&b, b"imagen b").unwrap();

        let copy_a = store_copy(&dir, &a).unwrap();
        assert_eq!(copy_a.file_name().unwrap(), "anuncio.png");
        assert_eq!(fs::read(&copy_a).unwrap(), b"imagen a");
        assert_eq!(store_copy(&dir, &a).unwrap(), copy_a, "el mismo archivo no se copia dos veces");
        // Copiar la copia (elegida desde la carpeta de la app) devuelve la misma.
        assert_eq!(store_copy(&dir, &copy_a).unwrap(), copy_a);
        let copy_b = store_copy(&dir, &b).unwrap();

        fs::remove_file(&a).unwrap();
        assert!(copy_a.is_file(), "la copia sigue aunque se borre el original");

        let kept = clean(&dir, &HashSet::from([copy_a.clone()]));
        assert_eq!(kept, 8);
        assert!(copy_a.is_file() && !copy_b.exists() && !copy_b.parent().unwrap().exists());
        let _ = fs::remove_dir_all(&base);
    }
}
