<script lang="ts">
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import type { OutputStyle } from "$lib/platform";

  let { style, onchange }: { style: OutputStyle; onchange: (style: OutputStyle) => void } = $props();

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
    <span class="label">{t("style.background")}</span>
    <div class="swatches">
      {#each backgrounds as b (b.value)}
        <button
          class="swatch"
          aria-pressed={style.background === b.value}
          onclick={() => onchange({ ...style, background: b.value })}
        >
          <span class="dot" style:background={b.color}></span>{t(b.label)}
        </button>
      {/each}
    </div>
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
  .dot { width: 16px; height: 16px; border-radius: 50%; border: 1px solid rgb(255 255 255 / 0.35); }
</style>
