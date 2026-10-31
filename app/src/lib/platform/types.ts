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
  | { kind: "pdf"; path: string; page: number };

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
  | { code: "internal"; message: string };

export interface OutputState {
  open: boolean;
  monitorName: string | null;
  content: OutputContent;
}

export interface Platform {
  listMonitors(): Promise<MonitorInfo[]>;
  openOutput(monitorIndex: number): Promise<OutputState>;
  closeOutput(): Promise<void>;
  setOutputContent(content: OutputContent): Promise<OutputState>;
  getOutputState(): Promise<OutputState>;
  /** Devuelve la función para dejar de escuchar. */
  onOutputState(handler: (state: OutputState) => void): Promise<() => void>;
  /** Abre el selector de archivos; devuelve [] si se cancela. */
  pickMedia(title: string, filterName: string): Promise<MediaFile[]>;
  /** URL que el webview puede cargar para un archivo local ya permitido. */
  mediaUrl(path: string): string;
  reportOutputError(message: string): Promise<void>;
  onOutputError(handler: (message: string) => void): Promise<() => void>;
}
