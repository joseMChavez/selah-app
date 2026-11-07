<script lang="ts" module>
  import type { OutputStyle } from "$lib/platform";

  export const DEFAULT_STYLE: OutputStyle = {
    fontScale: 1,
    align: "center",
    position: "middle",
    background: "black",
    font: "system",
    backgroundImage: null,
    logo: { image: null, corner: "topRight", size: "medium", opacity: 0.6 },
  };

  /** Alto del logo según el tamaño elegido, en % del alto de la pantalla. */
  const LOGO_HEIGHT: Record<OutputStyle["logo"]["size"], number> = { small: 8, medium: 12, large: 18 };

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
  import { renderPage } from "$lib/features/output/pdf";
  import { formatClock, formatCountdown, timerRemaining } from "$lib/features/clock/clock";
  import { fontFamily } from "$lib/features/output/fonts";
  import { i18n, t } from "$lib/i18n/index.svelte";

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
  let now = $state(Date.now());

  let src = $derived("path" in content ? platform.mediaUrl(content.path) : "");
  let name = $derived(("path" in content ? content.path.split(/[\\/]/).pop() : "") ?? "");

  let lyric = $derived.by(() => {
    if (content.kind === "text") {
      return {
        text: content.slides[content.index] ?? "",
        reference: "",
        // La autoría de una letra va solo en la primera diapositiva.
        copyright: content.index === 0 ? (content.credit ?? "") : "",
        second: null,
      };
    }
    if (content.kind === "scripture") {
      const verse = content.verses[content.index];
      const second = content.second && verse?.second ? verse.second : null;
      return {
        text: verse?.text ?? "",
        reference: verse ? `${verse.reference} · ${content.version}` : "",
        copyright: [content.copyright, content.second?.copyright].filter(Boolean).join(" · "),
        second: second && { text: second.text, reference: `${second.reference} · ${content.second!.version}` },
      };
    }
    return null;
  });

  let clock = $derived.by(() => {
    if (content.kind === "clock") return { text: formatClock(new Date(now), i18n.locale, content.hour12, content.seconds), expired: false, label: "" };
    if (content.kind === "timer") {
      const remaining = timerRemaining(content, now);
      return { text: formatCountdown(remaining), expired: remaining <= 0, label: content.label ?? "" };
    }
    return null;
  });

  // La imagen de fondo va detrás del texto, la Biblia y el reloj, y queda sola al quitar el contenido.
  // Una letra o una cuenta atrás (la pausa de un plan) pueden traer la suya.
  // El logo va encima de todo menos de la pantalla en negro y del patrón de prueba.
  let logo = $derived(
    style.logo?.image && content.kind !== "black" && content.kind !== "testPattern" ? platform.mediaUrl(style.logo.image) : null,
  );

  let backdrop = $derived.by(() => {
    if (!lyric && !clock && content.kind !== "empty") return null;
    const path = ((content.kind === "text" || content.kind === "timer") && content.background) || style.backgroundImage;
    return path ? platform.mediaUrl(path) : null;
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
    if (content.kind !== "clock" && content.kind !== "timer") return;
    now = Date.now();
    const timer = setInterval(() => (now = Date.now()), 200);
    return () => clearInterval(timer);
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
    void [lyric?.text, lyric?.reference, lyric?.second?.text, style.fontScale, style.position, style.font, width, height];
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
  class:preview
  class:pattern={content.kind === "testPattern"}
  class:expired={clock?.expired}
  class:logo-bottom={logo !== null && style.logo.corner.startsWith("bottom")}
  style:background-color={BACKGROUNDS[style.background]}
  style:font-family={fontFamily(style.font)}
  bind:clientWidth={width}
  bind:clientHeight={height}
>
  {#if backdrop}
    <img class="backdrop" src={backdrop} alt="" />
    <div class="shade"></div>
  {/if}
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
        {#if lyric.second}
          <hr class="divider" />
          <p class="text second">{lyric.second.text}</p>
          <p class="reference">{lyric.second.reference}</p>
        {/if}
      </div>
    </div>
    {#if lyric.copyright}<p class="copyright">{lyric.copyright}</p>{/if}
  {:else if clock}
    <!-- Sobre una imagen, la hora va en una placa oscura: tiene que leerse siempre, con cualquier fondo. -->
    <div class="clock-box" class:plate={backdrop !== null}>
      {#if clock.label}<p class="clock-label">{clock.label}</p>{/if}
      <!-- Tan grande como quepa a lo ancho: «10:45:30 p. m.» es mucho más largo que «5:00». -->
      <p class="clock" style:font-size="min({clock.label ? 24 : 30}cqh, {Math.round(130 / clock.text.length)}cqw)">{clock.text}</p>
    </div>
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
  {#if logo}
    <img
      class="logo {style.logo.corner}"
      src={logo}
      alt=""
      style:height="{LOGO_HEIGHT[style.logo.size]}cqh"
      style:opacity={style.logo.opacity}
      onerror={() => onerror?.(t("error.image", { name: style.logo.image?.split(/[\\/]/).pop() ?? "" }))}
    />
  {/if}
</div>

<style>
  /* Ocupa exactamente el recuadro de su contenedor (que debe estar posicionado).
     contain: strict impide que el texto se dibuje o se posicione fuera, aunque el contenedor
     tome su alto de aspect-ratio y el navegador no lo considere definido. */
  .view {
    container-type: size;
    contain: strict;
    position: absolute;
    inset: 0;
    color: #f5f5f0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }
  /* Las vistas previas nunca reciben el ratón: no pueden tapar los controles de la consola. */
  .view.preview { pointer-events: none; }
  .stage {
    position: absolute;
    inset: 6% 6%;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .stage.with-credit { bottom: 11%; }
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
  /* Marca de agua: encima de cualquier contenido, en una esquina, sin tapar el texto (que deja un 6 % de margen). */
  .logo {
    position: absolute;
    z-index: 2;
    width: auto;
    max-width: 22cqw;
    object-fit: contain;
    pointer-events: none;
  }
  /* Con el logo abajo, el copyright se estrecha para no quedar debajo del logo. */
  .view.logo-bottom .copyright { left: 26cqw; right: 26cqw; }
  .logo.topLeft { top: 3cqh; left: 2.5cqw; }
  .logo.topRight { top: 3cqh; right: 2.5cqw; }
  .logo.bottomLeft { bottom: 3cqh; left: 2.5cqw; }
  .logo.bottomRight { bottom: 3cqh; right: 2.5cqw; }
  .backdrop {
    position: absolute;
    inset: 0;
    object-fit: cover;
  }
  /* Oscurece la imagen lo justo para que el texto se lea sobre cualquier fondo. */
  .shade {
    position: absolute;
    inset: 0;
    background: rgb(0 0 0 / 0.4);
  }
  .divider {
    width: 30%;
    margin: 0.7em auto;
    border: 0;
    border-top: 0.06em solid rgb(245 245 240 / 0.45);
  }
  .text.second { color: rgb(245 245 240 / 0.88); }
  .clock {
    position: relative;
    margin: 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
    text-shadow: 0 0.03em 0.1em rgb(0 0 0 / 0.55);
  }
  .clock-box {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 92cqw;
  }
  .clock-box.plate {
    padding: 2cqh 4cqw;
    border-radius: 3cqh;
    background: rgb(0 0 0 / 0.55);
  }
  .clock-label {
    position: relative;
    margin: 0 0 1cqh;
    max-width: 88cqw;
    overflow: hidden;
    font-size: 8cqh;
    font-weight: 600;
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 0.04em 0.12em rgb(0 0 0 / 0.6);
  }
  /* Tiempo cumplido: un borde rojo que late despacio, visible desde el púlpito, sin tapar el fondo ni el logo.
     La cuenta sigue con «−» y en rojo claro, que se lee sobre la placa oscura. */
  .view.expired::after {
    content: "";
    position: absolute;
    inset: 0;
    border: 2.2cqh solid #e0262e;
    pointer-events: none;
    animation: expired 1.6s ease-in-out infinite alternate;
  }
  .view.expired .clock { color: #ff8a8a; }
  @keyframes expired {
    from { border-color: #9e1219; }
    to { border-color: #ff3b42; }
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
