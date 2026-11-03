// Los 66 libros en el orden de los archivos XML (book number 1–66).
// Nombres en español, inglés e italiano y abreviaturas habituales, para reconocer referencias en cualquiera de ellos.
export type BookLanguage = "es" | "en" | "it";

interface BookNames {
  es: string;
  en: string;
  it: string;
  /** Abreviaturas aceptadas, separadas por espacios (sin puntos). */
  aliases: string;
}

const B = (es: string, en: string, it: string, aliases = ""): BookNames => ({ es, en, it, aliases });

export const BOOKS: BookNames[] = [
  B("Génesis", "Genesis", "Genesi", "gn gen ge"),
  B("Éxodo", "Exodus", "Esodo", "ex exo es"),
  B("Levítico", "Leviticus", "Levitico", "lv lev"),
  B("Números", "Numbers", "Numeri", "nm num nu"),
  B("Deuteronomio", "Deuteronomy", "Deuteronomio", "dt deut"),
  B("Josué", "Joshua", "Giosuè", "jos js josh gs"),
  B("Jueces", "Judges", "Giudici", "jue jc jdg gdc"),
  B("Rut", "Ruth", "Rut", "rt ru"),
  B("1 Samuel", "1 Samuel", "1 Samuele", "1s 1sa 1sam 1sm"),
  B("2 Samuel", "2 Samuel", "2 Samuele", "2s 2sa 2sam 2sm"),
  B("1 Reyes", "1 Kings", "1 Re", "1r 1re 1rey 1ki 1kgs"),
  B("2 Reyes", "2 Kings", "2 Re", "2r 2re 2rey 2ki 2kgs"),
  B("1 Crónicas", "1 Chronicles", "1 Cronache", "1cr 1cro 1ch 1chr"),
  B("2 Crónicas", "2 Chronicles", "2 Cronache", "2cr 2cro 2ch 2chr"),
  B("Esdras", "Ezra", "Esdra", "esd ezr"),
  B("Nehemías", "Nehemiah", "Neemia", "neh ne"),
  B("Ester", "Esther", "Ester", "est"),
  B("Job", "Job", "Giobbe", "jb gb"),
  B("Salmos", "Psalms", "Salmi", "sal sl ps psa"),
  B("Proverbios", "Proverbs", "Proverbi", "pr prov pro"),
  B("Eclesiastés", "Ecclesiastes", "Ecclesiaste", "ec ecl ecc qo"),
  B("Cantares", "Song of Songs", "Cantico dei Cantici", "cnt cant ct song cantardeloscantares"),
  B("Isaías", "Isaiah", "Isaia", "is isa"),
  B("Jeremías", "Jeremiah", "Geremia", "jer jr ger"),
  B("Lamentaciones", "Lamentations", "Lamentazioni", "lm lam"),
  B("Ezequiel", "Ezekiel", "Ezechiele", "ez eze ezk"),
  B("Daniel", "Daniel", "Daniele", "dn dan"),
  B("Oseas", "Hosea", "Osea", "os hos"),
  B("Joel", "Joel", "Gioele", "jl gl"),
  B("Amós", "Amos", "Amos", "am"),
  B("Abdías", "Obadiah", "Abdia", "abd ob"),
  B("Jonás", "Jonah", "Giona", "jon"),
  B("Miqueas", "Micah", "Michea", "mi miq mic"),
  B("Nahúm", "Nahum", "Naum", "nah na"),
  B("Habacuc", "Habakkuk", "Abacuc", "hab ab"),
  B("Sofonías", "Zephaniah", "Sofonia", "sof zep"),
  B("Hageo", "Haggai", "Aggeo", "hag ag"),
  B("Zacarías", "Zechariah", "Zaccaria", "zac zec"),
  B("Malaquías", "Malachi", "Malachia", "mal ml"),
  B("Mateo", "Matthew", "Matteo", "mt mat"),
  B("Marcos", "Mark", "Marco", "mr mc mk mar"),
  B("Lucas", "Luke", "Luca", "lc lk luc"),
  B("Juan", "John", "Giovanni", "jn jua gv"),
  B("Hechos", "Acts", "Atti", "hch hech at act"),
  B("Romanos", "Romans", "Romani", "ro rm rom"),
  B("1 Corintios", "1 Corinthians", "1 Corinzi", "1co 1cor"),
  B("2 Corintios", "2 Corinthians", "2 Corinzi", "2co 2cor"),
  B("Gálatas", "Galatians", "Galati", "ga gal gl"),
  B("Efesios", "Ephesians", "Efesini", "ef efe eph"),
  B("Filipenses", "Philippians", "Filippesi", "fil flp php"),
  B("Colosenses", "Colossians", "Colossesi", "col"),
  B("1 Tesalonicenses", "1 Thessalonians", "1 Tessalonicesi", "1ts 1tes 1th 1te"),
  B("2 Tesalonicenses", "2 Thessalonians", "2 Tessalonicesi", "2ts 2tes 2th 2te"),
  B("1 Timoteo", "1 Timothy", "1 Timoteo", "1ti 1tim 1tm"),
  B("2 Timoteo", "2 Timothy", "2 Timoteo", "2ti 2tim 2tm"),
  B("Tito", "Titus", "Tito", "tit tt"),
  B("Filemón", "Philemon", "Filemone", "flm fm phm"),
  B("Hebreos", "Hebrews", "Ebrei", "he heb eb"),
  B("Santiago", "James", "Giacomo", "stg sant jas gc"),
  B("1 Pedro", "1 Peter", "1 Pietro", "1p 1pe 1ped 1pt"),
  B("2 Pedro", "2 Peter", "2 Pietro", "2p 2pe 2ped 2pt"),
  B("1 Juan", "1 John", "1 Giovanni", "1jn 1jua 1gv"),
  B("2 Juan", "2 John", "2 Giovanni", "2jn 2jua 2gv"),
  B("3 Juan", "3 John", "3 Giovanni", "3jn 3jua 3gv"),
  B("Judas", "Jude", "Giuda", "jud jds gd"),
  B("Apocalipsis", "Revelation", "Apocalisse", "ap apoc rev"),
];

/** Minúsculas, sin acentos, sin espacios ni puntos: "1 Co." -> "1co". */
export function normalizeBookName(text: string): string {
  return text
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[\s.]/g, "");
}

const exact = new Map<string, number>();
const fullNames: { book: number; name: string; language: BookLanguage }[] = [];

BOOKS.forEach((names, i) => {
  const book = i + 1;
  for (const language of ["es", "en", "it"] as const) {
    const name = normalizeBookName(names[language]);
    if (!exact.has(name)) exact.set(name, book);
    fullNames.push({ book, name, language });
  }
  for (const alias of names.aliases.split(" ").filter(Boolean)) {
    if (!exact.has(alias)) exact.set(alias, book);
  }
});

export function bookName(book: number, language: BookLanguage): string {
  return BOOKS[book - 1]?.[language] ?? `#${book}`;
}

/**
 * Libros que coinciden con lo escrito, el más probable primero.
 * Primero las coincidencias exactas (nombre o abreviatura); después los nombres que empiezan así,
 * dando prioridad al idioma indicado.
 */
export function matchBooks(input: string, language: BookLanguage): number[] {
  const query = normalizeBookName(input);
  if (!query) return [];
  const result: number[] = [];
  const exactBook = exact.get(query);
  if (exactBook) result.push(exactBook);
  const prefixed = fullNames
    .filter((n) => n.name.startsWith(query))
    .sort((a, b) => Number(b.language === language) - Number(a.language === language) || a.book - b.book);
  for (const { book } of prefixed) if (!result.includes(book)) result.push(book);
  return result;
}
