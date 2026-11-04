// El control remoto visto desde la consola: encender y apagar el servidor, y contarle a los teléfonos
// lo que hay en pantalla. Las órdenes del teléfono las aplica Console.svelte, igual que sus botones.
import { bookLanguage } from "$lib/features/bible/scripture";
import { currentBibles, visibleBibles } from "$lib/features/bible/bibles.svelte";
import type { DeckPosition } from "$lib/features/output/deck";
import { toMessage, type Message } from "$lib/core/messages";
import { setSetting, settings } from "$lib/core/settings.svelte";
import { platform, type OutputContent, type OutputState, type RemoteInfo } from "$lib/platform";
import type { RemoteItem, RemoteLive, RemoteStatus } from "./protocol";

const MAX_RECENT = 6;
const MAX_KEY = 400;

export const remote = $state({
  info: null as RemoteInfo | null,
  busy: false,
  error: null as Message | null,
});

/** Enciende o apaga el servidor según el ajuste «remoteEnabled». La consola lo llama cada vez que cambia. */
export async function syncRemote(enabled: boolean) {
  if (remote.busy) return;
  remote.busy = true;
  remote.error = null;
  try {
    const info = await platform.remoteInfo();
    if (enabled === info.running) remote.info = info;
    else remote.info = enabled ? await platform.remoteStart() : await platform.remoteStop();
  } catch (e) {
    remote.error = toMessage(e);
  } finally {
    remote.busy = false;
  }
  // Si el ajuste cambió mientras se encendía o apagaba, se aplica el último.
  if (remote.info && remote.info.running !== settings.remoteEnabled && !remote.error) syncRemote(settings.remoteEnabled);
}

export async function resetRemoteKey() {
  try {
    remote.info = await platform.remoteResetKey();
  } catch (e) {
    remote.error = toMessage(e);
  }
}

/** Mantiene al día cuántos teléfonos hay conectados. */
export function watchRemoteClients(): () => void {
  const unlisten = platform.onRemoteClients((count) => {
    if (remote.info) remote.info.clients = count;
  });
  return () => void unlisten.then((fn) => fn());
}

/** Lo que estuvo en pantalla hace poco, lo más reciente primero. Cada cosa una vez, en la posición en que quedó. */
const recent = $state<OutputContent[]>([]);

/** Qué es «lo mismo» para la lista: la misma presentación, el mismo pasaje, la misma letra… en cualquier posición.
 *  null para lo que no tiene sentido volver a poner (negro, vacío, patrón, temporizador). */
export function itemKey(content: OutputContent): string | null {
  const key = (() => {
    switch (content.kind) {
      case "pdf": return `pdf:${content.path}`;
      case "image": return `image:${content.path}`;
      case "video": return `video:${content.path}`;
      case "scripture": return `bible:${content.version}:${content.verses[0]?.reference}:${content.verses.length}`;
      case "text": return `text:${content.slides.length}:${content.slides[0]}`;
      case "clock": return "clock";
      default: return null;
    }
  })();
  return key && key.slice(0, MAX_KEY);
}

/** La consola lo llama con cada cambio en pantalla (fuera del seguimiento del $effect: lee y escribe la lista). */
export function rememberLive(content: OutputContent) {
  const key = itemKey(content);
  if (!key) return;
  const old = recent.findIndex((c) => itemKey(c) === key);
  if (old >= 0) recent.splice(old, 1);
  recent.unshift($state.snapshot(content) as OutputContent);
  recent.length = Math.min(recent.length, MAX_RECENT);
}

/** El contenido de un elemento de «Para mostrar»: lo preparado o lo reciente con esa clave. */
export function findItem(key: string, prepared: OutputContent): OutputContent | null {
  if (itemKey(prepared) === key) return prepared;
  const found = recent.find((c) => itemKey(c) === key);
  return found ? ($state.snapshot(found) as OutputContent) : null;
}

/** El teléfono elige versión: cambia la de arriba en la consola (si era la de abajo, deja de serlo). */
export function chooseBible(id: string) {
  if (!visibleBibles().some((b) => b.id === id)) return;
  if (settings.bible2 === id) setSetting("bible2", null);
  setSetting("bible", id);
}

function items(live: OutputContent, prepared: OutputContent, describe: (c: OutputContent) => string): RemoteItem[] {
  const liveKey = itemKey(live);
  const preparedKey = itemKey(prepared);
  const list: RemoteItem[] = [];
  if (preparedKey && preparedKey !== liveKey) list.push({ key: preparedKey, label: describe(prepared), prepared: true });
  for (const c of recent) {
    const key = itemKey(c)!;
    if (key !== liveKey && key !== preparedKey) list.push({ key, label: describe(c), prepared: false });
  }
  return list.slice(0, MAX_RECENT);
}

function liveOf(content: OutputContent, description: string): RemoteLive {
  switch (content.kind) {
    case "empty":
    case "black":
      return { kind: content.kind };
    case "scripture": {
      const verse = content.verses[content.index];
      return { kind: "scripture", reference: verse?.reference ?? "", text: verse?.text ?? "", version: content.version };
    }
    case "pdf":
      return { kind: "pdf", name: content.path.split(/[\\/]/).pop() ?? content.path };
    case "text":
      return { kind: "text", text: content.slides[content.index] ?? "" };
    default:
      return { kind: "other", description };
  }
}

/** Lo que verán los teléfonos. `description` es el texto de la consola para lo que el teléfono no sabe contar. */
export function remoteStatus(
  output: OutputState,
  prepared: OutputContent,
  position: DeckPosition | null,
  canUnblack: boolean,
  describe: (c: OutputContent) => string,
): RemoteStatus {
  const { bible } = currentBibles();
  const description = describe(output.content);
  const kind = output.content.kind;
  return {
    open: output.open,
    live: liveOf(output.content, description),
    position: position && { n: position.n, total: position.total, unit: kind === "pdf" ? "page" : kind === "scripture" ? "verse" : "slide" },
    canUnblack,
    confirmSend: settings.confirmSend,
    items: items(output.content, prepared, describe),
    bibles: visibleBibles().map((b) => ({ id: b.id, abbreviation: b.abbreviation, name: b.name })),
    bible: bible && { id: bible.id, abbreviation: bible.abbreviation, language: bookLanguage(bible), books: bible.books },
  };
}
