# Publicar una versión

Los instaladores se compilan en GitHub Actions (`.github/workflows/instaladores.yml`) y se publican en [Releases](https://github.com/joseMChavez/selah-app/releases). Ver D-013.

| Sistema | Archivos |
|---|---|
| Linux | `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE), `.AppImage` (cualquier distribución) |
| Windows | `.msi` y `-setup.exe` (NSIS) |
| macOS | `.dmg` universal (Intel y Apple Silicon) |

## Pasos

1. Con todo fusionado en `main`, crea y sube la etiqueta con la forma `vX.Y.Z`:
   ```sh
   git switch main && git pull
   git tag v0.2.2
   git push origin v0.2.2
   ```
   Los instaladores toman la versión de la etiqueta (`v0.2.2` → `0.2.2`); no hace falta cambiar ningún archivo.
2. Actions compila los tres sistemas (unos 15–25 min) y deja un **borrador** en Releases. Si la etiqueta no tiene la forma `vX.Y.Z`, falla al principio.
3. Revisa el borrador, prueba al menos un instalador y pulsa **Publish release**.
4. Opcional: para que la versión del código coincida, actualízala en `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml` y `app/package.json` en un PR. Solo afecta a las compilaciones locales.

Una etiqueta compila el código del commit al que apunta. Si una versión falla y hay que corregir algo, sube la siguiente etiqueta (`v0.2.3`) desde el `main` corregido.

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
