use tauri::{AppHandle, State};

use super::emit_settings;
use crate::core::db::{with_store, Db};
use crate::core::error::CommandError;

#[tauri::command]
pub fn settings_get(state: State<'_, Db>) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    with_store(&state, |store| store.settings())
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<'_, Db>,
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
    state: State<'_, Db>,
    keys: Vec<String>,
) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    with_store(&state, |store| store.reset_settings(&keys))?;
    emit_settings(&app, &state)
}

/// Dónde está la base (para mostrarlo en Ajustes); None si se trabaja en memoria.
#[tauri::command]
pub fn store_path(state: State<'_, Db>) -> Option<String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref()?.path().map(|p| p.display().to_string())
}
