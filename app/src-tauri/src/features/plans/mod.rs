// Planes del culto: lo que se va a presentar, preparado de antemano (D-019).
// Plugin interno «plans»: la interfaz llama a sus comandos como `plugin:plans|<comando>` (platform/tauri.ts).
mod commands;
pub mod domain;
pub mod repo;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Wry;

pub const NAME: &str = "plans";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::plans_list,
            commands::plan_save,
            commands::plan_delete,
            commands::plans_reset,
            commands::plan_allow_files,
        ])
        .build()
}
