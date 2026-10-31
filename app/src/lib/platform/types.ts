// Tipos compartidos entre la interfaz y el backend. No dependen de Tauri (ver D-010).

export interface MonitorInfo {
  index: number;
  name: string;
  width: number;
  height: number;
  x: number;
  y: number;
  scale: number;
  isPrimary: boolean;
}

export type OutputContent =
  | { kind: "empty" }
  | { kind: "black" }
  | { kind: "testPattern" }
  | { kind: "text"; slides: string[]; index: number }
  | { kind: "image"; path: string }
  | { kind: "video"; path: string; paused: boolean }
  | { kind: "pdf"; path: string; page: number }
  | { kind: "scripture"; verses: ScriptureVerse[]; index: number; version: string; copyright: string };

export interface ScriptureVerse {
  reference: string;
  text: string;
}

export interface OutputStyle {
  /** Tamaño máximo relativo (1 = normal). El texto se reduce solo si no cabe. */
  fontScale: number;
  align: "left" | "center" | "right";
  position: "top" | "middle" | "bottom";
  background: "black" | "navy" | "charcoal";
}

export interface BibleInfo {
  id: string;
  abbreviation: string;
  name: string;
  copyright: string;
  link: string | null;
  /** Idioma del texto bíblico, independiente del de la interfaz. */
  language: string | null;
  /** Incluida en la instalación (no se puede quitar). */
  builtin: boolean;
  books: { number: number; chapters: number }[];
}

export interface Verse {
  number: number;
  text: string;
}

export interface SearchHit {
  book: number;
  chapter: number;
  verse: number;
  text: string;
}

export type MediaKind = "image" | "video" | "pdf";

export interface MediaFile {
  path: string;
  name: string;
  kind: MediaKind;
}

/** Error devuelto por el backend; la interfaz lo traduce al idioma elegido. */
export type CommandError =
  | { code: "monitorDisconnected" }
  | { code: "fileMissing"; path: string }
  | { code: "invalidContent" }
  | { code: "invalidBible"; file: string; reason: string }
  | { code: "bibleNotFound" }
  | { code: "internal"; message: string };

export interface OutputState {
  open: boolean;
  monitorName: string | null;
  content: OutputContent;
  style: OutputStyle;
}

export interface Platform {
  listMonitors(): Promise<MonitorInfo[]>;
  openOutput(monitorIndex: number): Promise<OutputState>;
  closeOutput(): Promise<void>;
  setOutputContent(content: OutputContent): Promise<OutputState>;
  getOutputState(): Promise<OutputState>;
  setOutputStyle(style: OutputStyle): Promise<OutputState>;
  /** Devuelve la función para dejar de escuchar. */
  onOutputState(handler: (state: OutputState) => void): Promise<() => void>;
  /** Abre el selector de archivos; devuelve [] si se cancela. */
  pickMedia(title: string, filterName: string): Promise<MediaFile[]>;
  /** URL que el webview puede cargar para un archivo local ya permitido. */
  mediaUrl(path: string): string;
  reportOutputError(message: string): Promise<void>;
  /** Escribe un error de la interfaz en el registro de la app. */
  logError(message: string): Promise<void>;
  onOutputError(handler: (message: string) => void): Promise<() => void>;
  listBibles(): Promise<BibleInfo[]>;
  /** Abre el selector de archivos XML; devuelve las Biblias importadas ([] si se cancela). */
  importBibles(title: string, filterName: string): Promise<BibleInfo[]>;
  removeBible(id: string): Promise<void>;
  getChapter(bibleId: string, book: number, chapter: number): Promise<Verse[]>;
  searchBible(bibleId: string, query: string): Promise<SearchHit[]>;
}
