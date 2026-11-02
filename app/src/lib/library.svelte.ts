// Biblioteca de letras (canciones, liturgias, anuncios fijos) y la lista para el culto.
// Se guardan en la base de datos local; aquí se guarda una copia para la interfaz.
// Los cambios se aplican primero en la copia (la interfaz no espera) y luego en la base.
import { readLegacy, storage } from "$lib/settings.svelte";
import { platform, type Song } from "$lib/platform";

export type { Song };

export const library = $state({
  songs: [] as Song[],
  /** Ids de la biblioteca, en el orden en que se van a presentar. */
  setlist: [] as string[],
});

function failed(action: string, e: unknown) {
  storage.error = (e as { message?: string })?.message ?? String(e);
  platform.logError(`letras: ${action}: ${JSON.stringify(e)}`).catch(() => {});
}

const isSongs = (v: unknown): v is Song[] =>
  Array.isArray(v) && v.every((s) => typeof s?.id === "string" && typeof s?.body === "string");
const isIds = (v: unknown): v is string[] => Array.isArray(v) && v.every((x) => typeof x === "string");

/** Letras guardadas antes de la base de datos (localStorage): se pasan una vez si la base está vacía. */
async function migrateLegacy() {
  const songs = readLegacy("selah.songs", isSongs);
  const setlist = readLegacy("selah.setlist", isIds);
  if (!songs || library.songs.length > 0) return;
  for (const song of songs.value as Song[]) {
    await platform.songSave(song);
  }
  if (setlist) await platform.setlistSet(setlist.value as string[]);
}

export async function loadLibrary() {
  try {
    library.songs = await platform.songsList();
    await migrateLegacy();
    library.songs = await platform.songsList();
    library.setlist = await platform.setlistGet();
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
  library.setlist = library.setlist.filter((x) => x !== id);
  platform.songDelete(id).catch((e) => failed("borrar", e));
}

export function setSetlist(ids: string[]) {
  library.setlist = ids;
  platform.setlistSet(ids).catch((e) => failed("guardar la lista", e));
}

/** Borra la biblioteca entera y la lista. */
export async function resetLibrary() {
  await platform.songsReset();
  library.songs = [];
  library.setlist = [];
}
