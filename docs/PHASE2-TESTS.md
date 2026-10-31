# Fase 2 — Pruebas técnicas verticales

Cada prueba se repite en Linux, Windows y macOS (D-006). Los resultados alimentan los criterios para abandonar Tauri (D-010).

| # | Prueba | Linux | Windows | macOS |
|---|---|---|---|---|
| 1 | Consola + salida a pantalla completa en monitor elegido | en curso | pendiente | pendiente |
| 2 | Imagen, video y PDF en la salida | en curso | pendiente | pendiente |
| 3 | Cámara y audio en el webview | pendiente | pendiente | pendiente |
| 4 | OBS por obs-websocket: escena con cámara + texto, emisión de prueba | en curso | pendiente | pendiente |
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

- Selah también usa las escenas y fuentes de texto que ya existan en OBS: lista todas las escenas y deja elegir la fuente de texto que recibe el texto. Los diseños de Selah pasan a ser opcionales. Pendiente de probar con escenas propias del usuario (en esta prueba OBS solo tenía las de Selah).
- La conexión automática al abrir Selah reintenta 4 veces, cada 2 s: al reiniciar la app, el primer intento podía fallar.

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

## Prueba 4 — Transmisión con OBS (obs-websocket 5)

**Implementación:** `app/src-tauri/src/obs.rs` con el crate `obws` (MIT). Selah detecta OBS, activa su WebSocket editando `plugin_config/obs-websocket/config.json` (solo con OBS cerrado y con un botón explícito; conserva puerto y contraseña, crea una si no hay), abre OBS con `--disable-shutdown-check` y se conecta a `127.0.0.1`. Crea tres escenas propias (`Selah · Cámara`, `Selah · Cámara + texto`, `Selah · Espera`) y no toca nada más. La clave de emisión no sale de OBS. Vista previa: captura JPEG de la escena al aire cada segundo.

**Respuesta a D-008 («activar WebSocket sin abrir OBS manualmente»):** sí, editando su configuración con OBS cerrado. Con OBS abierto no sirve: OBS sobrescribe la configuración al salir.

**Emisión de prueba sin internet:** servidor RTMP local con ffmpeg:
`ffmpeg -listen 1 -i rtmp://127.0.0.1:1935/live/selah -c copy -f flv emision-prueba.flv`
Destino en Selah: servidor `rtmp://127.0.0.1:1935/live`, clave `selah`.

### Lista de comprobación

| Caso | Resultado esperado | Linux (Hyprland/Wayland) |
|---|---|---|
| a. Detectar OBS | Selah sabe si OBS está instalado, abierto y con WebSocket activo |✅ |
| b. Activar WebSocket | Con OBS cerrado se activa; con OBS abierto, Selah pide cerrarlo |✅ activado desde Selah con OBS cerrado |
| c. Abrir OBS y conectar | Un botón abre OBS y Selah se conecta solo en menos de 25 s |✅ |
| d. Crear diseños | Aparecen las 3 escenas de Selah con cámara, banda y espera; repetir no duplica nada |✅ (duplicado sin comprobar) |
| e. Cámara | La lista muestra la webcam; cambiarla se ve en la vista previa |⚠️ sin webcam: v4l2 «Unable to open device» |
| f. Cambiar de diseño | La escena cambia en OBS y la vista previa lo refleja en ~1 s | |
| g. Texto en la emisión | Poner/quitar texto cambia la emisión; la pantalla del templo no cambia | |
| h. Copiar desde el templo | Copia el versículo en pantalla con referencia y créditos; luego cambiar la proyección no cambia el texto copiado | |
| i. Emitir | Iniciar envía al servidor RTMP local; la barra superior muestra AL AIRE y la duración |✅ H.264 1280×720 30 fps + AAC estéreo, >1 min estable |
| j. Finalizar | Pide un segundo clic; el archivo `emision-prueba.flv` se reproduce | |
| k. OBS se cierra | Selah avisa de la pérdida de conexión y no se bloquea | |
| l. Independencia | Cambiar la proyección no cambia la emisión y viceversa | |

### Notas y hallazgos
