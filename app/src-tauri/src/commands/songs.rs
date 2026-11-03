// Biblioteca de letras y lista del culto (infra/store.rs).

use tauri::State;

use crate::domain::song::Song;
use crate::error::CommandError;
use crate::state::{AppState, with_store};

#[tauri::command]
pub fn songs_list(state: State<'_, AppState>) -> Result<Vec<Song>, CommandError> {
    with_store(&state, |store| store.songs())
}

#[tauri::command]
pub fn song_save(state: State<'_, AppState>, song: Song) -> Result<(), CommandError> {
    with_store(&state, |store| store.save_song(&song))
}

#[tauri::command]
pub fn song_delete(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    with_store(&state, |store| store.delete_song(&id))
}

#[tauri::command]
pub fn setlist_get(state: State<'_, AppState>) -> Result<Vec<String>, CommandError> {
    with_store(&state, |store| store.setlist())
}

#[tauri::command]
pub fn setlist_set(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), CommandError> {
    with_store(&state, |store| store.set_setlist(&ids))
}

#[tauri::command]
pub fn songs_reset(state: State<'_, AppState>) -> Result<(), CommandError> {
    with_store(&state, |store| store.reset_songs())
}
