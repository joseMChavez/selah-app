# Fase 2 — Pruebas técnicas verticales

Cada prueba se repite en Linux, Windows y macOS (D-006). Los resultados alimentan los criterios para abandonar Tauri (D-010).

| # | Prueba | Linux | Windows | macOS |
|---|---|---|---|---|
| 1 | Consola + salida a pantalla completa en monitor elegido | en curso | pendiente | pendiente |
| 2 | Imagen, video y PDF en la salida | pendiente | pendiente | pendiente |
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
