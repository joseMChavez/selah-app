use std::path::{Path, PathBuf};

use tauri::AppHandle;

use super::domain::{
    media_kind, MediaFile, MediaKind, IMAGE_EXTENSIONS, PDF_EXTENSIONS, PRESENTATION_EXTENSIONS,
    VIDEO_EXTENSIONS,
};
use super::scan;
use crate::core::assets::allow_media;
use crate::core::error::{internal, CommandError};
use crate::core::picker::{self, dialog_extensions};

#[tauri::command]
// Título y filtro llegan traducidos desde la interfaz.
pub async fn pick_media(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<Vec<MediaFile>, CommandError> {
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

    let mut files = Vec::new();
    for path in picked {
        if let Some(media) = to_media_file(&app, path)? {
            files.push(media);
        }
    }
    Ok(files)
}

/// Archivo compatible -> MediaFile, ya permitido en el protocolo asset. Otros tipos -> None.
pub fn to_media_file(app: &AppHandle, path: PathBuf) -> Result<Option<MediaFile>, CommandError> {
    let Some(kind) = media_kind(&path) else {
        return Ok(None);
    };
    allow_media(app, &path)?;
    Ok(Some(MediaFile {
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: path.to_string_lossy().into_owned(),
        kind,
    }))
}

/// Límite de archivos al añadir una carpeta: evita congelar la app si se elige, por ejemplo, la carpeta personal.
pub const MAX_FOLDER_FILES: usize = 2000;

/// Recorre la carpeta y sus subcarpetas (sin las ocultas) y devuelve los archivos compatibles, ordenados por ruta.
/// None si se cancela el diálogo; lista vacía si la carpeta no tiene nada compatible.
#[tauri::command]
pub async fn pick_media_folder(app: AppHandle, title: String) -> Result<Option<Vec<MediaFile>>, CommandError> {
    let request = picker::Request { title, filter: None, mode: picker::Mode::Folder };
    let Some(root) = picker::pick(&app, request).await.map_err(internal)?.and_then(|p| p.into_iter().next()) else {
        return Ok(None);
    };

    let found = scan::media_in_folder(root, MAX_FOLDER_FILES);

    let mut files = Vec::with_capacity(found.len());
    for path in found {
        if let Some(media) = to_media_file(&app, path)? {
            files.push(media);
        }
    }
    Ok(Some(files))
}

// --- Transmisión con OBS ---

/// Elige una imagen o GIF de fondo. None si se cancela.
#[tauri::command]
pub async fn pick_background(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<Option<String>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((filter_name, dialog_extensions(&[IMAGE_EXTENSIONS]))),
        mode: picker::Mode::File,
    };
    let Some(path) = picker::pick(&app, request).await.map_err(internal)?.and_then(|p| p.into_iter().next()) else {
        return Ok(None);
    };
    allow_media(&app, &path)?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Vuelve a permitir los fondos guardados (el permiso del protocolo asset no sobrevive a un reinicio).
/// Devuelve los que siguen existiendo; solo imágenes, para no abrir otros archivos.
#[tauri::command]
pub fn allow_backgrounds(app: AppHandle, paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|p| {
            let path = Path::new(p);
            path.is_file()
                && matches!(media_kind(path), Some(MediaKind::Image))
                && allow_media(&app, path).is_ok()
        })
        .collect()
}

// --- Base de datos local: ajustes, letras y lista ---
