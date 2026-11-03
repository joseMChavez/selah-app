// Transmisión con OBS (infra/obs.rs).

use tauri::State;

use crate::error::{CommandError, obs_failed};
use crate::infra::obs;
use crate::state::AppState;

#[tauri::command]
pub async fn obs_detect() -> obs::Detection {
    obs::detect()
}

#[tauri::command]
pub async fn obs_enable_websocket() -> Result<(), CommandError> {
    Ok(obs::enable_websocket()?)
}

#[tauri::command]
pub async fn obs_launch() -> Result<(), CommandError> {
    Ok(obs::launch()?)
}

#[tauri::command]
pub async fn obs_connect(state: State<'_, AppState>) -> Result<(), CommandError> {
    let client = obs::connect().await?;
    *state.obs.lock().await = Some(client);
    Ok(())
}

#[tauri::command]
pub async fn obs_disconnect(state: State<'_, AppState>) -> Result<(), CommandError> {
    if let Some(mut client) = state.obs.lock().await.take() {
        client.disconnect().await;
    }
    Ok(())
}

/// Se consulta cada segundo. Si OBS no responde, se da la conexión por perdida.
#[tauri::command]
pub async fn obs_status(state: State<'_, AppState>) -> Result<obs::Status, CommandError> {
    let mut guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    match obs::status(client).await {
        Ok(status) => Ok(status),
        Err(_) => {
            *guard = None;
            Err(CommandError::ObsNotConnected)
        }
    }
}

#[tauri::command]
pub async fn obs_setup(state: State<'_, AppState>, wait_text: String) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::setup(client, &wait_text).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_scene(state: State<'_, AppState>, scene: String) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::set_scene(client, &scene).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_show_text(
    state: State<'_, AppState>,
    source: String,
    text: String,
    credit: String,
) -> Result<obs::TextShown, CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::show_text(client, &source, &text, &credit).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_hide_text(state: State<'_, AppState>, source: String) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::hide_text(client, &source).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_cameras(state: State<'_, AppState>) -> Result<Vec<obs::Camera>, CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::cameras(client).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_camera(state: State<'_, AppState>, device: String) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::set_camera(client, &device).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_preview(state: State<'_, AppState>) -> Result<Option<String>, CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::preview(client).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_set_destination(state: State<'_, AppState>, server: String, key: String) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    obs::set_destination(client, &server, &key).await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_start_stream(state: State<'_, AppState>) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client.streaming().start().await.map_err(obs_failed)
}

#[tauri::command]
pub async fn obs_stop_stream(state: State<'_, AppState>) -> Result<(), CommandError> {
    let guard = state.obs.lock().await;
    let client = guard.as_ref().ok_or(CommandError::ObsNotConnected)?;
    client.streaming().stop().await.map_err(obs_failed)
}
