// Cada funcionalidad es un plugin interno de Tauri con sus comandos, su estado y su permiso
// (build.rs los declara leyendo los `commands.rs`). Ver docs/ARQUITECTURA.md.
pub mod bible;
pub mod media;
pub mod obs;
pub mod output;
pub mod presentations;
pub mod remote;
pub mod settings;
pub mod songs;
pub mod transfer;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// Cada #[tauri::command] de un `commands.rs` tiene que estar en el generate_handler! de su `mod.rs`:
    /// si falta, compila pero la interfaz recibe «command not found».
    #[test]
    fn todos_los_comandos_estan_registrados() {
        let features = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/features");
        for dir in fs::read_dir(&features).unwrap().flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
            let Ok(commands) = fs::read_to_string(dir.join("commands.rs")) else { continue };
            let module = fs::read_to_string(dir.join("mod.rs")).unwrap();
            let mut marked = false;
            for line in commands.lines().map(str::trim) {
                if line.starts_with("#[tauri::command") {
                    marked = true;
                } else if marked && line.starts_with("pub ") {
                    let name = line.trim_start_matches("pub ").trim_start_matches("async ").trim_start_matches("fn ");
                    let name = name.split(['(', '<']).next().unwrap();
                    assert!(
                        module.contains(&format!("commands::{name},")),
                        "{name} no está registrado en {}/mod.rs",
                        dir.display()
                    );
                    marked = false;
                }
            }
        }
    }
}
