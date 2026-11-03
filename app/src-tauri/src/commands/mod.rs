// Capa de aplicación: los comandos que llama la interfaz (app/src/lib/platform/tauri.ts).
// Son finos: validan lo que llega, llaman al dominio o a la infraestructura y traducen errores.
pub mod bible;
pub mod media;
pub mod obs;
pub mod office;
pub mod output;
pub mod settings;
pub mod songs;
