<script lang="ts">
  import { onMount } from "svelte";
  import OutputView from "$lib/OutputView.svelte";
  import { platform, type OutputContent } from "$lib/platform";

  let content = $state<OutputContent>({ kind: "empty" });
  let size = $state("");

  function measure() {
    const dpr = window.devicePixelRatio;
    size = `${Math.round(innerWidth * dpr)}×${Math.round(innerHeight * dpr)} px · escala ${dpr}`;
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
  <OutputView {content} diagnostics={size} />
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
