// Errores que devuelven los comandos a la interfaz (campo «code»; la interfaz los traduce en messages.ts).
// Cada funcionalidad convierte sus propios errores con un `impl From<…> for CommandError` en su módulo.
use serde::Serialize;

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
    /// El archivo de letras no se pudo leer (formato desconocido, dañado…).
    ImportFailed { file: String, reason: String },
    ExportFailed { reason: String },
    /// No se pudo leer o escribir la base de datos local.
    Storage { message: String },
    Internal { message: String },
}

pub fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::Internal {
        message: e.to_string(),
    }
}
