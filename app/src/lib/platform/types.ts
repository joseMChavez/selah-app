// Tipos compartidos entre la interfaz y el backend. No dependen de Tauri (ver D-010).
import type { RemoteCommand } from "$lib/features/remote/protocol";
import type { Board, BoardMessage, BoardOp, ScreenLayout } from "$lib/features/remote/board";

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
  | {
      kind: "text";
      slides: string[];
      index: number;
      /** Autoría de una letra; se muestra en la primera diapositiva. */
      credit?: string;
      /** Imagen o GIF propio de esta letra; tapa el fondo general. */
      background?: string | null;
    }
  | { kind: "image"; path: string }
  | { kind: "video"; path: string; paused: boolean }
  | { kind: "pdf"; path: string; page: number }
  | {
      kind: "scripture";
      verses: ScriptureVerse[];
      index: number;
      version: string;
      copyright: string;
      /** Segunda versión, debajo de la primera. Como mucho dos. */
      second?: { version: string; copyright: string } | null;
    }
  | { kind: "clock"; hour12: boolean; seconds: boolean }
  /** En marcha si endsAt (ms desde 1970) tiene valor; en pausa, remainingMs guarda lo que queda. */
  | {
      kind: "timer";
      durationMs: number;
      endsAt: number | null;
      remainingMs: number;
      /** Título sobre la cuenta («Receso»); vacío = solo la cuenta. */
      label?: string;
      /** Imagen o GIF propio de esta cuenta; tapa el fondo general. */
      background?: string | null;
    }
  /** Una hoja de la pizarra del control remoto (features/remote/board.ts). */
  | { kind: "board"; sheet: string };

export interface ScriptureVerse {
  reference: string;
  text: string;
  /** El mismo versículo en la segunda versión; null si esa versión no lo tiene. */
  second?: { reference: string; text: string } | null;
}

export type FontId = "system" | "montserrat" | "oswald" | "bebasNeue" | "lora" | "merriweather";

export interface OutputStyle {
  /** Tamaño máximo relativo (1 = normal). El texto se reduce solo si no cabe. */
  fontScale: number;
  align: "left" | "center" | "right";
  position: "top" | "middle" | "bottom";
  background: "black" | "navy" | "charcoal";
  font: FontId;
  /** Imagen o GIF detrás del texto, la Biblia y el reloj. */
  backgroundImage: string | null;
  logo: LogoStyle;
}

export type Corner = "topLeft" | "topRight" | "bottomLeft" | "bottomRight";

/** Logo o emblema como marca de agua en una esquina de la pantalla. */
export interface LogoStyle {
  /** null = sin logo. */
  image: string | null;
  corner: Corner;
  size: "small" | "medium" | "large";
  /** 0..1 */
  opacity: number;
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

/** «presentation» (PowerPoint, Impress) se convierte a PDF al prepararla. */
export type MediaKind = "image" | "video" | "pdf" | "presentation";

export interface MediaFile {
  path: string;
  name: string;
  kind: MediaKind;
}

export interface ObsDetection {
  installed: boolean;
  websocketEnabled: boolean;
  running: boolean;
  /** Abierto en modo seguro (sin WebSocket): hay que cerrarlo y abrirlo normal. */
  safeMode: boolean;
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

/** Letra guardada en la biblioteca. */
export interface Song {
  id: string;
  title: string;
  /** Quien la canta o interpreta. */
  singer: string;
  /** Quien escribió la letra. */
  writer: string;
  /** Texto completo; **** (o ***) en una línea separa las diapositivas. */
  body: string;
  /** Imagen o GIF propio; null = el fondo general. */
  background: string | null;
}

/** Ajustes tal como están en la base: clave → valor JSON. Las claves ausentes valen lo de por defecto. */
export type StoredSettings = Record<string, unknown>;

/** Error devuelto por el backend; la interfaz lo traduce al idioma elegido. */
export type CommandError =
  | { code: "monitorDisconnected" }
  | { code: "fileMissing"; path: string }
  | { code: "invalidContent" }
  | { code: "invalidBible"; file: string; reason: string }
  | { code: "bibleNotFound" }
  | { code: "obsNotInstalled" }
  | { code: "obsRunning" }
  | { code: "obsSafeMode" }
  | { code: "obsNotConnected" }
  | { code: "obs"; message: string }
  | { code: "storage"; message: string }
  | { code: "officeNotInstalled" }
  | { code: "conversionFailed"; file: string; reason: string }
  | { code: "officeBusy" }
  | { code: "officeCanceled" }
  | { code: "officeDownload"; message: string }
  | { code: "importFailed"; file: string; reason: string }
  | { code: "exportFailed"; reason: string }
  | { code: "copyFailed"; file: string; reason: string }
  | { code: "remoteFailed"; reason: string }
  | { code: "internal"; message: string };

/** LibreOffice para las presentaciones: el del sistema o la copia privada que Selah puede descargar. */
export interface OfficeStatus {
  system: boolean;
  privateCopy: boolean;
  /** Hay copia privada para este sistema. */
  downloadable: boolean;
  installing: boolean;
  version: string;
  /** Tamaño de la descarga en bytes. */
  size: number;
}

export interface OfficeProgress {
  downloaded: number;
  total: number;
  /** Ya descargado y comprobado; ahora se extrae. */
  extracting: boolean;
}

/** Una canción en la vista previa de una importación. */
export interface ImportSong {
  title: string;
  singer: string;
  slides: number;
}

export type ImportInvalidReason = "noTitle" | "noLyrics" | "repeated";

export interface ImportPreview {
  file: string;
  format: "selah" | "holyrics";
  fresh: ImportSong[];
  /** Ya están con la misma letra: no se tocan. */
  identical: number;
  /** Ya están con otra letra. */
  conflicts: ImportSong[];
  invalid: { title: string; reason: ImportInvalidReason }[];
  /** Canciones que trae como lista del culto; al importar pueden formar un plan. */
  setlist: number;
}

export type OnConflict = "skip" | "replace" | "keepBoth";

export interface ImportResult {
  added: number;
  replaced: number;
  skipped: number;
  invalid: number;
  /** Letras del plan creado con la lista del archivo; 0 si no se creó. */
  planSongs: number;
}

/** Un plan del culto: lo que se va a presentar, en orden (features/plans, D-019). */
export interface Plan {
  id: string;
  name: string;
  /** Día del culto, «AAAA-MM-DD»; null = sin fecha. */
  date: string | null;
  notes: string;
  items: PlanItem[];
}

export type PlanItem = { id: string } & (
  | { kind: "section"; title: string }
  | { kind: "song"; songId: string }
  | { kind: "media"; path: string; media: MediaKind }
  | { kind: "pause"; title: string; durationMs: number; background?: string | null }
  | {
      kind: "scripture";
      book: number;
      chapter: number;
      /** null = el capítulo entero. */
      verseStart: number | null;
      verseEnd: number | null;
      /** Versión fija (id de la Biblia); null = la elegida en la consola al mostrarlo. */
      bible?: string | null;
    }
);

export interface OutputState {
  open: boolean;
  monitorName: string | null;
  content: OutputContent;
  style: OutputStyle;
}

/** El control remoto, como lo muestra la consola en Ajustes. */
export interface RemoteInfo {
  running: boolean;
  /** Dirección para el teléfono, con la clave (la del QR). */
  url: string | null;
  /** El QR de `url` como SVG. */
  qr: string | null;
  /** false si no hay red: el teléfono no podrá llegar. */
  onNetwork: boolean;
  clients: number;
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
  /** Elige una imagen o GIF de fondo; null si se cancela. */
  pickBackground(title: string, filterName: string): Promise<string | null>;
  /** Vuelve a permitir fondos guardados tras reiniciar; devuelve los que siguen existiendo. */
  allowBackgrounds(paths: string[]): Promise<string[]>;
  /** Convierte una presentación a PDF (LibreOffice, PowerPoint o Keynote; o la toma de la caché) y devuelve la ruta del PDF. */
  convertPresentation(path: string): Promise<string>;
  /** Hay con qué convertir presentaciones. */
  officeAvailable(): Promise<boolean>;
  officeStatus(): Promise<OfficeStatus>;
  /** Descarga la copia privada de LibreOffice; termina cuando está lista. */
  officeInstall(): Promise<void>;
  officeCancel(): Promise<void>;
  /** Borra la copia privada (no un LibreOffice del sistema). */
  officeRemove(): Promise<void>;
  onOfficeProgress(handler: (progress: OfficeProgress) => void): Promise<() => void>;
  /** URL que el webview puede cargar para un archivo local ya permitido. */
  mediaUrl(path: string): string;
  reportOutputError(message: string): Promise<void>;
  /** Escribe un error de la interfaz en el registro de la app. */
  logError(message: string): Promise<void>;
  onOutputError(handler: (message: string) => void): Promise<() => void>;
  /** Avanzar (true) o retroceder lo que está en pantalla; lo pide la ventana de salida al pulsar una flecha. */
  outputNavigate(forward: boolean): Promise<void>;
  onOutputNavigate(handler: (forward: boolean) => void): Promise<() => void>;
  listBibles(): Promise<BibleInfo[]>;
  /** Abre el selector de archivos XML; devuelve las Biblias importadas ([] si se cancela). */
  importBibles(title: string, filterName: string): Promise<BibleInfo[]>;
  removeBible(id: string): Promise<void>;
  getChapter(bibleId: string, book: number, chapter: number): Promise<Verse[]>;
  searchBible(bibleId: string, query: string): Promise<SearchHit[]>;

  // Base de datos local
  settingsGet(): Promise<StoredSettings>;
  settingsSet(key: string, value: unknown): Promise<void>;
  /** Vuelve al valor inicial esas claves ([] = todas) y devuelve los ajustes que quedan. */
  settingsReset(keys: string[]): Promise<StoredSettings>;
  /** Cada cambio de ajustes, hecho desde cualquier ventana. */
  onSettingsChanged(handler: (settings: StoredSettings) => void): Promise<() => void>;
  /** Archivo de la base; null si se trabaja en memoria porque no se pudo abrir. */
  storePath(): Promise<string | null>;
  songsList(): Promise<Song[]>;
  songSave(song: Song): Promise<void>;
  songDelete(id: string): Promise<void>;
  /** Borra la biblioteca de letras (los planes las marcan como borradas). */
  songsReset(): Promise<void>;

  // Planes del culto (features/plans)
  /** Con fecha primero, del más próximo al más lejano; después los que no tienen fecha. */
  plansList(): Promise<Plan[]>;
  planSave(plan: Plan): Promise<void>;
  planDelete(id: string): Promise<void>;
  plansReset(): Promise<void>;
  /** Vuelve a permitir los archivos del plan guardado; devuelve los que ya no están en este equipo. */
  planAllowFiles(id: string): Promise<string[]>;
  /** Elige medios y los copia a la carpeta de la app; devuelve las copias, con el nombre original. [] si se cancela. */
  planPickMedia(title: string, filterName: string): Promise<MediaFile[]>;
  /** Elige el fondo de una pausa y lo copia a la carpeta de la app; null si se cancela. */
  planPickBackground(title: string, filterName: string): Promise<string | null>;
  /** Borra las copias que ya no usa ningún plan; devuelve lo que ocupan las que quedan, en bytes. */
  plansCleanFiles(): Promise<number>;

  // Control remoto desde el teléfono (features/remote)
  remoteInfo(): Promise<RemoteInfo>;
  remoteStart(): Promise<RemoteInfo>;
  remoteStop(): Promise<RemoteInfo>;
  /** Clave nueva: se desconectan los teléfonos y hay que volver a escanear el QR. */
  remoteResetKey(): Promise<RemoteInfo>;
  /** Lo que verán los teléfonos (features/remote/protocol.ts). */
  remotePublish(status: unknown): Promise<void>;
  /** Otro PDF en pantalla, de `total` páginas: se olvidan las miniaturas del anterior. */
  remoteThumbsStart(doc: string, total: number): Promise<boolean>;
  /** Una miniatura (JPEG) de la página `page` de `doc`, para los teléfonos. false si se descartó. */
  remoteThumb(doc: string, page: number, jpeg: Uint8Array): Promise<boolean>;
  onRemoteCommand(handler: (command: RemoteCommand) => void): Promise<() => void>;
  onRemoteClients(handler: (count: number) => void): Promise<() => void>;
  /** La pizarra entera: hojas y lo dibujado (features/remote/board.ts). */
  remoteBoard(): Promise<Board>;
  /** Un cambio en la pizarra desde la consola. false si no es válido. */
  remoteBoardOp(op: BoardOp): Promise<boolean>;
  /** La ventana de salida cuenta cómo es la pantalla y dónde quedó el texto, para dibujar encima desde el teléfono. */
  remoteScreen(screen: ScreenLayout | null): Promise<void>;
  onRemoteBoard(handler: (message: BoardMessage) => void): Promise<() => void>;

  // Importar y exportar letras (features/transfer)
  /** Elige un .selah o .hpls y devuelve la vista previa; null si se cancela. No guarda nada. */
  transferImportPick(title: string, filterName: string): Promise<ImportPreview | null>;
  /** Guarda lo de la última vista previa. Con planName, la lista del archivo se guarda además como un plan. */
  transferImportApply(onConflict: OnConflict, planName: string | null, importedSuffix: string): Promise<ImportResult>;
  transferImportCancel(): Promise<void>;
  /** Guarda la biblioteca (o solo las letras de un plan) en un .selah; null si se cancela. */
  transferExport(title: string, filterName: string, fileName: string, planId: string | null): Promise<{ path: string; songs: number } | null>;

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
