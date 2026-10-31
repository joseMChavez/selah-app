// Prueba técnica 1 de la Fase 2: consola + ventana de salida en un monitor elegido.
// El backend guarda qué hay en pantalla para que la salida pueda recuperarlo si se recarga.
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Monitor, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

const MAIN_LABEL: &str = "main";
const OUTPUT_LABEL: &str = "output";
const OUTPUT_STATE_EVENT: &str = "output-state";

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
    Text { text: String },
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
fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let primary = app.primary_monitor().map_err(|e| e.to_string())?;
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
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
fn place_fullscreen(window: &WebviewWindow, _monitor: &Monitor, index: usize) -> Result<(), String> {
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
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
fn place_fullscreen(window: &WebviewWindow, monitor: &Monitor, _index: usize) -> Result<(), String> {
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    window.set_position(*monitor.position()).map_err(|e| e.to_string())?;
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())
}

// async: en Windows crear ventanas desde un comando síncrono puede bloquear el hilo principal.
#[tauri::command]
async fn open_output(app: AppHandle, state: State<'_, AppState>, monitor_index: usize) -> Result<OutputState, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .get(monitor_index)
        .ok_or("El monitor elegido ya no está conectado")?;

    let window = match app.get_webview_window(OUTPUT_LABEL) {
        Some(window) => window,
        None => WebviewWindowBuilder::new(&app, OUTPUT_LABEL, WebviewUrl::App("output".into()))
            .title("Selah — Salida")
            .decorations(false)
            .visible(false)
            .build()
            .map_err(|e| e.to_string())?,
    };
    place_fullscreen(&window, monitor, monitor_index)?;

    let name = monitor_name(monitor, monitor_index);
    Ok(publish(&app, &state, |output| {
        output.open = true;
        output.monitor_name = Some(name);
    }))
}

#[tauri::command]
fn close_output(app: AppHandle) -> Result<(), String> {
    // El estado se actualiza en el evento Destroyed, que también cubre cierres no pedidos.
    if let Some(window) = app.get_webview_window(OUTPUT_LABEL) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_output_content(app: AppHandle, state: State<'_, AppState>, content: OutputContent) -> OutputState {
    publish(&app, &state, |output| output.content = content)
}

#[tauri::command]
fn get_output_state(state: State<'_, AppState>) -> OutputState {
    state.output.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            get_output_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
