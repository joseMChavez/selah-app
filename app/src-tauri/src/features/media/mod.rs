// Medios: imágenes, videos, PDF y presentaciones elegidos del equipo, y fondos.
// Plugin interno «media»: la interfaz llama a sus comandos como `plugin:media|<comando>` (platform/tauri.ts).
mod commands;
pub mod domain;
mod scan;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Wry;

pub const NAME: &str = "media";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::pick_media,
            commands::pick_media_folder,
            commands::pick_background,
            commands::allow_backgrounds,
        ])
        .build()
}
