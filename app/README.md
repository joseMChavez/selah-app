# EmetPro — app source

This folder holds the desktop app: a [Tauri 2](https://tauri.app/) backend in Rust (`src-tauri/`) and a [SvelteKit](https://svelte.dev/) interface in Svelte 5 and TypeScript (`src/`). What the app does and how to download it: [main README](../README.md).

## Commands

```sh
npm install
npm run tauri dev                  # run the app with hot reload
npm run check                      # type-check the interface
cd src-tauri && cargo test         # backend tests
npm run tauri build                # installers in src-tauri/target/release/bundle/
npx tauri build --debug --no-bundle  # quick debug binary: src-tauri/target/debug/emetpro
```

On Linux you also need GStreamer with gst-libav for video, plus the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

## Layout

The code is organized **by feature**, the same way on both sides:

| Feature | Backend (`src-tauri/src/features/`) | Interface (`src/lib/features/`) |
|---|---|---|
| Bible | `bible` | `bible` |
| Lyrics and texts | `songs` | `songs` |
| Import/export of lyrics | `transfer` | `transfer` |
| Images, videos, PDFs | `media` | `media` |
| Presentations (LibreOffice) | `presentations` | `presentations` |
| Service plans | `plans` | `plans` |
| Output window | `output` | `output` |
| Phone remote control and whiteboard | `remote` | `remote` |
| OBS streaming | `obs` | `stream` |
| Settings | `settings` | `settings` |
| Clock and timer | — | `clock` |

Each backend feature is an internal Tauri plugin; the interface calls it through `src/lib/platform/`. Shared pieces live in `src-tauri/src/core/` (SQLite database, errors, file access) and `src/lib/core/` (console layout, settings). Interface texts are in `src/lib/i18n/` (`es.ts` is the reference; `en.ts` and `it.ts` must have the same keys).

Data lives in the system's app data folder under `org.emetpro.app` (on Linux, `~/.local/share/org.emetpro.app/`): `emetpro.db` plus imported Bibles, backgrounds, plan media and the private LibreOffice copy. On first launch, `src-tauri/src/core/legacy.rs` copies the data of an earlier Selah install from `org.selah.app`.

More detail, in Spanish: [architecture](../docs/ARQUITECTURA.md) and [decisions log](../docs/DECISIONS.md). Code comments are in Spanish too.

## Recommended editor setup

[VS Code](https://code.visualstudio.com/) with the [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
