// Nombres de ventanas y de eventos que comparten el backend y la interfaz.

pub const MAIN_LABEL: &str = "main";

pub const OUTPUT_LABEL: &str = "output";

pub const OUTPUT_STATE_EVENT: &str = "output-state";

pub const OUTPUT_ERROR_EVENT: &str = "output-error";

/// Teclas de avance pulsadas en la ventana de salida: las aplica la consola, que sabe qué hay en pantalla.
pub const OUTPUT_NAVIGATE_EVENT: &str = "output-navigate";

/// Lleva todos los ajustes tras cada cambio: así la ventana de salida sigue el idioma de la consola.
pub const SETTINGS_EVENT: &str = "settings-changed";

/// Órdenes del control remoto (teléfono) para la consola: las aplica ella, que sabe qué hay en pantalla.
pub const REMOTE_COMMAND_EVENT: &str = "remote-command";

/// Cuántos teléfonos están conectados al control remoto.
pub const REMOTE_CLIENTS_EVENT: &str = "remote-clients";

/// Cambios en la pizarra del control remoto, para la consola y la salida (remote/board.rs).
pub const REMOTE_BOARD_EVENT: &str = "remote-board";
