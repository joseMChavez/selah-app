// Cada elemento de un plan: cómo se llama, qué le falta y qué se pone en pantalla. Lo usan el panel Plan y la consola
// (para el control remoto, que pide elementos del plan aunque el panel no esté a la vista).
import { t, tCount } from "$lib/i18n/index.svelte";
import { formatCountdown, preparedTimer } from "$lib/features/clock/clock";
import { screenKey } from "$lib/features/output/history.svelte";
import { library, songContent } from "$lib/features/songs/library.svelte";
import { splitSlides } from "$lib/features/songs/slides";
import { bibles } from "$lib/features/bible/bibles.svelte";
import { platform, type OutputContent } from "$lib/platform";
import { missingFiles, type PlanItem } from "./plans.svelte";
import { bibleFor, missingIn, referenceLabel, scriptureContent } from "./scripture";

const ICONS = { song: "♪", image: "🖼", video: "🎬", pdf: "📄", presentation: "📊", pause: "⏱", scripture: "📖" } as const;

/** El uso del plan en esta sesión: lo preparado desde él y lo que ya pasó por la pantalla. No se guarda. */
export const session = $state({
  /** Elemento preparado desde el plan y lo que se preparó con él (para saber cuándo llega a la pantalla). */
  current: null as { id: string; content: OutputContent } | null,
  /** Ids de los elementos que ya se mostraron, por plan. */
  shown: {} as Record<string, string[]>,
  /** Presentación que se está convirtiendo a PDF (id del elemento). */
  converting: null as string | null,
});

/** Presentación → PDF ya convertido, para no volver a pedirlo. */
const converted = new Map<string, string>();

export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function songOf(item: PlanItem) {
  return item.kind === "song" ? library.songs.find((s) => s.id === item.songId) : undefined;
}

/** Lo que impide mostrar un elemento: la letra se borró, el archivo ya no está o la versión no tiene la cita. */
export function problem(item: PlanItem): string | null {
  if (item.kind === "song" && !songOf(item)) return t("plans.songDeleted");
  if (item.kind === "scripture" && !bibles.loading && missingIn(item, bibleFor(item))) return t("plans.notInBible");
  if (item.kind === "media" && missingFiles.has(item.path)) return t("plans.fileMissing");
  return null;
}

export function title(item: PlanItem): string {
  switch (item.kind) {
    case "section": return item.title;
    case "song": return songOf(item)?.title ?? t("plans.song");
    case "media": return baseName(item.path);
    case "pause": return item.title || t("plans.pause");
    case "scripture": return referenceLabel(item, bibleFor(item));
  }
}

export function detail(item: PlanItem): string {
  switch (item.kind) {
    case "song": {
      const song = songOf(item);
      return song ? [song.singer, tCount("text.count", splitSlides(song.body).length)].filter(Boolean).join(" · ") : "";
    }
    case "media": return t(`kind.${item.media}`);
    case "scripture": {
      const bible = bibleFor(item);
      if (!bible) return t("plans.verses");
      const fixed = item.bible && bible.id === item.bible;
      return `${t("plans.verses")} · ${fixed ? bible.abbreviation : t("plans.consoleVersion", { version: bible.abbreviation })}`;
    }
    case "pause": {
      const bg = item.background && !missingFiles.has(item.background) ? ` · ${baseName(item.background)}` : "";
      return `${t("plans.pause")} · ${formatCountdown(item.durationMs)}${bg}`;
    }
    default: return "";
  }
}

export function icon(item: PlanItem): string {
  if (item.kind === "media") return ICONS[item.media];
  if (item.kind === "song" || item.kind === "pause" || item.kind === "scripture") return ICONS[item.kind];
  return "";
}

/** El contenido de un elemento para «Preparado»; null si no se puede mostrar. */
export async function contentOf(item: PlanItem): Promise<OutputContent | null> {
  switch (item.kind) {
    case "section": return null;
    case "scripture": return scriptureContent(item);
    case "song": {
      const song = songOf(item);
      return song ? songContent(song) : null;
    }
    case "pause": {
      const background = item.background && !missingFiles.has(item.background) ? item.background : null;
      return preparedTimer(item.durationMs, item.title, background);
    }
    case "media":
      switch (item.media) {
        case "image": return { kind: "image", path: item.path };
        case "video": return { kind: "video", path: item.path, paused: false };
        case "pdf": return { kind: "pdf", path: item.path, page: 1 };
        case "presentation": {
          // Una presentación se muestra como el PDF que hace de ella PowerPoint, Keynote o LibreOffice.
          let pdf = converted.get(item.path);
          if (!pdf) {
            session.converting = item.id;
            try {
              pdf = await platform.convertPresentation(item.path);
              converted.set(item.path, pdf);
            } finally {
              session.converting = null;
            }
          }
          return { kind: "pdf", path: pdf, page: 1 };
        }
      }
  }
}

/** Lo que está en pantalla es lo preparado desde el plan: se marca como mostrado. */
export function markShown(planId: string, onScreen: OutputContent) {
  const current = session.current;
  if (!current) return;
  const c = current.content;
  const same =
    c.kind === "timer"
      ? onScreen.kind === "timer" && onScreen.label === c.label && onScreen.durationMs === c.durationMs
      : screenKey(onScreen) !== null && screenKey(onScreen) === screenKey(c);
  const done = session.shown[planId] ?? [];
  if (same && !done.includes(current.id)) session.shown[planId] = [...done, current.id];
}
