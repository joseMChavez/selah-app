// Biblias: listar, importar, quitar, leer y buscar (infra/bible.rs).

use tauri::{AppHandle, State};

use crate::domain::media::dialog_extensions;
use crate::error::{CommandError, internal};
use crate::infra::{bible, picker};
use crate::state::{AppState, bibles_dir};

// Los comandos de Biblia son async para que leer los XML (varios MB) no congele la ventana.
#[tauri::command]
pub async fn list_bibles(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<bible::BibleInfo>, CommandError> {
    let dir = bibles_dir(&app)?;
    Ok(state.bibles.lock().unwrap().list(&dir))
}

#[tauri::command]
pub async fn import_bibles(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    filter_name: String,
) -> Result<Vec<bible::BibleInfo>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((filter_name, dialog_extensions(&[&["xml"]]))),
        mode: picker::Mode::Files,
    };
    let Some(picked) = picker::pick(&app, request).await.map_err(internal)? else {
        return Ok(Vec::new());
    };
    let dir = bibles_dir(&app)?;
    let mut imported = Vec::new();
    for path in picked {
        let info = state
            .bibles
            .lock()
            .unwrap()
            .import(&dir, &path)
            .map_err(|reason| CommandError::InvalidBible {
                file: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                reason,
            })?;
        imported.push(info);
    }
    Ok(imported)
}

#[tauri::command]
pub async fn remove_bible(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .remove(&dir, &id)
        .map_err(internal)
}

#[tauri::command]
pub async fn get_chapter(
    app: AppHandle,
    state: State<'_, AppState>,
    bible_id: String,
    book: u32,
    chapter: u32,
) -> Result<Vec<bible::Verse>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .chapter(&dir, &bible_id, book, chapter)
        .ok_or(CommandError::BibleNotFound)
}

#[tauri::command]
pub async fn search_bible(
    app: AppHandle,
    state: State<'_, AppState>,
    bible_id: String,
    query: String,
) -> Result<Vec<bible::SearchHit>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .bibles
        .lock()
        .unwrap()
        .search(&dir, &bible_id, &query)
        .ok_or(CommandError::BibleNotFound)
}
