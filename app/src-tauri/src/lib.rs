// Consola + ventana de salida en un monitor elegido, medios, Biblias, letras y transmisión.
// Este archivo solo arranca la app: registra los comandos y prepara el estado. Cómo se organiza el
// backend (commands → domain / infra) está en docs/ARQUITECTURA.md.
mod commands;
mod domain;
mod error;
mod infra;
mod state;

use tauri::{Manager, WindowEvent};

use crate::infra::{libreoffice, office, store};
use crate::state::{AppState, MAIN_LABEL, OUTPUT_LABEL, bibles_dir, publish};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            // La base se abre antes que nada: la interfaz lee sus ajustes al cargar.
            let dir = app.path().app_data_dir()?;
            *app.state::<AppState>().store.lock().unwrap() = Some(store::Store::open(&dir));
            office::set_private_soffice(libreoffice::soffice(&dir));
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
            commands::output::list_monitors,
            commands::output::open_output,
            commands::output::close_output,
            commands::output::set_output_content,
            commands::output::get_output_state,
            commands::output::set_output_style,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_reset,
            commands::settings::store_path,
            commands::songs::songs_list,
            commands::songs::song_save,
            commands::songs::song_delete,
            commands::songs::setlist_get,
            commands::songs::setlist_set,
            commands::songs::songs_reset,
            commands::media::pick_background,
            commands::media::convert_presentation,
            commands::office::office_available,
            commands::office::office_status,
            commands::office::office_install,
            commands::office::office_cancel,
            commands::office::office_remove,
            commands::media::allow_backgrounds,
            commands::media::pick_media,
            commands::media::pick_media_folder,
            commands::output::report_output_error,
            commands::output::output_navigate,
            commands::output::log_client_error,
            commands::bible::list_bibles,
            commands::bible::import_bibles,
            commands::bible::remove_bible,
            commands::bible::get_chapter,
            commands::bible::search_bible,
            commands::obs::obs_detect,
            commands::obs::obs_enable_websocket,
            commands::obs::obs_launch,
            commands::obs::obs_connect,
            commands::obs::obs_disconnect,
            commands::obs::obs_status,
            commands::obs::obs_setup,
            commands::obs::obs_set_scene,
            commands::obs::obs_show_text,
            commands::obs::obs_hide_text,
            commands::obs::obs_cameras,
            commands::obs::obs_set_camera,
            commands::obs::obs_preview,
            commands::obs::obs_set_destination,
            commands::obs::obs_start_stream,
            commands::obs::obs_stop_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

