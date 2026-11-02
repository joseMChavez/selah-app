<script lang="ts">
  // Reloj: la hora actual o una cuenta atrás que se pone en rojo al terminar.
  // El temporizador se prepara parado; empieza a contar al enviarlo a pantalla.
  import { t } from "$lib/i18n/index.svelte";
  import type { OutputContent } from "$lib/platform";

  let { onprepare }: { onprepare: (content: OutputContent) => void } = $props();

  const PRESETS = [1, 3, 5, 10, 15, 20, 30, 45];
  const MAX_MINUTES = 600;

  let mode = $state<"clock" | "timer">("clock");
  let hour12 = $state(!new Intl.DateTimeFormat(undefined, { hour: "numeric" }).resolvedOptions().hourCycle?.startsWith("h2"));
  let seconds = $state(false);
  let minutes = $state(5);
  let secs = $state(0);

  let durationMs = $derived(
    (Math.min(MAX_MINUTES, Math.max(0, Math.floor(minutes || 0))) * 60 + Math.min(59, Math.max(0, Math.floor(secs || 0)))) * 1000,
  );

  function update() {
    if (mode === "clock") onprepare({ kind: "clock", hour12, seconds });
    else onprepare(durationMs > 0 ? { kind: "timer", durationMs, endsAt: null, remainingMs: durationMs } : { kind: "empty" });
  }

  function choose(m: "clock" | "timer") {
    mode = m;
    update();
  }

  function preset(m: number) {
    minutes = m;
    secs = 0;
    update();
  }
</script>

<div class="clock-panel">
  <div class="segmented" role="tablist">
    <button role="tab" aria-selected={mode === "clock"} onclick={() => choose("clock")}>{t("clock.time")}</button>
    <button role="tab" aria-selected={mode === "timer"} onclick={() => choose("timer")}>{t("clock.timer")}</button>
  </div>

  {#if mode === "clock"}
    <p class="muted small">{t("clock.timeHint")}</p>
    <div class="segmented">
      <button aria-pressed={!hour12} onclick={() => { hour12 = false; update(); }}>{t("clock.h24")}</button>
      <button aria-pressed={hour12} onclick={() => { hour12 = true; update(); }}>{t("clock.h12")}</button>
    </div>
    <label class="check">
      <input type="checkbox" bind:checked={seconds} onchange={update} />
      {t("clock.seconds")}
    </label>
    <button class="btn" onclick={update}>{t("clock.prepare")}</button>
  {:else}
    <p class="muted small">{t("clock.timerHint")}</p>
    <div class="presets">
      {#each PRESETS as m (m)}
        <button class="chip" aria-pressed={durationMs === m * 60000} onclick={() => preset(m)}>{t("clock.minutes", { n: m })}</button>
      {/each}
    </div>
    <div class="duration">
      <label>
        <span class="label">{t("clock.min")}</span>
        <input class="field" type="number" min="0" max={MAX_MINUTES} bind:value={minutes} oninput={update} />
      </label>
      <span class="colon">:</span>
      <label>
        <span class="label">{t("clock.sec")}</span>
        <input class="field" type="number" min="0" max="59" bind:value={secs} oninput={update} />
      </label>
    </div>
    <p class="muted small">{t("clock.expiredHint")}</p>
    <button class="btn" onclick={update} disabled={durationMs === 0}>{t("clock.prepare")}</button>
  {/if}
</div>

<style>
  .clock-panel { display: flex; flex-direction: column; gap: 12px; align-items: flex-start; }
  .check { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .presets { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip {
    min-height: 36px;
    padding: 0 12px;
    font: inherit;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    cursor: pointer;
  }
  .chip[aria-pressed="true"] { border-color: var(--accent); background: var(--accent-soft); }
  .duration { display: flex; align-items: flex-end; gap: 8px; }
  .duration label { display: flex; flex-direction: column; gap: 4px; }
  .duration .field { width: 6em; font-size: 20px; font-variant-numeric: tabular-nums; }
  .colon { font-size: 24px; font-weight: 700; padding-bottom: 8px; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
</style>
