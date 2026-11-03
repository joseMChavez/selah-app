// Biblioteca de letras y lista del culto.
// Plugin interno «songs»: la interfaz llama a sus comandos como `plugin:songs|<comando>` (platform/tauri.ts).
mod commands;
pub mod domain;
mod repo;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Wry;

pub const NAME: &str = "songs";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::songs_list,
            commands::song_save,
            commands::song_delete,
            commands::setlist_get,
            commands::setlist_set,
            commands::songs_reset,
        ])
        .build()
}
