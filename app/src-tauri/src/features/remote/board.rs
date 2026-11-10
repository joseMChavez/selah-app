// Pizarra del control remoto (D-021): lo que se dibuja desde el teléfono, encima de lo que hay en pantalla o
// en hojas en blanco. Vive en memoria mientras el control remoto está encendido; al apagarlo se borra.
//
// Cada «capa» es lo dibujado sobre una cosa concreta: una página de un PDF, un versículo, una hoja… La clave
// de la capa la decide la interfaz (board.ts, layerKey); aquí solo se guarda y se reparte.
// Las operaciones son idempotentes (un trazo dice desde qué punto sigue), así el teléfono puede aplicarlas
// en cuanto las dibuja y otra vez cuando vuelven del servidor sin duplicar nada.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Las coordenadas van de 0 a COORD (enteros: más cortos en JSON que decimales). Se admite salirse un poco
/// del borde: un trazo que empieza en la orilla no se corta.
pub const COORD: i32 = 10_000;
const MARGIN: i32 = 1_000;
/// Números (x e y) en un solo mensaje y en un trazo entero.
const MAX_BATCH: usize = 2_000;
const MAX_STROKE: usize = 20_000;
const MAX_STROKES: usize = 3_000;
const MAX_LAYERS: usize = 300;
pub const MAX_SHEETS: usize = 30;
/// Ids que inventa el teléfono y claves de capa (incluyen la ruta de un PDF).
const MAX_ID: usize = 64;
const MAX_LAYER: usize = 512;
/// La disposición del texto en pantalla (unas decenas de palabras con su posición).
const MAX_LAYOUT: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    White,
    Black,
    Red,
    Yellow,
    Blue,
    Green,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Size {
    Thin,
    Medium,
    Thick,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    Pen,
    /// Resaltador: ancho y translúcido, para marcar palabras de un versículo.
    Marker,
}

/// Fondo de una hoja en blanco. Negro con tinta blanca es lo que mejor se ve proyectado.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Paper {
    #[default]
    Black,
    White,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub id: String,
    pub tool: Tool,
    pub color: Color,
    pub size: Size,
    /// x0, y0, x1, y1… entre 0 y COORD.
    pub points: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    pub id: String,
    pub paper: Paper,
}

/// Lo que cambia la pizarra. Llega del teléfono por el WebSocket (o de la consola) y se reparte tal cual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Op {
    /// Un trazo nuevo o su continuación: `points` sustituye desde el número `at` (0 = desde el principio).
    Draw { layer: String, stroke: String, tool: Tool, color: Color, size: Size, at: usize, points: Vec<i32> },
    /// Quitar un trazo entero (el borrador toca una línea y la quita).
    Erase { layer: String, stroke: String },
    /// Borrar todo lo dibujado sobre una cosa.
    Wipe { layer: String },
    AddSheet { id: String },
    SetPaper { id: String, paper: Paper },
}

impl Op {
    /// None si no es una operación de la pizarra o trae algo imposible.
    pub fn parse(text: &str) -> Option<Op> {
        let op: Op = serde_json::from_str(text).ok()?;
        op.is_valid().then_some(op)
    }

    fn is_valid(&self) -> bool {
        let id = |s: &str| !s.is_empty() && s.len() <= MAX_ID;
        let layer = |s: &str| !s.is_empty() && s.len() <= MAX_LAYER;
        match self {
            Op::Draw { layer: l, stroke, at, points, .. } => {
                layer(l)
                    && id(stroke)
                    && at % 2 == 0
                    && points.len() % 2 == 0
                    && points.len() <= MAX_BATCH
                    && at + points.len() <= MAX_STROKE
                    && points.iter().all(|p| (-MARGIN..=COORD + MARGIN).contains(p))
            }
            Op::Erase { layer: l, stroke } => layer(l) && id(stroke),
            Op::Wipe { layer: l } => layer(l),
            Op::AddSheet { id: i } | Op::SetPaper { id: i, .. } => id(i),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub sheets: Vec<Sheet>,
    pub layers: HashMap<String, Vec<Stroke>>,
    /// Dónde quedó cada palabra del texto en pantalla, medido por la ventana de salida (JSON tal cual).
    /// El teléfono lo dibuja de fondo para subrayar justo encima.
    pub screen: Option<serde_json::Value>,
}

impl Board {
    /// Aplica una operación. false si no cambió nada (o no cabe): entonces no hay que repartirla.
    pub fn apply(&mut self, op: &Op) -> bool {
        match op {
            Op::Draw { layer, stroke, tool, color, size, at, points } => {
                if !self.layers.contains_key(layer) && self.layers.len() >= MAX_LAYERS {
                    return false;
                }
                let strokes = self.layers.entry(layer.clone()).or_default();
                match strokes.iter_mut().find(|s| &s.id == stroke) {
                    Some(found) => {
                        // Un trozo que llega antes que el anterior no deja huecos: se espera al que falta.
                        if *at > found.points.len() {
                            return false;
                        }
                        found.points.truncate(*at);
                        found.points.extend_from_slice(points);
                    }
                    None => {
                        if *at != 0 || strokes.len() >= MAX_STROKES {
                            return false;
                        }
                        strokes.push(Stroke { id: stroke.clone(), tool: *tool, color: *color, size: *size, points: points.clone() });
                    }
                }
                true
            }
            Op::Erase { layer, stroke } => {
                let Some(strokes) = self.layers.get_mut(layer) else { return false };
                let before = strokes.len();
                strokes.retain(|s| &s.id != stroke);
                before != strokes.len()
            }
            Op::Wipe { layer } => self.layers.remove(layer).is_some_and(|strokes| !strokes.is_empty()),
            Op::AddSheet { id } => {
                if self.sheets.len() >= MAX_SHEETS || self.sheets.iter().any(|s| &s.id == id) {
                    return false;
                }
                self.sheets.push(Sheet { id: id.clone(), paper: Paper::default() });
                true
            }
            Op::SetPaper { id, paper } => match self.sheets.iter_mut().find(|s| &s.id == id) {
                Some(sheet) if sheet.paper != *paper => {
                    sheet.paper = *paper;
                    true
                }
                _ => false,
            },
        }
    }

    /// La ventana de salida cuenta cómo quedó el texto. false si es demasiado grande para ser eso.
    pub fn set_screen(&mut self, screen: Option<serde_json::Value>) -> bool {
        if screen.as_ref().is_some_and(|s| s.to_string().len() > MAX_LAYOUT) {
            return false;
        }
        self.screen = screen;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(at: usize, points: &[i32]) -> Op {
        Op::Draw {
            layer: "pdf:/a.pdf#2".into(),
            stroke: "s1".into(),
            tool: Tool::Pen,
            color: Color::Red,
            size: Size::Medium,
            at,
            points: points.to_vec(),
        }
    }

    #[test]
    fn lee_las_operaciones() {
        let op = Op::parse(r#"{"type":"draw","layer":"sheet:a","stroke":"s","tool":"marker","color":"yellow","size":"thick","at":0,"points":[1,2,3,4]}"#);
        assert!(matches!(op, Some(Op::Draw { tool: Tool::Marker, color: Color::Yellow, .. })));
        assert_eq!(Op::parse(r#"{"type":"wipe","layer":"x"}"#), Some(Op::Wipe { layer: "x".into() }));
        assert_eq!(Op::parse(r#"{"type":"setPaper","id":"a","paper":"white"}"#), Some(Op::SetPaper { id: "a".into(), paper: Paper::White }));
        // Una orden de la consola no es de la pizarra.
        assert_eq!(Op::parse(r#"{"type":"next"}"#), None);
    }

    #[test]
    fn rechaza_lo_imposible() {
        let bad = [
            r#"{"type":"draw","layer":"x","stroke":"s","tool":"pen","color":"red","size":"thin","at":0,"points":[1,2,3]}"#,
            r#"{"type":"draw","layer":"x","stroke":"s","tool":"pen","color":"red","size":"thin","at":1,"points":[1,2]}"#,
            r#"{"type":"draw","layer":"x","stroke":"s","tool":"pen","color":"red","size":"thin","at":0,"points":[1,99999]}"#,
            r#"{"type":"draw","layer":"x","stroke":"s","tool":"pen","color":"pink","size":"thin","at":0,"points":[1,2]}"#,
            r#"{"type":"draw","layer":"","stroke":"s","tool":"pen","color":"red","size":"thin","at":0,"points":[1,2]}"#,
            r#"{"type":"addSheet","id":""}"#,
        ];
        for json in bad {
            assert_eq!(Op::parse(json), None, "{json}");
        }
        let long = format!(r#"{{"type":"wipe","layer":"{}"}}"#, "x".repeat(600));
        assert_eq!(Op::parse(&long), None);
        let many = format!(r#"{{"type":"draw","layer":"x","stroke":"s","tool":"pen","color":"red","size":"thin","at":0,"points":[{}]}}"#, vec!["1"; MAX_BATCH + 2].join(","));
        assert_eq!(Op::parse(&many), None);
    }

    #[test]
    fn un_trazo_llega_por_partes_y_repetirlo_no_lo_duplica() {
        let mut board = Board::default();
        assert!(board.apply(&draw(0, &[1, 1, 2, 2])));
        assert!(board.apply(&draw(4, &[3, 3])));
        // El teléfono recibe su propio trozo de vuelta y lo aplica otra vez: queda igual.
        assert!(board.apply(&draw(4, &[3, 3])));
        assert!(!board.apply(&draw(10, &[9, 9])), "falta un trozo: no se deja un hueco");
        let strokes = &board.layers["pdf:/a.pdf#2"];
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].points, vec![1, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn borrar_un_trazo_y_todo() {
        let mut board = Board::default();
        board.apply(&draw(0, &[1, 1]));
        let layer = "pdf:/a.pdf#2".to_string();
        assert!(!board.apply(&Op::Erase { layer: layer.clone(), stroke: "otro".into() }));
        assert!(board.apply(&Op::Erase { layer: layer.clone(), stroke: "s1".into() }));
        assert!(board.layers[&layer].is_empty());
        board.apply(&draw(0, &[1, 1]));
        assert!(board.apply(&Op::Wipe { layer: layer.clone() }));
        assert!(!board.apply(&Op::Wipe { layer }), "ya estaba limpia");
    }

    #[test]
    fn hojas() {
        let mut board = Board::default();
        assert!(board.apply(&Op::AddSheet { id: "a".into() }));
        assert!(!board.apply(&Op::AddSheet { id: "a".into() }), "la misma hoja dos veces");
        assert_eq!(board.sheets[0].paper, Paper::Black, "negra con tinta blanca, lo que mejor se ve");
        assert!(board.apply(&Op::SetPaper { id: "a".into(), paper: Paper::White }));
        assert!(!board.apply(&Op::SetPaper { id: "a".into(), paper: Paper::White }));
        assert!(!board.apply(&Op::SetPaper { id: "b".into(), paper: Paper::White }));
        for i in 1..MAX_SHEETS {
            board.apply(&Op::AddSheet { id: format!("h{i}") });
        }
        assert!(!board.apply(&Op::AddSheet { id: "una-mas".into() }));
    }

    #[test]
    fn la_disposicion_del_texto_tiene_un_limite() {
        let mut board = Board::default();
        assert!(board.set_screen(Some(serde_json::json!({ "aspect": 1.78, "words": [] }))));
        assert!(!board.set_screen(Some(serde_json::json!({ "words": "x".repeat(MAX_LAYOUT) }))));
        assert!(board.screen.is_some(), "lo anterior se queda");
        assert!(board.set_screen(None));
    }
}
