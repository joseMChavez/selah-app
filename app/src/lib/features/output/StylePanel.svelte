<script lang="ts">
  import { FONTS } from "$lib/features/output/fonts";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/core/messages";
  import { platform, type Corner, type FontId, type LogoStyle, type OutputStyle } from "$lib/platform";

  let {
    style,
    onchange,
    onerror,
  }: { style: OutputStyle; onchange: (style: OutputStyle) => void; onerror: (message: Message) => void } = $props();

  const MIN_SCALE = 0.6;
  const MAX_SCALE = 1.6;
  const STEP = 0.1;

  const aligns: { value: OutputStyle["align"]; label: MessageKey }[] = [
    { value: "left", label: "style.left" },
    { value: "center", label: "style.center" },
    { value: "right", label: "style.right" },
  ];
  const positions: { value: OutputStyle["position"]; label: MessageKey }[] = [
    { value: "top", label: "style.top" },
    { value: "middle", label: "style.middle" },
    { value: "bottom", label: "style.bottom" },
  ];
  const backgrounds: { value: OutputStyle["background"]; label: MessageKey; color: string }[] = [
    { value: "black", label: "bg.black", color: "#000000" },
    { value: "navy", label: "bg.navy", color: "#0b1730" },
    { value: "charcoal", label: "bg.charcoal", color: "#1d1f22" },
  ];

  // Un logo en cada esquina; las flechas indican hacia dónde va.
  const corners: { value: Corner; label: MessageKey; arrow: string }[] = [
    { value: "topLeft", label: "logo.topLeft", arrow: "↖" },
    { value: "topRight", label: "logo.topRight", arrow: "↗" },
    { value: "bottomLeft", label: "logo.bottomLeft", arrow: "↙" },
    { value: "bottomRight", label: "logo.bottomRight", arrow: "↘" },
  ];
  const sizes: { value: LogoStyle["size"]; label: MessageKey }[] = [
    { value: "small", label: "logo.small" },
    { value: "medium", label: "logo.medium" },
    { value: "large", label: "logo.large" },
  ];
  const opacities: { value: number; label: MessageKey }[] = [
    { value: 0.35, label: "logo.soft" },
    { value: 0.6, label: "logo.halfway" },
    { value: 1, label: "logo.solid" },
  ];

  function changeLogo(change: Partial<LogoStyle>) {
    onchange({ ...style, logo: { ...style.logo, ...change } });
  }

  async function pickLogo() {
    try {
      const path = await platform.pickBackground(t("logo.dialogTitle"), t("logo.dialogFilter"));
      if (path) changeLogo({ image: path });
    } catch (e) {
      onerror(toMessage(e));
      platform.logError(`logo: no se pudo elegir: ${JSON.stringify(e)}`).catch(() => {});
    }
  }

  async function pickImage() {
    try {
      const path = await platform.pickBackground(t("background.dialogTitle"), t("background.dialogFilter"));
      if (path) onchange({ ...style, backgroundImage: path });
    } catch (e) {
      onerror(toMessage(e));
      platform.logError(`fondo: no se pudo elegir: ${JSON.stringify(e)}`).catch(() => {});
    }
  }

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function scale(delta: number) {
    const fontScale = Math.round(Math.min(MAX_SCALE, Math.max(MIN_SCALE, style.fontScale + delta)) * 10) / 10;
    onchange({ ...style, fontScale });
  }
</script>

<div class="style-panel">
  <p class="muted small">{t("style.applies")}</p>

  <div class="row">
    <span class="label">{t("style.size")}</span>
    <div class="size">
      <button class="btn" onclick={() => scale(-STEP)} disabled={style.fontScale <= MIN_SCALE} aria-label={t("style.smaller")}>A−</button>
      <span class="value">{Math.round(style.fontScale * 100)}%</span>
      <button class="btn" onclick={() => scale(STEP)} disabled={style.fontScale >= MAX_SCALE} aria-label={t("style.larger")}>A+</button>
    </div>
  </div>

  <div class="row">
    <span class="label">{t("style.align")}</span>
    <div class="segmented">
      {#each aligns as a (a.value)}
        <button aria-pressed={style.align === a.value} onclick={() => onchange({ ...style, align: a.value })}>{t(a.label)}</button>
      {/each}
    </div>
  </div>

  <div class="row">
    <span class="label">{t("style.position")}</span>
    <div class="segmented">
      {#each positions as p (p.value)}
        <button aria-pressed={style.position === p.value} onclick={() => onchange({ ...style, position: p.value })}>{t(p.label)}</button>
      {/each}
    </div>
  </div>

  <div class="row">
    <label class="label" for="style-font">{t("style.font")}</label>
    <select id="style-font" class="field compact" value={style.font} onchange={(e) => onchange({ ...style, font: e.currentTarget.value as FontId })}>
      {#each FONTS as f (f.id)}
        <option value={f.id} style:font-family={f.family}>{f.id === "system" ? t("font.system") : f.label}</option>
      {/each}
    </select>
  </div>

  <div class="row">
    <span class="label">{t("style.background")}</span>
    <div class="swatches">
      {#each backgrounds as b (b.value)}
        <button
          class="swatch"
          aria-pressed={style.background === b.value && !style.backgroundImage}
          onclick={() => onchange({ ...style, background: b.value, backgroundImage: null })}
        >
          <span class="dot" style:background={b.color}></span>{t(b.label)}
        </button>
      {/each}
      <button class="swatch" aria-pressed={style.backgroundImage !== null} onclick={pickImage} title={style.backgroundImage ?? undefined}>
        {#if style.backgroundImage}
          <img class="dot" src={platform.mediaUrl(style.backgroundImage)} alt="" />
        {/if}
        {t(style.backgroundImage ? "background.change" : "background.pick")}
      </button>
    </div>
  </div>
  {#if style.backgroundImage}
    <div class="row current">
      <span class="muted small name" title={style.backgroundImage}>{t("background.current", { name: baseName(style.backgroundImage) })}</span>
      <button class="btn ghost" onclick={() => onchange({ ...style, backgroundImage: null })}>{t("background.remove")}</button>
    </div>
  {/if}
  <p class="muted small">{t("background.hint")}</p>

  <div class="logo-block">
    <div class="row">
      <span class="label">{t("logo.title")}</span>
      <div class="swatches">
        {#if style.logo.image}
          <span class="muted small name" title={style.logo.image}>{baseName(style.logo.image)}</span>
          <button class="btn ghost" onclick={() => changeLogo({ image: null })}>{t("background.remove")}</button>
        {/if}
        <button class="swatch" onclick={pickLogo}>
          {#if style.logo.image}<img class="dot" src={platform.mediaUrl(style.logo.image)} alt="" />{/if}
          {t(style.logo.image ? "logo.change" : "logo.pick")}
        </button>
      </div>
    </div>
    {#if style.logo.image}
      <div class="row">
        <span class="label">{t("logo.corner")}</span>
        <div class="segmented">
          {#each corners as c (c.value)}
            <button aria-pressed={style.logo.corner === c.value} title={t(c.label)} aria-label={t(c.label)} onclick={() => changeLogo({ corner: c.value })}>{c.arrow}</button>
          {/each}
        </div>
      </div>
      <div class="row">
        <span class="label">{t("logo.size")}</span>
        <div class="segmented">
          {#each sizes as z (z.value)}
            <button aria-pressed={style.logo.size === z.value} onclick={() => changeLogo({ size: z.value })}>{t(z.label)}</button>
          {/each}
        </div>
      </div>
      <div class="row">
        <span class="label">{t("logo.opacity")}</span>
        <div class="segmented">
          {#each opacities as o (o.value)}
            <button aria-pressed={style.logo.opacity === o.value} onclick={() => changeLogo({ opacity: o.value })}>{t(o.label)}</button>
          {/each}
        </div>
      </div>
    {/if}
    <p class="muted small">{t("logo.hint")}</p>
  </div>
</div>

<style>
  .style-panel { display: flex; flex-direction: column; gap: 12px; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
  .label { font-size: 14px; font-weight: 600; }
  .size { display: flex; align-items: center; gap: 8px; }
  .value { min-width: 3.5em; text-align: center; font-variant-numeric: tabular-nums; }
  .swatches { display: flex; gap: 6px; flex-wrap: wrap; }
  .swatch {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 34px;
    padding: 0 10px;
    font: inherit;
    font-size: 14px;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }
  .swatch[aria-pressed="true"] { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); }
  .field.compact { width: auto; min-height: 36px; padding: 4px 8px; }
  img.dot { object-fit: cover; }
  .current { justify-content: flex-end; }
  .logo-block { display: flex; flex-direction: column; gap: 12px; padding-top: 12px; border-top: 1px solid var(--border); }
  .name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dot { width: 16px; height: 16px; border-radius: 50%; border: 1px solid rgb(255 255 255 / 0.35); }
</style>
