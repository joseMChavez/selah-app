<script lang="ts">
  import "../app.css";
  import "$lib/theme.svelte";
  import { platform } from "$lib/platform";

  // Cualquier error no controlado queda en el registro de la app para poder diagnosticarlo.
  window.addEventListener("error", (e) => {
    platform.logError(`${e.message} (${e.filename}:${e.lineno})`).catch(() => {});
  });
  window.addEventListener("unhandledrejection", (e) => {
    const reason = e.reason instanceof Error ? `${e.reason.message}\n${e.reason.stack}` : JSON.stringify(e.reason);
    platform.logError(`promesa sin capturar: ${reason}`).catch(() => {});
  });

  let { children } = $props();
</script>

{@render children()}
