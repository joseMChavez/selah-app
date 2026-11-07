use tauri::State;

use super::domain::Song;
use crate::core::db::{with_store, Db};
use crate::core::error::CommandError;

#[tauri::command]
pub fn songs_list(state: State<'_, Db>) -> Result<Vec<Song>, CommandError> {
    with_store(&state, |store| store.songs())
}

#[tauri::command]
pub fn song_save(state: State<'_, Db>, song: Song) -> Result<(), CommandError> {
    with_store(&state, |store| store.save_song(&song))
}

#[tauri::command]
pub fn song_delete(state: State<'_, Db>, id: String) -> Result<(), CommandError> {
    with_store(&state, |store| store.delete_song(&id))
}

#[tauri::command]
pub fn songs_reset(state: State<'_, Db>) -> Result<(), CommandError> {
    with_store(&state, |store| store.reset_songs())
}
