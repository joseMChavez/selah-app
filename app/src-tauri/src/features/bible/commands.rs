use tauri::{AppHandle, State};

use super::domain::{BibleInfo, SearchHit, Verse};
use super::{bibles_dir, BibleState};
use crate::core::error::{internal, CommandError};
use crate::core::picker::{self, dialog_extensions};

// Los comandos de Biblia son async para que leer los XML (varios MB) no congele la ventana.
#[tauri::command]
pub async fn list_bibles(
    app: AppHandle,
    state: State<'_, BibleState>,
) -> Result<Vec<BibleInfo>, CommandError> {
    let dir = bibles_dir(&app)?;
    Ok(state.0.lock().unwrap().list(&dir))
}

#[tauri::command]
pub async fn import_bibles(
    app: AppHandle,
    state: State<'_, BibleState>,
    title: String,
    filter_name: String,
) -> Result<Vec<BibleInfo>, CommandError> {
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
            .0
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
    state: State<'_, BibleState>,
    id: String,
) -> Result<(), CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .0
        .lock()
        .unwrap()
        .remove(&dir, &id)
        .map_err(internal)
}

#[tauri::command]
pub async fn get_chapter(
    app: AppHandle,
    state: State<'_, BibleState>,
    bible_id: String,
    book: u32,
    chapter: u32,
) -> Result<Vec<Verse>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .0
        .lock()
        .unwrap()
        .chapter(&dir, &bible_id, book, chapter)
        .ok_or(CommandError::BibleNotFound)
}

#[tauri::command]
pub async fn search_bible(
    app: AppHandle,
    state: State<'_, BibleState>,
    bible_id: String,
    query: String,
) -> Result<Vec<SearchHit>, CommandError> {
    let dir = bibles_dir(&app)?;
    state
        .0
        .lock()
        .unwrap()
        .search(&dir, &bible_id, &query)
        .ok_or(CommandError::BibleNotFound)
}
