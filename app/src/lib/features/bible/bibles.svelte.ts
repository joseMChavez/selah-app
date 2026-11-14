// Biblias disponibles, compartidas por el panel de la Biblia y la vista de Ajustes.
// Las importadas se pueden quitar (se borra la copia de EmetPro, no el archivo original);
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

/** Quita una importada de EmetPro, u oculta una incluida. Deja de estar elegida si lo estaba. */
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

/** La versión de nombre `abbreviation`; si hay varias con ese nombre, la elegida. */
export function bibleByAbbreviation(abbreviation: string): BibleInfo | null {
  const available = visibleBibles().filter((b) => b.abbreviation === abbreviation);
  return available.find((b) => b.id === settings.bible) ?? available[0] ?? null;
}

/** Vuelve a pantalla un versículo de otra versión (desde lo reciente, el plan o el teléfono): esa pasa a ser la
 *  elegida, para que lo que se busque después salga en la misma versión. Con o sin la de abajo, como estaba. */
export function followVersion(version: string, second: string | null) {
  const bible = bibleByAbbreviation(version);
  if (!bible) return;
  const below = second ? bibleByAbbreviation(second) : null;
  const { bible: current, second: currentSecond } = currentBibles();
  if (current?.id === bible.id && (currentSecond?.id ?? null) === (below?.id ?? null)) return;
  setSetting("bible", bible.id);
  setSetting("bible2", below && below.id !== bible.id ? below.id : null);
}
