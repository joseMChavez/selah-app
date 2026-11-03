// Selector de archivos como diálogo hijo de la consola: el gestor de ventanas lo pone encima y centrado.
// En Linux, el diálogo de tauri-plugin-dialog (rfd con GTK) se crea en un hilo aparte e ignora la ventana
// padre; en Hyprland o sway quedaba en mosaico debajo de las ventanas flotantes y parecía que no se abría.
// Aquí se crea en el hilo de GTK de la app, con transient_for. En Windows y macOS se usa el plugin con su padre.
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::state::MAIN_LABEL;

pub enum Mode {
    /// Uno o varios archivos.
    Files,
    File,
    Folder,
}

pub struct Request {
    /// Título traducido por la interfaz.
    pub title: String,
    /// Nombre del filtro y extensiones (sin punto). None = cualquier archivo.
    pub filter: Option<(String, Vec<String>)>,
    pub mode: Mode,
}

/// Rutas elegidas; None si se cancela.
pub async fn pick(app: &AppHandle, request: Request) -> Result<Option<Vec<PathBuf>>, String> {
    eprintln!("[selector] abierto: {}", request.title);
    let picked = open(app, request).await;
    match &picked {
        Ok(Some(paths)) => eprintln!("[selector] elegidos: {}", paths.len()),
        Ok(None) => eprintln!("[selector] cancelado"),
        Err(e) => eprintln!("[selector] error: {e}"),
    }
    picked
}

#[cfg(target_os = "linux")]
async fn open(app: &AppHandle, request: Request) -> Result<Option<Vec<PathBuf>>, String> {
    let window = app.get_webview_window(MAIN_LABEL).ok_or("no está la consola")?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let parent = window.clone();
    window
        .run_on_main_thread(move || {
            use gtk::prelude::*;
            let action = match request.mode {
                Mode::Folder => gtk::FileChooserAction::SelectFolder,
                Mode::Files | Mode::File => gtk::FileChooserAction::Open,
            };
            let parent = parent.gtk_window().ok();
            // Sin etiquetas propias: GTK usa «Abrir» y «Cancelar» en el idioma del sistema.
            let dialog = gtk::FileChooserNative::new(Some(&request.title), parent.as_ref(), action, None, None);
            dialog.set_modal(true);
            dialog.set_select_multiple(matches!(request.mode, Mode::Files));
            if let Some((name, extensions)) = &request.filter {
                let filter = gtk::FileFilter::new();
                filter.set_name(Some(name));
                for ext in extensions {
                    filter.add_pattern(&format!("*.{ext}"));
                }
                dialog.add_filter(filter);
            }
            let result = (dialog.run() == gtk::ResponseType::Accept).then(|| dialog.filenames());
            let _ = tx.send(result);
        })
        .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
async fn open(app: &AppHandle, request: Request) -> Result<Option<Vec<PathBuf>>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = app.dialog().file().set_title(request.title);
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        dialog = dialog.set_parent(&window);
    }
    if let Some((name, extensions)) = &request.filter {
        let extensions: Vec<&str> = extensions.iter().map(String::as_str).collect();
        dialog = dialog.add_filter(name, &extensions);
    }
    let picked = match request.mode {
        Mode::Files => dialog.blocking_pick_files(),
        Mode::File => dialog.blocking_pick_file().map(|f| vec![f]),
        Mode::Folder => dialog.blocking_pick_folder().map(|f| vec![f]),
    };
    picked
        .map(|files| files.into_iter().map(|f| f.into_path().map_err(|e| e.to_string())).collect::<Result<Vec<_>, _>>())
        .transpose()
}
