<script lang="ts">
  // Lo dibujado desde el teléfono, encima de lo que hay en pantalla (o sobre una hoja de la pizarra).
  // Lo usan la ventana de salida y las vistas previas de la consola: se ve igual en las dos.
  import { onMount } from "svelte";
  import { drawLayer, type Paper } from "./board";
  import { strokesOf, watchBoard } from "./board.svelte";

  let { layer, paper = null }: { layer: string | null; paper?: Paper | null } = $props();

  let canvas = $state<HTMLCanvasElement>();
  let width = $state(0);
  let height = $state(0);
  let visible = $derived(strokesOf(layer).length > 0);
  let frame = 0;

  onMount(() => {
    watchBoard();
    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    // Leer aquí todo lo que cambia el dibujo: la capa, los trazos (con cada punto nuevo) y el tamaño.
    void [canvas, width, height, paper, strokesOf(layer)];
    cancelAnimationFrame(frame);
    // Un dibujo por cuadro aunque lleguen muchos puntos seguidos.
    frame = requestAnimationFrame(() => {
      if (!canvas || width === 0 || height === 0) return;
      const dpr = window.devicePixelRatio || 1;
      const w = Math.round(width * dpr);
      const h = Math.round(height * dpr);
      if (canvas.width !== w) canvas.width = w;
      if (canvas.height !== h) canvas.height = h;
      const ctx = canvas.getContext("2d");
      if (ctx) drawLayer(ctx, strokesOf(layer), w, h, paper);
    });
  });
</script>

<div class="layer" bind:clientWidth={width} bind:clientHeight={height}>
  {#if visible}<canvas bind:this={canvas}></canvas>{/if}
</div>

<style>
  .layer {
    position: absolute;
    inset: 0;
    z-index: 3;
    pointer-events: none;
  }
  canvas {
    width: 100%;
    height: 100%;
  }
</style>
