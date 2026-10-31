<script lang="ts">
  // Recorrido rápido por una serie: miniaturas para diapositivas de texto, números para versículos.
  import OutputView from "$lib/OutputView.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { OutputContent, OutputStyle } from "$lib/platform";

  let {
    content,
    style,
    tone,
    disabled = false,
    onselect,
  }: {
    content: OutputContent;
    style: OutputStyle;
    /** "live" marca en rojo la posición actual: elegir otra cambia la pantalla al momento. */
    tone: "prepared" | "live";
    disabled?: boolean;
    onselect: (n: number) => void;
  } = $props();

  let strip = $state<HTMLOListElement>();

  // Mantiene visible la posición actual al avanzar.
  $effect(() => {
    if (content.kind !== "text" && content.kind !== "scripture") return;
    void content.index;
    strip?.querySelector<HTMLElement>("[aria-current='true']")?.scrollIntoView({ block: "nearest", inline: "nearest" });
  });
</script>

{#if content.kind === "text" && content.slides.length > 1}
  <ol class="thumbs {tone}" bind:this={strip}>
    {#each content.slides as _, i (i)}
      <li>
        <button
          class="thumb"
          aria-current={i === content.index}
          aria-label={t("position.slide", { n: i + 1, total: content.slides.length })}
          {disabled}
          onclick={() => onselect(i + 1)}
        >
          <span class="mini"><OutputView content={{ ...content, index: i }} {style} preview /></span>
          <span class="num">{i + 1}</span>
        </button>
      </li>
    {/each}
  </ol>
{:else if content.kind === "scripture" && content.verses.length > 1}
  <ol class="chips {tone}" bind:this={strip}>
    {#each content.verses as verse, i (i)}
      <li>
        <button
          class="chip"
          aria-current={i === content.index}
          title={verse.text}
          {disabled}
          onclick={() => onselect(i + 1)}
        >
          {verse.reference.split(":").pop()}
        </button>
      </li>
    {/each}
  </ol>
{/if}

<style>
  ol { list-style: none; margin: 0; padding: 2px; display: flex; gap: 8px; overflow-x: auto; }
  .thumbs li { flex: 0 0 132px; }
  .thumb {
    position: relative;
    width: 100%;
    padding: 3px;
    background: var(--surface-2);
    border: 2px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }
  .mini { display: block; aspect-ratio: 16 / 9; border-radius: 4px; overflow: hidden; pointer-events: none; }
  .num {
    position: absolute;
    left: 7px;
    bottom: 6px;
    padding: 0 6px;
    font-size: 12px;
    font-weight: 700;
    border-radius: 4px;
    background: rgb(0 0 0 / 0.65);
    color: #fff;
  }
  .chips { flex-wrap: wrap; overflow: visible; gap: 6px; }
  .chip {
    min-width: 40px;
    min-height: 36px;
    padding: 0 10px;
    font: inherit;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--text);
    background: var(--surface-2);
    border: 2px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }
  button:disabled { cursor: default; opacity: 0.6; }
  .prepared [aria-current="true"] { border-color: var(--accent); background: var(--accent-soft); }
  .live [aria-current="true"] { border-color: var(--live); background: var(--live-soft); }
</style>
