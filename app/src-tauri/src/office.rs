// Presentaciones (PowerPoint, Impress, Keynote) → PDF; luego se muestran como cualquier PDF.
// Un webview no sabe dibujar un PPTX, así que se convierte con lo que haya en el equipo, en este orden:
// LibreOffice (el del sistema o la copia privada de Selah, libreoffice.rs), que es el que mejor funciona;
// luego PowerPoint (COM) en Windows, y Keynote y PowerPoint en macOS. Si uno falla se prueba el siguiente. La conversión se guarda en caché y se rehace si el archivo cambia.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};

pub const PRESENTATION_EXTENSIONS: &[&str] = &["pptx", "ppt", "ppsx", "pps", "odp"];

/// Una presentación grande con muchas imágenes puede tardar; más que esto es que el programa se colgó.
const TIMEOUT: Duration = Duration::from_secs(180);

/// Las conversiones se hacen de una en una: LibreOffice no admite dos a la vez con el mismo perfil,
/// y PowerPoint y Keynote son una sola aplicación compartida.
static CONVERTING: Mutex<()> = Mutex::new(());

/// soffice de la copia privada (libreoffice.rs), si la hay. Se fija al arrancar y al instalarla o borrarla.
static PRIVATE_SOFFICE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_private_soffice(path: Option<PathBuf>) {
    *PRIVATE_SOFFICE.lock().unwrap_or_else(|e| e.into_inner()) = path;
}

const POWERPOINT_MAC: &str = "/Applications/Microsoft PowerPoint.app";
const KEYNOTE: &str = "/Applications/Keynote.app";

/// PowerPoint por COM, sin ventana. Las rutas llegan por variables de entorno, no dentro del script.
/// Si PowerPoint no estaba abierto con otras presentaciones, se cierra al terminar.
const POWERPOINT_WINDOWS_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$app = New-Object -ComObject PowerPoint.Application
try {
  # Open(archivo, solo lectura, sin título, sin ventana)
  $pres = $app.Presentations.Open($env:SELAH_ORIGEN, -1, 0, 0)
  try { $pres.SaveAs($env:SELAH_DESTINO, 32) } finally { $pres.Close() }
} finally {
  if ($app.Presentations.Count -eq 0) { $app.Quit() }
  [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($app)
}
"#;

/// AppleScript: las rutas llegan como argumentos (argv). Keynote y PowerPoint están en sandbox: el archivo
/// se resuelve como alias fuera del «tell», así el sistema les da permiso para abrirlo; con la ruta convertida
/// dentro del «tell», Keynote respondía que no podía abrir el archivo.
const POWERPOINT_MAC_SCRIPT: &[&str] = &[
    "on run argv",
    "set origen to (POSIX file (item 1 of argv)) as alias",
    "set destino to (POSIX file (item 2 of argv)) as string",
    "tell application \"Microsoft PowerPoint\"",
    "open origen",
    "set pres to active presentation",
    "save pres in destino as save as PDF",
    "close pres saving no",
    "end tell",
    "end run",
];

const KEYNOTE_SCRIPT: &[&str] = &[
    "on run argv",
    "set origen to (POSIX file (item 1 of argv)) as alias",
    "set destino to POSIX file (item 2 of argv)",
    "tell application \"Keynote\"",
    "set doc to open origen",
    "export doc to destino as PDF",
    "close doc saving no",
    "end tell",
    "end run",
];

pub enum OfficeError {
    /// No hay ningún programa capaz de convertir.
    NotInstalled,
    Failed(String),
}

/// Un programa capaz de convertir presentaciones a PDF.
enum Converter {
    PowerPointWindows,
    PowerPointMac,
    Keynote,
    LibreOffice(PathBuf),
}

impl Converter {
    fn name(&self) -> &'static str {
        match self {
            Self::PowerPointWindows | Self::PowerPointMac => "PowerPoint",
            Self::Keynote => "Keynote",
            Self::LibreOffice(_) => "LibreOffice",
        }
    }
}

/// Los convertidores de este equipo, en orden de preferencia: LibreOffice primero; en Mac, luego Keynote.
fn converters() -> Vec<Converter> {
    let mut list = Vec::new();
    let private = || PRIVATE_SOFFICE.lock().unwrap_or_else(|e| e.into_inner()).clone().filter(|p| p.is_file());
    if let Some(soffice) = find_soffice().or_else(private) {
        list.push(Converter::LibreOffice(soffice));
    }
    if cfg!(windows) && powerpoint_windows_installed() {
        list.push(Converter::PowerPointWindows);
    }
    if cfg!(target_os = "macos") {
        if Path::new(KEYNOTE).exists() {
            list.push(Converter::Keynote);
        }
        if Path::new(POWERPOINT_MAC).exists() {
            list.push(Converter::PowerPointMac);
        }
    }
    list
}

/// Hay algo con qué convertir presentaciones.
pub fn available() -> bool {
    !converters().is_empty()
}

/// PowerPoint registra su clase COM al instalarse.
fn powerpoint_windows_installed() -> bool {
    let mut command = Command::new("reg");
    command.args(["query", r"HKCR\PowerPoint.Application"]).stdout(Stdio::null()).stderr(Stdio::null());
    hide_window(&mut command);
    command.status().is_ok_and(|s| s.success())
}

/// LibreOffice instalado en el sistema: primero el del PATH, luego las rutas de instalación habituales.
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

/// En Windows, sin parpadeo de una consola negra.
fn hide_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}

/// Ejecuta con límite de tiempo. Err con la primera línea que el programa escribió como error.
fn run(mut command: Command, who: &str) -> Result<(), String> {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    hide_window(&mut command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if started.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("{who} tardó demasiado"));
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    if status.success() {
        return Ok(());
    }
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        use std::io::Read;
        let _ = pipe.read_to_string(&mut stderr);
    }
    let first = stderr.lines().map(str::trim).find(|l| !l.is_empty()).map(str::to_string);
    Err(first.unwrap_or_else(|| format!("{who} terminó con error ({status})")))
}

fn applescript(lines: &[&str], source: &Path, pdf: &Path) -> Command {
    let mut command = Command::new("osascript");
    for line in lines {
        command.arg("-e").arg(line);
    }
    command.arg(source).arg(pdf);
    command
}

fn command_for(converter: &Converter, source: &Path, pdf: &Path, dir: &Path, cache: &Path) -> Command {
    match converter {
        Converter::PowerPointWindows => {
            let mut command = Command::new("powershell");
            command
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", POWERPOINT_WINDOWS_SCRIPT])
                .env("SELAH_ORIGEN", source)
                .env("SELAH_DESTINO", pdf);
            command
        }
        Converter::PowerPointMac => applescript(POWERPOINT_MAC_SCRIPT, source, pdf),
        Converter::Keynote => applescript(KEYNOTE_SCRIPT, source, pdf),
        Converter::LibreOffice(soffice) => {
            let mut command = Command::new(soffice);
            command
                // Perfil propio: si el usuario tiene LibreOffice abierto, la conversión no choca con él.
                .arg(format!("-env:UserInstallation={}", file_url(&cache.join("perfil-libreoffice"))))
                .args(["--headless", "--norestore", "--nolockcheck", "--convert-to", "pdf", "--outdir"])
                .arg(dir)
                .arg(source);
            command
        }
    }
}

/// Convierte la presentación y devuelve la ruta del PDF (o la de la caché si ya existía).
pub fn convert(source: &Path, cache: &Path) -> Result<PathBuf, OfficeError> {
    let dir = cache_dir(source, cache).map_err(OfficeError::Failed)?;
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "presentacion".into());
    let pdf = dir.join(format!("{stem}.pdf"));
    if pdf.is_file() {
        return Ok(pdf);
    }
    let converters = converters();
    if converters.is_empty() {
        return Err(OfficeError::NotInstalled);
    }
    let _turn = CONVERTING.lock().unwrap_or_else(|e| e.into_inner());
    std::fs::create_dir_all(&dir).map_err(|e| OfficeError::Failed(e.to_string()))?;

    let mut failures = Vec::new();
    for converter in &converters {
        let result = run(command_for(converter, source, &pdf, &dir, cache), converter.name());
        if pdf.is_file() {
            return Ok(pdf);
        }
        let reason = result.err().unwrap_or_else(|| "no creó el PDF".into());
        eprintln!("[presentación] {} no pudo convertir: {reason}", converter.name());
        failures.push(format!("{}: {reason}", converter.name()));
    }
    Err(OfficeError::Failed(failures.join(" · ")))
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

    /// Las rutas nunca van dentro del script: un nombre con comillas no puede romperlo ni inyectar órdenes.
    #[test]
    fn rutas_fuera_de_los_scripts() {
        let source = Path::new("/tmp/Sermón \"especial\"'; rm -rf ~.pptx");
        let pdf = Path::new("/tmp/salida.pdf");
        let dir = Path::new("/tmp");

        let windows = command_for(&Converter::PowerPointWindows, source, pdf, dir, dir);
        let envs: Vec<_> = windows.get_envs().collect();
        assert!(envs.contains(&(std::ffi::OsStr::new("SELAH_ORIGEN"), Some(source.as_os_str()))));
        assert!(envs.contains(&(std::ffi::OsStr::new("SELAH_DESTINO"), Some(pdf.as_os_str()))));
        assert!(windows.get_args().all(|a| a != source.as_os_str()));

        for converter in [Converter::PowerPointMac, Converter::Keynote] {
            let mac = command_for(&converter, source, pdf, dir, dir);
            let args: Vec<_> = mac.get_args().collect();
            // Las dos últimas son las rutas (argv del AppleScript); el script no las contiene.
            assert_eq!(args[args.len() - 2..], [source.as_os_str(), pdf.as_os_str()]);
            assert!(args[..args.len() - 2].iter().all(|a| !a.to_string_lossy().contains("Sermón")));
        }
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
