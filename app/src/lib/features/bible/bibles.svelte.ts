// Biblias disponibles, compartidas por el panel de la Biblia y la vista de Ajustes.
// Las importadas se pueden quitar (se borra la copia de Selah, no el archivo original);
// las incluidas en la instalación no se borran: se ocultan, y Ajustes puede volver a mostrarlas.
import { platform, type BibleInfo } from "$lib/platform";
import { setSetting, settings } from "$lib/core/settings.svelte";

export const bibles = $state({
  all: [] as BibleInfo[],
  loading: true,
});

/** Las que aparecen para elegir: todas menos las ocultas. */
export function visibleBibles(): BibleInfo[] {
  return bibles.all.filter((b) => !settings.hiddenBibles.includes(b.id));
}

/** Las versiones elegidas: la de arriba (si la guardada ya no está, la primera) y la de abajo, si hay. */
export function currentBibles(): { bible: BibleInfo | null; second: BibleInfo | null } {
  const available = visibleBibles();
  const bible = available.find((b) => b.id === settings.bible) ?? available[0] ?? null;
  const second = settings.bible2 !== bible?.id ? (available.find((b) => b.id === settings.bible2) ?? null) : null;
  return { bible, second };
}

export async function loadBibles() {
  try {
    bibles.all = await platform.listBibles();
  } finally {
    bibles.loading = false;
  }
}

/** Abre el selector de XML; devuelve las importadas ([] si se cancela). */
export async function importBibles(title: string, filterName: string): Promise<BibleInfo[]> {
  const imported = await platform.importBibles(title, filterName);
  if (imported.length > 0) {
    await loadBibles();
    // Una versión importada otra vez vuelve a estar a la vista.
    const ids = imported.map((b) => b.id);
    if (settings.hiddenBibles.some((id) => ids.includes(id))) {
      setSetting("hiddenBibles", settings.hiddenBibles.filter((id) => !ids.includes(id)));
    }
  }
  return imported;
}

/** Quita una importada de Selah, u oculta una incluida. Deja de estar elegida si lo estaba. */
export async function removeBible(bible: BibleInfo) {
  if (bible.builtin) setSetting("hiddenBibles", [...settings.hiddenBibles, bible.id]);
  else {
    await platform.removeBible(bible.id);
    await loadBibles();
  }
  if (settings.bible2 === bible.id) setSetting("bible2", null);
  if (settings.bible === bible.id) {
    setSetting("bible", settings.bible2);
    setSetting("bible2", null);
  }
}

export function showBible(id: string) {
  setSetting("hiddenBibles", settings.hiddenBibles.filter((x) => x !== id));
}

/** Quita todas las importadas y oculta las incluidas. */
export async function removeAllBibles() {
  for (const b of bibles.all.filter((b) => !b.builtin)) await platform.removeBible(b.id);
  await loadBibles();
  setSetting("hiddenBibles", bibles.all.map((b) => b.id));
  setSetting("bible", null);
  setSetting("bible2", null);
}
