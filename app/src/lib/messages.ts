// Avisos para el operador. Se guarda la clave y no el texto, para que cambien de idioma con la interfaz.
import type { MessageKey } from "$lib/i18n/index.svelte";
import type { CommandError } from "$lib/platform";

export interface Message {
  key: MessageKey;
  params?: Record<string, string | number>;
}

/** Traduce un error de un comando del backend (o cualquier otro) a un aviso. */
export function toMessage(e: unknown): Message {
  const err = e as CommandError | null;
  switch (err?.code) {
    case "monitorDisconnected": return { key: "error.monitorDisconnected" };
    case "fileMissing": return { key: "error.fileMissing", params: { path: err.path } };
    case "invalidContent": return { key: "error.invalidContent" };
    case "invalidBible": return { key: "error.invalidBible", params: { file: err.file, reason: err.reason } };
    case "bibleNotFound": return { key: "error.bibleNotFound" };
    case "internal": return { key: "error.internal", params: { message: err.message } };
    default: return { key: "error.internal", params: { message: String(e) } };
  }
}
