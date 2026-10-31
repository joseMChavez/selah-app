// Consola + ventana de salida en un monitor elegido, medios y Biblias.
// El backend guarda qué hay en pantalla para que la salida pueda recuperarlo si se recarga.
mod bible;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, Monitor, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};
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
    Text {
        slides: Vec<String>,
        index: usize,
    },
    Image {
        path: String,
    },
    Video {
        path: String,
        paused: bool,
    },
    Pdf {
        path: String,
        page: u32,
    },
    /// Pasaje bíblico, un versículo por diapositiva. El copyright viaja con el contenido para mostrarse siempre.
    Scripture {
        verses: Vec<ScriptureVerse>,
        index: usize,
        version: String,
        copyright: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptureVerse {
    reference: String,
    text: String,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum VerticalPosition {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Background {
    #[default]
    Black,
    Navy,
    Charcoal,
}

/// Cómo se coloca el texto en la pantalla. Se aplica a lo que está al aire en cuanto cambia.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutputStyle {
    /// Tamaño máximo relativo; el texto se reduce solo si no cabe.
    font_scale: f32,
    align: TextAlign,
    position: VerticalPosition,
    background: Background,
}

impl Default for OutputStyle {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            align: TextAlign::Center,
            position: VerticalPosition::Middle,
            background: Background::Black,
        }
    }
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
    InvalidBible { file: String, reason: String },
    BibleNotFound,
    Internal { message: String },
}

fn internal(e: impl std::fmt::Display) -> CommandError {
    CommandError::Internal {
        message: e.to_string(),
    }
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
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(internal)
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputState {
    open: bool,
    monitor_name: Option<String>,
    content: OutputContent,
    style: OutputStyle,
}

#[derive(Default)]
struct AppState {
    output: Mutex<OutputState>,
    bibles: Mutex<bible::Library>,
}

fn bibles_dir(app: &AppHandle) -> Result<bible::BibleDirs, CommandError> {
    Ok(bible::BibleDirs {
        // Si no hay carpeta de recursos (no debería pasar), se sigue solo con las importadas.
        builtin: app.path().resource_dir().ok().map(|dir| dir.join("bibles")),
        user: app.path().app_data_dir().map_err(internal)?.join("bibles"),
    })
}

fn monitor_name(monitor: &Monitor, index: usize) -> String {
    monitor
        .name()
        .cloned()
        .unwrap_or_else(|| format!("Monitor {}", index + 1))
}

fn publish(
    app: &AppHandle,
    state: &AppState,
    update: impl FnOnce(&mut OutputState),
) -> OutputState {
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
fn place_fullscreen(
    window: &WebviewWindow,
    _monitor: &Monitor,
    index: usize,
) -> Result<(), CommandError> {
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
fn place_fullscreen(
    window: &WebviewWindow,
    monitor: &Monitor,
    _index: usize,
) -> Result<(), CommandError> {
    window.set_fullscreen(false).map_err(internal)?;
    window.set_position(*monitor.position()).map_err(internal)?;
    window.set_fullscreen(true).map_err(internal)?;
    window.show().map_err(internal)
}

// async: en Windows crear ventanas desde un comando síncrono puede bloquear el hilo principal.
#[tauri::command]
async fn open_output(
    app: AppHandle,
    state: State<'_, AppState>,
    monitor_index: usize,
) -> Result<OutputState, CommandError> {
    let monitors = app.available_monitors().map_err(internal)?;
    let monitor = monitors
        .get(monitor_index)
        .ok_or(CommandError::MonitorDisconnected)?;

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
fn set_output_content(
    app: AppHandle,
    state: State<'_, AppState>,
    content: OutputContent,
) -> Result<OutputState, CommandError> {
    let deck_len = match &content {
        OutputContent::Text { slides, index } => Some((slides.len(), *index)),
        OutputContent::Scripture { verses, index, .. } => Some((verses.len(), *index)),
        _ => None,
    };
    if deck_len.is_some_and(|(len, index)| index >= len) {
        return Err(CommandError::InvalidContent);
    }
    if let Some(path) = content.media_path() {
        let path = Path::new(path);
        if !path.is_file() {
            return Err(CommandError::FileMissing {
                path: path.display().to_string(),
            });
        }
        allow_media(&app, path)?;
    }
    Ok(publish(&app, &state, |output| output.content = content))
}

#[tauri::command]
// Título y filtro llegan traducidos desde la interfaz.
async fn pick_media(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<Vec<MediaFile>, CommandError> {
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
        if let Some(media) = to_media_file(&app, path)? {
            files.push(media);
        }
    }
    Ok(files)
}

/// Archivo compatible -> MediaFile, ya permitido en el protocolo asset. Otros tipos -> None.
fn to_media_file(app: &AppHandle, path: PathBuf) -> Result<Option<MediaFile>, CommandError> {
    let Some(kind) = media_kind(&path) else {
        return Ok(None);
    };
    allow_media(app, &path)?;
    Ok(Some(MediaFile {
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: path.to_string_lossy().into_owned(),
        kind,
    }))
}

/// Límite de archivos al añadir una carpeta: evita congelar la app si se elige, por ejemplo, la carpeta personal.
const MAX_FOLDER_FILES: usize = 2000;

/// Recorre la carpeta y sus subcarpetas (sin las ocultas) y devuelve los archivos compatibles, ordenados por ruta.
/// None si se cancela el diálogo; lista vacía si la carpeta no tiene nada compatible.
#[tauri::command]
async fn pick_media_folder(app: AppHandle, title: String) -> Result<Option<Vec<MediaFile>>, CommandError> {
    let Some(folder) = app.dialog().file().set_title(title).blocking_pick_folder() else {
        return Ok(None);
    };
    let root = folder.into_path().map_err(internal)?;

    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for path in entries.flatten().map(|e| e.path()) {
            let hidden = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.'));
            if hidden {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else if media_kind(&path).is_some() {
                found.push(path);
            }
        }
        if found.len() >= MAX_FOLDER_FILES {
            break;
        }
    }
    found.sort();
    found.truncate(MAX_FOLDER_FILES);

    let mut files = Vec::with_capacity(found.len());
    for path in found {
        if let Some(media) = to_media_file(&app, path)? {
            files.push(media);
        }
    }
    Ok(Some(files))
}

// Errores de la interfaz al registro de la app: sin esto, un fallo en el webview no deja rastro.
#[tauri::command]
fn log_client_error(window: tauri::Window, message: String) {
    eprintln!("[interfaz:{}] {message}", window.label());
}

// Los comandos de Biblia son async para que leer los XML (varios MB) no congele la ventana.
#[tauri::command]
async fn list_bibles(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<bible::BibleInfo>, CommandError> {
    let dir = bibles_dir(&app)?;
    Ok(state.bibles.lock().unwrap().list(&dir))
}

#[tauri::command]
async fn import_bibles(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    filter_name: String,
) -> Result<Vec<bible::BibleInfo>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter(filter_name, &["xml"])
        .blocking_pick_files()
    else {
        return Ok(Vec::new());
    };
    let dir = bibles_dir(&app)?;
    let mut imported = Vec::new();
    for file in picked {
        let path = file.into_path().map_err(internal)?;
        let info = state
            .bibles
            .lock()
            .unwrap()
            .import(&dir, &path)
            .map_err(|reason| CommandError::InvalidBible {
                file: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                reason,
            })?;
        imported.push(info);
    }
    Ok(imported)
}

#[tauri::command]
async fn remove_bible(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .remove(&dir, &id)
        .map_err(internal)
}

#[tauri::command]
async fn get_chapter(
    app: AppHandle,
    state: State<'_, AppState>,
    bible_id: String,
    book: u32,
    chapter: u32,
) -> Result<Vec<bible::Verse>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .chapter(&dir, &bible_id, book, chapter)
        .ok_or(CommandError::BibleNotFound)
}

#[tauri::command]
async fn search_bible(
    app: AppHandle,
    state: State<'_, AppState>,
    bible_id: String,
    query: String,
) -> Result<Vec<bible::SearchHit>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .search(&dir, &bible_id, &query)
        .ok_or(CommandError::BibleNotFound)
}

// La salida informa de archivos que no puede mostrar (formato no soportado, códec ausente).
#[tauri::command]
fn report_output_error(app: AppHandle, message: String) {
    let _ = app.emit(OUTPUT_ERROR_EVENT, message);
}

#[tauri::command]
fn set_output_style(app: AppHandle, state: State<'_, AppState>, style: OutputStyle) -> OutputState {
    publish(&app, &state, |output| output.style = style)
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
        .setup(|app| {
            // Las Biblias se leen en segundo plano al arrancar, para que estén listas al abrir el panel.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Ok(dirs) = bibles_dir(&handle) {
                    handle.state::<AppState>().bibles.lock().unwrap().ensure_loaded(&dirs);
                }
            });
            Ok(())
        })
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
            set_output_style,
            pick_media,
            pick_media_folder,
            report_output_error,
            log_client_error,
            list_bibles,
            import_bibles,
            remove_bible,
            get_chapter,
            search_bible
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
