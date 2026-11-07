// Consola + ventana de salida en un monitor elegido, medios, Biblias, letras y transmisión.
// Este archivo solo arranca la app: abre la base, registra cada funcionalidad (un plugin interno de Tauri)
// y cierra la app al cerrar la consola. Cómo se organiza el backend: docs/ARQUITECTURA.md.
mod core;
mod features;

use tauri::{Manager, WindowEvent};

use crate::core::db::{Db, Store};
use crate::core::events::MAIN_LABEL;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(features::output::plugin())
        .plugin(features::media::plugin())
        .plugin(features::presentations::plugin())
        .plugin(features::bible::plugin())
        .plugin(features::songs::plugin())
        .plugin(features::plans::plugin())
        .plugin(features::settings::plugin())
        .plugin(features::obs::plugin())
        .plugin(features::transfer::plugin())
        .plugin(features::remote::plugin())
        .manage(Db::default())
        .setup(|app| {
            // La base se abre antes que nada: la interfaz lee sus ajustes al cargar.
            let dir = app.path().app_data_dir()?;
            *app.state::<Db>().0.lock().unwrap() = Some(Store::open(&dir));
            Ok(())
        })
        .on_window_event(|window, event| {
            // Sin consola no hay forma de controlar la salida: cerrar la consola cierra la app.
            if window.label() == MAIN_LABEL && matches!(event, WindowEvent::Destroyed) {
                window.app_handle().exit(0);
            }
        })
        // Comandos del núcleo, comunes a toda la app (no pertenecen a ninguna funcionalidad).
        .invoke_handler(tauri::generate_handler![core::log::log_client_error])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
