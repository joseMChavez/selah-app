// Presentaciones (PowerPoint, Impress) → PDF con LibreOffice sin interfaz; luego se muestran como cualquier PDF.
// Un webview no sabe dibujar un PPTX, y LibreOffice es gratuito y está en Linux, Windows y macOS.
// La conversión se guarda en caché: la segunda vez es inmediata, y se rehace si el archivo cambia.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};

pub const PRESENTATION_EXTENSIONS: &[&str] = &["pptx", "ppt", "ppsx", "pps", "odp"];

/// Una presentación grande con muchas imágenes puede tardar; más que esto es que LibreOffice se colgó.
const TIMEOUT: Duration = Duration::from_secs(180);

/// LibreOffice no admite dos conversiones a la vez con el mismo perfil: se hacen de una en una.
static CONVERTING: Mutex<()> = Mutex::new(());

pub enum OfficeError {
    NotInstalled,
    Failed(String),
}

/// Ejecutable de LibreOffice: primero el del PATH, luego las rutas de instalación habituales.
pub fn find_soffice() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "soffice.exe" } else { "soffice" };
    if let Some(found) = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|dir| dir.join(exe))
        .find(|p| p.is_file())
    {
        return Some(found);
    }
    let mut known: Vec<PathBuf> = Vec::new();
    if cfg!(target_os = "macos") {
        known.push("/Applications/LibreOffice.app/Contents/MacOS/soffice".into());
    } else if cfg!(windows) {
        for var in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(dir) = std::env::var_os(var) {
                known.push(PathBuf::from(dir).join("LibreOffice").join("program").join("soffice.exe"));
            }
        }
    } else {
        known.push("/usr/lib/libreoffice/program/soffice".into());
        known.push("/opt/libreoffice/program/soffice".into());
        known.push("/snap/bin/libreoffice".into());
        known.push("/var/lib/flatpak/exports/bin/org.libreoffice.LibreOffice".into());
        if let Some(home) = std::env::var_os("HOME") {
            known.push(PathBuf::from(home).join(".local/share/flatpak/exports/bin/org.libreoffice.LibreOffice"));
        }
    }
    known.into_iter().find(|p| p.is_file())
}

/// Ruta como URL file:// para -env:UserInstallation (LibreOffice no acepta rutas sueltas ahí).
fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let mut url = String::from(if text.starts_with('/') { "file://" } else { "file:///" });
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => url.push(byte as char),
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

/// Carpeta de caché de esa versión del archivo (ruta, tamaño y fecha de modificación).
fn cache_dir(source: &Path, cache: &Path) -> Result<PathBuf, String> {
    let meta = std::fs::metadata(source).map_err(|e| e.to_string())?;
    let modified = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs());
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    meta.len().hash(&mut hasher);
    modified.hash(&mut hasher);
    Ok(cache.join(format!("{:016x}", hasher.finish())))
}

/// Convierte la presentación y devuelve la ruta del PDF (o la de la caché si ya existía).
pub fn convert(source: &Path, cache: &Path) -> Result<PathBuf, OfficeError> {
    let dir = cache_dir(source, cache).map_err(OfficeError::Failed)?;
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "presentacion".into());
    let pdf = dir.join(format!("{stem}.pdf"));
    if pdf.is_file() {
        return Ok(pdf);
    }
    let soffice = find_soffice().ok_or(OfficeError::NotInstalled)?;
    let _turn = CONVERTING.lock().unwrap_or_else(|e| e.into_inner());
    std::fs::create_dir_all(&dir).map_err(|e| OfficeError::Failed(e.to_string()))?;

    let mut command = Command::new(&soffice);
    command
        // Perfil propio: si el usuario tiene LibreOffice abierto, la conversión no choca con él.
        .arg(format!("-env:UserInstallation={}", file_url(&cache.join("perfil-libreoffice"))))
        .args(["--headless", "--norestore", "--nolockcheck", "--convert-to", "pdf", "--outdir"])
        .arg(&dir)
        .arg(source)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|e| OfficeError::Failed(e.to_string()))?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| OfficeError::Failed(e.to_string()))? {
            break status;
        }
        if started.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(OfficeError::Failed("LibreOffice tardó demasiado".into()));
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    if pdf.is_file() {
        return Ok(pdf);
    }
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        use std::io::Read;
        let _ = pipe.read_to_string(&mut stderr);
    }
    let reason = stderr.lines().find(|l| !l.trim().is_empty()).map(str::to_string);
    Err(OfficeError::Failed(reason.unwrap_or_else(|| format!("LibreOffice terminó sin crear el PDF ({status})"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_de_perfil() {
        assert_eq!(file_url(Path::new("/home/ana/.cache/selah/perfil")), "file:///home/ana/.cache/selah/perfil");
        assert_eq!(file_url(Path::new("/home/ana maría/x")), "file:///home/ana%20mar%C3%ADa/x");
        assert_eq!(file_url(Path::new(r"C:\Users\Ana Pérez\x")), "file:///C:/Users/Ana%20P%C3%A9rez/x");
    }

    /// Páginas de un PDF: objetos «/Type /Page» (sin contar el «/Type /Pages» raíz).
    fn count_pages(pdf: &[u8]) -> usize {
        (0..pdf.len())
            .filter(|&i| pdf[i..].starts_with(b"/Type"))
            .filter(|&i| {
                let rest = &pdf[i + 5..];
                let rest = &rest[rest.iter().take_while(|b| b.is_ascii_whitespace()).count()..];
                rest.starts_with(b"/Page") && rest.get(5) != Some(&b's')
            })
            .count()
    }

    /// Necesita LibreOffice instalado: `cargo test -- --ignored`.
    /// Crea un PPTX de dos diapositivas (a partir de una presentación en XML plano) y lo convierte.
    #[test]
    #[ignore]
    fn convierte_un_pptx() {
        let dir = std::env::temp_dir().join(format!("selah-office-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = |n: &str| format!(
            r#"<draw:page draw:name="{n}"><draw:frame svg:x="2cm" svg:y="2cm" svg:width="20cm" svg:height="4cm"><draw:text-box><text:p>Diapositiva {n}</text:p></draw:text-box></draw:frame></draw:page>"#
        );
        let fodp = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.presentation"><office:body><office:presentation>{}{}</office:presentation></office:body></office:document>"#,
            page("1"),
            page("2")
        );
        std::fs::write(dir.join("Sermón de prueba.fodp"), fodp).unwrap();
        let soffice = find_soffice().expect("LibreOffice instalado");
        let made = Command::new(soffice)
            .arg(format!("-env:UserInstallation={}", file_url(&dir.join("perfil"))))
            .args(["--headless", "--convert-to", "pptx", "--outdir"])
            .arg(&dir)
            .arg(dir.join("Sermón de prueba.fodp"))
            .status()
            .unwrap();
        let pptx = dir.join("Sermón de prueba.pptx");
        assert!(made.success() && pptx.is_file(), "no se pudo crear el PPTX de prueba");

        let pdf = convert(&pptx, &dir.join("cache")).ok().expect("convertir el PPTX");
        assert_eq!(pdf.file_name().unwrap(), "Sermón de prueba.pdf");
        let bytes = std::fs::read(&pdf).unwrap();
        let pages = count_pages(&bytes);
        assert!(pages >= 2, "el PDF debería tener las dos diapositivas ({pages})");
        // La segunda vez sale de la caché, sin llamar a LibreOffice.
        let started = Instant::now();
        assert_eq!(convert(&pptx, &dir.join("cache")).ok().expect("desde caché"), pdf);
        assert!(started.elapsed() < Duration::from_millis(100));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
