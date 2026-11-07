// El control remoto visto desde la consola: encender y apagar el servidor, y contarle a los teléfonos
// lo que hay en pantalla. Las órdenes del teléfono las aplica Console.svelte, igual que sus botones.
import { bookLanguage } from "$lib/features/bible/scripture";
import { currentBibles, visibleBibles } from "$lib/features/bible/bibles.svelte";
import type { DeckPosition } from "$lib/features/output/deck";
import { lastSeen, recentScreens, screenKey } from "$lib/features/output/history.svelte";
import { toMessage, type Message } from "$lib/core/messages";
import { setSetting, settings } from "$lib/core/settings.svelte";
import { platform, type OutputContent, type OutputState, type RemoteInfo } from "$lib/platform";
import { openPlan } from "$lib/features/plans/plans.svelte";
import { detail, icon, problem, session, title } from "$lib/features/plans/items.svelte";
import type { RemoteItem, RemoteLive, RemotePlan, RemoteStatus } from "./protocol";

const MAX_ITEMS = 6;

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

/** El contenido de un elemento de «Para mostrar». Si también estuvo en pantalla, vale cómo quedó allí
 *  (la presentación en la página a la que se llegó, no en la que se preparó). */
export function findItem(key: string, prepared: OutputContent): OutputContent | null {
  return lastSeen(key) ?? (screenKey(prepared) === key ? prepared : null);
}

/** El teléfono elige versión: cambia la de arriba en la consola (si era la de abajo, deja de serlo). */
export function chooseBible(id: string) {
  if (!visibleBibles().some((b) => b.id === id)) return;
  if (settings.bible2 === id) setSetting("bible2", null);
  setSetting("bible", id);
}

function items(live: OutputContent, prepared: OutputContent, describe: (c: OutputContent) => string): RemoteItem[] {
  const liveKey = screenKey(live);
  const preparedKey = screenKey(prepared);
  const list: RemoteItem[] = [];
  // Lo preparado solo si no estuvo ya en pantalla: si estuvo, se ofrece cómo quedó allí.
  if (preparedKey && preparedKey !== liveKey && !lastSeen(preparedKey)) {
    list.push({ key: preparedKey, label: describe(prepared), prepared: true });
  }
  for (const c of recentScreens()) {
    const key = screenKey(c)!;
    if (key !== liveKey) list.push({ key, label: describe(c), prepared: false });
  }
  return list.slice(0, MAX_ITEMS);
}

/** El plan abierto, contado para el teléfono: el predicador tiene sus versículos y lo demás a un toque. */
function planOf(): RemotePlan | null {
  const plan = openPlan.plan;
  if (!plan) return null;
  const shown = session.shown[plan.id] ?? [];
  return {
    name: plan.name,
    items: plan.items.map((item) =>
      item.kind === "section"
        ? { kind: "section", id: item.id, title: item.title }
        : {
            kind: "item",
            id: item.id,
            icon: icon(item),
            title: title(item),
            detail: problem(item) ?? detail(item),
            available: problem(item) === null,
            shown: shown.includes(item.id),
            current: session.current?.id === item.id,
          },
    ),
  };
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
    plan: planOf(),
    bibles: visibleBibles().map((b) => ({ id: b.id, abbreviation: b.abbreviation, name: b.name })),
    bible: bible && { id: bible.id, abbreviation: bible.abbreviation, language: bookLanguage(bible), books: bible.books },
  };
}
