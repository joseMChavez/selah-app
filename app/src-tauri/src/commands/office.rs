// LibreOffice para las presentaciones: estado y copia privada (infra/libreoffice.rs).

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{CommandError, internal};
use crate::infra::{libreoffice, office};

#[tauri::command]
pub fn office_available() -> bool {
    office::available()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeStatus {
    /// LibreOffice instalado en el sistema.
    system: bool,
    /// La copia privada de Selah está lista.
    private_copy: bool,
    /// Hay copia privada para este sistema.
    downloadable: bool,
    installing: bool,
    version: &'static str,
    /// Tamaño de la descarga en bytes.
    size: u64,
}

#[tauri::command]
pub fn office_status(app: AppHandle) -> Result<OfficeStatus, CommandError> {
    let data = app.path().app_data_dir().map_err(internal)?;
    Ok(OfficeStatus {
        system: office::find_soffice().is_some(),
        private_copy: libreoffice::soffice(&data).is_some(),
        downloadable: libreoffice::package().is_some(),
        installing: libreoffice::installing(),
        version: libreoffice::VERSION,
        size: libreoffice::package().map_or(0, |p| p.size),
    })
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeProgress {
    downloaded: u64,
    total: u64,
    extracting: bool,
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
    office::set_private_soffice(Some(soffice));
    Ok(())
}

#[tauri::command]
pub fn office_cancel() {
    libreoffice::cancel();
}

#[tauri::command]
pub fn office_remove(app: AppHandle) -> Result<(), CommandError> {
    let data = app.path().app_data_dir().map_err(internal)?;
    office::set_private_soffice(None);
    libreoffice::remove(&data)?;
    Ok(())
}
