// Formato propio de EmetPro para letras (.emetpro). Un solo archivo binario, no un zip:
//
//   «SELAHLTR»            8 bytes
//   versión               u16 little-endian (1)
//   largo del manifiesto  u32 little-endian
//   manifiesto            JSON UTF-8 (Manifest): canciones, lista del culto y descripción de los fondos
//   fondos                los bytes de cada imagen, uno tras otro, en el orden de `backgrounds`
//
// El manifiesto se puede leer con cualquier editor saltando los primeros 14 bytes. Cada fondo lleva su tamaño
// y su SHA-256: si no coinciden, el archivo se rechaza.
//
// Antes de llamarse EmetPro la app se llamaba Selah y estos archivos eran .selah. El contenido no cambió:
// la cabecera y el `format` siguen siendo los de entonces, así que un .selah se importa igual, y Selah 0.8 lee un
// .emetpro si se le cambia la extensión. No se cambian: son parte del formato, no el nombre de la app.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::features::transfer::domain::Incoming;

/// Extensión de los archivos que se exportan; la importación también acepta `.selah`.
pub const EXTENSION: &str = "emetpro";
const MAGIC: &[u8; 8] = b"SELAHLTR";
const VERSION: u16 = 1;
const FORMAT: &str = "selah-songs";
/// Más que esto no es un archivo de letras: evita reservar memoria por un largo corrupto.
const MAX_MANIFEST: usize = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub version: u16,
    /// Segundos desde 1970.
    pub exported_at: i64,
    /// «EmetPro 0.9.0» (o «Selah 0.8.2» en archivos anteriores): quién escribió el archivo.
    pub app: String,
    pub songs: Vec<SongEntry>,
    /// Ids de `songs`, en el orden de la lista del culto.
    #[serde(default)]
    pub setlist: Vec<String>,
    #[serde(default)]
    pub backgrounds: Vec<BackgroundEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub singer: String,
    #[serde(default)]
    pub writer: String,
    pub slides: Vec<String>,
    /// Índice en `backgrounds`.
    #[serde(default)]
    pub background: Option<usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundEntry {
    /// Nombre original, para conservar la extensión.
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

/// Un fondo dentro del archivo.
#[derive(Debug)]
pub struct Background {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Lo que trae un .emetpro (o un .selah).
#[derive(Debug)]
pub struct Package {
    pub songs: Vec<Incoming>,
    /// Índices de `songs`, en el orden de la lista del culto.
    pub setlist: Vec<usize>,
    pub backgrounds: Vec<Background>,
}

pub fn is_package(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

pub fn write(mut manifest: Manifest, backgrounds: &[Background]) -> Result<Vec<u8>, String> {
    manifest.format = FORMAT.into();
    manifest.version = VERSION;
    manifest.backgrounds = backgrounds
        .iter()
        .map(|b| BackgroundEntry { name: b.name.clone(), size: b.bytes.len() as u64, sha256: sha256(&b.bytes) })
        .collect();
    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(14 + json.len() + backgrounds.iter().map(|b| b.bytes.len()).sum::<usize>());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&json);
    for background in backgrounds {
        out.extend_from_slice(&background.bytes);
    }
    Ok(out)
}

pub fn read(bytes: &[u8]) -> Result<Package, String> {
    if !is_package(bytes) || bytes.len() < 14 {
        return Err("no es un archivo de letras de EmetPro".into());
    }
    let version = u16::from_le_bytes([bytes[8], bytes[9]]);
    if version > VERSION {
        return Err(format!("lo escribió una versión más nueva de EmetPro (formato {version}); actualiza EmetPro"));
    }
    let len = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]) as usize;
    if len > MAX_MANIFEST || 14 + len > bytes.len() {
        return Err("el archivo está incompleto o dañado".into());
    }
    let manifest: Manifest = serde_json::from_slice(&bytes[14..14 + len]).map_err(|e| format!("manifiesto ilegible: {e}"))?;
    if manifest.format != FORMAT {
        return Err(format!("formato desconocido: {}", manifest.format));
    }

    let mut offset = 14 + len;
    let mut backgrounds = Vec::with_capacity(manifest.backgrounds.len());
    for entry in &manifest.backgrounds {
        let end = usize::try_from(entry.size).ok().and_then(|s| offset.checked_add(s)).filter(|&e| e <= bytes.len());
        let Some(end) = end else {
            return Err("el archivo está incompleto o dañado".into());
        };
        let data = &bytes[offset..end];
        if sha256(data) != entry.sha256 {
            return Err(format!("el fondo «{}» está dañado", entry.name));
        }
        backgrounds.push(Background { name: entry.name.clone(), bytes: data.to_vec() });
        offset = end;
    }

    let index: std::collections::HashMap<&str, usize> =
        manifest.songs.iter().enumerate().map(|(i, s)| (s.id.as_str(), i)).collect();
    let setlist = manifest.setlist.iter().filter_map(|id| index.get(id.as_str()).copied()).collect();
    let songs = manifest
        .songs
        .into_iter()
        .map(|s| Incoming {
            title: s.title,
            singer: s.singer,
            writer: s.writer,
            slides: s.slides,
            background: s.background.filter(|&i| i < backgrounds.len()),
        })
        .collect();
    Ok(Package { songs, setlist, backgrounds })
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest {
            format: String::new(),
            version: 0,
            exported_at: 0,
            app: "prueba".into(),
            songs: vec![
                SongEntry {
                    id: "a".into(),
                    title: "Primera".into(),
                    singer: "Coro".into(),
                    writer: String::new(),
                    slides: vec!["Uno".into(), "Dos".into()],
                    background: Some(0),
                },
                SongEntry { id: "b".into(), title: "Segunda".into(), singer: String::new(), writer: "Ana".into(), slides: vec!["Tres".into()], background: None },
            ],
            setlist: vec!["b".into(), "a".into(), "no-existe".into()],
            backgrounds: Vec::new(),
        }
    }

    #[test]
    fn ida_y_vuelta() {
        let image = Background { name: "fondo.jpg".into(), bytes: vec![1, 2, 3, 4] };
        let bytes = write(manifest(), &[image]).unwrap();
        assert!(is_package(&bytes));
        let package = read(&bytes).unwrap();
        assert_eq!(package.songs.len(), 2);
        assert_eq!(package.songs[0].slides, ["Uno", "Dos"]);
        assert_eq!(package.songs[0].background, Some(0));
        assert_eq!(package.songs[1].writer, "Ana");
        assert_eq!(package.setlist, [1, 0], "la lista sigue el orden y descarta ids desconocidos");
        assert_eq!(package.backgrounds[0].bytes, [1, 2, 3, 4]);
    }

    #[test]
    fn rechaza_archivos_dañados() {
        let image = Background { name: "fondo.jpg".into(), bytes: vec![1, 2, 3, 4] };
        let bytes = write(manifest(), &[image]).unwrap();
        // Fondo cambiado: la suma no coincide.
        let mut altered = bytes.clone();
        *altered.last_mut().unwrap() = 9;
        assert!(read(&altered).unwrap_err().contains("dañado"));
        // Cortado a la mitad.
        assert!(read(&bytes[..bytes.len() - 2]).is_err());
        assert!(read(&bytes[..20]).is_err());
        // Otra cosa.
        assert!(read(b"PK\x03\x04 un zip").is_err());
        // Versión futura.
        let mut future = bytes.clone();
        future[8] = 9;
        assert!(read(&future).unwrap_err().contains("más nueva"));
    }
}
