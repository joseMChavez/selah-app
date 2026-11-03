// Tipos de archivo que Selah sabe mostrar.
use std::path::Path;

use serde::Serialize;

pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg", "avif"];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov", "webm", "mkv", "ogv"];
pub const PDF_EXTENSIONS: &[&str] = &["pdf"];
/// Se convierten a PDF para mostrarlas (infra/office.rs).
pub const PRESENTATION_EXTENSIONS: &[&str] = &["pptx", "ppt", "ppsx", "pps", "odp"];

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Image,
    Video,
    Pdf,
    /// PowerPoint o Impress: se convierte a PDF al prepararla (infra/office.rs).
    Presentation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub path: String,
    pub name: String,
    pub kind: MediaKind,
}

pub fn media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let ext = ext.as_str();
    if IMAGE_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Image)
    } else if VIDEO_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Video)
    } else if PDF_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Pdf)
    } else if PRESENTATION_EXTENSIONS.contains(&ext) {
        Some(MediaKind::Presentation)
    } else {
        None
    }
}

/// Extensiones para el filtro del selector de archivos, en minúsculas y en mayúsculas:
/// en Linux el filtro distingue mayúsculas y una foto «IMG_0001.JPG» no aparecería.
pub fn dialog_extensions(groups: &[&[&str]]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|g| g.iter())
        .flat_map(|ext| [ext.to_string(), ext.to_uppercase()])
        .collect()
}
