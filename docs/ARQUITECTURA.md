# Arquitectura

Selah tiene dos mitades que solo se hablan por comandos y eventos de Tauri:

- **Interfaz** (`app/src`): SvelteKit con Svelte 5. Nunca importa Tauri directamente: todo pasa por el adaptador `app/src/lib/platform` (D-010).
- **Backend** (`app/src-tauri/src`): Rust. Abre ventanas, lee archivos, guarda en SQLite y habla con OBS, LibreOffice, PowerPoint y Keynote.

## Backend por capas

```
src/
  lib.rs        arranque: módulos, estado inicial, lista de comandos (generate_handler!)
  main.rs       solo llama a lib::run()
  error.rs      CommandError: los errores que recibe la interfaz, como código + datos
  state.rs      AppState (lo compartido entre comandos), nombres de ventanas y eventos, ayudas comunes
  commands/     capa de aplicación: los #[tauri::command] que llama la interfaz
  domain/       tipos y reglas puras de Selah
  infra/        lo que toca el mundo exterior
```

| Capa | Qué va | Puede usar | No debe usar |
|---|---|---|---|
| `domain/` | Tipos (`OutputContent`, `OutputStyle`, `Song`, `MediaKind`) y reglas que se puedan probar sin nada alrededor | `serde`, `std` | Tauri, SQLite, red, procesos, `infra`, `commands` |
| `infra/` | Base de datos (`store.rs`), Biblias XML (`bible.rs`), OBS (`obs.rs`), conversión de presentaciones (`office.rs`), copia de LibreOffice (`libreoffice.rs`), selector de archivos (`picker.rs`) | `domain`, crates externos | `commands` |
| `commands/` | Un archivo por área de la interfaz. Cada comando valida lo que llega, llama a `domain`/`infra` y convierte errores a `CommandError` | todo | — |
| `state.rs`, `error.rs` | Piezas compartidas por los comandos | `domain`, `infra` | `commands` |

Las dependencias van siempre hacia abajo: `commands → infra → domain`. Si algo de `domain` necesita la base de datos o Tauri, no es dominio: va en `infra` o en el comando.

Cuando una operación tenga varios pasos (por ejemplo, importar letras: leer el archivo, validar, detectar repetidas, guardar), su lógica va en una función de `domain` o en un módulo nuevo `services/`, y el comando solo la llama. Así se puede probar sin abrir la app.

### Dónde está cada cosa

| Quiero cambiar… | Archivo |
|---|---|
| Qué se puede mostrar en pantalla, estilo, logo | `domain/output.rs`; comandos en `commands/output.rs` |
| Tipos de archivo admitidos | `domain/media.rs` |
| Elegir archivos, fondos, convertir presentaciones | `commands/media.rs` |
| Conversión a PDF (LibreOffice, PowerPoint, Keynote) | `infra/office.rs` |
| Descarga de la copia de LibreOffice | `infra/libreoffice.rs`; comandos en `commands/office.rs` |
| Tablas y migraciones de SQLite | `infra/store.rs` |
| Letras y lista del culto | `domain/song.rs`, `commands/songs.rs` |
| Ajustes | `commands/settings.rs` (los valores iniciales viven en la interfaz: `settings.svelte.ts`) |
| Biblias | `infra/bible.rs`, `commands/bible.rs` |
| OBS | `infra/obs.rs`, `commands/obs.rs` |
| Mensajes de error | variante en `error.rs` y su texto en `app/src/lib/messages.ts` + `i18n/*.ts` |

## Añadir un comando nuevo

1. **Backend:** escribe la función en el archivo de `commands/` que toque, con `#[tauri::command]` y `pub`. Si necesita estado, recibe `State<'_, AppState>`; si puede fallar, devuelve `Result<_, CommandError>`.
2. **Registro:** añádelo a `tauri::generate_handler![…]` en `lib.rs` como `commands::<archivo>::<nombre>`. Si no se registra, compila igual pero la interfaz recibe «command not found».
3. **Tipos de la interfaz:** declara la función en `app/src/lib/platform/types.ts` (interfaz `Platform`) y su llamada en `app/src/lib/platform/tauri.ts`. Los nombres de los argumentos van en camelCase en la interfaz (`filterName`) y en snake_case en Rust (`filter_name`): Tauri los traduce.
4. **Error nuevo:** variante en `CommandError` (`error.rs`), su forma en `CommandError` de `types.ts`, su caso en `messages.ts` y el texto en `i18n/es.ts`, `en.ts` e `it.ts` (`es.ts` es la referencia: los otros dos no compilan si les falta una clave).
5. **Pruebas:** `cargo test` en `app/src-tauri` y `npm run check` en `app`.

## Cuidado con…

- **Constantes en un `match`:** `match label { MAIN_LABEL => … }` solo compara si `MAIN_LABEL` está importada. Si no lo está, Rust la toma como una variable nueva que acepta cualquier valor y la rama siempre se cumple (el compilador avisa con «unused variable» o «should have a snake case name»: no ignores esos avisos).
- **Serde y la interfaz:** los tipos que viajan a la interfaz usan `#[serde(rename_all = "camelCase")]`. En los `enum` con campos hace falta también `rename_all_fields = "camelCase"`, o la interfaz recibe `duration_ms` en vez de `durationMs`.
- **Rutas de archivos en la salida:** la ventana de salida solo puede cargar archivos permitidos con `allow_media` (`state.rs`). Un archivo nuevo que se muestre en pantalla tiene que pasar por ahí.

## Interfaz

```
app/src/
  routes/          +page.svelte (consola) y output/ (ventana de salida)
  lib/
    platform/      único lugar que llama a Tauri (types.ts = contrato, tauri.ts = implementación)
    components/    paneles de la consola (Biblia, Textos, Medios, Reloj, Ajustes…)
    *.svelte.ts    estado compartido entre componentes (ajustes, letras, Biblias, OBS, LibreOffice)
    i18n/          textos en es, en, it
    Console.svelte, OutputView.svelte, OutputWindow.svelte
```

Las decisiones de diseño y sus motivos están en `DECISIONS.md`.
