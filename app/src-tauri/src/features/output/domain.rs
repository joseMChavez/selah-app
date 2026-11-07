// Lo que se muestra en la ventana de salida y con qué estilo. Tipos puros: sin Tauri.
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
// rename_all solo cambia el nombre de las variantes; rename_all_fields, el de sus campos (duration_ms → durationMs).
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OutputContent {
    #[default]
    Empty,
    Black,
    TestPattern,
    /// Serie completa de diapositivas: la salida conserva una copia fija aunque el operador edite el texto preparado.
    Text {
        slides: Vec<String>,
        index: usize,
        /// Autoría de una letra (cantante, escritor); se muestra en la primera diapositiva.
        #[serde(default)]
        credit: String,
        /// Imagen o GIF propio de esta letra; tapa el fondo general.
        #[serde(default)]
        background: Option<String>,
    },
    Image {
        path: String,
    },
    Video {
        path: String,
        paused: bool,
    },
    Pdf {
        path: String,
        page: u32,
    },
    /// Pasaje bíblico, un versículo por diapositiva. El copyright viaja con el contenido para mostrarse siempre.
    Scripture {
        verses: Vec<ScriptureVerse>,
        index: usize,
        version: String,
        copyright: String,
        /// Segunda versión, debajo de la primera (por ejemplo, en otro idioma). Como mucho dos.
        #[serde(default)]
        second: Option<SecondVersion>,
    },
    /// Hora actual; la dibuja la salida con su propio reloj.
    Clock {
        hour12: bool,
        seconds: bool,
    },
    /// Cuenta atrás. En marcha si ends_at (ms desde 1970) tiene valor; en pausa guarda lo que queda.
    /// Al llegar a cero sigue contando el tiempo excedido con un borde rojo.
    Timer {
        duration_ms: u64,
        ends_at: Option<u64>,
        remaining_ms: i64,
        /// Título sobre la cuenta («Receso», «Conferencia»); vacío = solo la cuenta.
        #[serde(default)]
        label: String,
        /// Imagen o GIF propio de esta cuenta; tapa el fondo general.
        #[serde(default)]
        background: Option<String>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecondVersion {
    pub version: String,
    pub copyright: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptureVerse {
    pub reference: String,
    pub text: String,
    /// El mismo versículo en la segunda versión; None si esa versión no lo tiene.
    #[serde(default)]
    pub second: Option<ParallelVerse>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelVerse {
    pub reference: String,
    pub text: String,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerticalPosition {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Background {
    #[default]
    Black,
    Navy,
    Charcoal,
}

/// Tipos de letra incluidos en la app (no dependen de las fuentes instaladas en cada equipo).
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Font {
    #[default]
    System,
    Montserrat,
    Oswald,
    BebasNeue,
    Lora,
    Merriweather,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Corner {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogoSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Logo o emblema de la iglesia, como marca de agua en una esquina de la pantalla.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Logo {
    /// None = sin logo.
    pub image: Option<String>,
    pub corner: Corner,
    pub size: LogoSize,
    /// 0..1; como marca de agua, por defecto semitransparente.
    pub opacity: f64,
}

impl Default for Logo {
    fn default() -> Self {
        Self { image: None, corner: Corner::TopRight, size: LogoSize::Medium, opacity: 0.6 }
    }
}

/// Cómo se coloca el texto en la pantalla. Se aplica a lo que está al aire en cuanto cambia.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStyle {
    /// Tamaño máximo relativo; el texto se reduce solo si no cabe.
    pub font_scale: f32,
    pub align: TextAlign,
    pub position: VerticalPosition,
    pub background: Background,
    #[serde(default)]
    pub font: Font,
    /// Imagen o GIF detrás del texto, la Biblia y el reloj.
    #[serde(default)]
    pub background_image: Option<String>,
    #[serde(default)]
    pub logo: Logo,
}

impl Default for OutputStyle {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            align: TextAlign::Center,
            position: VerticalPosition::Middle,
            background: Background::Black,
            font: Font::System,
            background_image: None,
            logo: Logo::default(),
        }
    }
}

impl OutputContent {
    pub fn media_path(&self) -> Option<&str> {
        match self {
            Self::Image { path } | Self::Video { path, .. } | Self::Pdf { path, .. } => Some(path),
            Self::Text { background, .. } | Self::Timer { background, .. } => background.as_deref(),
            _ => None,
        }
    }
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputState {
    pub open: bool,
    pub monitor_name: Option<String>,
    pub content: OutputContent,
    pub style: OutputStyle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn estilo_con_y_sin_logo() {
        // Un estilo guardado antes de que existiera el logo sigue valiendo: sin logo, esquina superior derecha.
        let old: OutputStyle = serde_json::from_str(
            r#"{"fontScale":1.1,"align":"center","position":"middle","background":"black","font":"system","backgroundImage":null}"#,
        )
        .unwrap();
        assert!(old.logo.image.is_none());
        assert!(matches!(old.logo.corner, Corner::TopRight));

        let json = r#"{"fontScale":1.5,"align":"left","position":"top","background":"navy","font":"oswald","backgroundImage":null,"logo":{"image":"/x/logo.png","corner":"bottomRight","size":"large","opacity":0.4}}"#;
        let style: OutputStyle = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_value(&style).unwrap(), serde_json::from_str::<serde_json::Value>(json).unwrap());
    }

    // Los contenidos llegan de la interfaz en camelCase: un campo mal nombrado solo falla al enviarlo.
    #[test]
    pub fn contenidos_de_la_interfaz() {
        for json in [
            r#"{"kind":"timer","durationMs":60000,"endsAt":null,"remainingMs":60000,"label":"Receso","background":null}"#,
            r#"{"kind":"clock","hour12":true,"seconds":false}"#,
            r#"{"kind":"text","slides":["a"],"index":0,"credit":"c","background":null}"#,
            r#"{"kind":"scripture","verses":[{"reference":"Jn 3:16","text":"t","second":{"reference":"Jn 3:16","text":"t"}}],"index":0,"version":"V","copyright":"","second":{"version":"W","copyright":""}}"#,
        ] {
            let content: OutputContent = serde_json::from_str(json).expect(json);
            assert_eq!(serde_json::to_value(&content).unwrap(), serde_json::from_str::<serde_json::Value>(json).unwrap());
        }
    }
}
