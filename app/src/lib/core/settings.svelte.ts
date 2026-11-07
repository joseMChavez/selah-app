// Ajustes de Selah, guardados en la base de datos local (SQLite) del backend.
// Cada ajuste tiene un valor inicial: restablecer es borrar la clave de la base.
// Las dos ventanas comparten los ajustes: el backend avisa de cada cambio a ambas.
import type { Locale } from "$lib/i18n/index.svelte";
import { platform, type OutputStyle, type StoredSettings } from "$lib/platform";
import type { ThemeChoice } from "$lib/core/theme.svelte";

const isString = (v: unknown): v is string => typeof v === "string";
const isStringOrNull = (v: unknown): v is string | null => v === null || typeof v === "string";
const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";

const SCHEMA = {
  /** null = el idioma del sistema. */
  locale: { initial: null as Locale | null, valid: isStringOrNull },
  theme: { initial: "system" as ThemeChoice, valid: (v: unknown) => v === "system" || v === "light" || v === "dark" },
  /** Pestaña de Proyección abierta (la consola comprueba que exista). */
  tab: { initial: "bible", valid: isString },
  view: { initial: "projection" as "projection" | "stream", valid: (v: unknown) => v === "projection" || v === "stream" },
  expanded: { initial: false, valid: isBoolean },
  /** Plan del culto abierto en la pestaña Plan; null = la lista de planes. */
  plan: { initial: null as string | null, valid: isStringOrNull },
  /** null = el estilo inicial de la pantalla. */
  outputStyle: {
    initial: null as OutputStyle | null,
    valid: (v: unknown) => v === null || (typeof v === "object" && "fontScale" in v),
  },
  bible: { initial: null as string | null, valid: isStringOrNull },
  /** Segunda versión debajo de la primera; null = solo una. */
  bible2: { initial: null as string | null, valid: isStringOrNull },
  /** Biblias que no aparecen para elegir (las incluidas no se pueden borrar: se ocultan). */
  hiddenBibles: { initial: [] as string[], valid: (v: unknown) => Array.isArray(v) && v.every(isString) },
  obsTextSource: { initial: null as string | null, valid: isStringOrNull },
  /** "" = el mensaje traducido de siempre. */
  obsWaitMessage: { initial: "", valid: isString },
  /** Preguntar antes de mostrar algo en la pantalla que ve la congregación. */
  confirmSend: { initial: true, valid: isBoolean },
  /** Conectar con OBS al abrir Selah si ya está abierto. */
  obsAutoConnect: { initial: true, valid: isBoolean },
  /** Servidor del control remoto encendido; se vuelve a encender al abrir Selah. */
  remoteEnabled: { initial: false, valid: isBoolean },
};

type Schema = typeof SCHEMA;
export type SettingKey = keyof Schema;
export type Settings = { [K in SettingKey]: Schema[K]["initial"] };

const KEYS = Object.keys(SCHEMA) as SettingKey[];

/** Grupos que se pueden restablecer por separado en la vista de Ajustes. */
export const SECTIONS = {
  interface: ["locale", "theme", "tab", "view", "expanded", "plan"],
  projection: ["outputStyle", "confirmSend"],
  bible: ["bible", "bible2", "hiddenBibles"],
  stream: ["obsTextSource", "obsWaitMessage", "obsAutoConnect"],
  remote: ["remoteEnabled"],
} satisfies Record<string, SettingKey[]>;

function initial(): Settings {
  return Object.fromEntries(KEYS.map((k) => [k, SCHEMA[k].initial])) as Settings;
}

export const settings = $state<Settings>(initial());

/** Último fallo al guardar; la consola lo muestra para que no pase desapercibido. */
export const storage = $state({ error: null as string | null });

function apply(stored: StoredSettings) {
  for (const key of KEYS) {
    const value = stored[key];
    // Un valor que no tiene la forma esperada (dato dañado, versión antigua) vale lo inicial.
    (settings as Record<SettingKey, unknown>)[key] = key in stored && SCHEMA[key].valid(value) ? value : SCHEMA[key].initial;
  }
}

function failed(action: string, e: unknown) {
  storage.error = (e as { message?: string })?.message ?? String(e);
  platform.logError(`ajustes: ${action}: ${JSON.stringify(e)}`).catch(() => {});
}

export function setSetting<K extends SettingKey>(key: K, value: Settings[K]) {
  settings[key] = value;
  platform.settingsSet(key, value).catch((e) => failed(`guardar ${key}`, e));
}

/** Vuelve al valor inicial esas claves; sin claves, todos los ajustes. */
export async function resetSettings(keys: SettingKey[] = []) {
  apply(await platform.settingsReset(keys));
}

// Antes de la base de datos, los ajustes estaban en localStorage. Se pasan una sola vez y se borran de allí.
const LEGACY: [string, SettingKey][] = [
  ["selah.locale", "locale"],
  ["selah.theme", "theme"],
  ["selah.tab", "tab"],
  ["selah.view", "view"],
  ["selah.expanded", "expanded"],
  ["selah.outputStyle", "outputStyle"],
  ["selah.bible", "bible"],
  ["selah.bible2", "bible2"],
  ["selah.obsTextSource", "obsTextSource"],
];

/** Lee un valor antiguo: unos se guardaban como JSON y otros como texto plano. */
export function readLegacy(storageKey: string, valid: (v: unknown) => boolean): { value: unknown } | null {
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw === null) return null;
    localStorage.removeItem(storageKey);
    let parsed: unknown = raw;
    try {
      parsed = JSON.parse(raw);
    } catch {
      // Texto plano.
    }
    if (valid(parsed)) return { value: parsed };
    if (valid(raw)) return { value: raw };
  } catch {
    // Sin almacenamiento antiguo: nada que migrar.
  }
  return null;
}

async function migrateLegacy(stored: StoredSettings) {
  for (const [storageKey, key] of LEGACY) {
    const legacy = readLegacy(storageKey, SCHEMA[key].valid);
    if (legacy && !(key in stored)) {
      (settings as Record<SettingKey, unknown>)[key] = legacy.value;
      await platform.settingsSet(key, legacy.value).catch((e) => failed(`migrar ${key}`, e));
    }
  }
}

/** Se llama una vez al cargar cada ventana, antes de dibujar nada. Si la base falla, se sigue con lo inicial. */
export async function loadSettings({ migrate }: { migrate: boolean }) {
  try {
    const stored = await platform.settingsGet();
    apply(stored);
    if (migrate) await migrateLegacy(stored);
  } catch (e) {
    failed("leer", e);
  }
  platform.onSettingsChanged(apply).catch(() => {});
}
