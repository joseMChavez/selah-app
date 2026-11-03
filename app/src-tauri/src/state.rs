// Estado compartido de la app y avisos a las ventanas.
use std::path::Path;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::domain::output::OutputState;
use crate::error::{internal, CommandError};
use crate::infra::{bible, store};

pub const MAIN_LABEL: &str = "main";
pub const OUTPUT_LABEL: &str = "output";
pub const OUTPUT_STATE_EVENT: &str = "output-state";
pub const OUTPUT_ERROR_EVENT: &str = "output-error";
/// Teclas de avance pulsadas en la ventana de salida: las aplica la consola, que sabe qué hay en pantalla.
pub const OUTPUT_NAVIGATE_EVENT: &str = "output-navigate";
/// Lleva todos los ajustes tras cada cambio: así la ventana de salida sigue el idioma de la consola.
pub const SETTINGS_EVENT: &str = "settings-changed";

#[derive(Default)]
pub struct AppState {
    pub output: Mutex<OutputState>,
    pub bibles: Mutex<bible::Library>,
    /// Conexión con OBS; None si no hay. Mutex de tokio: se mantiene durante las peticiones.
    pub obs: tauri::async_runtime::Mutex<Option<obws::Client>>,
    /// Se abre al arrancar (setup); None solo antes de eso.
    pub store: Mutex<Option<store::Store>>,
}

pub fn with_store<T>(
    state: &AppState,
    action: impl FnOnce(&mut store::Store) -> rusqlite::Result<T>,
) -> Result<T, CommandError> {
    let mut guard = state.store.lock().unwrap();
    let store = guard.as_mut().ok_or_else(|| CommandError::Storage {
        message: "la base de datos aún no está lista".into(),
    })?;
    action(store).map_err(|e| CommandError::Storage { message: e.to_string() })
}

pub fn emit_settings(app: &AppHandle, state: &AppState) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    let settings = with_store(state, |store| store.settings())?;
    let _ = app.emit(SETTINGS_EVENT, &settings);
    Ok(settings)
}

pub fn bibles_dir(app: &AppHandle) -> Result<bible::BibleDirs, CommandError> {
    Ok(bible::BibleDirs {
        // Si no hay carpeta de recursos (no debería pasar), se sigue solo con las importadas.
        builtin: app.path().resource_dir().ok().map(|dir| dir.join("bibles")),
        user: app.path().app_data_dir().map_err(internal)?.join("bibles"),
    })
}

pub fn publish(
    app: &AppHandle,
    state: &AppState,
    update: impl FnOnce(&mut OutputState),
) -> OutputState {
    let mut output = state.output.lock().unwrap();
    update(&mut output);
    let snapshot = output.clone();
    let _ = app.emit(OUTPUT_STATE_EVENT, &snapshot);
    snapshot
}

// El protocolo asset empieza sin acceso a nada; solo se abren los archivos que el usuario elige.
pub fn allow_media(app: &AppHandle, path: &Path) -> Result<(), CommandError> {
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(internal)
}
