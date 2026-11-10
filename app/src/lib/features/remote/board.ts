// Pizarra del control remoto (D-021): lo dibujado desde el teléfono, encima de lo que hay en pantalla o en hojas
// en blanco. Lo guarda el backend (src-tauri/src/features/remote/board.rs) mientras el control remoto está encendido.
// Este archivo no importa nada de Tauri: lo usan el teléfono (en un navegador), la consola y la ventana de salida,
// y los tres dibujan igual con drawLayer.
import type { OutputContent } from "$lib/platform/types";

/** Las coordenadas van de 0 a COORD sobre la pantalla entera (enteros: más cortos en JSON). */
export const COORD = 10_000;
/** Números (x e y) por mensaje: igual que MAX_BATCH en board.rs. */
export const MAX_BATCH = 2_000;
/** Igual que MAX_SHEETS en board.rs. */
export const MAX_SHEETS = 30;

export type InkColor = "white" | "black" | "red" | "yellow" | "blue" | "green";
export type InkSize = "thin" | "medium" | "thick";
export type InkTool = "pen" | "marker";
export type Paper = "black" | "white";

export const COLORS: InkColor[] = ["white", "black", "red", "yellow", "blue", "green"];
export const SIZES: InkSize[] = ["thin", "medium", "thick"];

export interface Stroke {
  id: string;
  tool: InkTool;
  color: InkColor;
  size: InkSize;
  /** x0, y0, x1, y1… entre 0 y COORD. */
  points: number[];
}

export interface Sheet {
  id: string;
  paper: Paper;
}

/** Una palabra del texto en pantalla y dónde quedó, entre 0 y COORD. */
export interface ScreenWord {
  t: string;
  x: number;
  y: number;
  w: number;
  h: number;
  /** Texto, referencia o segunda versión: el teléfono los pinta del mismo color que la pantalla. */
  k: "text" | "ref" | "second";
}

/** Cómo es la pantalla y, si hay texto, dónde quedó cada palabra. Lo mide la ventana de salida. */
export interface ScreenLayout {
  /** Ancho entre alto de la ventana de salida. */
  aspect: number;
  /** La capa del contenido medido (layerKey). */
  layer: string | null;
  /** Color de fondo de la pantalla. */
  paper: string;
  words: ScreenWord[];
}

export interface Board {
  sheets: Sheet[];
  layers: Record<string, Stroke[]>;
  screen: ScreenLayout | null;
}

export type BoardOp =
  /** Un trazo nuevo o su continuación: `points` sustituye desde el número `at`. Repetirla no duplica nada. */
  | { type: "draw"; layer: string; stroke: string; tool: InkTool; color: InkColor; size: InkSize; at: number; points: number[] }
  | { type: "erase"; layer: string; stroke: string }
  | { type: "wipe"; layer: string }
  | { type: "addSheet"; id: string }
  | { type: "setPaper"; id: string; paper: Paper };

/** Lo que reparte el backend: la pizarra entera, un cambio, o que se borró todo al apagar el control remoto. */
export type BoardMessage =
  | { type: "board"; board: Board }
  | { type: "op"; op: BoardOp }
  | { type: "screen"; screen: ScreenLayout | null }
  | { type: "reset" };

export function emptyBoard(): Board {
  return { sheets: [], layers: {}, screen: null };
}

/** La misma regla que Board::apply en board.rs (sin los límites, que ya aplicó el backend). */
export function applyOp(board: Board, op: BoardOp): void {
  switch (op.type) {
    case "draw": {
      const strokes = (board.layers[op.layer] ??= []);
      const found = strokes.find((s) => s.id === op.stroke);
      if (found) {
        if (op.at > found.points.length) return;
        found.points.length = op.at;
        found.points.push(...op.points);
      } else if (op.at === 0) {
        strokes.push({ id: op.stroke, tool: op.tool, color: op.color, size: op.size, points: [...op.points] });
      }
      return;
    }
    case "erase": {
      const strokes = board.layers[op.layer];
      if (strokes) board.layers[op.layer] = strokes.filter((s) => s.id !== op.stroke);
      return;
    }
    case "wipe":
      delete board.layers[op.layer];
      return;
    case "addSheet":
      if (!board.sheets.some((s) => s.id === op.id)) board.sheets.push({ id: op.id, paper: "black" });
      return;
    case "setPaper": {
      const sheet = board.sheets.find((s) => s.id === op.id);
      if (sheet) sheet.paper = op.paper;
      return;
    }
  }
}

/** Aplica un mensaje del backend. Devuelve la pizarra que queda (la misma, o una nueva si llegó entera). */
export function applyMessage(board: Board, message: BoardMessage): Board {
  switch (message.type) {
    case "board": return message.board;
    case "reset": return { ...emptyBoard(), screen: board.screen };
    case "screen": return { ...board, screen: message.screen };
    case "op":
      applyOp(board, message.op);
      return board;
  }
}

/** Ids cortos para trazos y hojas. Sin crypto.randomUUID: el teléfono abre la página por http y allí no existe. */
export function newId(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`;
}

/** Huella corta de un texto largo (FNV-1a), para que la clave de capa no crezca con la letra de una canción. */
function hash(text: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return (h >>> 0).toString(36);
}

function short(text: string): string {
  return text.length <= 120 ? text : `${text.slice(0, 80)}#${hash(text)}`;
}

/** Sobre qué se dibuja: cada página, versículo o diapositiva tiene sus propios trazos.
 *  null si en esa pantalla no se dibuja (vacía, en negro, patrón de prueba). */
export function layerKey(content: OutputContent): string | null {
  switch (content.kind) {
    case "board": return `sheet:${content.sheet}`;
    case "pdf": return `pdf:${content.page}:${short(content.path)}`;
    case "scripture": {
      const verse = content.verses[content.index];
      return verse ? `bible:${short(`${content.version}|${content.second?.version ?? ""}|${verse.reference}`)}` : null;
    }
    case "text": return `text:${content.index}:${short(content.slides[content.index] ?? "")}`;
    case "image": return `image:${short(content.path)}`;
    case "video": return `video:${short(content.path)}`;
    case "clock": return "clock";
    case "timer": return "timer";
    default: return null;
  }
}

export const PAPER_COLOR: Record<Paper, string> = { black: "#000000", white: "#ffffff" };

/** Tinta según el fondo, para que se lea desde lejos: en una hoja, el blanco y el negro son «la tinta que contrasta»
 *  (si se cambia el fondo, lo escrito sigue viéndose) y los colores se aclaran u oscurecen. Encima de lo que hay
 *  en pantalla, cada color es el que se eligió. */
const INK: Record<"black" | "white" | "over", Record<InkColor, string>> = {
  black: { white: "#ffffff", black: "#ffffff", red: "#ff5a52", yellow: "#ffd23f", blue: "#6aa8ff", green: "#4cd97b" },
  white: { white: "#111111", black: "#111111", red: "#d32f2f", yellow: "#d18f00", blue: "#1f5fd6", green: "#178a3c" },
  over: { white: "#ffffff", black: "#111111", red: "#ff3b30", yellow: "#ffd60a", blue: "#2f7bff", green: "#34c759" },
};

export function inkColor(color: InkColor, paper: Paper | null): string {
  return INK[paper ?? "over"][color];
}

/** Grosor en fracción del alto de la pantalla: se ve igual en el teléfono y en el proyector. */
const WIDTH: Record<InkTool, Record<InkSize, number>> = {
  pen: { thin: 0.005, medium: 0.01, thick: 0.02 },
  marker: { thin: 0.03, medium: 0.05, thick: 0.08 },
};

export function strokeWidth(tool: InkTool, size: InkSize, height: number): number {
  return Math.max(1, WIDTH[tool][size] * height);
}

function tracePath(ctx: CanvasRenderingContext2D, points: number[], w: number, h: number) {
  const x = (i: number) => (points[i] / COORD) * w;
  const y = (i: number) => (points[i + 1] / COORD) * h;
  ctx.beginPath();
  ctx.moveTo(x(0), y(0));
  if (points.length === 2) {
    // Un toque: un punto.
    ctx.lineTo(x(0) + 0.01, y(0));
    return;
  }
  // Curvas por los puntos medios: la línea sale suave aunque lleguen pocos puntos.
  for (let i = 2; i < points.length - 2; i += 2) {
    ctx.quadraticCurveTo(x(i), y(i), (x(i) + x(i + 2)) / 2, (y(i) + y(i + 2)) / 2);
  }
  ctx.lineTo(x(points.length - 2), y(points.length - 2));
}

/** Dibuja los trazos en un canvas de w×h píxeles. `paper` es el fondo de la hoja, o null encima de otra cosa. */
export function drawLayer(ctx: CanvasRenderingContext2D, strokes: readonly Stroke[], w: number, h: number, paper: Paper | null) {
  ctx.clearRect(0, 0, w, h);
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  for (const stroke of strokes) {
    if (stroke.points.length < 2) continue;
    const width = strokeWidth(stroke.tool, stroke.size, h);
    tracePath(ctx, stroke.points, w, h);
    if (stroke.tool === "marker") {
      // Translúcido: el texto de debajo se sigue leyendo. Más no se ve sobre un fondo oscuro.
      ctx.globalAlpha = 0.5;
      ctx.strokeStyle = inkColor(stroke.color, paper);
      ctx.lineWidth = width;
      ctx.stroke();
      ctx.globalAlpha = 1;
      continue;
    }
    if (paper === null) {
      // Encima de un versículo o una foto, un borde que contrasta: la línea se ve sobre cualquier fondo.
      ctx.strokeStyle = stroke.color === "black" ? "rgb(255 255 255 / 0.6)" : "rgb(0 0 0 / 0.45)";
      ctx.lineWidth = width + Math.max(2, h * 0.004);
      ctx.stroke();
    }
    ctx.strokeStyle = inkColor(stroke.color, paper);
    ctx.lineWidth = width;
    ctx.stroke();
  }
}

/** Distancia del punto (px, py) al segmento a-b. */
function segmentDistance(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const dx = bx - ax;
  const dy = by - ay;
  const length = dx * dx + dy * dy;
  const t = length === 0 ? 0 : Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / length));
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
}

/** Los trazos que toca el borrador en (x, y), en píxeles de un lienzo de w×h. `radius` también en píxeles. */
export function strokesAt(strokes: readonly Stroke[], x: number, y: number, w: number, h: number, radius: number): Stroke[] {
  const sx = w / COORD;
  const sy = h / COORD;
  return strokes.filter((stroke) => {
    const reach = radius + strokeWidth(stroke.tool, stroke.size, h) / 2;
    const p = stroke.points;
    if (p.length === 2) return Math.hypot(p[0] * sx - x, p[1] * sy - y) <= reach;
    for (let i = 0; i + 3 < p.length; i += 2) {
      if (segmentDistance(x, y, p[i] * sx, p[i + 1] * sy, p[i + 2] * sx, p[i + 3] * sy) <= reach) return true;
    }
    return false;
  });
}

/** Un trazo entero como operaciones de dibujo, en trozos que el backend acepta (para deshacer un borrado). */
export function redrawOps(layer: string, stroke: Stroke): BoardOp[] {
  const ops: BoardOp[] = [];
  for (let at = 0; at < Math.max(stroke.points.length, 1); at += MAX_BATCH) {
    ops.push({ type: "draw", layer, stroke: stroke.id, tool: stroke.tool, color: stroke.color, size: stroke.size, at, points: stroke.points.slice(at, at + MAX_BATCH) });
  }
  return ops;
}
