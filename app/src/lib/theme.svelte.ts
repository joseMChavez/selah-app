// Tema de la consola: seguir el sistema, claro u oscuro. Se guarda en la base de datos local.
import { setSetting, settings } from "$lib/settings.svelte";

export type ThemeChoice = "system" | "light" | "dark";

export const theme = {
  get choice(): ThemeChoice {
    return settings.theme;
  },
};

export function setTheme(choice: ThemeChoice) {
  setSetting("theme", choice);
}

export function applyTheme(choice: ThemeChoice) {
  // Sin atributo, las variables CSS siguen prefers-color-scheme.
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = choice;
}
