use std::sync::atomic::Ordering;

use qrcode::render::svg;
use qrcode::QrCode;
use serde::Serialize;
use tauri::{AppHandle, State};
use tokio::sync::watch;

use super::domain::url;
use super::server::{self, Shared};
use super::{save_key, RemoteState, Running};
use crate::core::error::CommandError;

/// Lo que la consola muestra en Ajustes.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    running: bool,
    /// Dirección completa, con la clave (la del QR).
    url: Option<String>,
    /// El QR de `url`, como SVG.
    qr: Option<String>,
    /// false si no se encontró red: el teléfono no podrá llegar.
    on_network: bool,
    clients: usize,
}

fn info(state: &RemoteState) -> RemoteInfo {
    let clients = state.clients.load(Ordering::SeqCst);
    let Some(port) = state.running.lock().unwrap().as_ref().map(|r| r.port) else {
        return RemoteInfo { running: false, url: None, qr: None, on_network: false, clients };
    };
    let ip = server::lan_ip();
    let address = url(&ip.to_string(), port, &state.key.borrow());
    let qr = QrCode::new(address.as_bytes())
        .ok()
        .map(|code| code.render::<svg::Color>().min_dimensions(220, 220).quiet_zone(true).build());
    RemoteInfo { running: true, url: Some(address), qr, on_network: !ip.is_loopback(), clients }
}

#[tauri::command]
pub fn remote_info(state: State<'_, RemoteState>) -> RemoteInfo {
    info(&state)
}

#[tauri::command]
pub async fn remote_start(app: AppHandle, state: State<'_, RemoteState>) -> Result<RemoteInfo, CommandError> {
    if state.running.lock().unwrap().is_some() {
        return Ok(info(&state));
    }
    let (stop, stop_rx) = watch::channel(false);
    let shared = Shared {
        app,
        key: state.key.subscribe(),
        status: state.status.subscribe(),
        stop: stop_rx,
        clients: state.clients.clone(),
    };
    let port = server::start(shared).await.map_err(|e| CommandError::RemoteFailed { reason: e.to_string() })?;
    let mut running = state.running.lock().unwrap();
    if running.is_none() {
        *running = Some(Running { port, _stop: stop });
    }
    drop(running);
    Ok(info(&state))
}

#[tauri::command]
pub fn remote_stop(state: State<'_, RemoteState>) -> RemoteInfo {
    // Soltar `stop` apaga el servidor y desconecta los teléfonos.
    state.running.lock().unwrap().take();
    info(&state)
}

/// Clave nueva: los teléfonos conectados se desconectan y hay que volver a escanear el QR.
#[tauri::command]
pub fn remote_reset_key(app: AppHandle, state: State<'_, RemoteState>) -> RemoteInfo {
    state.key.send_replace(save_key(&app));
    info(&state)
}

/// La consola publica lo que hay en pantalla; se reenvía tal cual a los teléfonos.
#[tauri::command]
pub fn remote_publish(state: State<'_, RemoteState>, status: serde_json::Value) {
    state.status.send_replace(serde_json::json!({ "type": "status", "status": status }).to_string());
}
