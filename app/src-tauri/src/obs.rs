// Transmisión con OBS por obs-websocket 5 (D-008).
// Selah prepara OBS (activa el WebSocket con permiso del usuario y con OBS cerrado), lo abre y se conecta.
// Crea sus propias escenas «Selah · …» y solo toca esas: el resto de la configuración de OBS queda igual.
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use obws::Client;
use obws::common::{Alignment, BoundsType};
use obws::requests::inputs::{Create as CreateInput, InputId, SetSettings};
use obws::requests::scene_items::{
    Bounds, CreateSceneItem, Id as SceneItemLookup, Position, SceneItemTransform, SetEnabled, SetTransform,
};
use obws::requests::sources::{SourceId, TakeScreenshot};
use serde::Serialize;
use serde_json::{Value, json};

pub const SCENE_CAMERA: &str = "Selah · Cámara";
pub const SCENE_CAMERA_TEXT: &str = "Selah · Cámara + texto";
pub const SCENE_WAIT: &str = "Selah · Espera";
const SELAH_SCENES: [&str; 3] = [SCENE_CAMERA, SCENE_CAMERA_TEXT, SCENE_WAIT];

const INPUT_CAMERA: &str = "Selah · Cámara (dispositivo)";
const INPUT_BAND: &str = "Selah · Banda";
const INPUT_TEXT: &str = "Selah · Texto";
const INPUT_CREDIT: &str = "Selah · Créditos";
const INPUT_WAIT_BG: &str = "Selah · Fondo de espera";
const INPUT_WAIT_TEXT: &str = "Selah · Texto de espera";

const DEFAULT_PORT: u16 = 4455;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const PREVIEW_WIDTH: u32 = 640;

pub enum ObsError {
    NotInstalled,
    /// OBS está abierto: si se cambia su configuración ahora, OBS la sobrescribe al cerrar.
    Running,
    Failed(String),
}

impl<E: std::fmt::Display> From<E> for ObsError {
    fn from(e: E) -> Self {
        ObsError::Failed(e.to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    pub installed: bool,
    pub websocket_enabled: bool,
    pub running: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub active: bool,
    pub reconnecting: bool,
    pub duration_ms: i64,
    pub congestion: f32,
    pub skipped_frames: u32,
    pub total_frames: u32,
}

/// Destino de la emisión. La clave nunca sale de OBS: solo se indica si hay una.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    pub kind: String,
    pub server: String,
    pub has_key: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub obs_version: String,
    pub program_scene: Option<String>,
    /// Todas las escenas de OBS, en el orden en que OBS las muestra (las del usuario y las de Selah).
    pub scenes: Vec<String>,
    /// Fuentes de texto de OBS que Selah puede rellenar.
    pub text_sources: Vec<String>,
    /// Las tres escenas de Selah existen en OBS.
    pub selah_ready: bool,
    pub has_camera: bool,
    pub streaming: StreamInfo,
    pub destination: Option<Destination>,
}

#[derive(Serialize)]
pub struct Camera {
    pub id: String,
    pub name: String,
}

// --- Detección, preparación y arranque (sin conexión) ---

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from)
}

/// Carpetas de configuración de OBS posibles, la nativa primero.
fn config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            dirs.push(PathBuf::from(appdata).join("obs-studio"));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = home() {
            dirs.push(home.join("Library/Application Support/obs-studio"));
        }
    } else {
        let config = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).or_else(|| home().map(|h| h.join(".config")));
        if let Some(config) = config {
            dirs.push(config.join("obs-studio"));
        }
        if let Some(home) = home() {
            dirs.push(home.join(".var/app/com.obsproject.Studio/config/obs-studio"));
        }
    }
    dirs
}

fn websocket_config_path() -> Option<PathBuf> {
    let dirs = config_dirs();
    let dir = dirs.iter().find(|d| d.is_dir()).or(dirs.first())?;
    Some(dir.join("plugin_config/obs-websocket/config.json"))
}

fn read_websocket_config() -> Option<Value> {
    let text = std::fs::read_to_string(websocket_config_path()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(name)).find(|p| p.is_file())
}

/// Programa y argumentos para abrir OBS.
fn launcher() -> Option<(PathBuf, Vec<&'static str>)> {
    if cfg!(target_os = "windows") {
        let base = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or_else(|| "C:\\Program Files".into());
        let exe = base.join("obs-studio\\bin\\64bit\\obs64.exe");
        return exe.is_file().then_some((exe, vec![]));
    }
    if cfg!(target_os = "macos") {
        let exe = PathBuf::from("/Applications/OBS.app/Contents/MacOS/OBS");
        return exe.is_file().then_some((exe, vec![]));
    }
    if let Some(exe) = find_in_path("obs") {
        return Some((exe, vec![]));
    }
    let flatpak = find_in_path("flatpak")?;
    let installed = Command::new(&flatpak).args(["info", "com.obsproject.Studio"]).output().ok()?.status.success();
    installed.then_some((flatpak, vec!["run", "com.obsproject.Studio"]))
}

fn is_running() -> bool {
    if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq obs64.exe", "/NH"])
            .output()
            .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("obs64.exe"))
    } else {
        let name = if cfg!(target_os = "macos") { "OBS" } else { "obs" };
        Command::new("pgrep").args(["-x", name]).output().is_ok_and(|o| o.status.success())
    }
}

pub fn detect() -> Detection {
    let config = read_websocket_config();
    Detection {
        installed: launcher().is_some(),
        websocket_enabled: config.as_ref().and_then(|c| c["server_enabled"].as_bool()).unwrap_or(false),
        running: is_running(),
    }
}

/// Activa el servidor WebSocket en la configuración de OBS. Solo con OBS cerrado.
/// Conserva el puerto y la contraseña que hubiera; si no hay contraseña, crea una.
pub fn enable_websocket() -> Result<(), ObsError> {
    if is_running() {
        return Err(ObsError::Running);
    }
    let path = websocket_config_path().ok_or(ObsError::NotInstalled)?;
    let mut config: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    config["server_enabled"] = json!(true);
    config["auth_required"] = json!(true);
    config["alerts_enabled"] = json!(false);
    config["first_load"] = json!(false);
    if config["server_port"].as_u64().is_none() {
        config["server_port"] = json!(DEFAULT_PORT);
    }
    if config["server_password"].as_str().is_none_or(str::is_empty) {
        config["server_password"] = json!(uuid::Uuid::new_v4().simple().to_string()[..16]);
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn launch() -> Result<(), ObsError> {
    if is_running() {
        return Ok(());
    }
    let (program, args) = launcher().ok_or(ObsError::NotInstalled)?;
    let mut command = Command::new(&program);
    // Sin el aviso de «OBS no se cerró bien», que bloquearía el arranque esperando al usuario.
    command.args(args).arg("--disable-shutdown-check");
    // En Windows OBS necesita arrancar desde su propia carpeta.
    if let Some(dir) = program.parent().filter(|_| cfg!(target_os = "windows")) {
        command.current_dir(dir);
    }
    command.spawn()?;
    Ok(())
}

// --- Con conexión ---

pub async fn connect() -> Result<Client, ObsError> {
    let config = read_websocket_config().ok_or(ObsError::NotInstalled)?;
    let port = config["server_port"].as_u64().and_then(|p| u16::try_from(p).ok()).unwrap_or(DEFAULT_PORT);
    let password = config["auth_required"]
        .as_bool()
        .unwrap_or(true)
        .then(|| config["server_password"].as_str().unwrap_or_default().to_string());
    let client = tokio::time::timeout(CONNECT_TIMEOUT, Client::connect("127.0.0.1", port, password))
        .await
        .map_err(|_| ObsError::Failed("OBS no responde".into()))??;
    Ok(client)
}

pub async fn status(client: &Client) -> obws::error::Result<Status> {
    let version = client.general().version().await?;
    let mut scenes = client.scenes().list().await?.scenes;
    // OBS lista las escenas al revés de como las muestra en su ventana.
    scenes.sort_by(|a, b| b.index.cmp(&a.index));
    let names: Vec<String> = scenes.into_iter().map(|s| s.id.name).collect();
    let program = client.scenes().current_program_scene().await.ok().map(|s| s.id.name);
    let inputs = client.inputs().list(None).await?;
    let stream = client.streaming().status().await?;
    let destination = client.config().stream_service_settings::<Value>().await.ok().map(|s| Destination {
        server: s.settings["server"].as_str().unwrap_or_default().to_string(),
        has_key: s.settings["key"].as_str().is_some_and(|k| !k.is_empty()),
        kind: s.r#type,
    });
    let text_sources = inputs
        .iter()
        .filter(|i| is_text_kind(&i.kind) && i.id.name != INPUT_CREDIT && i.id.name != INPUT_WAIT_TEXT)
        .map(|i| i.id.name.clone())
        .collect();
    Ok(Status {
        obs_version: version.obs_studio_version.to_string(),
        program_scene: program,
        selah_ready: SELAH_SCENES.iter().all(|s| names.iter().any(|n| n == s)),
        scenes: names,
        text_sources,
        has_camera: inputs.iter().any(|i| i.id.name == INPUT_CAMERA),
        streaming: StreamInfo {
            active: stream.active,
            reconnecting: stream.reconnecting,
            duration_ms: stream.duration.whole_milliseconds() as i64,
            congestion: stream.congestion,
            skipped_frames: stream.skipped_frames,
            total_frames: stream.total_frames,
        },
        destination,
    })
}

/// Clase de fuente de OBS disponible en este sistema, la primera que exista de la lista.
fn pick_kind(kinds: &[String], wanted: &[&str]) -> Option<String> {
    wanted.iter().find(|w| kinds.iter().any(|k| k == *w)).map(|w| w.to_string())
}

/// Nombre del ajuste del dispositivo de cámara según la clase de fuente.
fn camera_property(kind: &str) -> &'static str {
    match kind {
        "dshow_input" => "video_device_id",
        "v4l2_input" => "device_id",
        _ => "device",
    }
}

fn text_settings(kind: &str, text: &str, size: f32, width: f32, color: u32) -> Value {
    if kind.starts_with("text_gdiplus") {
        // GDI+ usa color BGR sin alfa.
        json!({ "text": text, "font": { "face": "Arial", "size": size as i64 }, "color": color & 0x00FF_FFFF,
                "extents": true, "extents_cx": width as i64, "extents_cy": (size * 3.2) as i64, "extents_wrap": true })
    } else {
        json!({ "text": text, "font": { "face": "Sans Serif", "size": size as i64, "style": "Regular", "flags": 0 },
                "color1": color, "color2": color, "word_wrap": true, "custom_width": width as i64, "drop_shadow": true })
    }
}

/// Crea la fuente en la escena si no existe en OBS; si ya existe, la añade a la escena si falta.
/// Devuelve el id del elemento en esa escena.
async fn ensure_item(
    client: &Client,
    existing_inputs: &[String],
    scene: &str,
    input: &str,
    kind: &str,
    settings: Value,
) -> obws::error::Result<i64> {
    if !existing_inputs.iter().any(|i| i == input) {
        let created = client
            .inputs()
            .create(CreateInput { scene: scene.into(), input, kind, settings: Some(settings), enabled: Some(true) })
            .await?;
        return Ok(created.scene_item_id);
    }
    let lookup = SceneItemLookup { scene: scene.into(), source: input, search_offset: None };
    match client.scene_items().id(lookup).await {
        Ok(id) => Ok(id),
        Err(_) => {
            let create = CreateSceneItem { scene: scene.into(), source: SourceId::Name(input), enabled: Some(true) };
            client.scene_items().create(create).await
        }
    }
}

async fn place(client: &Client, scene: &str, item_id: i64, x: f32, y: f32, bounds: Option<(f32, f32)>, alignment: Alignment) -> obws::error::Result<()> {
    let transform = SceneItemTransform {
        position: Some(Position { x: Some(x), y: Some(y) }),
        alignment: Some(alignment),
        bounds: bounds.map(|(width, height)| Bounds {
            r#type: Some(BoundsType::ScaleInner),
            alignment: Some(Alignment::CENTER),
            width: Some(width),
            height: Some(height),
        }),
        ..Default::default()
    };
    client.scene_items().set_transform(SetTransform { scene: scene.into(), item_id, transform }).await
}

/// Crea o completa las escenas de Selah. Se puede repetir sin duplicar nada.
pub async fn setup(client: &Client, wait_text: &str) -> obws::error::Result<()> {
    let video = client.config().video_settings().await?;
    let (w, h) = (video.base_width as f32, video.base_height as f32);
    let kinds = client.inputs().list_kinds(false).await?;
    let camera_kind = pick_kind(&kinds, &["v4l2_input", "dshow_input", "macos-avcapture", "av_capture_input_v2", "av_capture_input"]);
    let text_kind = pick_kind(&kinds, &["text_ft2_source_v2", "text_gdiplus_v3", "text_gdiplus_v2", "text_gdiplus"])
        .unwrap_or_else(|| "text_ft2_source_v2".into());
    let color_kind = pick_kind(&kinds, &["color_source_v3", "color_source_v2", "color_source"]).unwrap_or_else(|| "color_source_v3".into());

    let scenes: Vec<String> = client.scenes().list().await?.scenes.into_iter().map(|s| s.id.name).collect();
    for scene in SELAH_SCENES {
        if !scenes.iter().any(|s| s == scene) {
            client.scenes().create(scene).await?;
        }
    }
    let mut inputs: Vec<String> = client.inputs().list(None).await?.into_iter().map(|i| i.id.name).collect();
    let top_left = Alignment::TOP | Alignment::LEFT;

    // Cámara a pantalla completa en las dos escenas con cámara.
    if let Some(kind) = &camera_kind {
        for scene in [SCENE_CAMERA, SCENE_CAMERA_TEXT] {
            let id = ensure_item(client, &inputs, scene, INPUT_CAMERA, kind, json!({})).await?;
            inputs.push(INPUT_CAMERA.into());
            place(client, scene, id, 0.0, 0.0, Some((w, h)), top_left).await?;
        }
    }

    // Banda inferior semitransparente con el texto y los créditos encima.
    let band = ensure_item(client, &inputs, SCENE_CAMERA_TEXT, INPUT_BAND, &color_kind,
        json!({ "color": 0xB400_0000u32, "width": w as i64, "height": (h * 0.24) as i64 })).await?;
    place(client, SCENE_CAMERA_TEXT, band, 0.0, h * 0.76, None, top_left).await?;
    let text = ensure_item(client, &inputs, SCENE_CAMERA_TEXT, INPUT_TEXT, &text_kind,
        text_settings(&text_kind, "", h * 0.042, w * 0.9, 0xFFFF_FFFF)).await?;
    place(client, SCENE_CAMERA_TEXT, text, w * 0.05, h * 0.785, None, top_left).await?;
    let credit = ensure_item(client, &inputs, SCENE_CAMERA_TEXT, INPUT_CREDIT, &text_kind,
        text_settings(&text_kind, "", h * 0.018, w * 0.9, 0xFFC8_C8C8)).await?;
    place(client, SCENE_CAMERA_TEXT, credit, w * 0.05, h * 0.955, None, top_left).await?;

    // Pantalla de espera: fondo azul noche y un mensaje centrado.
    let bg = ensure_item(client, &inputs, SCENE_WAIT, INPUT_WAIT_BG, &color_kind,
        json!({ "color": 0xFF30_170Bu32, "width": w as i64, "height": h as i64 })).await?;
    place(client, SCENE_WAIT, bg, 0.0, 0.0, None, top_left).await?;
    let wait = ensure_item(client, &inputs, SCENE_WAIT, INPUT_WAIT_TEXT, &text_kind,
        text_settings(&text_kind, wait_text, h * 0.07, w * 0.8, 0xFFFF_FFFF)).await?;
    place(client, SCENE_WAIT, wait, w / 2.0, h / 2.0, None, Alignment::CENTER).await?;
    set_input_text(client, INPUT_WAIT_TEXT, wait_text).await
}

async fn set_input_text(client: &Client, input: &str, text: &str) -> obws::error::Result<()> {
    let settings = json!({ "text": text });
    client.inputs().set_settings(SetSettings { input: InputId::Name(input), settings: &settings, overlay: Some(true) }).await
}

/// Cambia el texto de la banda. No cambia de escena: eso lo decide el operador.
fn is_text_kind(kind: &str) -> bool {
    kind.starts_with("text_ft2_source") || kind.starts_with("text_gdiplus")
}

/// Dónde quedó visible el texto al ponerlo.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TextShown {
    /// En la escena que ya estaba al aire.
    Current,
    /// La escena al aire no tenía esa fuente: se cambió a una que sí la tiene.
    Switched { scene: String },
    /// Ninguna escena contiene esa fuente: el texto se cambió pero no se ve.
    NotInScene,
}

async fn item_in(client: &Client, scene: &str, source: &str) -> Option<i64> {
    client.scene_items().id(SceneItemLookup { scene: scene.into(), source, search_offset: None }).await.ok()
}

async fn set_item_visible(client: &Client, scene: &str, item_id: i64, enabled: bool) -> obws::error::Result<()> {
    client.scene_items().set_enabled(SetEnabled { scene: scene.into(), item_id, enabled }).await
}

/// Escribe el texto en la fuente elegida (de Selah o del usuario) y la hace visible en la emisión.
/// Los créditos van en su propia fuente con el diseño de Selah; con fuentes del usuario, en la última línea.
pub async fn show_text(client: &Client, source: &str, text: &str, credit: &str) -> obws::error::Result<TextShown> {
    if source == INPUT_TEXT {
        set_input_text(client, INPUT_TEXT, text).await?;
        set_input_text(client, INPUT_CREDIT, credit).await?;
    } else if credit.is_empty() {
        set_input_text(client, source, text).await?;
    } else {
        set_input_text(client, source, &format!("{text}\n{credit}")).await?;
    }

    let program = client.scenes().current_program_scene().await?.id.name;
    if let Some(item) = item_in(client, &program, source).await {
        set_item_visible(client, &program, item, true).await?;
        return Ok(TextShown::Current);
    }
    for scene in client.scenes().list().await?.scenes {
        let name = scene.id.name;
        if let Some(item) = item_in(client, &name, source).await {
            set_item_visible(client, &name, item, true).await?;
            client.scenes().set_current_program_scene(name.as_str()).await?;
            return Ok(TextShown::Switched { scene: name });
        }
    }
    Ok(TextShown::NotInScene)
}

/// Oculta la fuente de texto en la escena al aire. Con el diseño de Selah se vuelve a «Cámara»,
/// para no dejar la banda vacía.
pub async fn hide_text(client: &Client, source: &str) -> obws::error::Result<()> {
    let program = client.scenes().current_program_scene().await?.id.name;
    if source == INPUT_TEXT && program == SCENE_CAMERA_TEXT {
        return client.scenes().set_current_program_scene(SCENE_CAMERA).await;
    }
    match item_in(client, &program, source).await {
        Some(item) => set_item_visible(client, &program, item, false).await,
        None => Ok(()),
    }
}

pub async fn set_scene(client: &Client, scene: &str) -> obws::error::Result<()> {
    client.scenes().set_current_program_scene(scene).await
}

pub async fn cameras(client: &Client) -> obws::error::Result<Vec<Camera>> {
    let inputs = client.inputs().list(None).await?;
    let Some(input) = inputs.iter().find(|i| i.id.name == INPUT_CAMERA) else { return Ok(Vec::new()) };
    let items = client.inputs().properties_list_property_items(InputId::Name(INPUT_CAMERA), camera_property(&input.kind)).await?;
    Ok(items
        .into_iter()
        .filter(|i| i.enabled)
        .filter_map(|i| Some(Camera { id: i.value.as_str()?.to_string(), name: i.name }))
        .collect())
}

pub async fn set_camera(client: &Client, device: &str) -> obws::error::Result<()> {
    let inputs = client.inputs().list(None).await?;
    let Some(input) = inputs.iter().find(|i| i.id.name == INPUT_CAMERA) else { return Ok(()) };
    let settings = json!({ camera_property(&input.kind): device });
    client.inputs().set_settings(SetSettings { input: InputId::Name(INPUT_CAMERA), settings: &settings, overlay: Some(true) }).await
}

/// Imagen de lo que está saliendo en la emisión, como data URL JPEG.
pub async fn preview(client: &Client) -> obws::error::Result<Option<String>> {
    let Ok(scene) = client.scenes().current_program_scene().await else { return Ok(None) };
    let shot = TakeScreenshot {
        source: SourceId::Name(&scene.id.name),
        format: "jpg",
        width: Some(PREVIEW_WIDTH),
        height: None,
        compression_quality: Some(70),
    };
    client.sources().take_screenshot(shot).await.map(Some)
}

/// RTMP personalizado. Si la clave llega vacía se conserva la que ya tenía OBS.
pub async fn set_destination(client: &Client, server: &str, key: &str) -> obws::error::Result<()> {
    let current = client.config().stream_service_settings::<Value>().await.ok();
    let key = if key.is_empty() {
        current.as_ref().and_then(|c| c.settings["key"].as_str()).unwrap_or_default().to_string()
    } else {
        key.to_string()
    };
    client.config().set_stream_service_settings("rtmp_custom", &json!({ "server": server, "key": key })).await
}
