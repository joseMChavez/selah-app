<script lang="ts">
  import { onMount } from "svelte";
  import OutputView, { DEFAULT_STYLE } from "$lib/OutputView.svelte";
  import { platform, type OutputContent, type OutputStyle } from "$lib/platform";
  import { t } from "$lib/i18n/index.svelte";

  let content = $state<OutputContent>({ kind: "empty" });
  let style = $state<OutputStyle>(DEFAULT_STYLE);
  let dpr = $state(1);
  let width = $state(0);
  let height = $state(0);
  // Derivado para que cambie también al cambiar de idioma.
  let size = $derived(`${width}×${height} px · ${t("pattern.scale")} ${dpr}`);

  function measure() {
    dpr = window.devicePixelRatio;
    width = Math.round(innerWidth * dpr);
    height = Math.round(innerHeight * dpr);
  }

  onMount(() => {
    measure();
    // Recupera lo que estaba en pantalla si la ventana se recarga.
    const apply = (state: { content: OutputContent; style: OutputStyle }) => {
      content = state.content;
      style = state.style;
    };
    platform.getOutputState().then(apply);
    const unlisten = platform.onOutputState(apply);
    return () => unlisten.then((fn) => fn());
  });
</script>

<svelte:window onresize={measure} />

<div class="output">
  <OutputView {content} {style} diagnostics={size} onerror={(message) => platform.reportOutputError(message)} />
</div>

<style>
  :global(html, body) {
    margin: 0;
    /* Gana a los estilos de la consola: la salida siempre empieza en negro. */
    background: #000 !important;
    overflow: hidden;
  }
  .output {
    position: relative;
    width: 100vw;
    height: 100vh;
    cursor: none;
  }
</style>
