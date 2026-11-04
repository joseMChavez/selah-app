// El control remoto visto desde la consola: encender y apagar el servidor, y contarle a los teléfonos
// lo que hay en pantalla. Las órdenes del teléfono las aplica Console.svelte, igual que sus botones.
import { bookLanguage } from "$lib/features/bible/scripture";
import { currentBibles } from "$lib/features/bible/bibles.svelte";
import type { DeckPosition } from "$lib/features/output/deck";
import { toMessage, type Message } from "$lib/core/messages";
import { settings } from "$lib/core/settings.svelte";
import { platform, type OutputContent, type OutputState, type RemoteInfo } from "$lib/platform";
import type { RemoteLive, RemoteStatus } from "./protocol";

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
export function remoteStatus(output: OutputState, position: DeckPosition | null, canUnblack: boolean, description: string): RemoteStatus {
  const { bible } = currentBibles();
  const kind = output.content.kind;
  return {
    open: output.open,
    live: liveOf(output.content, description),
    position: position && { n: position.n, total: position.total, unit: kind === "pdf" ? "page" : kind === "scripture" ? "verse" : "slide" },
    canUnblack,
    confirmSend: settings.confirmSend,
    bible: bible && { id: bible.id, abbreviation: bible.abbreviation, language: bookLanguage(bible), books: bible.books },
  };
}
