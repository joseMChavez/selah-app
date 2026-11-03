// Tipos de letra para la pantalla. Van dentro de la app (licencia SIL OFL 1.1, paquetes @fontsource):
// la salida se ve igual en cualquier equipo y sin conexión. Solo los alfabetos latinos y los pesos que se usan.
import "@fontsource/montserrat/latin-500.css";
import "@fontsource/montserrat/latin-ext-500.css";
import "@fontsource/montserrat/latin-600.css";
import "@fontsource/montserrat/latin-ext-600.css";
import "@fontsource/oswald/latin-500.css";
import "@fontsource/oswald/latin-ext-500.css";
import "@fontsource/oswald/latin-600.css";
import "@fontsource/oswald/latin-ext-600.css";
import "@fontsource/bebas-neue/latin-400.css";
import "@fontsource/bebas-neue/latin-ext-400.css";
import "@fontsource/lora/latin-500.css";
import "@fontsource/lora/latin-ext-500.css";
import "@fontsource/lora/latin-600.css";
import "@fontsource/lora/latin-ext-600.css";
import "@fontsource/merriweather/latin-500.css";
import "@fontsource/merriweather/latin-ext-500.css";
import "@fontsource/merriweather/latin-600.css";
import "@fontsource/merriweather/latin-ext-600.css";
import type { FontId } from "$lib/platform";

const SANS = `"Inter", "Segoe UI", system-ui, sans-serif`;

export const FONTS: { id: FontId; label: string; family: string }[] = [
  { id: "system", label: "Sistema", family: SANS },
  { id: "montserrat", label: "Montserrat", family: `"Montserrat", ${SANS}` },
  { id: "oswald", label: "Oswald", family: `"Oswald", ${SANS}` },
  { id: "bebasNeue", label: "Bebas Neue", family: `"Bebas Neue", ${SANS}` },
  { id: "lora", label: "Lora", family: `"Lora", Georgia, serif` },
  { id: "merriweather", label: "Merriweather", family: `"Merriweather", Georgia, serif` },
];

export function fontFamily(id: FontId | undefined): string {
  return (FONTS.find((f) => f.id === id) ?? FONTS[0]).family;
}
