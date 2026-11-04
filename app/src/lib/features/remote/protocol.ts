// Lo que se dicen el teléfono y la consola a través del servidor del control remoto (src-tauri/src/features/remote).
// Este archivo no importa nada de Tauri: lo usa también la página del teléfono, que corre en un navegador.

/** Órdenes del teléfono. Deben coincidir con `domain::Command` del backend. */
export type RemoteCommand =
  | { type: "next" }
  | { type: "previous" }
  | { type: "black" }
  | { type: "clear" }
  | { type: "scripture"; book: number; chapter: number; verse: number };

/** Lo que está en pantalla, contado para el teléfono. */
export type RemoteLive =
  | { kind: "empty" }
  | { kind: "black" }
  | { kind: "scripture"; reference: string; text: string; version: string }
  | { kind: "pdf"; name: string }
  | { kind: "text"; text: string }
  /** Imagen, video, reloj…: el teléfono muestra la descripción de la consola. */
  | { kind: "other"; description: string };

/** Lo que la consola publica cada vez que cambia algo. */
export interface RemoteStatus {
  /** La ventana de salida está abierta. */
  open: boolean;
  live: RemoteLive;
  /** Posición en lo que está en pantalla, si se puede avanzar (versículos, páginas, diapositivas). */
  position: { n: number; total: number | null; unit: "verse" | "page" | "slide" } | null;
  /** En negro con algo a lo que volver. */
  canUnblack: boolean;
  /** Preguntar antes de mostrar algo nuevo (el mismo ajuste de la consola). */
  confirmSend: boolean;
  /** La Biblia elegida en la consola: el teléfono busca en ella. */
  bible: {
    id: string;
    abbreviation: string;
    /** Idioma de los nombres de libro. */
    language: string | null;
    books: { number: number; chapters: number }[];
  } | null;
}

/** Mensaje del servidor al teléfono. `status` es null hasta que la consola publica por primera vez. */
export interface ServerMessage {
  type: "status";
  status: RemoteStatus | null;
}
