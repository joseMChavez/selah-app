// Archivos que la ventana de salida puede cargar (protocolo asset).
use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::core::error::{internal, CommandError};

// El protocolo asset empieza sin acceso a nada; solo se abren los archivos que el usuario elige.
pub fn allow_media(app: &AppHandle, path: &Path) -> Result<(), CommandError> {
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(internal)
}
