<script lang="ts">
  // Dibuja un contenido de salida. Lo usan la ventana de salida y las miniaturas de la consola,
  // así la vista previa se ve igual que la pantalla real.
  import type { OutputContent } from "$lib/platform";

  let { content, diagnostics = "" }: { content: OutputContent; diagnostics?: string } = $props();
</script>

<div class="view" class:pattern={content.kind === "testPattern"}>
  {#if content.kind === "text"}
    <p class="text">{content.text}</p>
  {:else if content.kind === "testPattern"}
    <div class="corner tl"></div>
    <div class="corner tr"></div>
    <div class="corner bl"></div>
    <div class="corner br"></div>
    <p class="text">Patrón de prueba</p>
    {#if diagnostics}<p class="diag">{diagnostics}</p>{/if}
  {/if}
</div>

<style>
  .view {
    container-type: size;
    position: relative;
    width: 100%;
    height: 100%;
    background: #000;
    color: #fff;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    font-family: system-ui, sans-serif;
  }
  .text {
    margin: 0 6cqw;
    font-size: 7cqh;
    line-height: 1.25;
    text-align: center;
    white-space: pre-wrap;
  }
  .diag {
    margin-top: 2cqh;
    font-size: 3cqh;
    font-family: monospace;
    opacity: 0.8;
  }
  /* El borde y las esquinas permiten comprobar a ojo que la salida cubre todo el monitor. */
  .pattern {
    box-sizing: border-box;
    border: 0.6cqh solid #ff3b30;
    background:
      linear-gradient(#222 1px, transparent 1px) 0 0 / 10cqw 10cqh,
      linear-gradient(90deg, #222 1px, transparent 1px) 0 0 / 10cqw 10cqh,
      #000;
  }
  .corner {
    position: absolute;
    width: 6cqh;
    height: 6cqh;
    background: #ffcc00;
  }
  .tl { top: 0; left: 0; }
  .tr { top: 0; right: 0; }
  .bl { bottom: 0; left: 0; }
  .br { bottom: 0; right: 0; }
</style>
