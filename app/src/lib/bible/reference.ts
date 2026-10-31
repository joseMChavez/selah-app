import { matchBooks, type BookLanguage } from "./books";

export interface Reference {
  book: number;
  chapter: number;
  /** Sin versículos = capítulo completo. */
  verseStart: number | null;
  verseEnd: number | null;
}

export interface ParsedQuery {
  /** Referencia completa si se reconoció libro y capítulo. */
  reference: Reference | null;
  /** Libros posibles para lo escrito, para sugerirlos mientras se escribe. */
  books: number[];
}

// "1 Juan 3:16-18", "jn 3 16", "Sal 23", "1co13.4". El libro puede empezar por 1, 2 o 3.
const PATTERN = /^\s*([1-3]?\s*[^\d\s:.,-][^\d:,-]*?)\s*(?:(\d+)\s*(?:[:.,\s]\s*(\d+)\s*(?:-\s*(\d+))?)?)?\s*$/u;

export function parseQuery(input: string, language: BookLanguage): ParsedQuery {
  const match = PATTERN.exec(input);
  if (!match) return { reference: null, books: [] };
  const [, bookText, chapter, verseStart, verseEnd] = match;
  const books = matchBooks(bookText, language);
  if (books.length === 0 || !chapter) return { reference: null, books };

  const start = verseStart ? Number(verseStart) : null;
  const end = verseEnd ? Number(verseEnd) : start;
  return {
    reference: {
      book: books[0],
      chapter: Number(chapter),
      verseStart: start,
      verseEnd: end !== null && start !== null ? Math.max(start, end) : null,
    },
    books,
  };
}
