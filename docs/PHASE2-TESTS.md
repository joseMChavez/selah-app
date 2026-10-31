# Fase 2 — Pruebas técnicas verticales

Cada prueba se repite en Linux, Windows y macOS (D-006). Los resultados alimentan los criterios para abandonar Tauri (D-010).

| # | Prueba | Linux | Windows | macOS |
|---|---|---|---|---|
| 1 | Consola + salida a pantalla completa en monitor elegido | en curso | pendiente | pendiente |
| 2 | Imagen, video y PDF en la salida | en curso | pendiente | pendiente |
| 3 | Cámara y audio en el webview | pendiente | pendiente | pendiente |
| 4 | OBS por obs-websocket: escena con cámara + texto, emisión de prueba | pendiente | pendiente | pendiente |
| 5 | Pasaje desde una Biblia local e importación de un archivo permitido | pendiente | pendiente | pendiente |

## Prueba 1 — Salida en monitor secundario

**Implementación:** `app/src-tauri/src/lib.rs`. En Linux se usa `gtk_window.fullscreen_on_monitor()` porque Wayland no permite posicionar ventanas; en Windows y macOS, `set_position` + `set_fullscreen`. La interfaz solo habla con el backend a través de `app/src/lib/platform/` (D-010).

**Cómo ejecutar:** `cd app && npm run tauri dev`.

### Lista de comprobación

| Caso | Resultado esperado | Linux (Hyprland/Wayland) |
|---|---|---|
| a. Detectar monitores | La consola lista todos los monitores con nombre, resolución y escala | |
| b. Abrir salida en monitor secundario | Ventana sin bordes, a pantalla completa, solo en el monitor elegido; la consola no se mueve | |
| c. Cobertura completa | Con «Patrón de prueba», el borde rojo y las 4 esquinas amarillas se ven enteros; la resolución mostrada coincide con la del monitor | |
| d. Mover salida al otro monitor | La salida cambia de monitor sin cerrarse | |
| e. Preparar sin enviar | Editar el texto cambia la vista previa, no la pantalla | |
| f. Enviar a pantalla | El texto aparece en la salida y en «En pantalla» | |
| g. Pantalla en negro | La salida queda en negro y la consola lo indica con texto | |
| h. Desconectar el monitor de salida | La consola avisa en menos de 3 s; la app no se cierra | |
| i. Reconectar | El monitor vuelve a la lista y la salida se puede abrir otra vez | |
| j. Cerrar la salida | La ventana desaparece y el estado pasa a «Salida cerrada» | |
| k. Cerrar la consola | La app termina, incluida la salida | |
| l. Cursor | El puntero no se ve sobre la salida | |
| m. Escalas distintas | Con monitores de escala distinta (1.5 y 1.0) el texto se ve proporcionado en ambos | |

### Notas y hallazgos

- **2026-10-31 — Dibujo y clics desplazados en WebKitGTK (Linux).** Con Hyprland/Wayland, escala 1,5, `GDK_SCALE=2` y gráfica híbrida Intel + NVIDIA GTX 1650 Ti, WebKitGTK 2.52 pintaba elementos fuera de sitio y los clics caían en el botón vecino o en capas invisibles. Causa: el renderizador DMA-BUF. Solución: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, que la app activa sola al arrancar (`src-tauri/src/main.rs`). Además, la vista de salida usa `contain: strict` y las vistas previas ignoran el ratón, para que ningún contenido pueda tapar la consola. Revisar en otros equipos Linux (D-010: criterio «pantalla completa estable en Linux»).

## Prueba 2 — Imagen, video y PDF en la salida

**Implementación:** los archivos se eligen con el diálogo del sistema (`tauri-plugin-dialog`) y se sirven con el protocolo `asset`, que empieza sin acceso a nada: el backend solo permite cada archivo elegido. Antes de enviar a pantalla se comprueba que el archivo siga existiendo. Los PDF se dibujan con pdf.js (`pdfjs-dist`, Apache-2.0) porque WebKit no los muestra dentro de la página. El video usa `<video>` y depende de los códecs del sistema: GStreamer en Linux, Media Foundation en Windows, AVFoundation en macOS.

**Archivos de prueba:** `samples/media/` (ignorado por git), generados con ffmpeg: H.264/AAC en .mp4 y .mov, VP9/Opus y AV1/Opus en .webm, HEVC/AAC en .mp4, imágenes PNG 1080p, JPG vertical, WebP de 12 MP, un JPG corrupto y un PDF de 3 páginas con una apaisada.

### Lista de comprobación

| Caso | Resultado esperado | Linux (Hyprland/Wayland) |
|---|---|---|
| a. Imagen PNG/JPG/WebP | Se ve entera, sin deformar, con bandas negras si la proporción no es 16:9 | |
| b. Imagen de 12 MP | Aparece en menos de 1 s | |
| c. Archivo corrupto | La consola muestra «La salida informa: No se pudo mostrar la imagen…»; la app sigue funcionando | |
| d. Video H.264 .mp4 y .mov | Se reproduce con sonido en la salida y sin sonido en la consola | |
| e. Video VP9 y AV1 (.webm) | Se reproduce, o la consola informa de códec no soportado | |
| f. Video HEVC | Se reproduce, o la consola informa de códec no soportado | |
| g. Pausar y reanudar | El video se detiene y continúa desde el mismo punto | |
| h. Fluidez | Video 1080p30 sin saltos visibles; anotar uso de CPU (`top`) | |
| i. PDF: preparar página | Las flechas de «Preparado» cambian la vista previa, no la pantalla | |
| j. PDF: página siguiente en pantalla | La salida cambia de página al instante; la página apaisada se ve entera | |
| k. Archivo borrado antes de enviar | Al pulsar «Enviar a pantalla» aparece «El archivo ya no existe…» y la pantalla no cambia | |
| l. Cambiar de video a texto | El video se detiene y deja de sonar | |

### Notas y hallazgos

**Añadido durante la prueba:** texto en diapositivas separadas por `***`, navegación de lo que está en pantalla (botones, miniaturas, flechas ← →) e interfaz en español, inglés e italiano. Las flechas fallaban porque WebKit no quita el foco del cuadro de texto al pulsar un botón y porque los botones de monitor las capturaban; corregido, pendiente de confirmar.
