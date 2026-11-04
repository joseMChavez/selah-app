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

## Icono

La fuente es `app/src-tauri/icons/icon.svg` (también `app/static/favicon.svg`). Tras cambiarla, se regeneran todos los tamaños:
```sh
cd app
rsvg-convert -w 1024 -h 1024 src-tauri/icons/icon.svg -o /tmp/selah-icon.png
npx tauri icon /tmp/selah-icon.png -o src-tauri/icons
rm -rf src-tauri/icons/android src-tauri/icons/ios src-tauri/icons/64x64.png   # Selah no los usa
cp src-tauri/icons/icon.svg static/favicon.svg
rsvg-convert -w 128 -h 128 static/favicon.svg -o static/favicon.png
```

## Avisos al instalar

Los instaladores no están firmados:
- **Windows:** SmartScreen dice «Windows protegió su PC» → «Más información» → «Ejecutar de todas formas».
- **macOS:** clic derecho sobre Selah en Aplicaciones → «Abrir» → «Abrir». Si dice que está dañado: `xattr -cr /Applications/Selah.app`.
- **Linux:** el AppImage necesita permiso de ejecución (`chmod +x Selah_*.AppImage`).

## Historial

| Etiqueta | Resultado | Qué pasó |
|---|---|---|
| `v0.2.0` | ❌ Fallida | El código seguía en `0.1.0` y el flujo exigía que la etiqueta coincidiera con la versión de los archivos. No compiló nada. |
| `v0.2.1` | ❌ Fallida | La misma causa. Aprendizaje: el flujo pasó a tomar la versión de la etiqueta (PR #5). |
| `v0.2.2` | ⚠️ Parcial | Windows y macOS compilaron; Linux falló al instalar dependencias: `libgstreamer1.0-dev` choca con libunwind en ubuntu-22.04. Aprendizaje: instalar solo los paquetes necesarios (PR #6). Queda un borrador sin instalador de Linux. |
| `v0.2.3` | Pendiente | Primera versión con instaladores para los tres sistemas. |
| `v0.3.0` | Pendiente | Letras (biblioteca, lista del culto, fondos de imagen o GIF, tipos de letra), Biblia en dos versiones, reloj y temporizador, presentaciones de PowerPoint (con LibreOffice), logo como marca de agua, «Enviar a pantalla» abre la salida sola, y ajustes y letras guardados en SQLite con opción de restablecer (D-014). |
| `v0.3.1` | ❌ Descartada | La etiqueta se creó sobre `06dcd44` (el código de la 0.3.0) antes de fusionar #9 y #10: los instaladores dicen 0.3.1 pero no traen PowerPoint ni Keynote. Aprendizaje: subir la versión en `main` y etiquetar ese commit. Su borrador no se publica. |
| `v0.3.2` | Pendiente | Presentaciones también con PowerPoint (Windows), Keynote o PowerPoint (macOS), no solo con LibreOffice. Actions con Node 24. |
| `v0.3.3` | Pendiente | Keynote y PowerPoint en macOS: el archivo se pasa como alias para que la app, en sandbox, pueda abrirlo (antes Keynote decía que no podía abrir el archivo). |
| `v0.4.0` | Pendiente | Presentaciones con LibreOffice primero; si no hay, Selah ofrece descargar su propia copia (26.2.6, SHA-256 comprobado) sin instalarla en el sistema, y se puede borrar desde Ajustes (D-015). |
| `v0.5.0` | Pendiente | Importar y exportar letras: formato propio `.selah` (con lista del culto y fondos) y listas de Holyrics `.hpls`, con vista previa y elección para las repetidas (D-017). Preguntar antes de enviar a la pantalla de la iglesia. OBS: ya no arranca en modo seguro tras un cierre forzado, y si lo está, Selah lo explica. Una línea con `**` separa diapositivas. Código organizado por funcionalidad, cada una como plugin interno (D-016). |
| `v0.6.0` | Pendiente | Control remoto desde el teléfono (D-018): se activa en Ajustes y se vincula con un QR en la misma red wifi. Busca y muestra versículos, cambia la versión de la Biblia, avanza o retrocede versículos y diapositivas, pone la pantalla en negro y vuelve a lo reciente (la presentación en la página donde quedó). Pide confirmación si la consola la pide. En pantalla completa y sin zoom. Puede requerir abrir los puertos 7300–7309 en el cortafuegos. |
