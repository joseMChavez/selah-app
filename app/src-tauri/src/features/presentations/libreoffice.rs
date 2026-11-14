// Copia privada de LibreOffice para convertir presentaciones cuando el equipo no tiene con qué (o falla).
// Con permiso del usuario se descarga de The Document Foundation, se comprueba su SHA-256 y se guarda en la
// carpeta de datos de EmetPro, sin instalarla en el sistema: AppImage extraído en Linux, MSI extraído sin
// administrador en Windows y LibreOffice.app copiada del .dmg en macOS. La versión es fija (probada);
// las sumas son las que publica download.documentfoundation.org (y la del AppImage, calculada al fijarla).
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

pub const VERSION: &str = "26.2.6";

pub struct Package {
    /// Primero el servidor de réplicas; si la versión ya salió de «stable», el archivo histórico.
    urls: &'static [&'static str],
    sha256: &'static str,
    pub size: u64,
}

#[cfg(all(windows, target_arch = "x86_64"))]
static PACKAGE: Option<Package> = Some(Package {
    urls: &[
        "https://download.documentfoundation.org/libreoffice/stable/26.2.6/win/x86_64/LibreOffice_26.2.6_Win_x86-64.msi",
        "https://downloadarchive.documentfoundation.org/libreoffice/old/26.2.6.3/win/x86_64/LibreOffice_26.2.6.3_Win_x86-64.msi",
    ],
    sha256: "f9877032fd908beb9c0ddf06df4af5c2e85f419c42e14876c4cce5aae5fb2660",
    size: 373_252_096,
});

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
static PACKAGE: Option<Package> = Some(Package {
    urls: &[
        "https://download.documentfoundation.org/libreoffice/stable/26.2.6/mac/aarch64/LibreOffice_26.2.6_MacOS_aarch64.dmg",
        "https://downloadarchive.documentfoundation.org/libreoffice/old/26.2.6.3/mac/aarch64/LibreOffice_26.2.6.3_MacOS_aarch64.dmg",
    ],
    sha256: "94bb3248df074c225490a8a6d1d9dc87c7d6783dbb7a8e9f0d0c3d94348552af",
    size: 297_798_926,
});

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
static PACKAGE: Option<Package> = Some(Package {
    urls: &[
        "https://download.documentfoundation.org/libreoffice/stable/26.2.6/mac/x86_64/LibreOffice_26.2.6_MacOS_x86-64.dmg",
        "https://downloadarchive.documentfoundation.org/libreoffice/old/26.2.6.3/mac/x86_64/LibreOffice_26.2.6.3_MacOS_x86-64.dmg",
    ],
    sha256: "135b8a95b8133396d54bf8e726dbc0066145efa0d785963fc3d4592acbfcfe5b",
    size: 308_308_875,
});

// TDF no publica AppImage; el enlace de libreoffice.org lleva a los de LibreItalia.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
static PACKAGE: Option<Package> = Some(Package {
    urls: &["https://appimages.libreitalia.org/LibreOffice-26.2.6.3.basic-x86_64.AppImage"],
    sha256: "9f808e6313cbf9e0629ee8ce34c81fa8861591797aa2746cc811d46b87e702e0",
    size: 295_773_376,
});

#[cfg(not(any(
    all(windows, target_arch = "x86_64"),
    all(target_os = "macos", any(target_arch = "aarch64", target_arch = "x86_64")),
    all(target_os = "linux", target_arch = "x86_64"),
)))]
static PACKAGE: Option<Package> = None;

/// Lo que hay que descargar en este equipo; None si no hay paquete para este sistema.
pub fn package() -> Option<&'static Package> {
    PACKAGE.as_ref()
}

pub enum InstallError {
    Busy,
    Canceled,
    Failed(String),
}

static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

/// Una sola instalación a la vez; se libera aunque falle.
struct Turn;

impl Turn {
    fn take() -> Option<Self> {
        BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).ok()?;
        CANCEL.store(false, Ordering::SeqCst);
        Some(Self)
    }
}

impl Drop for Turn {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::SeqCst);
    }
}

pub fn installing() -> bool {
    BUSY.load(Ordering::SeqCst)
}

pub fn cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}

fn root(data: &Path) -> PathBuf {
    data.join("libreoffice")
}

/// Archivo con la ruta de soffice dentro de la copia: buscarlo cada vez sería recorrer miles de archivos.
fn marker(data: &Path) -> PathBuf {
    root(data).join("soffice.txt")
}

/// soffice de la copia privada, si está completa.
pub fn soffice(data: &Path) -> Option<PathBuf> {
    let relative = std::fs::read_to_string(marker(data)).ok()?;
    let path = root(data).join(relative.trim());
    path.is_file().then_some(path)
}

pub struct Progress {
    pub downloaded: u64,
    pub total: u64,
    /// Ya descargado y comprobado; ahora se extrae.
    pub extracting: bool,
}

/// Descarga, comprueba y extrae; devuelve la ruta de soffice.
pub async fn install(data: &Path, mut progress: impl FnMut(Progress) + Send) -> Result<PathBuf, InstallError> {
    let package = package().ok_or_else(|| InstallError::Failed("no hay LibreOffice para este sistema".into()))?;
    let _turn = Turn::take().ok_or(InstallError::Busy)?;
    let root = root(data);
    std::fs::create_dir_all(&root).map_err(failed)?;
    let download = root.join("descarga.part");

    let mut errors = Vec::new();
    let mut ok = false;
    for url in package.urls {
        match fetch(url, &download, package, &mut progress).await {
            Ok(()) => {
                ok = true;
                break;
            }
            Err(e) if CANCEL.load(Ordering::SeqCst) => {
                let _ = std::fs::remove_file(&download);
                return Err(e);
            }
            Err(InstallError::Failed(reason)) => {
                eprintln!("[libreoffice] {url}: {reason}");
                errors.push(reason);
            }
            Err(e) => return Err(e),
        }
    }
    if !ok {
        let _ = std::fs::remove_file(&download);
        return Err(InstallError::Failed(errors.join(" · ")));
    }

    progress(Progress { downloaded: package.size, total: package.size, extracting: true });
    let data = data.to_path_buf();
    tauri::async_runtime::spawn_blocking(move || unpack(&data, &download))
        .await
        .map_err(failed)?
        .map_err(InstallError::Failed)
}

async fn fetch(url: &str, file: &Path, package: &Package, progress: &mut impl FnMut(Progress)) -> Result<(), InstallError> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("EmetPro/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(failed)?;
    let mut response = client.get(url).send().await.and_then(|r| r.error_for_status()).map_err(failed)?;
    let mut out = std::fs::File::create(file).map_err(failed)?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut reported = Instant::now();
    while let Some(chunk) = response.chunk().await.map_err(failed)? {
        if CANCEL.load(Ordering::SeqCst) {
            return Err(InstallError::Canceled);
        }
        out.write_all(&chunk).map_err(failed)?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        if reported.elapsed() > Duration::from_millis(250) {
            reported = Instant::now();
            progress(Progress { downloaded, total: package.size, extracting: false });
        }
    }
    out.flush().map_err(failed)?;
    check(&format!("{:x}", hasher.finalize()), downloaded, package).map_err(InstallError::Failed)
}

/// La descarga tiene que ser exactamente el archivo fijado: si no, no se ejecuta nada de ella.
fn check(sha256: &str, size: u64, package: &Package) -> Result<(), String> {
    if size != package.size {
        return Err(format!("descarga incompleta ({size} de {} bytes)", package.size));
    }
    if sha256 != package.sha256 {
        return Err("la descarga no coincide con la suma SHA-256 publicada".into());
    }
    Ok(())
}

/// Extrae en una carpeta temporal y solo al final la deja en su sitio: una copia a medias nunca se usa.
fn unpack(data: &Path, download: &Path) -> Result<PathBuf, String> {
    let root = root(data);
    let staging = root.join(format!("{VERSION}.tmp"));
    let target = root.join(VERSION);
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let result = extract(download, &staging).and_then(|()| {
        find_soffice(&staging).ok_or_else(|| "no se encontró soffice en el paquete".to_string())
    });
    let _ = std::fs::remove_file(download);
    let found = match result {
        Ok(found) => found,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e);
        }
    };
    let relative = found.strip_prefix(&staging).map_err(|e| e.to_string())?.to_path_buf();
    let _ = std::fs::remove_file(marker(data));
    let _ = std::fs::remove_dir_all(&target);
    std::fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    let relative = Path::new(VERSION).join(relative);
    std::fs::write(marker(data), relative.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    // Versiones anteriores que hayan quedado.
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() && entry.file_name() != VERSION {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
    Ok(root.join(relative))
}

#[cfg(target_os = "linux")]
fn extract(download: &Path, staging: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(download, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    // El propio AppImage se extrae (en squashfs-root); así no hace falta FUSE para usarlo.
    let mut command = Command::new(download);
    command.arg("--appimage-extract").current_dir(staging);
    run(command, "el AppImage")
}

#[cfg(windows)]
fn extract(download: &Path, staging: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    // Instalación administrativa: solo copia los archivos, sin registro ni permisos de administrador.
    let mut command = Command::new("msiexec");
    command.arg("/a").arg(download).arg("/qn").raw_arg(format!("TARGETDIR=\"{}\"", staging.display()));
    run(command, "msiexec")
}

#[cfg(target_os = "macos")]
fn extract(download: &Path, staging: &Path) -> Result<(), String> {
    let mount = staging.join("montaje");
    std::fs::create_dir_all(&mount).map_err(|e| e.to_string())?;
    let mut attach = Command::new("hdiutil");
    attach.args(["attach", "-nobrowse", "-readonly", "-noautoopen", "-mountpoint"]).arg(&mount).arg(download);
    run(attach, "hdiutil")?;
    let mut copy = Command::new("ditto");
    copy.arg(mount.join("LibreOffice.app")).arg(staging.join("LibreOffice.app"));
    let copied = run(copy, "ditto");
    let mut detach = Command::new("hdiutil");
    detach.args(["detach", "-force"]).arg(&mount);
    let _ = run(detach, "hdiutil");
    let _ = std::fs::remove_dir(&mount);
    copied
}

#[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
fn extract(_download: &Path, _staging: &Path) -> Result<(), String> {
    Err("no hay LibreOffice para este sistema".into())
}

#[allow(dead_code)]
fn run(mut command: Command, who: &str) -> Result<(), String> {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    let output = command.output().map_err(|e| format!("{who}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first = stderr.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or_default();
    Err(format!("{who} terminó con error ({}) {first}", output.status).trim().to_string())
}

/// soffice dentro de la copia: en «program» (Linux y Windows) o en «Contents/MacOS» (macOS).
fn find_soffice(dir: &Path) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "soffice.exe" } else { "soffice" };
    let mut pending = vec![(dir.to_path_buf(), 0)];
    while let Some((dir, depth)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() && depth < 8 {
                pending.push((path, depth + 1));
            } else if kind.is_file() && entry.file_name() == exe {
                let parent = dir.file_name().and_then(|n| n.to_str());
                if matches!(parent, Some("program" | "MacOS")) {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Borra la copia privada (no toca un LibreOffice instalado en el sistema).
pub fn remove(data: &Path) -> Result<(), InstallError> {
    let _turn = Turn::take().ok_or(InstallError::Busy)?;
    match std::fs::remove_dir_all(root(data)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(failed(e)),
        _ => Ok(()),
    }
}

fn failed(e: impl std::fmt::Display) -> InstallError {
    InstallError::Failed(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emetpro-lo-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn encuentra_soffice_en_program() {
        let dir = temp("buscar");
        let exe = if cfg!(windows) { "soffice.exe" } else { "soffice" };
        // Un soffice fuera de «program» (como el enlace de usr/bin) no vale.
        std::fs::create_dir_all(dir.join("squashfs-root/usr/bin")).unwrap();
        std::fs::write(dir.join("squashfs-root/usr/bin").join(exe), "").unwrap();
        assert_eq!(find_soffice(&dir), None);
        let program = dir.join("squashfs-root/opt/libreoffice26.2/program");
        std::fs::create_dir_all(&program).unwrap();
        std::fs::write(program.join(exe), "").unwrap();
        assert_eq!(find_soffice(&dir), Some(program.join(exe)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn comprueba_tamano_y_suma() {
        let package = Package { urls: &[], sha256: "abc", size: 10 };
        assert!(check("abc", 10, &package).is_ok());
        assert!(check("abc", 9, &package).is_err());
        assert!(check("abd", 10, &package).is_err());
    }

    #[test]
    fn la_copia_se_lee_del_marcador() {
        let data = temp("marcador");
        assert_eq!(soffice(&data), None);
        let program = root(&data).join(VERSION).join("program");
        std::fs::create_dir_all(&program).unwrap();
        std::fs::write(program.join("soffice"), "").unwrap();
        std::fs::write(marker(&data), format!("{VERSION}/program/soffice\n")).unwrap();
        assert_eq!(soffice(&data), Some(program.join("soffice")));
        assert!(remove(&data).is_ok());
        assert_eq!(soffice(&data), None);
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn hay_paquete_para_los_sistemas_de_los_instaladores() {
        if cfg!(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64"))) {
            let package = package().expect("paquete");
            assert_eq!(package.sha256.len(), 64);
            assert!(package.urls.iter().all(|u| u.starts_with("https://")));
        }
    }

    /// Descarga de verdad (~300 MB) y extrae en una carpeta temporal: `cargo test -- --ignored instala_la_copia`.
    #[test]
    #[ignore]
    fn instala_la_copia() {
        let data = temp("instalar");
        let soffice = tauri::async_runtime::block_on(install(&data, |p| {
            eprintln!("{} / {} {}", p.downloaded, p.total, if p.extracting { "extrayendo" } else { "" });
        }))
        .unwrap_or_else(|e| panic!("{}", match e {
            InstallError::Failed(m) => m,
            InstallError::Busy => "ocupado".into(),
            InstallError::Canceled => "cancelado".into(),
        }));
        assert!(soffice.is_file());
        assert_eq!(super::soffice(&data), Some(soffice));
    }
}
