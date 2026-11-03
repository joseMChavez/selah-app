// Ajustes guardados en la base local (infra/store.rs).

use tauri::{AppHandle, State};

use crate::error::CommandError;
use crate::state::{AppState, emit_settings, with_store};

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    with_store(&state, |store| store.settings())
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), CommandError> {
    with_store(&state, |store| store.set_setting(&key, &value))?;
    emit_settings(&app, &state).map(|_| ())
}

/// Vuelve al valor inicial esas claves; lista vacía = todos los ajustes.
#[tauri::command]
pub fn settings_reset(
    app: AppHandle,
    state: State<'_, AppState>,
    keys: Vec<String>,
) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    with_store(&state, |store| store.reset_settings(&keys))?;
    emit_settings(&app, &state)
}

/// Dónde está la base (para mostrarlo en Ajustes); None si se trabaja en memoria.
#[tauri::command]
pub fn store_path(state: State<'_, AppState>) -> Option<String> {
    let guard = state.store.lock().unwrap();
    guard.as_ref()?.path().map(|p| p.display().to_string())
}
