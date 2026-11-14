// Reglas de la importación, sin archivos ni base de datos: qué canciones del archivo son válidas, cuáles ya
// están en la biblioteca y qué se guarda según lo que elija el usuario para las repetidas.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::features::songs::domain::Song;

/// Una canción leída de un archivo, antes de entrar en la biblioteca.
#[derive(Clone, Debug, PartialEq)]
pub struct Incoming {
    pub title: String,
    pub singer: String,
    pub writer: String,
    pub slides: Vec<String>,
    /// Índice en los fondos que trae el archivo (solo en .emetpro y .selah).
    pub background: Option<usize>,
}

/// Por qué una canción del archivo no se importa. La interfaz lo traduce.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Invalid {
    NoTitle,
    NoLyrics,
    /// El archivo trae dos veces la misma canción: se importa la primera.
    Repeated,
}

/// Qué hacer con una canción que ya está en la biblioteca con otra letra.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OnConflict {
    /// No tocar la de la biblioteca.
    Skip,
    /// Poner la letra nueva en la canción existente: conserva su id, su sitio en la lista y su fondo
    /// si el archivo no trae otro.
    Replace,
    /// Añadirla aparte, con «(importada)» en el título.
    KeepBoth,
}

/// Clasificación de las canciones del archivo frente a la biblioteca.
#[derive(Debug, Default, PartialEq)]
pub struct Plan {
    pub fresh: Vec<usize>,
    /// Ya existe con la misma letra: se salta sin preguntar.
    pub identical: Vec<(usize, String)>,
    /// Ya existe (mismo título y cantante) con otra letra: (canción del archivo, id en la biblioteca).
    pub conflicts: Vec<(usize, String)>,
    pub invalid: Vec<(usize, Invalid)>,
}

/// Lo que hay que guardar y, para cada canción del archivo, con qué id quedó (para la lista del culto).
#[derive(Debug, Default)]
pub struct Resolution {
    pub save: Vec<Song>,
    pub ids: HashMap<usize, String>,
    pub added: usize,
    pub replaced: usize,
    pub skipped: usize,
}

/// Separador que se escribe al pasar diapositivas a texto (el mismo que propone el editor).
pub const SEPARATOR: &str = "\n****\n";

/// Diapositivas de una letra, con la misma regla que la interfaz (features/songs/slides.ts): corta una línea
/// que solo tenga dos o más asteriscos, o tres o más seguidos en cualquier sitio.
pub fn split_slides(body: &str) -> Vec<String> {
    let mut slides = Vec::new();
    let mut current = String::new();
    for line in body.split('\n') {
        let bare = line.trim_end_matches('\r');
        let stars = bare.chars().filter(|&c| c == '*').count();
        if stars >= 2 && bare.chars().all(|c| c == '*' || c == ' ' || c == '\t') {
            slides.push(std::mem::take(&mut current));
            continue;
        }
        let mut pieces = split_runs(bare).into_iter();
        if let Some(first) = pieces.next() {
            current.push_str(&first);
        }
        for piece in pieces {
            slides.push(std::mem::take(&mut current));
            current.push_str(&piece);
        }
        current.push('\n');
    }
    slides.push(current);
    slides.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Corta una línea por los tramos de tres o más asteriscos seguidos.
fn split_runs(line: &str) -> Vec<String> {
    let mut pieces = vec![String::new()];
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' {
            let run = chars[i..].iter().take_while(|&&c| c == '*').count();
            if run >= 3 {
                pieces.push(String::new());
            } else {
                pieces.last_mut().unwrap().extend(&chars[i..i + run]);
            }
            i += run;
        } else {
            pieces.last_mut().unwrap().push(chars[i]);
            i += 1;
        }
    }
    pieces
}

/// Título y cantante comparables: sin mayúsculas, acentos ni espacios de más.
pub fn song_key(title: &str, singer: &str) -> String {
    let plain = |text: &str| {
        text.chars()
            .flat_map(char::to_lowercase)
            .map(|c| match c {
                'á' | 'à' | 'ä' | 'â' => 'a',
                'é' | 'è' | 'ë' | 'ê' => 'e',
                'í' | 'ì' | 'ï' | 'î' => 'i',
                'ó' | 'ò' | 'ö' | 'ô' => 'o',
                'ú' | 'ù' | 'ü' | 'û' => 'u',
                c => c,
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    format!("{}\u{1f}{}", plain(title), plain(singer))
}

pub fn validate(song: &Incoming) -> Option<Invalid> {
    if song.title.trim().is_empty() {
        Some(Invalid::NoTitle)
    } else if song.slides.iter().all(|s| s.trim().is_empty()) {
        Some(Invalid::NoLyrics)
    } else {
        None
    }
}

/// Compara cada canción del archivo con la biblioteca.
pub fn plan(existing: &[Song], incoming: &[Incoming]) -> Plan {
    let by_key: HashMap<String, &Song> = existing.iter().map(|s| (song_key(&s.title, &s.singer), s)).collect();
    let mut seen = std::collections::HashSet::new();
    let mut plan = Plan::default();
    for (i, song) in incoming.iter().enumerate() {
        if let Some(reason) = validate(song) {
            plan.invalid.push((i, reason));
            continue;
        }
        let key = song_key(&song.title, &song.singer);
        if !seen.insert(key.clone()) {
            plan.invalid.push((i, Invalid::Repeated));
            continue;
        }
        match by_key.get(&key) {
            None => plan.fresh.push(i),
            Some(old) if split_slides(&old.body) == song.slides => plan.identical.push((i, old.id.clone())),
            Some(old) => plan.conflicts.push((i, old.id.clone())),
        }
    }
    plan
}

/// Qué se guarda. `background(i)` da la ruta ya guardada del fondo i del archivo; `new_id` crea ids.
pub fn resolve(
    existing: &[Song],
    incoming: &[Incoming],
    plan: &Plan,
    on_conflict: OnConflict,
    imported_suffix: &str,
    background: impl Fn(usize) -> Option<String>,
    mut new_id: impl FnMut() -> String,
) -> Resolution {
    let mut out = Resolution::default();
    let to_song = |id: String, song: &Incoming| Song {
        id,
        title: song.title.trim().to_string(),
        singer: song.singer.trim().to_string(),
        writer: song.writer.trim().to_string(),
        body: song.slides.join(SEPARATOR),
        background: song.background.and_then(&background),
    };
    for &i in &plan.fresh {
        let song = to_song(new_id(), &incoming[i]);
        out.ids.insert(i, song.id.clone());
        out.save.push(song);
        out.added += 1;
    }
    for (i, id) in &plan.identical {
        out.ids.insert(*i, id.clone());
        out.skipped += 1;
    }
    for (i, id) in &plan.conflicts {
        match on_conflict {
            OnConflict::Skip => {
                out.ids.insert(*i, id.clone());
                out.skipped += 1;
            }
            OnConflict::Replace => {
                let old = existing.iter().find(|s| &s.id == id);
                let mut song = to_song(id.clone(), &incoming[*i]);
                if song.background.is_none() {
                    song.background = old.and_then(|s| s.background.clone());
                }
                out.ids.insert(*i, song.id.clone());
                out.save.push(song);
                out.replaced += 1;
            }
            OnConflict::KeepBoth => {
                let mut song = to_song(new_id(), &incoming[*i]);
                song.title = format!("{} {imported_suffix}", song.title);
                out.ids.insert(*i, song.id.clone());
                out.save.push(song);
                out.added += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(id: &str, title: &str, singer: &str, body: &str) -> Song {
        Song { id: id.into(), title: title.into(), singer: singer.into(), writer: String::new(), body: body.into(), background: None }
    }

    fn incoming(title: &str, singer: &str, slides: &[&str]) -> Incoming {
        Incoming {
            title: title.into(),
            singer: singer.into(),
            writer: String::new(),
            slides: slides.iter().map(|s| s.to_string()).collect(),
            background: None,
        }
    }

    #[test]
    fn diapositivas_como_en_la_interfaz() {
        assert_eq!(split_slides("Estrofa\n**\nCoro"), ["Estrofa", "Coro"]);
        assert_eq!(split_slides("Estrofa\r\n****\r\nCoro\n * * \nFinal"), ["Estrofa", "Coro", "Final"]);
        assert_eq!(split_slides("Estrofa *** Coro"), ["Estrofa", "Coro"]);
        assert_eq!(split_slides("Una **palabra** fuerte\nOtra"), ["Una **palabra** fuerte\nOtra"]);
        assert_eq!(split_slides("\n**\n\n"), Vec::<String>::new());
    }

    #[test]
    fn clave_sin_acentos_ni_mayusculas() {
        assert_eq!(song_key("Santo por Siempre ", " Hillsong"), song_key("santo  por siempre", "hillsong"));
        assert_eq!(song_key("Canción", ""), song_key("CANCION", ""));
        assert_ne!(song_key("Admirable", "A"), song_key("Admirable", "B"));
    }

    #[test]
    fn clasifica_nuevas_iguales_repetidas_e_invalidas() {
        let library = [song("1", "Admirable", "Danilo Montero", "Con poder\n****\nCoro"), song("2", "Glorioso día", "", "Vieja")];
        let file = [
            incoming("Nueva", "", &["Letra"]),
            incoming("admirable", "danilo montero", &["Con poder", "Coro"]),
            incoming("Glorioso Dia", "", &["Nueva letra"]),
            incoming("  ", "", &["Sin título"]),
            incoming("Vacía", "", &[" "]),
            incoming("Nueva", "", &["Otra vez"]),
        ];
        let plan = plan(&library, &file);
        assert_eq!(plan.fresh, [0]);
        assert_eq!(plan.identical, [(1, "1".to_string())]);
        assert_eq!(plan.conflicts, [(2, "2".to_string())]);
        assert_eq!(plan.invalid, [(3, Invalid::NoTitle), (4, Invalid::NoLyrics), (5, Invalid::Repeated)]);
    }

    #[test]
    fn sustituir_conserva_id_y_fondo() {
        let mut old = song("2", "Glorioso día", "", "Vieja");
        old.background = Some("/fondos/a.jpg".into());
        let library = [old];
        let file = [incoming("Glorioso Día", "", &["Nueva", "Coro"])];
        let plan = plan(&library, &file);
        let mut n = 0;
        let ids = || {
            n += 1;
            format!("nuevo-{n}")
        };
        let r = resolve(&library, &file, &plan, OnConflict::Replace, "(importada)", |_| None, ids);
        assert_eq!(r.replaced, 1);
        assert_eq!(r.save[0].id, "2");
        assert_eq!(r.save[0].body, "Nueva\n****\nCoro");
        assert_eq!(r.save[0].background.as_deref(), Some("/fondos/a.jpg"));
    }

    #[test]
    fn saltar_y_conservar_las_dos() {
        let library = [song("2", "Glorioso día", "", "Vieja")];
        let file = [incoming("Glorioso día", "", &["Nueva"]), incoming("Otra", "", &["x"])];
        let plan = plan(&library, &file);

        let skip = resolve(&library, &file, &plan, OnConflict::Skip, "(importada)", |_| None, || "n".into());
        assert_eq!((skip.added, skip.replaced, skip.skipped), (1, 0, 1));
        assert_eq!(skip.ids[&0], "2", "la saltada apunta a la de la biblioteca (para la lista)");

        let mut n = 0;
        let both = resolve(&library, &file, &plan, OnConflict::KeepBoth, "(importada)", |_| None, || {
            n += 1;
            format!("n{n}")
        });
        assert_eq!(both.added, 2);
        assert!(both.save.iter().any(|s| s.title == "Glorioso día (importada)"));
    }
}
