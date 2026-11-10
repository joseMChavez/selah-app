// Lo que se dicen el teléfono y la consola a través del servidor del control remoto (src-tauri/src/features/remote).
// Este archivo no importa nada de Tauri: lo usa también la página del teléfono, que corre en un navegador.
import type { BoardMessage } from "./board";

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
  | { type: "planItem"; id: string }
  /** Ir a la posición n (desde 1) de lo que está en pantalla: página, diapositiva o versículo. */
  | { type: "goTo"; n: number }
  /** Poner en pantalla una hoja de la pizarra (board.ts). */
  | { type: "showSheet"; id: string };

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

/** Lo que se puede elegir de lo que está en pantalla. La posición actual es `position.n`. */
export type RemoteStrip =
  /** Páginas de un PDF: sus miniaturas están en /api/thumb?doc=…&n=…, mientras siga en pantalla. */
  | { kind: "pages"; doc: string; total: number }
  /** Diapositivas de texto o versículos: una etiqueta corta por cada uno. */
  | { kind: "labels"; labels: string[] };

/** Lo que está en pantalla, contado para el teléfono. */
export type RemoteLive =
  | { kind: "empty" }
  | { kind: "black" }
  | { kind: "scripture"; reference: string; text: string; version: string }
  | { kind: "pdf"; name: string }
  | { kind: "text"; text: string }
  /** Una hoja de la pizarra. */
  | { kind: "board"; sheet: string }
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
  /** Para ver y elegir dentro de lo que está en pantalla; null si no hay nada que elegir. */
  strip: RemoteStrip | null;
  /** Sobre qué se dibuja en la pizarra ahora; null si no se puede (pantalla vacía o en negro). */
  canvas: RemoteCanvas | null;
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

/** Lo que está en pantalla, para dibujar encima: la capa de sus trazos y qué poner de fondo en el teléfono. */
export interface RemoteCanvas {
  /** La capa (board.ts, layerKey). */
  layer: string;
  under:
    /** Una hoja de la pizarra: su fondo está en la pizarra. */
    | { kind: "sheet"; id: string }
    /** Una página de PDF: la miniatura de /api/thumb. */
    | { kind: "page"; doc: string; n: number }
    /** Texto o versículo: dónde quedó cada palabra llega en la pizarra (screen), medido por la ventana de salida. */
    | { kind: "text"; text: string }
    /** Imagen, video, reloj…: solo se describe. */
    | { kind: "other"; description: string };
}

/** Mensaje del servidor al teléfono. `status` es null hasta que la consola publica por primera vez.
 *  Los de la pizarra (board.ts) llegan por el mismo WebSocket. */
export type ServerMessage = { type: "status"; status: RemoteStatus | null } | BoardMessage;
