use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::domain::{plan, resolve, Incoming, Invalid, OnConflict};
use super::formats::selah::{self, Background, Manifest, SongEntry};
use super::formats::holyrics;
use super::{Pending, TransferState};
use crate::core::assets::allow_media;
use crate::core::db::{now_secs, with_store, Db};
use crate::core::error::{internal, CommandError};
use crate::core::picker::{self, dialog_extensions};
use crate::features::plans::domain::{ItemContent, Plan, PlanItem};
use crate::features::transfer::domain::split_slides;

/// Un archivo de letras no pesa más que esto, ni con fondos.
const MAX_FILE: u64 = 512 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSummary {
    title: String,
    singer: String,
    slides: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidSong {
    title: String,
    reason: Invalid,
}

/// Lo que se va a importar, para que el usuario lo vea antes de confirmar.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    file: String,
    /// "selah" u "holyrics".
    format: &'static str,
    fresh: Vec<SongSummary>,
    /// Ya están en la biblioteca con la misma letra: no se tocan.
    identical: usize,
    /// Ya están con otra letra: el usuario elige qué hacer.
    conflicts: Vec<SongSummary>,
    invalid: Vec<InvalidSong>,
    /// Canciones que trae como lista del culto (en .hpls, todas); al importar pueden formar un plan.
    setlist: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    added: usize,
    replaced: usize,
    skipped: usize,
    invalid: usize,
    /// Letras del plan creado con la lista del archivo; 0 si no se creó.
    plan_songs: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    path: String,
    songs: usize,
}

fn summary(song: &Incoming) -> SongSummary {
    SongSummary { title: song.title.trim().to_string(), singer: song.singer.trim().to_string(), slides: song.slides.len() }
}

/// Lee un archivo de letras en cualquiera de los formatos que se reconocen.
fn read_file(path: &Path) -> Result<Pending, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_FILE {
        return Err("el archivo es demasiado grande".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if selah::is_selah(&bytes) {
        let package = selah::read(&bytes)?;
        Ok(Pending { file, format: "selah", songs: package.songs, setlist: package.setlist, backgrounds: package.backgrounds })
    } else if holyrics::is_hpls(&bytes) {
        // Una lista de Holyrics es una lista del culto: todas sus canciones, en su orden.
        let songs = holyrics::read(&bytes)?;
        let setlist = (0..songs.len()).collect();
        Ok(Pending { file, format: "holyrics", songs, setlist, backgrounds: Vec::new() })
    } else {
        Err("no es un archivo de letras de Selah (.selah) ni una lista de Holyrics (.hpls)".into())
    }
}

/// Elige un archivo, lo lee y devuelve la vista previa. No guarda nada: eso lo hace `transfer_import_apply`.
#[tauri::command]
pub async fn transfer_import_pick(
    app: AppHandle,
    state: State<'_, TransferState>,
    db: State<'_, Db>,
    title: String,
    filter_name: String,
) -> Result<Option<ImportPreview>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((filter_name, dialog_extensions(&[&["selah", "hpls"]]))),
        mode: picker::Mode::File,
    };
    let Some(path) = picker::pick(&app, request).await.map_err(internal)?.and_then(|p| p.into_iter().next()) else {
        return Ok(None);
    };
    let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let pending = tauri::async_runtime::spawn_blocking(move || read_file(&path))
        .await
        .map_err(internal)?
        .map_err(|reason| CommandError::ImportFailed { file, reason })?;

    let existing = with_store(&db, |store| store.songs())?;
    let plan = plan(&existing, &pending.songs);
    let preview = ImportPreview {
        file: pending.file.clone(),
        format: pending.format,
        fresh: plan.fresh.iter().map(|&i| summary(&pending.songs[i])).collect(),
        identical: plan.identical.len(),
        conflicts: plan.conflicts.iter().map(|(i, _)| summary(&pending.songs[*i])).collect(),
        invalid: plan
            .invalid
            .iter()
            .map(|&(i, reason)| InvalidSong { title: pending.songs[i].title.trim().to_string(), reason })
            .collect(),
        setlist: pending.setlist.len(),
    };
    *state.0.lock().unwrap() = Some(pending);
    Ok(Some(preview))
}

/// Guarda lo leído por `transfer_import_pick`. `imported_suffix` es «(importada)» en el idioma de la interfaz.
/// Con `plan_name`, la lista del archivo se guarda además como un plan con ese nombre.
#[tauri::command]
pub fn transfer_import_apply(
    app: AppHandle,
    state: State<'_, TransferState>,
    db: State<'_, Db>,
    on_conflict: OnConflict,
    plan_name: Option<String>,
    imported_suffix: String,
) -> Result<ImportResult, CommandError> {
    let pending = state.0.lock().unwrap().take().ok_or_else(|| internal("no hay nada leído para importar"))?;
    let file = pending.file.clone();
    // La biblioteca puede haber cambiado desde la vista previa: se vuelve a comparar.
    let existing = with_store(&db, |store| store.songs())?;
    let plan = plan(&existing, &pending.songs);

    // Solo se guardan los fondos de las canciones que de verdad se van a guardar.
    let used: HashSet<usize> = plan
        .fresh
        .iter()
        .chain(plan.conflicts.iter().map(|(i, _)| i))
        .filter_map(|&i| pending.songs[i].background)
        .collect();
    let dir = app.path().app_data_dir().map_err(internal)?.join("fondos");
    let mut saved = std::collections::HashMap::new();
    for i in used {
        let path = save_background(&dir, &pending.backgrounds[i])
            .map_err(|reason| CommandError::ImportFailed { file: file.clone(), reason })?;
        allow_media(&app, &path)?;
        saved.insert(i, path.to_string_lossy().into_owned());
    }

    let resolution = resolve(
        &existing,
        &pending.songs,
        &plan,
        on_conflict,
        &imported_suffix,
        |i| saved.get(&i).cloned(),
        || uuid::Uuid::new_v4().to_string(),
    );
    let new_plan = plan_name.filter(|name| !name.trim().is_empty()).map(|name| Plan {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        date: None,
        notes: String::new(),
        items: pending
            .setlist
            .iter()
            .filter_map(|i| resolution.ids.get(i))
            .map(|id| PlanItem { id: uuid::Uuid::new_v4().to_string(), content: ItemContent::Song { song_id: id.clone() } })
            .collect(),
    });
    let new_plan = new_plan.filter(|p| !p.items.is_empty());
    with_store(&db, |store| store.import_songs(&resolution.save, new_plan.as_ref()))?;
    Ok(ImportResult {
        added: resolution.added,
        replaced: resolution.replaced,
        skipped: resolution.skipped,
        invalid: plan.invalid.len(),
        plan_songs: new_plan.map_or(0, |p| p.items.len()),
    })
}

/// El usuario cerró la vista previa sin importar.
#[tauri::command]
pub fn transfer_import_cancel(state: State<'_, TransferState>) {
    state.0.lock().unwrap().take();
}

/// Guarda el fondo con el nombre de su contenido: importar dos veces el mismo no lo duplica.
fn save_background(dir: &Path, background: &Background) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    let hash = format!("{:x}", Sha256::digest(&background.bytes));
    let ext = Path::new(&background.name)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("img")
        .to_ascii_lowercase();
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.{ext}", &hash[..16]));
    if !path.is_file() {
        std::fs::write(&path, &background.bytes).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

/// Exporta la biblioteca entera (o solo las letras de un plan, como lista del culto) a un .selah, con sus fondos.
#[tauri::command]
pub async fn transfer_export(
    app: AppHandle,
    db: State<'_, Db>,
    title: String,
    filter_name: String,
    file_name: String,
    plan_id: Option<String>,
) -> Result<Option<ExportResult>, CommandError> {
    let request = picker::Request {
        title,
        filter: Some((filter_name, dialog_extensions(&[&["selah"]]))),
        mode: picker::Mode::Save { name: file_name },
    };
    let Some(mut path) = picker::pick(&app, request).await.map_err(internal)?.and_then(|p| p.into_iter().next()) else {
        return Ok(None);
    };
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("selah")) {
        path.as_mut_os_string().push(".selah");
    }

    let mut songs = with_store(&db, |store| store.songs())?;
    // La lista del archivo: las letras del plan, en su orden y sin repetir.
    let mut setlist: Vec<String> = Vec::new();
    if let Some(id) = plan_id {
        let plan = with_store(&db, |store| store.plan(&id))?.ok_or_else(|| internal("el plan ya no existe"))?;
        for item in plan.items {
            match item.content {
                ItemContent::Song { song_id } if !setlist.contains(&song_id) => setlist.push(song_id),
                _ => {}
            }
        }
        songs = setlist.iter().filter_map(|id| songs.iter().find(|s| &s.id == id).cloned()).collect();
    }
    let export = |reason: String| CommandError::ExportFailed { reason };

    let mut backgrounds: Vec<Background> = Vec::new();
    let mut index_of = std::collections::HashMap::<String, usize>::new();
    let mut entries = Vec::with_capacity(songs.len());
    for song in &songs {
        // Un fondo que ya no existe en el disco se omite: la letra se exporta igual.
        let background = song.background.as_ref().and_then(|p| {
            if let Some(&i) = index_of.get(p) {
                return Some(i);
            }
            let bytes = std::fs::read(p).ok()?;
            let name = Path::new(p).file_name()?.to_string_lossy().into_owned();
            backgrounds.push(Background { name, bytes });
            index_of.insert(p.clone(), backgrounds.len() - 1);
            Some(backgrounds.len() - 1)
        });
        entries.push(SongEntry {
            id: song.id.clone(),
            title: song.title.clone(),
            singer: song.singer.clone(),
            writer: song.writer.clone(),
            slides: split_slides(&song.body),
            background,
        });
    }
    let manifest = Manifest {
        format: String::new(),
        version: 0,
        exported_at: now_secs(),
        app: format!("Selah {}", env!("CARGO_PKG_VERSION")),
        songs: entries,
        setlist: setlist.into_iter().filter(|id| songs.iter().any(|s| &s.id == id)).collect(),
        backgrounds: Vec::new(),
    };
    let bytes = selah::write(manifest, &backgrounds).map_err(export)?;
    // Se escribe aparte y se renombra al final: un corte a mitad no deja un archivo roto con el nombre bueno.
    let partial = path.with_extension("selah.part");
    std::fs::write(&partial, &bytes).map_err(|e| export(e.to_string()))?;
    std::fs::rename(&partial, &path).map_err(|e| export(e.to_string()))?;
    Ok(Some(ExportResult { path: path.to_string_lossy().into_owned(), songs: songs.len() }))
}
