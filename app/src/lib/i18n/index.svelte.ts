// Idioma de la interfaz. Es independiente del idioma de cada Biblia o medio.
// La preferencia se guarda en la base de datos local (ajuste «locale»; sin valor, el del sistema).
import { setSetting, settings } from "$lib/settings.svelte";
import en from "./en";
import es, { type MessageKey } from "./es";
import it from "./it";

export const locales = {
  es: { name: "Español", messages: es },
  en: { name: "English", messages: en },
  it: { name: "Italiano", messages: it },
} as const;

export type Locale = keyof typeof locales;

function isLocale(value: unknown): value is Locale {
  return typeof value === "string" && value in locales;
}

const systemLocale: Locale = (() => {
  const system = navigator.language.slice(0, 2);
  return isLocale(system) ? system : "es";
})();

export const i18n = {
  /** El elegido, o el del sistema si no se eligió ninguno (o el guardado ya no existe). */
  get locale(): Locale {
    return isLocale(settings.locale) ? settings.locale : systemLocale;
  },
};

export function setLocale(locale: Locale | null) {
  setSetting("locale", locale);
}

export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  // Si faltara una traducción, se muestra la clave: un texto ausente no debe romper la consola.
  const template: string = locales[i18n.locale].messages[key] ?? es[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, name) => (name in params ? String(params[name]) : match));
}

export type { MessageKey };
