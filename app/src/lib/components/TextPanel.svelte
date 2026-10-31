<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import type { OutputContent } from "$lib/platform";
  import { splitSlides } from "$lib/slides";

  let { onprepare }: { onprepare: (content: OutputContent) => void } = $props();

  let draft = $state(t("text.sample"));
  let slides = $derived(splitSlides(draft));

  // Escribir solo cambia «Preparado»: la pantalla no se toca hasta pulsar «Enviar a pantalla».
  function update() {
    onprepare(slides.length > 0 ? { kind: "text", slides, index: 0 } : { kind: "empty" });
  }
</script>

<div class="text-panel">
  <label class="label" for="text-draft">{t("text.label")}</label>
  <textarea id="text-draft" class="field" bind:value={draft} oninput={update} onfocus={update} rows="12"></textarea>
  <p class="muted small">
    {t("text.hint")}
    {#if slides.length > 1}· {t("text.count", { count: slides.length })}{/if}
  </p>
</div>

<style>
  .text-panel { display: flex; flex-direction: column; gap: 8px; height: 100%; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
  textarea { flex: 1; min-height: 220px; resize: vertical; font-size: 16px; line-height: 1.5; }
</style>
