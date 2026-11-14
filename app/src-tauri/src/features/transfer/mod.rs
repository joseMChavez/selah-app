// Importar y exportar letras: el formato propio de EmetPro (.emetpro; también lee los .selah de antes) y las listas de Holyrics (.hpls).
// Plugin interno «transfer»: la interfaz llama a sus comandos como `plugin:transfer|<comando>` (platform/tauri.ts).
//
// Sirve de ejemplo de funcionalidad autónoma (docs/ARQUITECTURA.md): todo lo suyo está en esta carpeta y en
// app/src/lib/features/transfer/. Fuera solo se registra (lib.rs, features/mod.rs, capabilities, la lista de
// paneles) y usa piezas públicas de otras: la base (core::db), los tipos y la escritura de letras (songs).
mod commands;
pub mod domain;
pub mod formats;
mod repo;

use std::sync::Mutex;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Wry};

use self::domain::Incoming;
use self::formats::emetpro::Background;

pub const NAME: &str = "transfer";

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new(NAME)
        .invoke_handler(tauri::generate_handler![
            commands::transfer_import_pick,
            commands::transfer_import_apply,
            commands::transfer_import_cancel,
            commands::transfer_export,
        ])
        .setup(|app, _api| {
            app.manage(TransferState::default());
            Ok(())
        })
        .build()
}

/// Un archivo leído que espera a que el usuario confirme la importación.
pub struct Pending {
    pub file: String,
    pub format: &'static str,
    pub songs: Vec<Incoming>,
    /// Índices de `songs` en el orden de la lista del culto.
    pub setlist: Vec<usize>,
    pub backgrounds: Vec<Background>,
}

#[derive(Default)]
pub struct TransferState(pub Mutex<Option<Pending>>);
