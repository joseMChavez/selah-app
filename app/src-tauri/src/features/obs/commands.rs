use tauri::State;

use super::{client, obs_failed, ObsState};
use crate::core::error::CommandError;

#[tauri::command]
pub async fn obs_detect() -> client::Detection {
    client::detect()
}

#[tauri::command]
pub async fn obs_enable_websocket() -> Result<(), CommandError> {
    Ok(client::enable_websocket()?)
}

#[tauri::command]
pub async fn obs_launch() -> Result<(), CommandError> {
    Ok(client::launch()?)
}

#[tauri::command]
pub async fn obs_connect(state: State<'_, ObsState>) -> Result<(), CommandError> {
    let client = client::connect().await?;
    // Antes de que la interfaz pregunte por las escenas: si eran las de Selah, ya tienen el nombre nuevo.
    if let Err(e) = client::rename_legacy(&client).await {
        eprintln!("[obs] no se pudieron renombrar las escenas de Selah: {e}");
    }
    *state.0.lock().await = Some(client);
    Ok(())
}

#[tauri::command]
pub async fn obs_disconnect(state: State<'_, ObsState>) -> Result<(), CommandError> {
    if let Some(mut client) = state.0.lock().await.take() {
        client.disconnect().await;
    }
    Ok(())
}

/// Se consulta cada segundo. Si OBS no responde, se da la conexión por perdida.
#[tauri::command]
pub async fn obs_status(state: State<'_, ObsState>) -> Result<client::Status, CommandError> {
    let mut guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    match client::status(client).await {
        Ok(status) => Ok(status),
        Err(_) => {
            *guard = None;
            Err(CommandError::ObsNotConnected)
        }
    }
}

#[tauri::command]
pub async fn obs_setup(state: State<'_, ObsState>, wait_text: String) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::setup(client, &wait_text).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_scene(state: State<'_, ObsState>, scene: String) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::set_scene(client, &scene).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_show_text(
    state: State<'_, ObsState>,
    source: String,
    text: String,
    credit: String,
) -> Result<client::TextShown, CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::show_text(client, &source, &text, &credit).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_hide_text(state: State<'_, ObsState>, source: String) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::hide_text(client, &source).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_cameras(state: State<'_, ObsState>) -> Result<Vec<client::Camera>, CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::cameras(client).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_camera(state: State<'_, ObsState>, device: String) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::set_camera(client, &device).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_preview(state: State<'_, ObsState>) -> Result<Option<String>, CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::preview(client).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_destination(state: State<'_, ObsState>, server: String, key: String) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client::set_destination(client, &server, &key).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_start_stream(state: State<'_, ObsState>) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client.streaming().start().await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_stop_stream(state: State<'_, ObsState>) -> Result<(), CommandError> {
    let guard = state.0.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client.streaming().stop().await.map_err(obs_failed)
}
