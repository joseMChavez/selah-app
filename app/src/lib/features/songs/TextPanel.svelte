<script lang="ts">
  // Textos: una biblioteca de letras guardadas y el texto libre de siempre. Lo que se presenta en un culto,
  // en orden, va en un plan (features/plans).
  import { t, tCount } from "$lib/i18n/index.svelte";
  import { addToOpenPlan, openPlan } from "$lib/features/plans/plans.svelte";
  import {
    allowedBackgrounds,
    allowSongBackgrounds,
    deleteSong,
    library,
    saveSong,
    songContent,
    type Song,
  } from "$lib/features/songs/library.svelte";
  import { toMessage, type Message } from "$lib/core/messages";
  import { platform, type OutputContent } from "$lib/platform";
  import { splitSlides } from "$lib/features/songs/slides";

  let {
    onprepare,
    onerror,
  }: {
    onprepare: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  type Mode = "library" | "free";

  let mode = $state<Mode>("library");
  let draft = $state(t("text.sample"));
  let slides = $derived(splitSlides(draft));
  let filter = $state("");
  /** Letra que se está editando (copia); null = no se edita nada. */
  let editing = $state<Song | null>(null);
  let current = $state<string | null>(null);
  let confirmDelete = $state<string | null>(null);
  /** Letras añadidas al plan abierto en esta visita, para confirmarlo en el botón. */
  let added = $state(new Set<string>());
  /** Aviso breve tras añadir al plan. */
  let notice = $state<string | null>(null);
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;
  let shown = $derived.by(() => {
    const q = filter.trim().toLocaleLowerCase();
    if (!q) return library.songs;
    return library.songs.filter((s) => [s.title, s.singer, s.writer, s.body].some((f) => f.toLocaleLowerCase().includes(q)));
  });
  let editingSlides = $derived(editing ? splitSlides(editing.body) : []);

  const modes: { id: Mode; label: "songs.library" | "songs.free" }[] = [
    { id: "library", label: "songs.library" },
    { id: "free", label: "songs.free" },
  ];

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function prepareSong(song: Song) {
    current = song.id;
    onprepare(songContent(song));
  }

  // Escribir solo cambia «Preparado»: la pantalla no se toca hasta pulsar «Enviar a pantalla».
  function updateFree() {
    current = null;
    onprepare(slides.length > 0 ? { kind: "text", slides, index: 0 } : { kind: "empty" });
  }

  function startNew() {
    editing = { id: crypto.randomUUID(), title: "", singer: "", writer: "", body: "", background: null };
  }

  function startEdit(song: Song) {
    editing = { ...song };
    prepareSong(song);
  }

  function saveEditing() {
    if (!editing) return;
    const song = { ...editing, title: editing.title.trim() || t("songs.untitled") };
    saveSong(song);
    prepareSong(song);
    editing = null;
  }

  async function pickBackground() {
    if (!editing) return;
    try {
      const path = await platform.pickBackground(t("background.dialogTitle"), t("background.dialogFilter"));
      if (path && editing) {
        allowedBackgrounds.add(path);
        editing.background = path;
        onprepare(songContent(editing));
      }
    } catch (e) {
      onerror(toMessage(e));
      platform.logError(`fondo de letra: no se pudo elegir: ${JSON.stringify(e)}`).catch(() => {});
    }
  }

  function remove(id: string) {
    if (confirmDelete !== id) {
      confirmDelete = id;
      return;
    }
    deleteSong(id);
    confirmDelete = null;
  }

  function addToPlan(song: Song) {
    addToOpenPlan({ id: crypto.randomUUID(), kind: "song", songId: song.id });
    added = new Set([...added, song.id]);
    notice = t("plans.addedSong", { title: song.title, plan: openPlan.plan?.name ?? "" });
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => (notice = null), 3000);
  }

  // Al cambiar de plan, los botones vuelven a ofrecer «+ Plan».
  $effect(() => {
    void openPlan.id;
    added = new Set();
  });

  function chooseMode(m: Mode) {
    mode = m;
    editing = null;
    confirmDelete = null;
  }

  // Cada fondo nuevo que llega a la biblioteca (al abrir, al importar letras) se vuelve a permitir.
  $effect(() => {
    void library.songs;
    allowSongBackgrounds();
  });
</script>

<div class="text-panel">
  <div class="segmented modes" role="tablist">
    {#each modes as m (m.id)}
      <button role="tab" aria-selected={mode === m.id} onclick={() => chooseMode(m.id)}>
        {t(m.label)}
      </button>
    {/each}
  </div>

  {#if editing}
    <form class="editor" onsubmit={(e) => { e.preventDefault(); saveEditing(); }}>
      <label class="field-row">
        <span class="label">{t("songs.title")}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="field" bind:value={editing.title} autofocus />
      </label>
      <div class="pair">
        <label class="field-row">
          <span class="label">{t("songs.singer")}</span>
          <input class="field" bind:value={editing.singer} />
        </label>
        <label class="field-row">
          <span class="label">{t("songs.writer")}</span>
          <input class="field" bind:value={editing.writer} />
        </label>
      </div>
      <label class="field-row grow">
        <span class="label">{t("songs.lyrics")}</span>
        <textarea class="field" bind:value={editing.body} oninput={() => editing && onprepare(songContent(editing))} rows="12" placeholder={t("songs.lyricsPlaceholder")}></textarea>
      </label>
      <p class="muted small">
        {t("songs.hint")}
        {#if editingSlides.length > 0}· {tCount("text.count", editingSlides.length)}{/if}
      </p>
      <div class="row">
        <span class="label">{t("songs.background")}</span>
        {#if editing.background}
          <span class="muted small name" title={editing.background}>{baseName(editing.background)}</span>
          <button type="button" class="btn ghost" onclick={() => editing && (editing.background = null)}>{t("background.remove")}</button>
        {:else}
          <span class="muted small">{t("songs.backgroundDefault")}</span>
        {/if}
        <button type="button" class="btn" onclick={pickBackground}>{t("background.pick")}</button>
      </div>
      <div class="row end">
        <button type="button" class="btn ghost" onclick={() => (editing = null)}>{t("songs.cancel")}</button>
        <button type="submit" class="btn primary" disabled={editingSlides.length === 0}>{t("songs.save")}</button>
      </div>
    </form>
  {:else if mode === "library"}
    <div class="actions">
      <button class="btn primary" onclick={startNew}>{t("songs.new")}</button>
      <input class="field search" bind:value={filter} placeholder={t("songs.search")} aria-label={t("songs.search")} />
    </div>
    {#if notice}<p class="notice small" role="status">✓ {notice}</p>{/if}
    {#if library.songs.length === 0}
      <p class="muted">{t("songs.empty")}</p>
    {:else if shown.length === 0}
      <p class="muted">{t("songs.noMatches", { query: filter })}</p>
    {:else}
      <ul class="items">
        {#each shown as song (song.id)}
          <li class:chosen={song.id === current}>
            <button class="item" onclick={() => prepareSong(song)}>
              <span class="name">
                {song.title}
                <span class="muted small sub">{[song.singer, tCount("text.count", splitSlides(song.body).length)].filter(Boolean).join(" · ")}</span>
              </span>
            </button>
            {#if openPlan.id}
              <button class="btn ghost small-btn" class:on={added.has(song.id)} onclick={() => addToPlan(song)} title={t("songs.addToPlanHint")}>
                {added.has(song.id) ? t("songs.inPlan") : t("songs.addToPlan")}
              </button>
            {/if}
            <button class="btn ghost small-btn" onclick={() => startEdit(song)}>{t("songs.edit")}</button>
            <button class="icon-btn remove" class:confirm={confirmDelete === song.id} onclick={() => remove(song.id)} aria-label={t("songs.delete", { title: song.title })} title={t("songs.delete", { title: song.title })}>
              {confirmDelete === song.id ? t("songs.confirmDelete") : "✕"}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <label class="label" for="text-draft">{t("text.label")}</label>
    <textarea id="text-draft" class="field free" bind:value={draft} oninput={updateFree} onfocus={updateFree} rows="12"></textarea>
    <p class="muted small">
      {t("text.hint")}
      {#if slides.length > 1}· {tCount("text.count", slides.length)}{/if}
    </p>
  {/if}
</div>

<style>
  .text-panel { display: flex; flex-direction: column; gap: 10px; height: 100%; }
  .modes { align-self: flex-start; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
  textarea { flex: 1; min-height: 220px; resize: vertical; font-size: 16px; line-height: 1.5; }
  .editor { display: flex; flex-direction: column; gap: 10px; flex: 1; }
  .field-row { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
  .field-row.grow { flex: 1; }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .row.end { justify-content: flex-end; }
  .row .name { max-width: 14em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .actions { display: flex; gap: 8px; }
  .notice {
    margin: 0;
    padding: 6px 10px;
    border-radius: 8px;
    color: var(--accent);
    background: var(--accent-soft);
    font-weight: 600;
  }
  .search { flex: 1; min-width: 0; }

  .items { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  li { display: flex; align-items: center; gap: 2px; border: 1px solid transparent; border-radius: 8px; }
  li:hover { background: var(--surface-2); }
  li.chosen { border-color: var(--accent); background: var(--accent-soft); }
  .item {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 46px;
    padding: 6px 10px;
    font: inherit;
    text-align: left;
    color: var(--text);
    background: transparent;
    border: 0;
    cursor: pointer;
  }
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; font-weight: 600; }
  .name, .sub { text-overflow: ellipsis; white-space: nowrap; }
  .sub { font-weight: 400; overflow: hidden; }
  .small-btn { min-height: 34px; padding: 0 10px; font-size: 13px; }
  .small-btn.on { color: var(--accent); }
  .icon-btn {
    flex: none;
    min-width: 36px;
    min-height: 36px;
    font: inherit;
    font-size: 13px;
    color: var(--muted);
    background: transparent;
    border: 0;
    border-radius: 8px;
    cursor: pointer;
  }
  .icon-btn:disabled { opacity: 0.35; cursor: default; }
  .icon-btn:not(:disabled):hover { background: var(--surface); color: var(--text); }
  .icon-btn.remove:hover, .icon-btn.confirm { color: var(--live); background: var(--live-soft); }
  .icon-btn.confirm { padding: 0 10px; font-weight: 650; }
</style>
