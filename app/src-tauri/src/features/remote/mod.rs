// Control remoto desde el teléfono: pasar versículos y diapositivas, pantalla en negro, buscar en la Biblia.
// Plugin interno «remote»: la interfaz llama a sus comandos como `plugin:remote|<comando>` (platform/tauri.ts).
//
// La consola sigue mandando: el servidor (server.rs) solo pasa a la consola las órdenes del teléfono
// (evento «remote-command») y reparte a los teléfonos lo que la consola publica (remote_publish).
// La interfaz del teléfono es la ruta /remote de la misma app (app/src/lib/features/remote/).
mod board;
mod commands;
pub mod domain;
mod server;
mod thumbs;

use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use tauri::plugin::{Builder, TauriPlugin};
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tokio::sync::{broadcast, watch};

use self::board::{Board, Op};
use self::domain::{is_key, new_key};
use self::thumbs::Thumbs;
use crate::core::events::REMOTE_BOARD_EVENT;

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
            commands::remote_board,
            commands::remote_board_op,
            commands::remote_screen,
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
                board: Arc::new(BoardHub::new()),
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
    /// La pizarra (board.rs): se borra al apagar el control remoto.
    board: Arc<BoardHub>,
}

/// Cuántos mensajes de la pizarra puede ir atrasado un teléfono antes de mandarle la pizarra entera.
const BOARD_BACKLOG: usize = 512;

/// La pizarra y su reparto: a los teléfonos por `tx` (el WebSocket de cada uno) y a las ventanas de Selah
/// (consola y salida) por el evento REMOTE_BOARD_EVENT. Los mensajes son los mismos para todos.
pub struct BoardHub {
    board: Mutex<Board>,
    tx: broadcast::Sender<String>,
}

impl BoardHub {
    fn new() -> Self {
        BoardHub { board: Mutex::new(Board::default()), tx: broadcast::channel(BOARD_BACKLOG).0 }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    /// Toda la pizarra, para un teléfono que se conecta o que se quedó atrás.
    pub fn snapshot(&self) -> String {
        json!({ "type": "board", "board": &*self.board.lock().unwrap() }).to_string()
    }

    pub fn board(&self) -> serde_json::Value {
        serde_json::to_value(&*self.board.lock().unwrap()).unwrap_or_default()
    }

    pub fn apply(&self, app: &AppHandle, op: Op) {
        if self.board.lock().unwrap().apply(&op) {
            self.send(app, json!({ "type": "op", "op": op }));
        }
    }

    /// Cómo quedó el texto en la pantalla. Solo les interesa a los teléfonos.
    pub fn set_screen(&self, screen: Option<serde_json::Value>) {
        if self.board.lock().unwrap().set_screen(screen.clone()) {
            let _ = self.tx.send(json!({ "type": "screen", "screen": screen }).to_string());
        }
    }

    /// Al apagar el control remoto: las hojas y lo dibujado se olvidan (lo medido de la pantalla sigue valiendo).
    pub fn reset(&self, app: &AppHandle) {
        let mut board = self.board.lock().unwrap();
        let screen = board.screen.take();
        *board = Board { screen, ..Board::default() };
        drop(board);
        let _ = app.emit(REMOTE_BOARD_EVENT, json!({ "type": "reset" }));
        let _ = self.tx.send(self.snapshot());
    }

    fn send(&self, app: &AppHandle, message: impl Serialize) {
        let _ = app.emit(REMOTE_BOARD_EVENT, &message);
        if let Ok(text) = serde_json::to_string(&message) {
            let _ = self.tx.send(text);
        }
    }
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
