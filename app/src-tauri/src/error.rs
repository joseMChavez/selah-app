// Errores que devuelven los comandos a la interfaz (campo «code»; la interfaz los traduce en messages.ts).
use serde::Serialize;

use crate::infra::{libreoffice, obs};

// Los errores viajan como códigos para que la interfaz los muestre en el idioma elegido.
#[derive(Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum CommandError {
    MonitorDisconnected,
    FileMissing { path: String },
    InvalidContent,
    InvalidBible { file: String, reason: String },
    BibleNotFound,
    ObsNotInstalled,
    /// Hay que cerrar OBS para cambiar su configuración.
    ObsRunning,
    ObsNotConnected,
    Obs { message: String },
    /// Para mostrar presentaciones hace falta LibreOffice.
    OfficeNotInstalled,
    ConversionFailed { file: String, reason: String },
    /// Ya se está descargando LibreOffice (o borrando la copia).
    OfficeBusy,
    /// El usuario canceló la descarga.
    OfficeCanceled,
    OfficeDownload { message: String },
    /// No se pudo leer o escribir la base de datos local.
    Storage { message: String },
    Internal { message: String },
}

pub fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::Internal {
        message: e.to_string(),
    }
}

impl From<obs::ObsError> for CommandError {
    fn from(e: obs::ObsError) -> Self {
        match e {
            obs::ObsError::NotInstalled => CommandError::ObsNotInstalled,
            obs::ObsError::Running => CommandError::ObsRunning,
            obs::ObsError::Failed(message) => CommandError::Obs { message },
        }
    }
}

pub fn obs_failed(e: obws::error::Error) -> CommandError {
    CommandError::Obs { message: e.to_string() }
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
