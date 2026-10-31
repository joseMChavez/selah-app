<script lang="ts">
  // Dibuja un contenido de salida. Lo usan la ventana de salida y las miniaturas de la consola,
  // así la vista previa se ve igual que la pantalla real.
  import { platform, type OutputContent } from "$lib/platform";
  import { renderPage } from "$lib/pdf";
  import { t } from "$lib/i18n/index.svelte";

  let {
    content,
    diagnostics = "",
    preview = false,
    onerror,
  }: {
    content: OutputContent;
    diagnostics?: string;
    /** En miniatura el video no se reproduce: evita decodificarlo dos veces y que suene en la consola. */
    preview?: boolean;
    onerror?: (message: string) => void;
  } = $props();

  let video = $state<HTMLVideoElement>();
  let canvas = $state<HTMLCanvasElement>();
  let width = $state(0);
  let height = $state(0);

  let src = $derived("path" in content ? platform.mediaUrl(content.path) : "");
  let name = $derived(("path" in content ? content.path.split(/[\\/]/).pop() : "") ?? "");

  function videoError() {
    // 4 = MEDIA_ERR_SRC_NOT_SUPPORTED: suele indicar que falta el códec en el sistema.
    const hint = video?.error?.code === 4 ? ` ${t("error.codec")}` : "";
    onerror?.(t("error.video", { name }) + hint);
  }

  $effect(() => {
    if (!video || content.kind !== "video" || preview) return;
    if (content.paused) {
      video.pause();
    } else {
      video.play().catch((e) => onerror?.(t("error.videoStart", { error: String(e) })));
    }
  });

  $effect(() => {
    if (!canvas || content.kind !== "pdf" || width === 0 || height === 0) return;
    const controller = new AbortController();
    renderPage(src, content.page, canvas, { width, height }, controller.signal).catch(() => onerror?.(t("error.pdf", { name })));
    return () => controller.abort();
  });
</script>

<div
  class="view"
  class:pattern={content.kind === "testPattern"}
  bind:clientWidth={width}
  bind:clientHeight={height}
>
  {#if content.kind === "text"}
    <p class="text">{content.slides[content.index] ?? ""}</p>
  {:else if content.kind === "image"}
    <img {src} alt="" onerror={() => onerror?.(t("error.image", { name }))} />
  {:else if content.kind === "video"}
    {#key src}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video bind:this={video} {src} muted={preview} preload={preview ? "metadata" : "auto"} onerror={videoError}></video>
    {/key}
  {:else if content.kind === "pdf"}
    <canvas bind:this={canvas}></canvas>
  {:else if content.kind === "testPattern"}
    <div class="corner tl"></div>
    <div class="corner tr"></div>
    <div class="corner bl"></div>
    <div class="corner br"></div>
    <p class="text">{t("pattern.title")}</p>
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
  img,
  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
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
