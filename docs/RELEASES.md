# Publicar una versión

Los instaladores se compilan en GitHub Actions (`.github/workflows/instaladores.yml`) y se publican en [Releases](https://github.com/joseMChavez/selah-app/releases). Ver D-013.

| Sistema | Archivos |
|---|---|
| Linux | `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE), `.AppImage` (cualquier distribución) |
| Windows | `.msi` y `-setup.exe` (NSIS) |
| macOS | `.dmg` universal (Intel y Apple Silicon) |

## Pasos

1. Desde `main`, con todo fusionado, cambia la versión en los tres archivos:
   - `app/src-tauri/tauri.conf.json` → `"version"`
   - `app/src-tauri/Cargo.toml` → `version`
   - `app/package.json` → `"version"`
2. Haz commit del cambio de versión y fusiónalo en `main`.
3. Crea y sube la etiqueta, que debe coincidir con la versión:
   ```sh
   git tag v0.2.0
   git push origin v0.2.0
   ```
4. Actions compila los tres sistemas (unos 15–25 min) y deja un **borrador** en Releases. Si la etiqueta no coincide con la versión, falla al principio.
5. Revisa el borrador, prueba al menos un instalador y pulsa **Publish release**.

## Compilar sin publicar

En GitHub: **Actions → Instaladores → Run workflow**. Los instaladores quedan como artefactos de esa ejecución, durante 90 días.

En local (solo Linux, e incluye las Biblias de `resources/bibles/`):
```sh
cd app
npm run tauri build
```
Los archivos quedan en `app/src-tauri/target/release/bundle/`.

## Avisos al instalar

Los instaladores no están firmados:
- **Windows:** SmartScreen dice «Windows protegió su PC» → «Más información» → «Ejecutar de todas formas».
- **macOS:** clic derecho sobre Selah en Aplicaciones → «Abrir» → «Abrir». Si dice que está dañado: `xattr -cr /Applications/Selah.app`.
- **Linux:** el AppImage necesita permiso de ejecución (`chmod +x Selah_*.AppImage`).
