// Versículos en un plan (D-019): la lista que da el predicador («Juan 3:16-18; Sal 23») y cómo se muestra cada cita.
import { bibles, currentBibles, loadBibles } from "$lib/features/bible/bibles.svelte";
import { bookName } from "$lib/features/bible/books";
import { bookLanguage, buildScripture } from "$lib/features/bible/scripture";
import { i18n } from "$lib/i18n/index.svelte";
import { platform, type BibleInfo, type OutputContent, type PlanItem } from "$lib/platform";

export { parseList, type ParsedLine } from "./references";

export type ScriptureItem = Extract<PlanItem, { kind: "scripture" }>;

/** La versión con la que se mostrará: la fija del elemento (si sigue en Selah) o la elegida en la consola. */
export function bibleFor(item: ScriptureItem): BibleInfo | null {
  return (item.bible && bibles.all.find((b) => b.id === item.bible)) || currentBibles().bible;
}

/** «Juan 3:16-18», «Salmos 23», en el idioma de esa Biblia. */
export function referenceLabel(item: ScriptureItem, bible: BibleInfo | null): string {
  const name = bookName(item.book, bible ? bookLanguage(bible) : i18n.locale);
  if (item.verseStart === null) return `${name} ${item.chapter}`;
  const end = item.verseEnd !== null && item.verseEnd !== item.verseStart ? `-${item.verseEnd}` : "";
  return `${name} ${item.chapter}:${item.verseStart}${end}`;
}

/** La versión no tiene ese libro o capítulo. */
export function missingIn(item: ScriptureItem, bible: BibleInfo | null): boolean {
  if (!bible) return true;
  const book = bible.books.find((b) => b.number === item.book);
  return !book || item.chapter < 1 || item.chapter > book.chapters;
}

/** El pasaje listo para «Preparado», con la segunda versión de la consola debajo si la hay.
 *  null si la Biblia no tiene esos versículos. */
export async function scriptureContent(item: ScriptureItem): Promise<OutputContent | null> {
  if (bibles.all.length === 0) await loadBibles();
  const bible = bibleFor(item);
  if (!bible || missingIn(item, bible)) return null;
  const { second: consoleSecond } = currentBibles();
  const second = consoleSecond && consoleSecond.id !== bible.id ? consoleSecond : null;
  const [verses, secondVerses] = await Promise.all([
    platform.getChapter(bible.id, item.book, item.chapter),
    second ? platform.getChapter(second.id, item.book, item.chapter).catch(() => null) : Promise.resolve(null),
  ]);
  const start = item.verseStart ?? 1;
  const end = item.verseStart === null ? Number.MAX_SAFE_INTEGER : (item.verseEnd ?? item.verseStart);
  const shown = verses.filter((v) => v.number >= start && v.number <= end);
  if (shown.length === 0) return null;
  return buildScripture(bible, second, item.book, item.chapter, shown, shown[0].number, secondVerses);
}
