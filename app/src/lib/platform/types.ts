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
  | { kind: "text"; text: string };

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
}
