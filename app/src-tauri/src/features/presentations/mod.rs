// Presentaciones (PowerPoint, Impress, Keynote) → PDF, y la copia privada de LibreOffice.
// Plugin interno «presentations»: la interfaz llama a sus comandos como `plugin:presentations|<comando>` (platform/tauri.ts).
mod commands;
pub mod convert;
pub mod libreoffice;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Wry};

use crate::core::error::CommandError;

pub const NAME: &str = "presentations";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::convert_presentation,
            commands::office_available,
            commands::office_status,
            commands::office_install,
            commands::office_cancel,
            commands::office_remove,
        ])
        .setup(|app, _api| {
            if let Ok(dir) = app.path().app_data_dir() {
                convert::set_private_soffice(libreoffice::soffice(&dir));
            }
            Ok(())
        })
        .build()
}

impl From<libreoffice::InstallError> for CommandError {
    fn from(e: libreoffice::InstallError) -> Self {
        match e {
            libreoffice::InstallError::Busy => CommandError::OfficeBusy,
            libreoffice::InstallError::Canceled => CommandError::OfficeCanceled,
            libreoffice::InstallError::Failed(message) => CommandError::OfficeDownload { message },
        }
    }
}
