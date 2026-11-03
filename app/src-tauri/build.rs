// Cada carpeta de src/features es un plugin interno de Tauri. Tauri exige declarar sus comandos para darles
// permiso: se leen aquí de su `commands.rs` (las funciones con #[tauri::command]), así no hay que mantener
// una lista a mano. El permiso «<plugin>:default» permite todos sus comandos (capabilities/default.json).
use std::fs;
use std::path::Path;

fn command_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut marked = false;
    for line in source.lines().map(str::trim) {
        if line.starts_with("#[tauri::command") {
            marked = true;
        } else if marked && (line.starts_with("pub fn ") || line.starts_with("pub async fn ")) {
            let rest = line.trim_start_matches("pub ").trim_start_matches("async ").trim_start_matches("fn ");
            names.push(rest.split(['(', '<']).next().unwrap_or_default().to_string());
            marked = false;
        }
    }
    names
}

fn main() {
    let mut attributes = tauri_build::Attributes::new();
    let features = Path::new("src/features");
    println!("cargo:rerun-if-changed=src/features");
    let mut dirs: Vec<_> = fs::read_dir(features).expect("src/features").flatten().map(|e| e.path()).collect();
    dirs.sort();
    for dir in dirs.into_iter().filter(|d| d.is_dir()) {
        let commands = dir.join("commands.rs");
        println!("cargo:rerun-if-changed={}", commands.display());
        let Ok(source) = fs::read_to_string(&commands) else { continue };
        let name: &'static str = dir.file_name().unwrap().to_string_lossy().into_owned().leak();
        let names: Vec<&'static str> = command_names(&source).into_iter().map(|n| &*n.leak()).collect();
        attributes = attributes.plugin(
            name,
            tauri_build::InlinedPlugin::new()
                .commands(names.leak())
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        );
    }
    tauri_build::try_build(attributes).expect("tauri-build");
}
