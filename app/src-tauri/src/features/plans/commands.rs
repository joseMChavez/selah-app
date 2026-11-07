use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};

use super::domain::Plan;
use super::files;
use crate::core::assets::allow_media;
use crate::core::db::{with_store, Db};
use crate::core::error::{internal, CommandError};
use crate::core::picker::{self, dialog_extensions};
use crate::features::media::domain::{
    media_kind, MediaFile, MediaKind, IMAGE_EXTENSIONS, PDF_EXTENSIONS, PRESENTATION_EXTENSIONS, VIDEO_EXTENSIONS,
};

fn media_dir(app: &AppHandle) -> Result<PathBuf, CommandError> {
    Ok(app.path().app_data_dir().map_err(internal)?.join(files::DIR))
}

/// Copia el archivo a la carpeta de la app (fuera del hilo de la interfaz: un video puede tardar) y lo permite.
async fn copy_in(app: &AppHandle, source: PathBuf) -> Result<PathBuf, CommandError> {
    let dir = media_dir(app)?;
    let file = source.display().to_string();
    let copy = tauri::async_runtime::spawn_blocking(move || files::store_copy(&dir, &source))
        .await
        .map_err(internal)?
        .map_err(|e| CommandError::CopyFailed { file, reason: e.to_string() })?;
    allow_media(app, &copy)?;
    Ok(copy)
}

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

/// Elige medios para un plan y los copia a la carpeta de la app; devuelve las copias (con el nombre original).
#[tauri::command]
pub async fn plan_pick_media(app: AppHandle, title: String, filter_name: String) -> Result<Vec<MediaFile>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((
            filter_name,
            dialog_extensions(&[IMAGE_EXTENSIONS, VIDEO_EXTENSIONS, PDF_EXTENSIONS, PRESENTATION_EXTENSIONS]),
        )),
        mode: picker::Mode::Files,
    };
    let Some(picked) = picker::pick(&app, request).await.map_err(internal)? else {
        return Ok(Vec::new());
    };
    let mut copies = Vec::new();
    for path in picked {
        let Some(kind) = media_kind(&path) else { continue };
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let copy = copy_in(&app, path).await?;
        copies.push(MediaFile { name, path: copy.to_string_lossy().into_owned(), kind });
    }
    Ok(copies)
}

/// Elige la imagen de fondo de una pausa y la copia a la carpeta de la app. None si se cancela.
#[tauri::command]
pub async fn plan_pick_background(app: AppHandle, title: String, filter_name: String) -> Result<Option<String>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((filter_name, dialog_extensions(&[IMAGE_EXTENSIONS]))),
        mode: picker::Mode::File,
    };
    let Some(path) = picker::pick(&app, request).await.map_err(internal)?.and_then(|p| p.into_iter().next()) else {
        return Ok(None);
    };
    if !matches!(media_kind(&path), Some(MediaKind::Image)) {
        return Ok(None);
    }
    Ok(Some(copy_in(&app, path).await?.to_string_lossy().into_owned()))
}

/// Borra las copias que ya no usa ningún plan; devuelve lo que ocupan las que quedan, en bytes.
#[tauri::command]
pub async fn plans_clean_files(app: AppHandle, state: State<'_, Db>) -> Result<u64, CommandError> {
    let plans = with_store(&state, |store| store.plans())?;
    let used: HashSet<PathBuf> = plans
        .iter()
        .flat_map(|p| p.items.iter().filter_map(|i| i.content.file()))
        .map(PathBuf::from)
        .collect();
    let dir = media_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || files::clean(&dir, &used)).await.map_err(internal)
}
