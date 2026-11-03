// Transmisión con OBS: detectar, configurar su WebSocket, escenas de Selah, texto y emisión.
// Plugin interno «obs»: la interfaz llama a sus comandos como `plugin:obs|<comando>` (platform/tauri.ts).
pub mod client;
mod commands;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Wry};

use crate::core::error::CommandError;

pub const NAME: &str = "obs";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::obs_detect,
            commands::obs_enable_websocket,
            commands::obs_launch,
            commands::obs_connect,
            commands::obs_disconnect,
            commands::obs_status,
            commands::obs_setup,
            commands::obs_set_scene,
            commands::obs_show_text,
            commands::obs_hide_text,
            commands::obs_cameras,
            commands::obs_set_camera,
            commands::obs_preview,
            commands::obs_set_destination,
            commands::obs_start_stream,
            commands::obs_stop_stream,
        ])
        .setup(|app, _api| {
            app.manage(ObsState::default());
            Ok(())
        })
        .build()
}

/// Conexión con OBS; None si no hay. Mutex de tokio: se mantiene durante las peticiones.
#[derive(Default)]
pub struct ObsState(pub tauri::async_runtime::Mutex<Option<obws::Client>>);

impl From<client::ObsError> for CommandError {
    fn from(e: client::ObsError) -> Self {
        match e {
            client::ObsError::NotInstalled => CommandError::ObsNotInstalled,
            client::ObsError::Running => CommandError::ObsRunning,
            client::ObsError::SafeMode => CommandError::ObsSafeMode,
            client::ObsError::Failed(message) => CommandError::Obs { message },
        }
    }
}

pub fn obs_failed(e: obws::error::Error) -> CommandError {
    CommandError::Obs { message: e.to_string() }
}
