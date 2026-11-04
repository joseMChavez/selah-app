// Cómo se arma un pasaje para la pantalla. Lo usan el panel de la Biblia y el control remoto,
// para que un versículo se vea igual venga de donde venga.
import { bookName, type BookLanguage } from "./books";
import { currentBibles, loadBibles, bibles } from "./bibles.svelte";
import { i18n, locales } from "$lib/i18n/index.svelte";
import { platform, type BibleInfo, type OutputContent, type Verse } from "$lib/platform";

/** Los nombres de libro siguen el idioma de la Biblia, no el de la interfaz. */
export function bookLanguage(bible: BibleInfo | null): BookLanguage {
  return bible?.language && bible.language in locales ? (bible.language as BookLanguage) : i18n.locale;
}

/** Los versículos `verses` de un capítulo, empezando en el número `start`, con la segunda versión debajo si la hay. */
export function buildScripture(
  bible: BibleInfo,
  second: BibleInfo | null,
  book: number,
  chapter: number,
  verses: Verse[],
  start: number,
  secondVerses: Verse[] | null,
): OutputContent {
  const language = bookLanguage(bible);
  const secondLanguage = bookLanguage(second);
  const index = verses.findIndex((v) => v.number === start);
  const parallel = (n: number) => {
    const text = secondVerses?.find((v) => v.number === n)?.text;
    return text ? { reference: `${bookName(book, secondLanguage)} ${chapter}:${n}`, text } : null;
  };
  return {
    kind: "scripture",
    verses: verses.map((v) => ({ reference: `${bookName(book, language)} ${chapter}:${v.number}`, text: v.text, second: parallel(v.number) })),
    index: Math.max(index, 0),
    version: bible.abbreviation,
    copyright: bible.copyright,
    second: second && secondVerses ? { version: second.abbreviation, copyright: second.copyright } : null,
  };
}

/** El capítulo entero con las versiones elegidas en la consola, en el versículo pedido.
 *  null si no hay Biblia o no tiene ese capítulo o versículo. */
export async function loadScripture(book: number, chapter: number, verse: number): Promise<OutputContent | null> {
  if (bibles.all.length === 0) await loadBibles();
  const { bible, second } = currentBibles();
  if (!bible) return null;
  const [verses, secondVerses] = await Promise.all([
    platform.getChapter(bible.id, book, chapter).catch(() => null),
    second ? platform.getChapter(second.id, book, chapter).catch(() => null) : Promise.resolve(null),
  ]);
  if (!verses?.some((v) => v.number === verse)) return null;
  return buildScripture(bible, second, book, chapter, verses, verse, secondVerses);
}
