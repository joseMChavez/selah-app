// Prueba técnica 1 de la Fase 2: consola + ventana de salida en un monitor elegido.
// El backend guarda qué hay en pantalla para que la salida pueda recuperarlo si se recarga.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Monitor, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_dialog::DialogExt;

const MAIN_LABEL: &str = "main";
const OUTPUT_LABEL: &str = "output";
const OUTPUT_STATE_EVENT: &str = "output-state";
const OUTPUT_ERROR_EVENT: &str = "output-error";

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg", "avif"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov", "webm", "mkv", "ogv"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorInfo {
    index: usize,
    name: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    scale: f64,
    is_primary: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum OutputContent {
    #[default]
    Empty,
    Black,
    TestPattern,
    /// Serie completa de diapositivas: la salida conserva una copia fija aunque el operador edite el texto preparado.
    Text { slides: Vec<String>, index: usize },
    Image { path: String },
    Video { path: String, paused: bool },
    Pdf { path: String, page: u32 },
}

impl OutputContent {
    fn media_path(&self) -> Option<&str> {
        match self {
            Self::Image { path } | Self::Video { path, .. } | Self::Pdf { path, .. } => Some(path),
            _ => None,
        }
    }
}

// Los errores viajan como códigos para que la interfaz los muestre en el idioma elegido.
#[derive(Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
enum CommandError {
    MonitorDisconnected,
    FileMissing { path: String },
    InvalidContent,
    Internal { message: String },
}

fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::Internal { message: e.to_string() }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum MediaKind {
    Image,
    Video,
    Pdf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaFile {
    path: String,
    name: String,
    kind: MediaKind,
}

fn media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let ext = ext.as_str();
    if IMAGE_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Image)
    } else if VIDEO_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Video)
    } else if PDF_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Pdf)
    } else {
        None
    }
}

// El protocolo asset empieza sin acceso a nada; solo se abren los archivos que el usuario elige.
fn allow_media(app: &AppHandle, path: &Path) -> Result<(), CommandError> {
    app.asset_protocol_scope().allow_file(path).map_err(internal)
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputState {
    open: bool,
    monitor_name: Option<String>,
    content: OutputContent,
}

#[derive(Default)]
struct AppState {
    output: Mutex<OutputState>,
}

fn monitor_name(monitor: &Monitor, index: usize) -> String {
    monitor.name().cloned().unwrap_or_else(|| format!("Monitor {}", index + 1))
}

fn publish(app: &AppHandle, state: &AppState, update: impl FnOnce(&mut OutputState)) -> OutputState {
    let mut output = state.output.lock().unwrap();
    update(&mut output);
    let snapshot = output.clone();
    let _ = app.emit(OUTPUT_STATE_EVENT, &snapshot);
    snapshot
}

#[tauri::command]
fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, CommandError> {
    let primary = app.primary_monitor().map_err(internal)?;
    let monitors = app.available_monitors().map_err(internal)?;
    Ok(monitors
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            index,
            name: monitor_name(m, index),
            width: m.size().width,
            height: m.size().height,
            x: m.position().x,
            y: m.position().y,
            scale: m.scale_factor(),
            is_primary: primary
                .as_ref()
                .is_some_and(|p| p.name() == m.name() && p.position() == m.position()),
        })
        .collect())
}

// En Wayland no se puede posicionar una ventana; GTK sí permite pedir pantalla completa en un monitor concreto.
// El índice de GDK coincide con el de available_monitors() porque tao los enumera en el mismo orden.
#[cfg(target_os = "linux")]
fn place_fullscreen(window: &WebviewWindow, _monitor: &Monitor, index: usize) -> Result<(), CommandError> {
    let w = window.clone();
    window
        .run_on_main_thread(move || {
            use gtk::prelude::*;
            if let Ok(gtk_window) = w.gtk_window() {
                gtk_window.unfullscreen();
                if let Some(screen) = GtkWindowExt::screen(&gtk_window) {
                    gtk_window.fullscreen_on_monitor(&screen, index as i32);
                }
            }
            let _ = w.show();
        })
        .map_err(internal)
}

#[cfg(not(target_os = "linux"))]
fn place_fullscreen(window: &WebviewWindow, monitor: &Monitor, _index: usize) -> Result<(), CommandError> {
    window.set_fullscreen(false).map_err(internal)?;
    window.set_position(*monitor.position()).map_err(internal)?;
    window.set_fullscreen(true).map_err(internal)?;
    window.show().map_err(internal)
}

// async: en Windows crear ventanas desde un comando síncrono puede bloquear el hilo principal.
#[tauri::command]
async fn open_output(app: AppHandle, state: State<'_, AppState>, monitor_index: usize) -> Result<OutputState, CommandError> {
    let monitors = app.available_monitors().map_err(internal)?;
    let monitor = monitors.get(monitor_index).ok_or(CommandError::MonitorDisconnected)?;

    let window = match app.get_webview_window(OUTPUT_LABEL) {
        Some(window) => window,
        None => WebviewWindowBuilder::new(&app, OUTPUT_LABEL, WebviewUrl::App("output".into()))
            .title("Selah — Salida")
            .decorations(false)
            .visible(false)
            .build()
            .map_err(internal)?,
    };
    place_fullscreen(&window, monitor, monitor_index)?;

    let name = monitor_name(monitor, monitor_index);
    Ok(publish(&app, &state, |output| {
        output.open = true;
        output.monitor_name = Some(name);
    }))
}

#[tauri::command]
fn close_output(app: AppHandle) -> Result<(), CommandError> {
    // El estado se actualiza en el evento Destroyed, que también cubre cierres no pedidos.
    if let Some(window) = app.get_webview_window(OUTPUT_LABEL) {
        window.close().map_err(internal)?;
    }
    Ok(())
}

// Se comprueba antes de enviar para que un archivo borrado o movido no deje la pantalla en un estado roto.
#[tauri::command]
fn set_output_content(app: AppHandle, state: State<'_, AppState>, content: OutputContent) -> Result<OutputState, CommandError> {
    if let OutputContent::Text { slides, index } = &content {
        if *index >= slides.len() {
            return Err(CommandError::InvalidContent);
        }
    }
    if let Some(path) = content.media_path() {
        let path = Path::new(path);
        if !path.is_file() {
            return Err(CommandError::FileMissing { path: path.display().to_string() });
        }
        allow_media(&app, path)?;
    }
    Ok(publish(&app, &state, |output| output.content = content))
}

#[tauri::command]
// Título y filtro llegan traducidos desde la interfaz.
async fn pick_media(app: AppHandle, title: String, filter_name: String) -> Result<Vec<MediaFile>, CommandError> {
    let all: Vec<&str> = [IMAGE_EXTENSIONS, VIDEO_EXTENSIONS, PDF_EXTENSIONS].concat();
    let Some(picked) = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter(filter_name, &all)
        .blocking_pick_files()
    else {
        return Ok(Vec::new());
    };

    let mut files = Vec::new();
    for file in picked {
        let path: PathBuf = file.into_path().map_err(internal)?;
        let Some(kind) = media_kind(&path) else { continue };
        allow_media(&app, &path)?;
        files.push(MediaFile {
            name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            path: path.to_string_lossy().into_owned(),
            kind,
        });
    }
    Ok(files)
}

// La salida informa de archivos que no puede mostrar (formato no soportado, códec ausente).
#[tauri::command]
fn report_output_error(app: AppHandle, message: String) {
    let _ = app.emit(OUTPUT_ERROR_EVENT, message);
}

#[tauri::command]
fn get_output_state(state: State<'_, AppState>) -> OutputState {
    state.output.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .on_window_event(|window, event| {
            let app = window.app_handle();
            match (window.label(), event) {
                (OUTPUT_LABEL, WindowEvent::Destroyed) => {
                    publish(app, &app.state::<AppState>(), |output| {
                        output.open = false;
                        output.monitor_name = None;
                    });
                }
                // Sin consola no hay forma de controlar la salida: cerrar la consola cierra la app.
                (MAIN_LABEL, WindowEvent::Destroyed) => app.exit(0),
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_monitors,
            open_output,
            close_output,
            set_output_content,
            get_output_state,
            pick_media,
            report_output_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
