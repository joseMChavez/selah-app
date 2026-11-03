// Ajustes guardados en la base local; cada cambio se avisa a todas las ventanas.
// Plugin interno «settings»: la interfaz llama a sus comandos como `plugin:settings|<comando>` (platform/tauri.ts).
mod commands;
mod repo;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, Emitter, Wry};

use crate::core::db::{with_store, Db};
use crate::core::error::CommandError;
use crate::core::events::SETTINGS_EVENT;

pub const NAME: &str = "settings";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_set,
            commands::settings_reset,
            commands::store_path,
        ])
        .build()
}

pub fn emit_settings(app: &AppHandle, state: &Db) -> Result<serde_json::Map<String, serde_json::Value>, CommandError> {
    let settings = with_store(state, |store| store.settings())?;
    let _ = app.emit(SETTINGS_EVENT, &settings);
    Ok(settings)
}
