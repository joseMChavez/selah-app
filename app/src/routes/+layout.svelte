<script lang="ts">
  import "../app.css";
  import { i18n } from "$lib/i18n/index.svelte";
  import { loadLibrary } from "$lib/features/songs/library.svelte";
  import { platform } from "$lib/platform";
  import { loadSettings } from "$lib/core/settings.svelte";
  import { applyTheme, theme } from "$lib/core/theme.svelte";

  // Cualquier error no controlado queda en el registro de la app para poder diagnosticarlo.
  window.addEventListener("error", (e) => {
    platform.logError(`${e.message} (${e.filename}:${e.lineno})`).catch(() => {});
  });
  window.addEventListener("unhandledrejection", (e) => {
    const reason = e.reason instanceof Error ? `${e.reason.message}\n${e.reason.stack}` : JSON.stringify(e.reason);
    platform.logError(`promesa sin capturar: ${reason}`).catch(() => {});
  });

  let { children } = $props();

  // Los ajustes se leen antes de dibujar: así no se ve un instante con el idioma o el tema equivocados.
  // La biblioteca de letras y la migración desde localStorage solo hacen falta en la consola.
  const isOutput = location.pathname.startsWith("/output");
  const ready = isOutput ? loadSettings({ migrate: false }) : loadSettings({ migrate: true }).then(loadLibrary);

  $effect(() => applyTheme(theme.choice));
  $effect(() => {
    document.documentElement.lang = i18n.locale;
  });
</script>

{#await ready then}
  {@render children()}
{/await}
