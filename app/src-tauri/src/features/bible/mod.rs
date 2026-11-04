// Biblias: incluidas e importadas (XML de Beblia, D-012), capítulos y búsqueda.
// Plugin interno «bible»: la interfaz llama a sus comandos como `plugin:bible|<comando>` (platform/tauri.ts).
mod commands;
pub mod domain;
mod library;
mod xml;

use std::sync::Mutex;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, Manager, Wry};

use self::domain::Verse;
use self::library::{BibleDirs, Library};
use crate::core::error::{internal, CommandError};

pub const NAME: &str = "bible";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::list_bibles,
            commands::import_bibles,
            commands::remove_bible,
            commands::get_chapter,
            commands::search_bible,
        ])
        .setup(|app, _api| {
            app.manage(BibleState::default());
            // Se leen en segundo plano al arrancar, para que estén listas al abrir el panel.
            let handle = app.clone();
            std::thread::spawn(move || {
                if let Ok(dirs) = bibles_dir(&handle) {
                    handle.state::<BibleState>().0.lock().unwrap().ensure_loaded(&dirs);
                }
            });
            Ok(())
        })
        .build()
}

#[derive(Default)]
pub struct BibleState(pub Mutex<Library>);

pub fn bibles_dir(app: &AppHandle) -> Result<BibleDirs, CommandError> {
    Ok(BibleDirs {
        // Si no hay carpeta de recursos (no debería pasar), se sigue solo con las importadas.
        builtin: app.path().resource_dir().ok().map(|dir| dir.join("bibles")),
        user: app.path().app_data_dir().map_err(internal)?.join("bibles"),
    })
}

/// Los versículos de un capítulo; None si no está esa Biblia o ese capítulo. Para otras funcionalidades
/// (el control remoto), que no deben tocar `BibleState` directamente.
pub fn chapter(app: &AppHandle, bible_id: &str, book: u32, chapter: u32) -> Option<Vec<Verse>> {
    let dirs = bibles_dir(app).ok()?;
    app.state::<BibleState>().0.lock().unwrap().chapter(&dirs, bible_id, book, chapter)
}
