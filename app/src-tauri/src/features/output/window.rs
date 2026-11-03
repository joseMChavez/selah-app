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
