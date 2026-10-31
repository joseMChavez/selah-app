// Tema de la consola: seguir el sistema, claro u oscuro. Se recuerda en este equipo.
export type ThemeChoice = "system" | "light" | "dark";

const STORAGE_KEY = "selah.theme";

function initialTheme(): ThemeChoice {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "light" || saved === "dark" || saved === "system") return saved;
  } catch {
    // Sin almacenamiento: se sigue el sistema.
  }
  return "system";
}

export const theme = $state({ choice: initialTheme() });

function apply(choice: ThemeChoice) {
  // Sin atributo, las variables CSS siguen prefers-color-scheme.
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = choice;
}

apply(theme.choice);

export function setTheme(choice: ThemeChoice) {
  theme.choice = choice;
  apply(choice);
  try {
    localStorage.setItem(STORAGE_KEY, choice);
  } catch {
    // El cambio se aplica aunque no se recuerde.
  }
}
