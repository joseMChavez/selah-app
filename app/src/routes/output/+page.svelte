<script lang="ts">
  import { onMount } from "svelte";
  import OutputView from "$lib/OutputView.svelte";
  import { platform, type OutputContent } from "$lib/platform";
  import { t } from "$lib/i18n/index.svelte";

  let content = $state<OutputContent>({ kind: "empty" });
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
    platform.getOutputState().then((state) => (content = state.content));
    const unlisten = platform.onOutputState((state) => (content = state.content));
    return () => unlisten.then((fn) => fn());
  });
</script>

<svelte:window onresize={measure} />

<div class="output">
  <OutputView {content} diagnostics={size} onerror={(message) => platform.reportOutputError(message)} />
</div>

<style>
  :global(html, body) {
    margin: 0;
    background: #000;
    overflow: hidden;
  }
  .output {
    width: 100vw;
    height: 100vh;
    cursor: none;
  }
</style>
