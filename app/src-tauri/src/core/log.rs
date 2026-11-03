// Registro de la app: los errores de la interfaz también se escriben aquí, junto a los del backend.

// Errores de la interfaz al registro de la app: sin esto, un fallo en el webview no deja rastro.
#[tauri::command]
pub fn log_client_error(window: tauri::Window, message: String) {
    eprintln!("[interfaz:{}] {message}", window.label());
}
