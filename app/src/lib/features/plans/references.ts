// La lista de citas que da el predicador («Juan 3:16-18; Sal 23»). Sin Svelte ni Tauri: se puede probar sola.
import type { BookLanguage } from "$lib/features/bible/books";
import { parseQuery, type Reference } from "$lib/features/bible/reference";

const LANGUAGES: BookLanguage[] = ["es", "en", "it"];

/** Una línea de la lista y lo que se entendió de ella. */
export interface ParsedLine {
  text: string;
  reference: Reference | null;
}

/** Separa por «;» o por líneas (la coma ya separa capítulo y versículo: «Jn 3,16»). Reconoce los libros en el idioma
 *  de la Biblia elegida y, si no, en los demás: el predicador puede escribir «John 3:16» con una Biblia en español. */
export function parseList(text: string, language: BookLanguage): ParsedLine[] {
  const order = [language, ...LANGUAGES.filter((l) => l !== language)];
  return text
    .split(/[;\n]+/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      for (const l of order) {
        const { reference } = parseQuery(line, l);
        if (reference) return { text: line, reference };
      }
      return { text: line, reference: null };
    });
}
