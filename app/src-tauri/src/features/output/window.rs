// La ventana de salida en el sistema: monitores y pantalla completa en el monitor elegido.
use serde::Serialize;
use tauri::{Monitor, WebviewWindow};

use crate::core::error::{internal, CommandError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub is_primary: bool,
}

pub fn monitor_name(monitor: &Monitor, index: usize) -> String {
    monitor
        .name()
        .cloned()
        .unwrap_or_else(|| format!("Monitor {}", index + 1))
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

#[cfg(target_os = "windows")]
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

/// En macOS la pantalla completa se pone en el monitor donde ya está la ventana, así que primero hay que llevarla
/// al elegido. Dos trampas:
/// - macOS mide en puntos y tao pasa la posición de píxeles a puntos con la escala de la ventana (la Retina del
///   portátil, 2×), no con la del monitor de destino: un proyector a 1× a la derecha de un portátil de 1512 puntos
///   está en x = 1512, y la ventana acababa en x = 756, todavía en el portátil. Se da la posición ya en puntos.
/// - AppKit mueve la ventana más tarde (y no durante la animación de salir de pantalla completa): se espera a que
///   esté en el monitor elegido antes de pedir la pantalla completa.
#[cfg(target_os = "macos")]
pub fn place_fullscreen(
    window: &WebviewWindow,
    monitor: &Monitor,
    _index: usize,
) -> Result<(), CommandError> {
    use std::thread::sleep;
    use std::time::Duration;
    use tauri::{LogicalPosition, LogicalSize};

    const STEP: Duration = Duration::from_millis(50);
    /// La animación de salir de pantalla completa dura menos de un segundo.
    const TRIES: u32 = 30;

    if window.is_fullscreen().unwrap_or(false) {
        window.set_fullscreen(false).map_err(internal)?;
        for _ in 0..TRIES {
            if !window.is_fullscreen().unwrap_or(false) {
                break;
            }
            sleep(STEP);
        }
    }
    let scale = monitor.scale_factor();
    let origin: LogicalPosition<f64> = monitor.position().to_logical(scale);
    let size: LogicalSize<f64> = monitor.size().to_logical(scale);
    let on_target = || {
        window
            .current_monitor()
            .ok()
            .flatten()
            .is_some_and(|m| m.name() == monitor.name() && m.position() == monitor.position())
    };
    for _ in 0..TRIES {
        window.set_position(origin).map_err(internal)?;
        window.set_size(size).map_err(internal)?;
        window.show().map_err(internal)?;
        sleep(STEP);
        if on_target() {
            break;
        }
    }
    window.set_fullscreen(true).map_err(internal)
}
