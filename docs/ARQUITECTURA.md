# Arquitectura

Selah se organiza **por funcionalidad**: Biblia, letras, medios, presentaciones, salida, ajustes, OBS. Cada una vive en su carpeta, en el backend y en la interfaz, con todo lo suyo dentro. Un núcleo común da lo que comparten todas. El núcleo no conoce a ninguna funcionalidad; ellas dependen de él.

Así se puede tocar una funcionalidad sin abrir las demás, y los plugins de otros desarrolladores se enchufan igual que las funcionalidades propias (D-016).

Las dos mitades solo se hablan por comandos y eventos de Tauri:

- **Interfaz** (`app/src`): SvelteKit con Svelte 5. Nunca importa Tauri directamente: todo pasa por `app/src/lib/platform` (D-010).
- **Backend** (`app/src-tauri/src`): Rust.

## Backend

```
src/
  lib.rs                 arranque: abre la base, registra cada funcionalidad, cierra la app al cerrar la consola
  core/                  lo común
    db.rs                SQLite: abrir, migraciones (MIGRATIONS), Store, Db (estado), with_store
    error.rs             CommandError: los errores que recibe la interfaz, como código + datos
    events.rs            nombres de ventanas y de eventos
    assets.rs            allow_media: qué archivos puede cargar la ventana de salida
    picker.rs            selector de archivos del sistema
    log.rs               comando log_client_error (errores de la interfaz al registro)
  features/
    <funcionalidad>/
      mod.rs             el plugin interno: nombre, comandos registrados, estado, arranque
      commands.rs        los #[tauri::command]: validan, llaman y traducen errores
      domain.rs          tipos y reglas puras (sin Tauri, SQLite, red ni procesos)
      …                  la implementación: repo.rs (SQLite), xml.rs, client.rs (OBS), window.rs…
```

| Funcionalidad | Qué hace | Archivos |
|---|---|---|
| `output` | Ventana de salida: monitores, abrir/cerrar, contenido y estilo | `domain.rs` (OutputContent, OutputStyle, OutputState), `window.rs` (pantalla completa) |
| `media` | Elegir archivos y carpetas, fondos | `domain.rs` (tipos de archivo), `scan.rs` (recorrer carpetas) |
| `presentations` | PowerPoint/Impress/Keynote → PDF, copia privada de LibreOffice | `convert.rs`, `libreoffice.rs` |
| `bible` | Biblias incluidas e importadas, capítulos, búsqueda | `domain.rs` (tipos, normalizar, buscar), `xml.rs`, `library.rs` |
| `songs` | Biblioteca de letras y lista del culto | `domain.rs` (Song), `repo.rs` |
| `settings` | Ajustes por clave | `repo.rs` |
| `obs` | Transmisión con OBS | `client.rs` |

### Reglas

- **Dependencias hacia el núcleo:** una funcionalidad usa `core` y, si lo necesita, el `domain` de otra (por ejemplo, `presentations` usa los tipos de archivo de `media`). Nunca los comandos ni el estado de otra.
- **`domain.rs` es puro:** si algo necesita Tauri, la base o el disco, no va ahí. Así se prueba con `cargo test` sin abrir la app.
- **El estado es de cada funcionalidad:** `OutputStore`, `BibleState`, `ObsState`… Cada una lo registra en su `setup`. La base (`Db`) es del núcleo porque la comparten varias; cada funcionalidad escribe sus consultas en su propio `impl Store` (`features/songs/repo.rs`).
- **Tablas nuevas:** se añade una entrada al final de `MIGRATIONS` en `core/db.rs`. Nunca se edita una entrada publicada.

### Cada funcionalidad es un plugin interno de Tauri

`mod.rs` construye el plugin con `tauri::plugin::Builder::new(NAME)` y lo registra en `lib.rs` (`.plugin(features::x::plugin())`).

- **Llamadas:** la interfaz llama a sus comandos como `plugin:<nombre>|<comando>`, por ejemplo `plugin:bible|list_bibles`.
- **Permiso:** cada plugin tiene uno, `<nombre>:default`, que permite todos sus comandos. Se da a las ventanas en `capabilities/default.json`.
- **Lista de comandos:** `build.rs` la lee del `commands.rs` de cada carpeta de `features/`, así que no hay que mantenerla a mano.

## Añadir…

### …un comando a una funcionalidad existente

1. Escríbelo en `features/<x>/commands.rs` con `#[tauri::command]` y `pub`. Si necesita estado, recibe `State<'_, XState>`; si puede fallar, devuelve `Result<_, CommandError>`.
2. Añádelo a `generate_handler![…]` en `features/<x>/mod.rs`. Si te olvidas, `cargo test` falla con «… no está registrado».
3. En la interfaz: declara la función en `app/src/lib/platform/types.ts` (interfaz `Platform`) y su llamada en `platform/tauri.ts` como `invoke("plugin:<x>|<comando>", { … })`. Los argumentos van en camelCase en la interfaz (`filterName`) y en snake_case en Rust (`filter_name`): Tauri los traduce.

### …una funcionalidad nueva

1. Backend: carpeta `features/<nueva>/` con `mod.rs` (copia uno pequeño, como `songs/mod.rs`) y `commands.rs`; `pub mod <nueva>;` en `features/mod.rs`; `.plugin(features::<nueva>::plugin())` en `lib.rs`; `"<nueva>:default"` en `capabilities/default.json`.
2. Interfaz: carpeta `app/src/lib/features/<nueva>/` con su panel y un `index.ts` que exporte su `SourcePanel` (id, texto, icono, componente); añádelo a la lista de `app/src/lib/features/index.ts`. La consola lo muestra en la barra lateral, sin tocar `Console.svelte`.
3. El panel recibe `PanelProps` (`core/panels.ts`): lo preparado, lo que está en pantalla, `onprepare`, `onsend` y `onerror`. Usa los que necesites.

### …un error nuevo

Variante en `CommandError` (`core/error.rs`). Si viene de un error propio de la funcionalidad, la conversión va en su módulo (`impl From<MiError> for CommandError`, como en `features/obs/mod.rs`). Después: su forma en `CommandError` de `platform/types.ts`, su caso en `core/messages.ts` y el texto en `i18n/es.ts`, `en.ts` e `it.ts`. `es.ts` es la referencia: los otros dos no compilan si les falta una clave.

## Interfaz

```
app/src/
  routes/                +page.svelte (consola) y output/ (ventana de salida)
  lib/
    platform/            único lugar que llama a Tauri (types.ts = contrato, tauri.ts = implementación)
    i18n/                textos en es, en, it
    core/                Console.svelte (la consola), panels.ts (contrato de paneles), settings, theme, messages
    features/
      index.ts           paneles de contenido, en el orden de la barra lateral
      bible/  songs/  media/  clock/      panel + estado + index.ts
      output/            OutputView (lo que se dibuja en pantalla), OutputWindow, StylePanel, SlideStrip
      presentations/     descarga de LibreOffice
      stream/            OBS
      settings/          vista de Ajustes
```

## Cuidado con…

- **Constantes en un `match`:** `match label { MAIN_LABEL => … }` solo compara si `MAIN_LABEL` está importada. Si no lo está, Rust la toma como una variable nueva que acepta cualquier valor y la rama siempre se cumple. El compilador avisa con «unused variable» o «should have a snake case name»: no ignores esos avisos.
- **Serde y la interfaz:** los tipos que viajan a la interfaz usan `#[serde(rename_all = "camelCase")]`. En los `enum` con campos hace falta también `rename_all_fields = "camelCase"`, o la interfaz recibe `duration_ms` en vez de `durationMs`.
- **Archivos en la salida:** la ventana de salida solo carga archivos permitidos con `allow_media` (`core/assets.rs`).
- **Código de un solo sistema** (`#[cfg(target_os = …)]`, por ejemplo `features/output/window.rs` y `core/picker.rs`): en Linux no se compila la parte de Windows o macOS. Antes de fusionar, lanza «Instaladores» a mano en la rama (workflow_dispatch) para compilar en los tres.

## Plugins externos (próximo paso)

Están pensados sobre estas mismas piezas:

- Su panel será un `SourcePanel` más, que carga el HTML del plugin en un `iframe` aislado y le da una API (`selah.screen`, `selah.storage`, `selah.keys`…) en vez de acceso a Tauri.
- Su contenido en pantalla será un tipo nuevo de `OutputContent`, dibujado en la salida también dentro de un `iframe`.
- Sus permisos se declararán en su `plugin.json` y se aceptarán al instalarlo, igual que cada funcionalidad tiene hoy su `<x>:default`.

Las decisiones de diseño y sus motivos están en `DECISIONS.md`.
