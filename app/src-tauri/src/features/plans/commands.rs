use std::path::Path;

use tauri::{AppHandle, State};

use super::domain::Plan;
use crate::core::assets::allow_media;
use crate::core::db::{with_store, Db};
use crate::core::error::CommandError;
use crate::features::media::domain::media_kind;

#[tauri::command]
pub fn plans_list(state: State<'_, Db>) -> Result<Vec<Plan>, CommandError> {
    with_store(&state, |store| store.plans())
}

#[tauri::command]
pub fn plan_save(state: State<'_, Db>, plan: Plan) -> Result<(), CommandError> {
    with_store(&state, |store| store.save_plan(&plan))
}

#[tauri::command]
pub fn plan_delete(state: State<'_, Db>, id: String) -> Result<(), CommandError> {
    with_store(&state, |store| store.delete_plan(&id))
}

#[tauri::command]
pub fn plans_reset(state: State<'_, Db>) -> Result<(), CommandError> {
    with_store(&state, |store| store.reset_plans())
}

/// Vuelve a permitir los archivos de un plan guardado (el permiso del protocolo asset no sobrevive a un reinicio).
/// Las rutas salen del plan en la base, no de la interfaz. Devuelve las que ya no están en este equipo.
#[tauri::command]
pub fn plan_allow_files(app: AppHandle, state: State<'_, Db>, id: String) -> Result<Vec<String>, CommandError> {
    let Some(plan) = with_store(&state, |store| store.plan(&id))? else {
        return Ok(Vec::new());
    };
    let mut missing = Vec::new();
    for file in plan.items.iter().filter_map(|i| i.content.file()) {
        let path = Path::new(file);
        let usable = path.is_file() && media_kind(path).is_some() && allow_media(&app, path).is_ok();
        if !usable && !missing.iter().any(|m| m == file) {
            missing.push(file.to_string());
        }
    }
    Ok(missing)
}
