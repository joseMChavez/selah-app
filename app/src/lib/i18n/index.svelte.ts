// Idioma de la interfaz. Es independiente del idioma de cada Biblia o medio.
// La preferencia se guarda en localStorage por ahora; en la Fase 3 pasa a la base de datos local.
import en from "./en";
import es, { type MessageKey } from "./es";
import it from "./it";

export const locales = {
  es: { name: "Español", messages: es },
  en: { name: "English", messages: en },
  it: { name: "Italiano", messages: it },
} as const;

export type Locale = keyof typeof locales;

const STORAGE_KEY = "selah.locale";

function isLocale(value: unknown): value is Locale {
  return typeof value === "string" && value in locales;
}

function initialLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (isLocale(saved)) return saved;
  } catch {
    // Sin almacenamiento disponible: se usa el idioma del sistema.
  }
  const system = navigator.language.slice(0, 2);
  return isLocale(system) ? system : "es";
}

export const i18n = $state({ locale: initialLocale() });
document.documentElement.lang = i18n.locale;

export function setLocale(locale: Locale) {
  i18n.locale = locale;
  document.documentElement.lang = locale;
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // La preferencia no se recordará, pero el cambio se aplica igual.
  }
}

// La ventana de salida comparte almacenamiento con la consola: así cambia de idioma a la vez.
window.addEventListener("storage", (event) => {
  if (event.key === STORAGE_KEY && isLocale(event.newValue)) {
    i18n.locale = event.newValue;
    document.documentElement.lang = event.newValue;
  }
});

export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  const template = locales[i18n.locale].messages[key] ?? es[key];
  return template.replace(/\{(\w+)\}/g, (match, name) => (name in params ? String(params[name]) : match));
}

export type { MessageKey };
