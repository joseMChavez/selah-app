<script lang="ts">
  // Importar letras (de un .selah o de una lista de Holyrics) y exportar la biblioteca a un .selah.
  // Primero se ve qué va a entrar y se elige qué hacer con las repetidas; nada se guarda hasta confirmar.
  import type { PanelProps } from "$lib/core/panels";
  import { toMessage } from "$lib/core/messages";
  import { library, loadLibrary } from "$lib/features/songs/library.svelte";
  import { loadPlans, openPlan, plans } from "$lib/features/plans/plans.svelte";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import { platform, type ImportPreview, type ImportResult, type OnConflict } from "$lib/platform";

  let { onerror }: PanelProps = $props();

  const REASONS: Record<string, MessageKey> = {
    noTitle: "transfer.invalid.noTitle",
    noLyrics: "transfer.invalid.noLyrics",
    repeated: "transfer.invalid.repeated",
  };
  const choices: { id: OnConflict; label: MessageKey; hint: MessageKey }[] = [
    { id: "skip", label: "transfer.conflict.skip", hint: "transfer.conflict.skipHint" },
    { id: "replace", label: "transfer.conflict.replace", hint: "transfer.conflict.replaceHint" },
    { id: "keepBoth", label: "transfer.conflict.keepBoth", hint: "transfer.conflict.keepBothHint" },
  ];

  let preview = $state<ImportPreview | null>(null);
  let onConflict = $state<OnConflict>("skip");
  /** Guardar la lista del archivo como un plan, y con qué nombre. */
  let makePlan = $state(false);
  let planName = $state("");
  /** Plan cuyas letras se exportan. */
  let exportPlanId = $state<string | null>(openPlan.id);
  let busy = $state(false);
  let result = $state<ImportResult | null>(null);
  let exported = $state<{ path: string; songs: number } | null>(null);

  let importable = $derived(preview ? preview.fresh.length + (onConflict === "skip" ? 0 : preview.conflicts.length) : 0);
  let withSongs = $derived(plans.all.filter((p) => p.items.some((i) => i.kind === "song")));
  let exportPlan = $derived(withSongs.find((p) => p.id === exportPlanId) ?? withSongs[0] ?? null);
  let exportPlanSongs = $derived(exportPlan ? new Set(exportPlan.items.flatMap((i) => (i.kind === "song" ? [i.songId] : []))).size : 0);
  let creatingPlan = $derived(makePlan && preview !== null && preview.setlist > 0 && planName.trim() !== "");

  function fileStem(file: string): string {
    return (file.split(/[\\/]/).pop() ?? file).replace(/\.[^.]+$/, "");
  }

  async function pick() {
    result = null;
    exported = null;
    try {
      busy = true;
      const p = await platform.transferImportPick(t("transfer.dialogTitle"), t("transfer.dialogFilter"));
      if (!p) return;
      preview = p;
      onConflict = "skip";
      // Una lista de Holyrics es la lista de un culto: lo normal es querer usarla como plan.
      makePlan = p.format === "holyrics";
      planName = fileStem(p.file);
    } catch (e) {
      onerror(toMessage(e));
    } finally {
      busy = false;
    }
  }

  async function apply() {
    try {
      busy = true;
      result = await platform.transferImportApply(onConflict, creatingPlan ? planName.trim() : null, t("transfer.importedSuffix"));
      preview = null;
      await Promise.all([loadLibrary(), loadPlans()]);
    } catch (e) {
      onerror(toMessage(e));
    } finally {
      busy = false;
    }
  }

  function cancel() {
    preview = null;
    platform.transferImportCancel().catch(() => {});
  }

  async function exportSongs(planId: string | null) {
    result = null;
    exported = null;
    const date = new Date().toISOString().slice(0, 10);
    const plan = plans.all.find((p) => p.id === planId);
    const name = plan ? `${plan.name} ${plan.date ?? date}.selah` : `${t("transfer.fileLibrary")} ${date}.selah`;
    try {
      busy = true;
      exported = await platform.transferExport(t("transfer.exportTitle"), t("transfer.exportFilter"), name, planId);
    } catch (e) {
      onerror(toMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="transfer">
  <section class="block">
    <h3>{t("transfer.importHeading")}</h3>
    <p class="muted small">{t("transfer.importIntro")}</p>

    {#if !preview}
      <div class="row">
        <button class="btn primary" onclick={pick} disabled={busy}>{t("transfer.pick")}</button>
        {#if library.songs.length > 0}
          <span class="muted small">{t("transfer.backupHint")}</span>
        {/if}
      </div>
    {:else}
      <div class="preview" aria-live="polite">
        <p class="file">
          <strong>{preview.file}</strong>
          <span class="muted small">{t(preview.format === "holyrics" ? "transfer.formatHolyrics" : "transfer.formatSelah")}</span>
        </p>

        {#if preview.fresh.length > 0}
          <details open={preview.fresh.length <= 8}>
            <summary>{t("transfer.fresh", { count: preview.fresh.length })}</summary>
            <ul>
              {#each preview.fresh as song, i (i)}
                <li>{song.title}{#if song.singer}<span class="muted small"> · {song.singer}</span>{/if}<span class="muted small"> · {t("text.count", { count: song.slides })}</span></li>
              {/each}
            </ul>
          </details>
        {/if}

        {#if preview.identical > 0}
          <p class="muted small">{t("transfer.identical", { count: preview.identical })}</p>
        {/if}

        {#if preview.conflicts.length > 0}
          <details open>
            <summary>{t("transfer.conflicts", { count: preview.conflicts.length })}</summary>
            <ul>
              {#each preview.conflicts as song, i (i)}
                <li>{song.title}{#if song.singer}<span class="muted small"> · {song.singer}</span>{/if}</li>
              {/each}
            </ul>
          </details>
          <fieldset class="choices">
            <legend class="small">{t("transfer.conflictQuestion")}</legend>
            {#each choices as c (c.id)}
              <label class="choice">
                <input type="radio" name="on-conflict" value={c.id} bind:group={onConflict} />
                <span>{t(c.label)}<span class="muted small"> — {t(c.hint)}</span></span>
              </label>
            {/each}
          </fieldset>
        {/if}

        {#if preview.invalid.length > 0}
          <details>
            <summary class="warn">{t("transfer.invalid", { count: preview.invalid.length })}</summary>
            <ul>
              {#each preview.invalid as song, i (i)}
                <li>{song.title || t("songs.untitled")}<span class="muted small"> — {t(REASONS[song.reason])}</span></li>
              {/each}
            </ul>
          </details>
        {/if}

        {#if preview.setlist > 0}
          <label class="check">
            <input type="checkbox" bind:checked={makePlan} />
            {t("transfer.makePlan", { count: preview.setlist })}
          </label>
          {#if makePlan}
            <input class="field plan-name" bind:value={planName} aria-label={t("plans.name")} maxlength="80" />
          {/if}
        {/if}

        <div class="row end">
          <button class="btn ghost" onclick={cancel} disabled={busy}>{t("songs.cancel")}</button>
          <button class="btn primary" onclick={apply} disabled={busy || (importable === 0 && !creatingPlan)}>
            {t("transfer.apply", { count: importable })}
          </button>
        </div>
      </div>
    {/if}

    {#if result}
      <p class="ok small" role="status">
        {t("transfer.done", { added: result.added, replaced: result.replaced, skipped: result.skipped })}
        {#if result.planSongs > 0}· {t("transfer.donePlan", { count: result.planSongs })}{/if}
      </p>
    {/if}
  </section>

  <section class="block">
    <h3>{t("transfer.exportHeading")}</h3>
    <p class="muted small">{t("transfer.exportIntro")}</p>
    <div class="row">
      <button class="btn" onclick={() => exportSongs(null)} disabled={busy || library.songs.length === 0}>
        {t("transfer.exportLibrary", { count: library.songs.length })}
      </button>
    </div>
    {#if withSongs.length > 0}
      <div class="row">
        <select class="field" value={exportPlan?.id} onchange={(e) => (exportPlanId = e.currentTarget.value)} aria-label={t("transfer.exportPlanPick")}>
          {#each withSongs as p (p.id)}
            <option value={p.id}>{p.name}{p.date ? ` · ${p.date}` : ""}</option>
          {/each}
        </select>
        <button class="btn" onclick={() => exportSongs(exportPlan!.id)} disabled={busy || !exportPlan}>
          {t("transfer.exportPlan", { count: exportPlanSongs })}
        </button>
      </div>
    {/if}
    {#if exported}
      <p class="ok small" role="status">{t("transfer.exported", { count: exported.songs })} <code title={exported.path}>{exported.path}</code></p>
    {/if}
  </section>
</div>

<style>
  .transfer { display: flex; flex-direction: column; gap: 18px; }
  .block { display: flex; flex-direction: column; gap: 8px; }
  h3 { margin: 0; font-size: 1rem; }
  p { margin: 0; }
  .row { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
  .row.end { justify-content: flex-end; }
  .preview { display: flex; flex-direction: column; gap: 10px; padding: 12px; border: 1px solid var(--border); border-radius: 8px; }
  .file { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px; }
  ul { margin: 6px 0 0; padding-left: 20px; max-height: 180px; overflow: auto; }
  li { margin: 2px 0; }
  summary { cursor: pointer; }
  .choices { display: flex; flex-direction: column; gap: 4px; margin: 0; padding: 8px 10px; border: 1px solid var(--border); border-radius: 6px; }
  .choice, .check { display: flex; gap: 8px; align-items: baseline; }
  code { overflow-wrap: anywhere; }
  .plan-name { max-width: 24em; }
</style>
