// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK con el renderizador DMA-BUF dibuja y recibe clics en posiciones equivocadas en
    // Wayland con escalado fraccionario y gráficas híbridas/NVIDIA. Se desactiva salvo que el
    // usuario lo haya decidido ya con la variable de entorno.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // SAFETY: se ejecuta antes de crear cualquier hilo; nadie más lee el entorno a la vez.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
    app_lib::run()
}
