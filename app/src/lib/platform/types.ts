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

export interface ObsDetection {
  installed: boolean;
  websocketEnabled: boolean;
  running: boolean;
}

export interface ObsStatus {
  obsVersion: string;
  programScene: string | null;
  /** Todas las escenas de OBS, en el orden de su ventana: las del usuario y las de Selah. */
  scenes: string[];
  /** Fuentes de texto de OBS que Selah puede rellenar. */
  textSources: string[];
  /** Las escenas de Selah existen en OBS. */
  selahReady: boolean;
  hasCamera: boolean;
  streaming: {
    active: boolean;
    reconnecting: boolean;
    durationMs: number;
    congestion: number;
    skippedFrames: number;
    totalFrames: number;
  };
  /** La clave de emisión nunca sale de OBS: solo se indica si existe. */
  destination: { kind: string; server: string; hasKey: boolean } | null;
}

/** Dónde quedó visible el texto al ponerlo en la emisión. */
export type ObsTextShown = { kind: "current" } | { kind: "switched"; scene: string } | { kind: "notInScene" };

export interface ObsCamera {
  id: string;
  name: string;
}

/** Error devuelto por el backend; la interfaz lo traduce al idioma elegido. */
export type CommandError =
  | { code: "monitorDisconnected" }
  | { code: "fileMissing"; path: string }
  | { code: "invalidContent" }
  | { code: "invalidBible"; file: string; reason: string }
  | { code: "bibleNotFound" }
  | { code: "obsNotInstalled" }
  | { code: "obsRunning" }
  | { code: "obsNotConnected" }
  | { code: "obs"; message: string }
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
  /** Elige una carpeta y devuelve sus archivos compatibles, subcarpetas incluidas (null si se cancela). */
  pickMediaFolder(title: string): Promise<MediaFile[] | null>;
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

  // Transmisión con OBS
  obsDetect(): Promise<ObsDetection>;
  /** Activa el WebSocket de OBS en su configuración. Solo con OBS cerrado. */
  obsEnableWebsocket(): Promise<void>;
  obsLaunch(): Promise<void>;
  obsConnect(): Promise<void>;
  obsDisconnect(): Promise<void>;
  obsStatus(): Promise<ObsStatus>;
  /** Crea o completa las escenas de Selah en OBS. */
  obsSetup(waitText: string): Promise<void>;
  obsSetScene(scene: string): Promise<void>;
  /** Escribe el texto en esa fuente de OBS y la hace visible en la emisión. */
  obsShowText(source: string, text: string, credit: string): Promise<ObsTextShown>;
  /** Oculta esa fuente de texto en la escena al aire. */
  obsHideText(source: string): Promise<void>;
  obsCameras(): Promise<ObsCamera[]>;
  obsSetCamera(device: string): Promise<void>;
  /** Imagen (data URL) de lo que sale en la emisión. */
  obsPreview(): Promise<string | null>;
  /** Clave vacía = conservar la que ya tiene OBS. */
  obsSetDestination(server: string, key: string): Promise<void>;
  obsStartStream(): Promise<void>;
  obsStopStream(): Promise<void>;
}
