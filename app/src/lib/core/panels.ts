// Paneles de «Contenido» de la consola (Biblia, Textos, Medios, Reloj…). Cada funcionalidad declara el suyo en
// features/<x>/index.ts y la consola arma la barra lateral recorriendo features/index.ts: para añadir un panel
// no hay que tocar la consola. Es el mismo contrato que usarán los plugins externos.
import type { Component } from "svelte";
import type { Message } from "$lib/core/messages";
import type { MessageKey } from "$lib/i18n/index.svelte";
import type { OutputContent } from "$lib/platform";

/** Lo que la consola da a todos los paneles; cada uno usa lo que necesita. */
export interface PanelProps {
  /** Lo preparado en la vista previa. */
  prepared: OutputContent;
  /** Lo que está en pantalla ahora. */
  live: OutputContent;
  onprepare: (content: OutputContent) => void;
  /** Prepara y envía a pantalla de una vez (doble clic, Enter). */
  onsend: (content: OutputContent) => void;
  onerror: (message: Message) => void;
}

export interface SourcePanel {
  /** Se guarda en los ajustes («tab»): no cambiarlo una vez publicado. */
  id: string;
  label: MessageKey;
  /** Trazo SVG de 24×24; hereda el color del texto. */
  icon: string;
  component: Component<PanelProps>;
}
