// El servidor del control remoto: sirve la página del teléfono, una API mínima y un WebSocket.
//
//   GET /remote?key=…    la página (la misma interfaz de Selah, ruta /remote)
//   GET /_app/…          sus archivos
//   GET /api/chapter     versículos de un capítulo, para elegir en el teléfono
//   GET /api/thumb       miniatura de una página del PDF en pantalla (thumbs.rs)
//   GET /remote.webmanifest?key=…   para instalarla como app (PWA); con una clave válida, la app abre ya vinculada
//   GET /ws?key=…        WebSocket: el teléfono manda órdenes (domain::Command) y recibe lo que hay en pantalla;
//                        también dibuja en la pizarra (board::Op) y recibe lo que dibujan los demás
//
// Todo lo que no es la página pide la clave. Las órdenes no se aplican aquí: se pasan a la consola.
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::watch;

use super::board::Op;
use super::domain::{same_key, Command, PORTS};
use super::thumbs::{Lookup, Thumbs};
use super::BoardHub;
use crate::core::events::{MAIN_LABEL, REMOTE_CLIENTS_EVENT, REMOTE_COMMAND_EVENT};
use crate::features::bible;

/// Lo que comparten todas las conexiones.
#[derive(Clone)]
pub struct Shared {
    pub app: AppHandle,
    /// La clave vigente: al cambiarla, los teléfonos conectados con la anterior se desconectan.
    pub key: watch::Receiver<String>,
    /// Lo último que publicó la consola (JSON), tal cual se manda al teléfono.
    pub status: watch::Receiver<String>,
    /// Se cierra (o cambia) al apagar el control remoto.
    pub stop: watch::Receiver<bool>,
    pub clients: Arc<AtomicUsize>,
    pub thumbs: Arc<Mutex<Thumbs>>,
    pub thumbs_added: watch::Receiver<u64>,
    pub board: Arc<BoardHub>,
}

impl Shared {
    fn allows(&self, key: Option<&str>) -> bool {
        key.is_some_and(|k| same_key(k, &self.key.borrow()))
    }

    fn connected(&self, change: isize) {
        let now = if change > 0 {
            self.clients.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.clients.fetch_sub(1, Ordering::SeqCst) - 1
        };
        let _ = self.app.emit_to(MAIN_LABEL, REMOTE_CLIENTS_EVENT, now);
    }
}

/// Abre el primer puerto libre de `PORTS` y sirve hasta que `stop` cambie. Devuelve el puerto.
pub async fn start(shared: Shared) -> std::io::Result<u16> {
    let mut last_error = None;
    for port in PORTS {
        // En todas las interfaces: el teléfono llega por la red local.
        match TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await {
            Ok(listener) => {
                let mut stop = shared.stop.clone();
                let router = Router::new()
                    .route("/", get(|| async { Redirect::to("/remote") }))
                    .route("/ws", get(socket))
                    .route("/api/chapter", get(chapter))
                    .route("/api/thumb", get(thumb))
                    .route("/remote.webmanifest", get(manifest))
                    .fallback(get(asset))
                    .with_state(shared);
                tauri::async_runtime::spawn(async move {
                    let shutdown = async move {
                        let _ = stop.changed().await;
                    };
                    if let Err(e) = axum::serve(listener, router).with_graceful_shutdown(shutdown).await {
                        eprintln!("control remoto: {e}");
                    }
                });
                return Ok(port);
            }
            Err(e) => last_error = Some(e),
        }
    }
    Err(last_error.unwrap_or_else(|| std::io::Error::other("sin puertos")))
}

/// La IP de este equipo en la red local. Conectar un socket UDP no envía nada: solo elige la interfaz
/// por la que saldría el tráfico. Sin red, 127.0.0.1 (la consola avisa).
pub fn lan_ip() -> IpAddr {
    UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .and_then(|socket| {
            socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9))?;
            socket.local_addr()
        })
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

#[derive(Deserialize)]
struct KeyQuery {
    key: Option<String>,
}

async fn socket(State(shared): State<Shared>, Query(query): Query<KeyQuery>, upgrade: WebSocketUpgrade) -> Response {
    if !shared.allows(query.key.as_deref()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    upgrade.on_upgrade(move |socket| client(shared, socket))
}

async fn client(shared: Shared, mut socket: WebSocket) {
    let mut status = shared.status.clone();
    let mut key = shared.key.clone();
    let mut stop = shared.stop.clone();
    // Antes de mandar la pizarra entera: así no se pierde nada de lo que se dibuje entretanto.
    let mut board = shared.board.subscribe();
    let my_key = key.borrow_and_update().clone();
    shared.connected(1);

    let first = status.borrow_and_update().clone();
    let sent = socket.send(Message::Text(first.into())).await.is_ok()
        && socket.send(Message::Text(shared.board.snapshot().into())).await.is_ok();
    if sent {
        loop {
            tokio::select! {
                changed = status.changed() => {
                    if changed.is_err() { break; }
                    let text = status.borrow_and_update().clone();
                    if socket.send(Message::Text(text.into())).await.is_err() { break; }
                }
                message = board.recv() => {
                    let text = match message {
                        Ok(text) => text,
                        // Se quedó atrás (Wi‑Fi lento): la pizarra entera en vez de lo que se perdió.
                        Err(RecvError::Lagged(_)) => shared.board.snapshot(),
                        Err(RecvError::Closed) => break,
                    };
                    if socket.send(Message::Text(text.into())).await.is_err() { break; }
                }
                changed = key.changed() => {
                    if changed.is_err() || *key.borrow_and_update() != my_key { break; }
                }
                _ = stop.changed() => break,
                message = socket.recv() => match message {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(command) = Command::parse(&text) {
                            let _ = shared.app.emit_to(MAIN_LABEL, REMOTE_COMMAND_EVENT, &command);
                        } else if let Some(op) = Op::parse(&text) {
                            shared.board.apply(&shared.app, op);
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    Some(Ok(_)) => {}
                },
            }
        }
    }
    let _ = socket.send(Message::Close(None)).await;
    shared.connected(-1);
}

/// El manifiesto de la app instalada. Lleva la clave en `start_url` solo si es la vigente: el teléfono guarda
/// lo instalado aparte del navegador y, sin ella, la app abriría sin vincular.
async fn manifest(State(shared): State<Shared>, Query(query): Query<KeyQuery>) -> Response {
    let start = match query.key.as_deref() {
        Some(key) if shared.allows(Some(key)) => format!("/remote?key={key}"),
        _ => "/remote".to_string(),
    };
    let body = super::domain::manifest(&start);
    ([(header::CONTENT_TYPE, "application/manifest+json"), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

#[derive(Deserialize)]
struct ChapterQuery {
    key: Option<String>,
    bible: String,
    book: u32,
    chapter: u32,
}

async fn chapter(State(shared): State<Shared>, Query(query): Query<ChapterQuery>) -> Response {
    if !shared.allows(query.key.as_deref()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let app = shared.app.clone();
    // Leer una Biblia la primera vez toma un momento: fuera del hilo del servidor.
    let verses = tauri::async_runtime::spawn_blocking(move || bible::chapter(&app, &query.bible, query.book, query.chapter)).await;
    match verses {
        Ok(Some(verses)) => Json(verses).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Cuánto espera el teléfono a una miniatura que la consola aún está dibujando.
const THUMB_WAIT: Duration = Duration::from_secs(8);

#[derive(Deserialize)]
struct ThumbQuery {
    key: Option<String>,
    doc: String,
    n: u32,
}

/// Una miniatura del PDF en pantalla. Si la consola aún no la dibujó, espera un poco a que llegue.
async fn thumb(State(shared): State<Shared>, Query(query): Query<ThumbQuery>) -> Response {
    if !shared.allows(query.key.as_deref()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut added = shared.thumbs_added.clone();
    let found = tokio::time::timeout(THUMB_WAIT, async {
        loop {
            added.borrow_and_update();
            match shared.thumbs.lock().unwrap().get(&query.doc, query.n) {
                Lookup::Ready(jpeg) => return Some(jpeg),
                Lookup::Gone => return None,
                Lookup::Pending => {}
            }
            if added.changed().await.is_err() {
                return None;
            }
        }
    })
    .await;
    match found {
        // El id del documento va en la dirección: la misma dirección es siempre la misma imagen.
        Ok(Some(jpeg)) => ([(header::CONTENT_TYPE, "image/jpeg"), (header::CACHE_CONTROL, "private, max-age=3600")], jpeg.to_vec()).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        // Aún no llegó: el teléfono vuelve a pedirla.
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// Los archivos de la interfaz, los mismos que usa la consola. Solo la ruta /remote sirve de página.
async fn asset(State(shared): State<Shared>, uri: Uri) -> Response {
    let path = uri.path();
    let is_file = path.starts_with("/_app/") || path.rsplit('/').next().is_some_and(|name| name.contains('.'));
    if !is_file && path != "/remote" {
        return Redirect::to("/remote").into_response();
    }
    match shared.app.asset_resolver().get(path.to_string()) {
        Some(asset) => {
            // Los de /_app/immutable/ llevan un hash en el nombre: pueden quedarse en caché.
            let cache = if path.starts_with("/_app/immutable/") { "public, max-age=31536000, immutable" } else { "no-cache" };
            ([(header::CONTENT_TYPE, asset.mime_type), (header::CACHE_CONTROL, cache.to_string())], asset.bytes).into_response()
        }
        // En `tauri dev` la interfaz no va dentro del ejecutable: hay que probar con un build.
        None => (StatusCode::SERVICE_UNAVAILABLE, "Selah: esta compilacion no incluye la interfaz (tauri dev). Usa un build.").into_response(),
    }
}
