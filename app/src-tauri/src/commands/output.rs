// Ventana de salida: monitores, abrir y cerrar, contenido y estilo.
use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Monitor, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::domain::output::{OutputContent, OutputState, OutputStyle};
use crate::error::{CommandError, internal};
use crate::state::{AppState, MAIN_LABEL, OUTPUT_ERROR_EVENT, OUTPUT_LABEL, OUTPUT_NAVIGATE_EVENT, allow_media, publish};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    index: usize,
    name: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    scale: f64,
    is_primary: bool,
}

pub fn monitor_name(monitor: &Monitor, index: usize) -> String {
    monitor
        .name()
        .cloned()
        .unwrap_or_else(|| format!("Monitor {}", index + 1))
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, CommandError> {
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
pub fn place_fullscreen(
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
pub fn place_fullscreen(
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
pub async fn open_output(
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
    // El teclado vuelve a la consola (si el sistema lo permite): las flechas avanzan desde allí.
    // Se encola detrás de la colocación de la salida, que también corre en el hilo principal.
    if let Some(console) = app.get_webview_window(MAIN_LABEL) {
        let target = console.clone();
        let _ = console.run_on_main_thread(move || {
            let _ = target.set_focus();
        });
    }

    let name = monitor_name(monitor, monitor_index);
    Ok(publish(&app, &state, |output| {
        output.open = true;
        output.monitor_name = Some(name);
    }))
}

#[tauri::command]
pub fn close_output(app: AppHandle) -> Result<(), CommandError> {
    // El estado se actualiza en el evento Destroyed, que también cubre cierres no pedidos.
    if let Some(window) = app.get_webview_window(OUTPUT_LABEL) {
        window.close().map_err(internal)?;
    }
    Ok(())
}

// Se comprueba antes de enviar para que un archivo borrado o movido no deje la pantalla en un estado roto.
#[tauri::command]
pub fn set_output_content(
    app: AppHandle,
    state: State<'_, AppState>,
    content: OutputContent,
) -> Result<OutputState, CommandError> {
    let deck_len = match &content {
        OutputContent::Text { slides, index, .. } => Some((slides.len(), *index)),
        OutputContent::Scripture { verses, index, .. } => Some((verses.len(), *index)),
        _ => None,
    };
    let empty_timer = matches!(content, OutputContent::Timer { duration_ms: 0, .. });
    if empty_timer || deck_len.is_some_and(|(len, index)| index >= len) {
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

// Errores de la interfaz al registro de la app: sin esto, un fallo en el webview no deja rastro.
#[tauri::command]
pub fn log_client_error(window: tauri::Window, message: String) {
    eprintln!("[interfaz:{}] {message}", window.label());
}

// La salida informa de archivos que no puede mostrar (formato no soportado, códec ausente).
#[tauri::command]
pub fn report_output_error(app: AppHandle, message: String) {
    let _ = app.emit(OUTPUT_ERROR_EVENT, message);
}

/// Si el foco quedó en la salida, sus flechas (o un presentador inalámbrico) mueven igual lo que está en pantalla.
#[tauri::command]
pub fn output_navigate(app: AppHandle, forward: bool) {
    let _ = app.emit(OUTPUT_NAVIGATE_EVENT, forward);
}

#[tauri::command]
pub fn set_output_style(app: AppHandle, state: State<'_, AppState>, mut style: OutputStyle) -> OutputState {
    // Un fondo o un logo que ya no existe (archivo movido, otro equipo) se quita en vez de dejar la pantalla rota.
    for image in [&mut style.background_image, &mut style.logo.image] {
        if let Some(path) = image.as_deref() {
            let path = Path::new(path);
            if !path.is_file() || allow_media(&app, path).is_err() {
                *image = None;
            }
        }
    }
    style.logo.opacity = style.logo.opacity.clamp(0.1, 1.0);
    publish(&app, &state, |output| output.style = style)
}

#[tauri::command]
pub fn get_output_state(state: State<'_, AppState>) -> OutputState {
    state.output.lock().unwrap().clone()
}
