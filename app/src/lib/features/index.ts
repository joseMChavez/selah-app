// Paneles de contenido de la consola, en el orden de la barra lateral. Un panel nuevo se añade aquí.
import type { SourcePanel } from "$lib/core/panels";
import { biblePanel } from "./bible";
import { clockPanel } from "./clock";
import { mediaPanel } from "./media";
import { planPanel } from "./plans";
import { textPanel } from "./songs";
import { transferPanel } from "./transfer";

export const sourcePanels: SourcePanel[] = [planPanel, biblePanel, textPanel, mediaPanel, clockPanel, transferPanel];
