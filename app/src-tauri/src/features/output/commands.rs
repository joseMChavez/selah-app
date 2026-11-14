use std::path::Path;

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use super::domain::{OutputContent, OutputState, OutputStyle};
use super::window::{monitor_name, place_fullscreen, MonitorInfo};
use super::{publish, OutputStore};
use crate::core::assets::allow_media;
use crate::core::error::{internal, CommandError};
use crate::core::events::{MAIN_LABEL, OUTPUT_ERROR_EVENT, OUTPUT_LABEL, OUTPUT_NAVIGATE_EVENT};

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

// async: en Windows crear ventanas desde un comando síncrono puede bloquear el hilo principal.
#[tauri::command]
pub async fn open_output(
    app: AppHandle,
    state: State<'_, OutputStore>,
    monitor_index: usize,
) -> Result<OutputState, CommandError> {
    let monitors = app.available_monitors().map_err(internal)?;
    let monitor = monitors
        .get(monitor_index)
        .ok_or(CommandError::MonitorDisconnected)?;

    let window = match app.get_webview_window(OUTPUT_LABEL) {
        Some(window) => window,
        None => WebviewWindowBuilder::new(&app, OUTPUT_LABEL, WebviewUrl::App("output".into()))
            .title("EmetPro — Salida")
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
    state: State<'_, OutputStore>,
    content: OutputContent,
) -> Result<OutputState, CommandError> {
    let deck_len = match &content {
        OutputContent::Text { slides, index, .. } => Some((slides.len(), *index)),
        OutputContent::Scripture { verses, index, .. } => Some((verses.len(), *index)),
        _ => None,
    };
    let empty_timer = matches!(content, OutputContent::Timer { duration_ms: 0, .. });
    let no_sheet = matches!(&content, OutputContent::Board { sheet } if sheet.is_empty() || sheet.len() > 64);
    if empty_timer || no_sheet || deck_len.is_some_and(|(len, index)| index >= len) {
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
pub fn set_output_style(app: AppHandle, state: State<'_, OutputStore>, mut style: OutputStyle) -> OutputState {
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
pub fn get_output_state(state: State<'_, OutputStore>) -> OutputState {
    state.0.lock().unwrap().clone()
}
