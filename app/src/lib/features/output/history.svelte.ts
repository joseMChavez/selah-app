// Lo que pasó por la pantalla hace poco, cada cosa una vez y en la posición en que quedó.
// Sirve para volver a una presentación en su página después de mostrar un versículo, desde la consola
// (Medios) o desde el teléfono («Para mostrar» del control remoto).
import type { OutputContent } from "$lib/platform";

const MAX_RECENT = 6;
const MAX_KEY = 400;

/** Lo más reciente primero. */
const recent = $state<OutputContent[]>([]);

/** Qué es «lo mismo»: la misma presentación, el mismo pasaje, la misma letra… en cualquier posición.
 *  null para lo que no tiene sentido recordar (negro, vacío, patrón, temporizador). */
export function screenKey(content: OutputContent): string | null {
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

/** La consola lo llama con cada cambio en pantalla, fuera del seguimiento de su $effect (lee y escribe la lista). */
export function rememberLive(content: OutputContent) {
  const key = screenKey(content);
  if (!key) return;
  const old = recent.findIndex((c) => screenKey(c) === key);
  if (old >= 0) recent.splice(old, 1);
  recent.unshift($state.snapshot(content) as OutputContent);
  recent.length = Math.min(recent.length, MAX_RECENT);
}

/** Lo reciente, lo más nuevo primero. */
export function recentScreens(): readonly OutputContent[] {
  return recent;
}

/** Cómo quedó en pantalla lo que tiene esa clave, o null si no está entre lo reciente. */
export function lastSeen(key: string): OutputContent | null {
  const found = recent.find((c) => screenKey(c) === key);
  return found ? ($state.snapshot(found) as OutputContent) : null;
}

/** Una presentación que se vuelve a elegir desde el principio sigue en la página donde quedó. */
export function resume(content: OutputContent): OutputContent {
  if (content.kind !== "pdf" || content.page !== 1) return content;
  const seen = lastSeen(screenKey(content)!);
  return seen?.kind === "pdf" ? { ...content, page: seen.page } : content;
}
