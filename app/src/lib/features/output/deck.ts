// Contenidos que se recorren paso a paso: diapositivas de texto, versículos y páginas de PDF.
// Las mismas reglas valen para «Preparado» y para «En pantalla».
import type { MessageKey } from "$lib/i18n/index.svelte";
import type { OutputContent } from "$lib/platform";

export interface DeckPosition {
  /** Posición actual, empezando en 1. */
  n: number;
  /** null mientras no se sabe (páginas de un PDF que aún se está leyendo). */
  total: number | null;
  label: MessageKey;
}

export function deckPosition(content: OutputContent, pdfPages: Record<string, number>): DeckPosition | null {
  switch (content.kind) {
    case "text": return { n: content.index + 1, total: content.slides.length, label: "position.slide" };
    case "scripture": return { n: content.index + 1, total: content.verses.length, label: "position.verse" };
    case "pdf": return { n: content.page, total: pdfPages[content.path] ?? null, label: "position.page" };
    default: return null;
  }
}

/** El mismo contenido en la posición n, o null si n está fuera de rango o no cambia nada. */
export function moveDeck(content: OutputContent, n: number, pdfPages: Record<string, number>): OutputContent | null {
  const position = deckPosition(content, pdfPages);
  if (!position || n < 1 || n === position.n || (position.total !== null && n > position.total)) return null;
  switch (content.kind) {
    case "text": return { ...content, index: n - 1 };
    case "scripture": return { ...content, index: n - 1 };
    case "pdf": return { ...content, page: n };
    default: return null;
  }
}
