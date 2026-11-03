// Ventana de salida: qué se muestra, con qué estilo y en qué monitor.
// Plugin interno «output»: la interfaz llama a sus comandos como `plugin:output|<comando>` (platform/tauri.ts).
mod commands;
pub mod domain;
mod window;

use std::sync::Mutex;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent, Wry};

use self::domain::OutputState;
use crate::core::events::{OUTPUT_LABEL, OUTPUT_STATE_EVENT};

pub const NAME: &str = "output";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::list_monitors,
            commands::open_output,
            commands::close_output,
            commands::set_output_content,
            commands::report_output_error,
            commands::output_navigate,
            commands::set_output_style,
            commands::get_output_state,
        ])
        .setup(|app, _api| {
            app.manage(OutputStore::default());
            Ok(())
        })
        .on_event(|app, event| {
            // Si se cierra la ventana de salida (Alt+F4, el sistema…), la consola debe saberlo.
            if let RunEvent::WindowEvent { label, event: WindowEvent::Destroyed, .. } = event
                && label == OUTPUT_LABEL
            {
                publish(app, &app.state::<OutputStore>(), |output| {
                    output.open = false;
                    output.monitor_name = None;
                });
            }
        })
        .build()
}

/// Lo que hay en la salida ahora. La salida lo pide al cargar para recuperarse si se recarga.
#[derive(Default)]
pub struct OutputStore(pub Mutex<OutputState>);

pub fn publish(
    app: &AppHandle,
    state: &OutputStore,
    update: impl FnOnce(&mut OutputState),
) -> OutputState {
    let mut output = state.0.lock().unwrap();
    update(&mut output);
    let snapshot = output.clone();
    let _ = app.emit(OUTPUT_STATE_EVENT, &snapshot);
    snapshot
}
