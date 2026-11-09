// Control remoto desde el teléfono: pasar versículos y diapositivas, pantalla en negro, buscar en la Biblia.
// Plugin interno «remote»: la interfaz llama a sus comandos como `plugin:remote|<comando>` (platform/tauri.ts).
//
// La consola sigue mandando: el servidor (server.rs) solo pasa a la consola las órdenes del teléfono
// (evento «remote-command») y reparte a los teléfonos lo que la consola publica (remote_publish).
// La interfaz del teléfono es la ruta /remote de la misma app (app/src/lib/features/remote/).
mod commands;
pub mod domain;
mod server;
mod thumbs;

use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, Manager, Wry};
use tokio::sync::watch;

use self::domain::{is_key, new_key};
use self::thumbs::Thumbs;

pub const NAME: &str = "remote";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::remote_info,
            commands::remote_start,
            commands::remote_stop,
            commands::remote_reset_key,
            commands::remote_publish,
            commands::remote_thumbs_start,
            commands::remote_thumb,
        ])
        .setup(|app, _api| {
            let key = load_key(app);
            app.manage(RemoteState {
                key: watch::channel(key).0,
                status: watch::channel("null".to_string()).0,
                running: Mutex::new(None),
                clients: Arc::new(AtomicUsize::new(0)),
                thumbs: Arc::new(Mutex::new(Thumbs::default())),
                thumbs_added: watch::channel(0).0,
            });
            Ok(())
        })
        .build()
}

pub struct RemoteState {
    key: watch::Sender<String>,
    status: watch::Sender<String>,
    running: Mutex<Option<Running>>,
    clients: Arc<AtomicUsize>,
    /// Miniaturas del PDF en pantalla (thumbs.rs).
    thumbs: Arc<Mutex<Thumbs>>,
    /// Cambia cada vez que llega una miniatura: /api/thumb espera a la que pidió el teléfono.
    thumbs_added: watch::Sender<u64>,
}

/// Un servidor encendido. Al soltar `stop`, se apaga y se cierran sus conexiones.
struct Running {
    port: u16,
    _stop: watch::Sender<bool>,
}

fn key_file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join("remote-key"))
}

/// La clave se guarda para que el QR escaneado siga valiendo de un culto a otro.
fn load_key(app: &AppHandle) -> String {
    let saved = key_file(app).and_then(|f| std::fs::read_to_string(f).ok()).map(|k| k.trim().to_string());
    match saved {
        Some(key) if is_key(&key) => key,
        _ => save_key(app),
    }
}

/// Clave nueva, guardada. Si no se puede guardar, vale hasta cerrar Selah.
fn save_key(app: &AppHandle) -> String {
    let key = new_key();
    if let Some(file) = key_file(app) {
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&file, &key) {
            eprintln!("control remoto: no se pudo guardar la clave: {e}");
        }
    }
    key
}
