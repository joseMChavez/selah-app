// La pizarra vista desde las ventanas de Selah (consola y salida): una copia que sigue al backend.
// Los puntos no pasan por $state (serían miles de proxies): la pizarra es un objeto normal y `version` avisa
// de cada cambio a quien dibuja.
import { platform } from "$lib/platform";
import { applyMessage, emptyBoard, type Board, type Paper, type Stroke } from "./board";

let board: Board = emptyBoard();
const changes = $state({ version: 0 });
let watching = false;

function changed() {
  changes.version++;
}

/** Empieza a seguir la pizarra (una vez por ventana). */
export function watchBoard() {
  if (watching) return;
  watching = true;
  // Primero escuchar y luego pedirla entera: repetir un cambio que ya venía en ella no duplica nada.
  platform
    .onRemoteBoard((message) => {
      board = applyMessage(board, message);
      changed();
    })
    .catch(() => {});
  platform
    .remoteBoard()
    .then((full) => {
      board = { ...full, layers: { ...board.layers, ...full.layers } };
      changed();
    })
    .catch(() => {});
}

/** Lo dibujado sobre una capa. Leerlo dentro de un $effect lo vuelve a ejecutar con cada cambio. */
export function strokesOf(layer: string | null): readonly Stroke[] {
  void changes.version;
  return (layer && board.layers[layer]) || [];
}

export function sheetPaper(id: string): Paper {
  void changes.version;
  return board.sheets.find((s) => s.id === id)?.paper ?? "black";
}

/** Posición de la hoja (1, 2…) para nombrarla «Hoja 2». */
export function sheetNumber(id: string): number {
  void changes.version;
  return board.sheets.findIndex((s) => s.id === id) + 1;
}
