// Biblioteca de letras (canciones, liturgias, anuncios fijos). Lo que se presenta en un culto, en orden, son los planes
// (features/plans). Se guardan en la base de datos local; aquí se guarda una copia para la interfaz.
// Los cambios se aplican primero en la copia (la interfaz no espera) y luego en la base.
import { SvelteSet } from "svelte/reactivity";
import { readLegacy, storage } from "$lib/core/settings.svelte";
import { t } from "$lib/i18n/index.svelte";
import { platform, type OutputContent, type Song } from "$lib/platform";
import { splitSlides } from "./slides";

export type { Song };

export const library = $state({
  songs: [] as Song[],
});

/** Fondos de letras que existen en este equipo y ya se pueden cargar; los demás se ignoran para no impedir mostrar la letra. */
export const allowedBackgrounds = new SvelteSet<string>();

function credit(song: Song): string {
  return [song.title, song.singer, song.writer && t("songs.writtenBy", { name: song.writer })].filter(Boolean).join(" · ");
}

/** La letra lista para «Preparado», desde la primera diapositiva. */
export function songContent(song: Song): OutputContent {
  const slides = splitSlides(song.body);
  if (slides.length === 0) return { kind: "empty" };
  const background = song.background && allowedBackgrounds.has(song.background) ? song.background : null;
  return { kind: "text", slides, index: 0, credit: credit(song), background };
}

function failed(action: string, e: unknown) {
  storage.error = (e as { message?: string })?.message ?? String(e);
  platform.logError(`letras: ${action}: ${JSON.stringify(e)}`).catch(() => {});
}

const isSongs = (v: unknown): v is Song[] =>
  Array.isArray(v) && v.every((s) => typeof s?.id === "string" && typeof s?.body === "string");

/** Letras guardadas antes de la base de datos (localStorage): se pasan una vez si la base está vacía. */
async function migrateLegacy() {
  const songs = readLegacy("selah.songs", isSongs);
  if (!songs || library.songs.length > 0) return;
  for (const song of songs.value as Song[]) {
    await platform.songSave(song);
  }
}

export async function loadLibrary() {
  try {
    library.songs = await platform.songsList();
    await migrateLegacy();
    library.songs = await platform.songsList();
  } catch (e) {
    failed("leer", e);
  }
}

export function saveSong(song: Song) {
  const i = library.songs.findIndex((s) => s.id === song.id);
  if (i >= 0) library.songs[i] = song;
  else library.songs.push(song);
  library.songs.sort((a, b) => a.title.localeCompare(b.title));
  platform.songSave(song).catch((e) => failed("guardar", e));
}

export function deleteSong(id: string) {
  library.songs = library.songs.filter((s) => s.id !== id);
  platform.songDelete(id).catch((e) => failed("borrar", e));
}

/** Borra la biblioteca entera. */
export async function resetLibrary() {
  await platform.songsReset();
  library.songs = [];
}

// El permiso para leer los fondos no sobrevive a un reinicio de la app. Se pide para cada fondo nuevo que llegue
// a la biblioteca (al abrir, al importar letras), una sola vez por ruta.
const requested = new Set<string>();
export function allowSongBackgrounds() {
  const missing = [...new Set(library.songs.map((s) => s.background))].filter((p): p is string => p !== null && !requested.has(p));
  if (missing.length === 0) return;
  for (const p of missing) requested.add(p);
  platform.allowBackgrounds(missing).then((ok) => ok.forEach((p) => allowedBackgrounds.add(p))).catch(() => {});
}
