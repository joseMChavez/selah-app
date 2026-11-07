// Lo que se dicen el teléfono y la consola a través del servidor del control remoto (src-tauri/src/features/remote).
// Este archivo no importa nada de Tauri: lo usa también la página del teléfono, que corre en un navegador.

/** Órdenes del teléfono. Deben coincidir con `domain::Command` del backend. */
export type RemoteCommand =
  | { type: "next" }
  | { type: "previous" }
  | { type: "black" }
  | { type: "clear" }
  | { type: "scripture"; book: number; chapter: number; verse: number }
  | { type: "setBible"; id: string }
  | { type: "show"; key: string }
  /** Un elemento del plan abierto en la consola, por su id. */
  | { type: "planItem"; id: string };

/** Algo que se puede volver a poner en pantalla desde el teléfono: lo preparado en la consola o lo reciente. */
export interface RemoteItem {
  key: string;
  /** Cómo lo describe la consola: «Juan 3:16 (LBLA)», «Presentación.pptx, página 5»… */
  label: string;
  /** Lo preparado en la consola (aún no enviado). */
  prepared: boolean;
}

/** El plan abierto en la consola, para tenerlo a un toque (D-019). */
export interface RemotePlan {
  name: string;
  items: RemotePlanItem[];
}

export type RemotePlanItem =
  | { kind: "section"; id: string; title: string }
  | {
      kind: "item";
      id: string;
      icon: string;
      title: string;
      detail: string;
      /** false si no se puede mostrar (letra borrada, archivo que no está…). */
      available: boolean;
      /** Ya pasó por la pantalla en este culto. */
      shown: boolean;
      /** Es lo último preparado o mostrado desde el plan. */
      current: boolean;
    };

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
  /** Lo preparado y lo que estuvo en pantalla hace poco, lo más reciente primero (sin lo que está ahora). */
  items: RemoteItem[];
  /** El plan abierto en la consola; null si no hay. */
  plan: RemotePlan | null;
  /** Las versiones que se pueden elegir. */
  bibles: { id: string; abbreviation: string; name: string }[];
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
