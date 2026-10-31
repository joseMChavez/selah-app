<script lang="ts" module>
  import type { OutputStyle } from "$lib/platform";

  export const DEFAULT_STYLE: OutputStyle = { fontScale: 1, align: "center", position: "middle", background: "black" };

  const BACKGROUNDS: Record<OutputStyle["background"], string> = {
    black: "#000000",
    navy: "#0b1730",
    charcoal: "#1d1f22",
  };
</script>

<script lang="ts">
  // Dibuja un contenido de salida. Lo usan la ventana de salida y las miniaturas de la consola,
  // así la vista previa se ve igual que la pantalla real.
  import { platform, type OutputContent } from "$lib/platform";
  import { renderPage } from "$lib/pdf";
  import { t } from "$lib/i18n/index.svelte";

  let {
    content,
    style = DEFAULT_STYLE,
    diagnostics = "",
    preview = false,
    onerror,
  }: {
    content: OutputContent;
    style?: OutputStyle;
    diagnostics?: string;
    /** En miniatura el video no se reproduce: evita decodificarlo dos veces y que suene en la consola. */
    preview?: boolean;
    onerror?: (message: string) => void;
  } = $props();

  let video = $state<HTMLVideoElement>();
  let canvas = $state<HTMLCanvasElement>();
  let stage = $state<HTMLDivElement>();
  let body = $state<HTMLDivElement>();
  let width = $state(0);
  let height = $state(0);

  let src = $derived("path" in content ? platform.mediaUrl(content.path) : "");
  let name = $derived(("path" in content ? content.path.split(/[\\/]/).pop() : "") ?? "");

  let lyric = $derived.by(() => {
    if (content.kind === "text") return { text: content.slides[content.index] ?? "", reference: "", copyright: "" };
    if (content.kind === "scripture") {
      const verse = content.verses[content.index];
      return {
        text: verse?.text ?? "",
        reference: verse ? `${verse.reference} · ${content.version}` : "",
        copyright: content.copyright,
      };
    }
    return null;
  });

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
    renderPage(src, content.page, canvas, { width, height }, controller.signal).catch(() =>
      onerror?.(t("error.pdf", { name })),
    );
    return () => controller.abort();
  });

  // Ajuste automático: el texto usa el tamaño máximo elegido y se reduce lo justo para caber.
  // Así un versículo largo no se corta y uno corto no queda diminuto.
  $effect(() => {
    void [lyric?.text, lyric?.reference, style.fontScale, style.position, width, height];
    if (!stage || !body || height === 0) return;
    const fits = () => body!.scrollHeight <= stage!.clientHeight + 1 && body!.scrollWidth <= stage!.clientWidth + 1;
    let low = height * 0.022;
    let high = height * 0.08 * style.fontScale;
    body.style.fontSize = `${high}px`;
    if (fits()) return;
    for (let i = 0; i < 10; i++) {
      const mid = (low + high) / 2;
      body.style.fontSize = `${mid}px`;
      if (fits()) low = mid;
      else high = mid;
    }
    body.style.fontSize = `${low}px`;
  });
</script>

<div
  class="view"
  class:pattern={content.kind === "testPattern"}
  style:background-color={BACKGROUNDS[style.background]}
  bind:clientWidth={width}
  bind:clientHeight={height}
>
  {#if lyric}
    <div
      class="stage pos-{style.position}"
      class:with-credit={lyric.copyright !== ""}
      style:text-align={style.align}
      bind:this={stage}
    >
      <div class="body" bind:this={body}>
        <p class="text">{lyric.text}</p>
        {#if lyric.reference}<p class="reference">{lyric.reference}</p>{/if}
      </div>
    </div>
    {#if lyric.copyright}<p class="copyright">{lyric.copyright}</p>{/if}
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
    <p class="pattern-title">{t("pattern.title")}</p>
    {#if diagnostics}<p class="diag">{diagnostics}</p>{/if}
  {/if}
</div>

<style>
  .view {
    container-type: size;
    position: relative;
    width: 100%;
    height: 100%;
    color: #f5f5f0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    font-family: "Inter", "Segoe UI", system-ui, sans-serif;
  }
  .stage {
    position: absolute;
    inset: 6cqh 6cqw;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .stage.with-credit { bottom: 11cqh; }
  .stage.pos-top { justify-content: flex-start; }
  .stage.pos-bottom { justify-content: flex-end; }
  .body {
    width: 100%;
    line-height: 1.28;
    overflow-wrap: anywhere;
    /* Sombra suave: el texto se lee aunque el fondo cambie. */
    text-shadow: 0 0.04em 0.12em rgb(0 0 0 / 0.55);
  }
  .text {
    margin: 0;
    white-space: pre-wrap;
    font-weight: 500;
  }
  .reference {
    margin: 0.55em 0 0;
    font-size: 0.6em;
    font-weight: 600;
    letter-spacing: 0.02em;
    color: #f2d58a;
  }
  .copyright {
    position: absolute;
    left: 6cqw;
    right: 6cqw;
    bottom: 3cqh;
    margin: 0;
    font-size: 2cqh;
    line-height: 1.3;
    text-align: center;
    color: rgb(245 245 240 / 0.62);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  img,
  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .pattern-title {
    margin: 0;
    font-size: 7cqh;
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
      #000 !important;
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
