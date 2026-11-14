use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{convert, libreoffice};
use crate::core::assets::allow_media;
use crate::core::error::{internal, CommandError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeStatus {
    /// LibreOffice instalado en el sistema.
    system: bool,
    /// La copia privada de EmetPro está lista.
    private_copy: bool,
    /// Hay copia privada para este sistema.
    downloadable: bool,
    installing: bool,
    version: &'static str,
    /// Tamaño de la descarga en bytes.
    size: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeProgress {
    downloaded: u64,
    total: u64,
    extracting: bool,
}

/// Convierte una presentación a PDF (o la toma de la caché) y devuelve la ruta del PDF, lista para mostrarse.
#[tauri::command]
pub async fn convert_presentation(app: AppHandle, path: String) -> Result<String, CommandError> {
    let source = PathBuf::from(&path);
    if !source.is_file() {
        return Err(CommandError::FileMissing { path });
    }
    let cache = app.path().app_cache_dir().map_err(internal)?.join("presentaciones");
    let file = source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    // LibreOffice tarda unos segundos: fuera del hilo de los comandos.
    let pdf = tauri::async_runtime::spawn_blocking(move || convert::convert(&source, &cache))
        .await
        .map_err(internal)?
        .map_err(|e| match e {
            convert::OfficeError::NotInstalled => CommandError::OfficeNotInstalled,
            convert::OfficeError::Failed(reason) => {
                eprintln!("[presentación] {file}: {reason}");
                CommandError::ConversionFailed { file, reason }
            }
        })?;
    allow_media(&app, &pdf)?;
    Ok(pdf.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn office_available() -> bool {
    convert::available()
}

#[tauri::command]
pub fn office_status(app: AppHandle) -> Result<OfficeStatus, CommandError> {
    let data = app.path().app_data_dir().map_err(internal)?;
    Ok(OfficeStatus {
        system: convert::find_soffice().is_some(),
        private_copy: libreoffice::soffice(&data).is_some(),
        downloadable: libreoffice::package().is_some(),
        installing: libreoffice::installing(),
        version: libreoffice::VERSION,
        size: libreoffice::package().map_or(0, |p| p.size),
    })
}

/// Descarga la copia privada de LibreOffice (solo cuando el usuario lo pide). Avisa del avance con «office-progress».
#[tauri::command]
pub async fn office_install(app: AppHandle) -> Result<(), CommandError> {
    let data = app.path().app_data_dir().map_err(internal)?;
    let emitter = app.clone();
    let soffice = libreoffice::install(&data, move |p| {
        let _ = emitter.emit(
            "office-progress",
            OfficeProgress { downloaded: p.downloaded, total: p.total, extracting: p.extracting },
        );
    })
    .await?;
    eprintln!("[libreoffice] copia lista: {}", soffice.display());
    convert::set_private_soffice(Some(soffice));
    Ok(())
}

#[tauri::command]
pub fn office_cancel() {
    libreoffice::cancel();
}

#[tauri::command]
pub fn office_remove(app: AppHandle) -> Result<(), CommandError> {
    let data = app.path().app_data_dir().map_err(internal)?;
    convert::set_private_soffice(None);
    libreoffice::remove(&data)?;
    Ok(())
}
