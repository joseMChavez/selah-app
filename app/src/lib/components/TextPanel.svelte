<script lang="ts">
  // Textos: una biblioteca de letras guardadas, la lista del culto y el texto libre de siempre.
  import { onMount } from "svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { deleteSong, library, saveSong, setSetlist, type Song } from "$lib/library.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { platform, type OutputContent } from "$lib/platform";
  import { splitSlides } from "$lib/slides";

  let {
    onprepare,
    onerror,
  }: {
    onprepare: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  type Mode = "setlist" | "library" | "free";

  let mode = $state<Mode>(library.setlist.length > 0 ? "setlist" : "library");
  let draft = $state(t("text.sample"));
  let slides = $derived(splitSlides(draft));
  let filter = $state("");
  /** Letra que se está editando (copia); null = no se edita nada. */
  let editing = $state<Song | null>(null);
  let current = $state<string | null>(null);
  let confirmDelete = $state<string | null>(null);
  /** Fondos que existen en este equipo; los demás se ignoran para no impedir mostrar la letra. */
  let backgrounds = $state(new Set<string>());

  let setlistSongs = $derived(library.setlist.map((id) => library.songs.find((s) => s.id === id)).filter((s) => s !== undefined));
  let shown = $derived.by(() => {
    const q = filter.trim().toLocaleLowerCase();
    if (!q) return library.songs;
    return library.songs.filter((s) => [s.title, s.singer, s.writer, s.body].some((f) => f.toLocaleLowerCase().includes(q)));
  });
  let editingSlides = $derived(editing ? splitSlides(editing.body) : []);

  const modes: { id: Mode; label: "songs.setlist" | "songs.library" | "songs.free" }[] = [
    { id: "setlist", label: "songs.setlist" },
    { id: "library", label: "songs.library" },
    { id: "free", label: "songs.free" },
  ];

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function credit(song: Song): string {
    return [song.title, song.singer, song.writer && t("songs.writtenBy", { name: song.writer })].filter(Boolean).join(" · ");
  }

  function songContent(song: Song): OutputContent {
    const slides = splitSlides(song.body);
    if (slides.length === 0) return { kind: "empty" };
    const background = song.background && backgrounds.has(song.background) ? song.background : null;
    return { kind: "text", slides, index: 0, credit: credit(song), background };
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
        backgrounds = new Set([...backgrounds, path]);
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

  function toggleInSetlist(id: string) {
    setSetlist(library.setlist.includes(id) ? library.setlist.filter((x) => x !== id) : [...library.setlist, id]);
  }

  function move(i: number, delta: -1 | 1) {
    const ids = [...library.setlist];
    const j = i + delta;
    if (j < 0 || j >= ids.length) return;
    [ids[i], ids[j]] = [ids[j], ids[i]];
    setSetlist(ids);
  }

  function chooseMode(m: Mode) {
    mode = m;
    editing = null;
    confirmDelete = null;
  }

  onMount(() => {
    // El permiso para leer los fondos no sobrevive a un reinicio de la app.
    const paths = [...new Set(library.songs.map((s) => s.background).filter((p) => p !== null))];
    if (paths.length > 0) platform.allowBackgrounds(paths).then((ok) => (backgrounds = new Set(ok))).catch(() => {});
  });
</script>

<div class="text-panel">
  <div class="segmented modes" role="tablist">
    {#each modes as m (m.id)}
      <button role="tab" aria-selected={mode === m.id} onclick={() => chooseMode(m.id)}>
        {t(m.label)}
        {#if m.id === "setlist" && library.setlist.length > 0}<span class="count">{library.setlist.length}</span>{/if}
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
        {#if editingSlides.length > 0}· {t("text.count", { count: editingSlides.length })}{/if}
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
  {:else if mode === "setlist"}
    {#if setlistSongs.length === 0}
      <p class="muted">{t("songs.setlistEmpty")}</p>
    {:else}
      <p class="muted small">{t("songs.setlistHint")}</p>
      <ol class="items">
        {#each setlistSongs as song, i (song.id)}
          <li class:chosen={song.id === current}>
            <button class="item" onclick={() => prepareSong(song)}>
              <span class="num">{i + 1}</span>
              <span class="name">{song.title}<span class="muted small sub">{song.singer}</span></span>
            </button>
            <button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label={t("songs.moveUp")}>▲</button>
            <button class="icon-btn" onclick={() => move(i, 1)} disabled={i === setlistSongs.length - 1} aria-label={t("songs.moveDown")}>▼</button>
            <button class="icon-btn remove" onclick={() => toggleInSetlist(song.id)} aria-label={t("songs.removeFromSetlist", { title: song.title })}>✕</button>
          </li>
        {/each}
      </ol>
      <button class="btn ghost clear" onclick={() => setSetlist([])}>{t("songs.clearSetlist")}</button>
    {/if}
  {:else if mode === "library"}
    <div class="actions">
      <button class="btn primary" onclick={startNew}>{t("songs.new")}</button>
      <input class="field search" bind:value={filter} placeholder={t("songs.search")} aria-label={t("songs.search")} />
    </div>
    {#if library.songs.length === 0}
      <p class="muted">{t("songs.empty")}</p>
    {:else if shown.length === 0}
      <p class="muted">{t("songs.noMatches", { query: filter })}</p>
    {:else}
      <ul class="items">
        {#each shown as song (song.id)}
          {@const inSetlist = library.setlist.includes(song.id)}
          <li class:chosen={song.id === current}>
            <button class="item" onclick={() => prepareSong(song)}>
              <span class="name">
                {song.title}
                <span class="muted small sub">{[song.singer, t("text.count", { count: splitSlides(song.body).length })].filter(Boolean).join(" · ")}</span>
              </span>
            </button>
            <button class="btn ghost small-btn" class:on={inSetlist} onclick={() => toggleInSetlist(song.id)}>
              {inSetlist ? t("songs.inSetlist") : t("songs.addToSetlist")}
            </button>
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
      {#if slides.length > 1}· {t("text.count", { count: slides.length })}{/if}
    </p>
  {/if}
</div>

<style>
  .text-panel { display: flex; flex-direction: column; gap: 10px; height: 100%; }
  .modes { align-self: flex-start; }
  .count {
    margin-left: 6px;
    padding: 0 6px;
    font-size: 12px;
    border-radius: 999px;
    /* Se lee igual en la pestaña elegida (texto claro) y en las demás. */
    background: color-mix(in srgb, currentColor 18%, transparent);
  }
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
  .search { flex: 1; min-width: 0; }
  .clear { align-self: flex-start; }

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
  .num { min-width: 1.5em; font-weight: 700; color: var(--accent); font-variant-numeric: tabular-nums; }
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
