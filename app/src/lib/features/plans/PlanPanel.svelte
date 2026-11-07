<script lang="ts">
  // Plan: lo que se va a presentar en un culto, preparado de antemano y en orden (D-019).
  // Sin plan abierto se ven todos los planes; con uno abierto, se edita y se usa en el mismo sitio:
  // un clic prepara el elemento, doble clic lo envía a pantalla, «Siguiente» prepara el que sigue.
  import { t, i18n } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/core/messages";
  import { formatCountdown, preparedTimer } from "$lib/features/clock/clock";
  import { screenKey } from "$lib/features/output/history.svelte";
  import { library, songContent } from "$lib/features/songs/library.svelte";
  import { bibles, currentBibles, visibleBibles } from "$lib/features/bible/bibles.svelte";
  import { bookLanguage } from "$lib/features/bible/scripture";
  import { bibleFor, missingIn, parseList, referenceLabel, scriptureContent, type ScriptureItem } from "./scripture";
  import { splitSlides } from "$lib/features/songs/slides";
  import { platform, type MediaFile, type OutputContent } from "$lib/platform";
  import {
    choosePlan,
    createPlan,
    deletePlan,
    duplicatePlan,
    missingFiles,
    openPlan,
    plans,
    savePlan,
    today,
    type Plan,
    type PlanItem,
  } from "./plans.svelte";

  let {
    live,
    onprepare,
    onsend,
    onerror,
  }: {
    live: OutputContent;
    onprepare: (content: OutputContent) => void;
    onsend: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  const ICONS = { song: "♪", image: "🖼", video: "🎬", pdf: "📄", presentation: "📊", pause: "⏱", scripture: "📖" } as const;
  const PAUSE_PRESETS = [5, 10, 15, 20, 30];
  const MAX_MINUTES = 600;

  type Pause = Extract<PlanItem, { kind: "pause" }>;

  let plan = $derived(openPlan.plan);
  /** Elemento preparado desde el plan, y lo que se preparó con él (para saber cuándo llega a la pantalla). */
  let current = $state<{ id: string; content: OutputContent } | null>(null);
  /** Elementos que ya pasaron por la pantalla en esta sesión, por plan. */
  let shown = $state<Record<string, string[]>>({});
  let confirmDelete = $state<string | null>(null);
  let showPast = $state(false);

  /** Qué se está añadiendo o editando debajo de la lista. */
  let adding = $state<"song" | "scripture" | null>(null);
  let songFilter = $state("");
  /** La lista de citas que se está escribiendo o pegando, y con qué versión se mostrarán ("" = la de la consola). */
  let scriptureText = $state("");
  let scriptureBible = $state("");
  let addingScripture = $state(false);
  let scriptureLines = $derived(parseList(scriptureText, bookLanguage(currentBibles().bible)));
  let scriptureReady = $derived(scriptureLines.filter((l) => l.reference !== null).length);
  /** Pausa en el formulario: nueva (sin id en el plan) o una que se edita. */
  let pauseDraft = $state<{ id: string; title: string; minutes: number; seconds: number; background: string | null; isNew: boolean } | null>(null);
  /** Sección cuyo título se está escribiendo. */
  let editingSection = $state<string | null>(null);

  /** Presentación → PDF ya convertido, para no volver a pedirlo. */
  const converted = new Map<string, string>();
  let converting = $state<string | null>(null);
  /** Copiando medios elegidos a la carpeta de la app. */
  let copying = $state(false);
  /** Aviso breve tras añadir algo («Admirable» añadida al plan). */
  let notice = $state<string | null>(null);
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;

  function notify(text: string) {
    notice = text;
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => (notice = null), 3000);
  }

  const now = today();
  let upcoming = $derived(plans.all.filter((p) => p.date === null || p.date >= now));
  let past = $derived(plans.all.filter((p) => p.date !== null && p.date < now).reverse());
  let presentable = $derived(plan ? plan.items.filter((i) => i.kind !== "section") : []);
  let songMatches = $derived.by(() => {
    const q = songFilter.trim().toLocaleLowerCase();
    const songs = q ? library.songs.filter((s) => [s.title, s.singer, s.body].some((f) => f.toLocaleLowerCase().includes(q))) : library.songs;
    return songs.slice(0, 50);
  });

  function formatDate(date: string | null): string {
    if (!date) return t("plans.noDate");
    const [y, m, d] = date.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(i18n.locale, { weekday: "long", day: "numeric", month: "long", year: "numeric" });
  }

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function songOf(item: PlanItem) {
    return item.kind === "song" ? library.songs.find((s) => s.id === item.songId) : undefined;
  }

  /** Lo que impide mostrar un elemento: la letra se borró o el archivo ya no está. */
  function problem(item: PlanItem): string | null {
    if (item.kind === "song" && !songOf(item)) return t("plans.songDeleted");
    if (item.kind === "scripture" && !bibles.loading && missingIn(item, bibleFor(item))) return t("plans.notInBible");
    if (item.kind === "media" && missingFiles.has(item.path)) return t("plans.fileMissing");
    return null;
  }

  function title(item: PlanItem): string {
    switch (item.kind) {
      case "section": return item.title;
      case "song": return songOf(item)?.title ?? t("plans.song");
      case "media": return baseName(item.path);
      case "pause": return item.title || t("plans.pause");
      case "scripture": return referenceLabel(item, bibleFor(item));
    }
  }

  function detail(item: PlanItem): string {
    switch (item.kind) {
      case "song": {
        const song = songOf(item);
        return song ? [song.singer, t("text.count", { count: splitSlides(song.body).length })].filter(Boolean).join(" · ") : "";
      }
      case "media": return t(`kind.${item.media}`);
      case "scripture": {
        const bible = bibleFor(item);
        if (!bible) return t("plans.verses");
        const fixed = item.bible && bible.id === item.bible;
        return `${t("plans.verses")} · ${fixed ? bible.abbreviation : t("plans.consoleVersion", { version: bible.abbreviation })}`;
      }
      case "pause": {
        const bg = item.background && !missingFiles.has(item.background) ? ` · ${baseName(item.background)}` : "";
        return `${t("plans.pause")} · ${formatCountdown(item.durationMs)}${bg}`;
      }
      default: return "";
    }
  }

  function icon(item: PlanItem): string {
    if (item.kind === "media") return ICONS[item.media];
    if (item.kind === "song" || item.kind === "pause" || item.kind === "scripture") return ICONS[item.kind];
    return "";
  }

  /** El contenido de un elemento para «Preparado»; null si no se puede mostrar. */
  async function contentOf(item: PlanItem): Promise<OutputContent | null> {
    switch (item.kind) {
      case "section": return null;
      case "scripture": return scriptureContent(item);
      case "song": {
        const song = songOf(item);
        return song ? songContent(song) : null;
      }
      case "pause": {
        const background = item.background && !missingFiles.has(item.background) ? item.background : null;
        return preparedTimer(item.durationMs, item.title, background);
      }
      case "media":
        switch (item.media) {
          case "image": return { kind: "image", path: item.path };
          case "video": return { kind: "video", path: item.path, paused: false };
          case "pdf": return { kind: "pdf", path: item.path, page: 1 };
          case "presentation": {
            // Una presentación se muestra como el PDF que hace de ella PowerPoint, Keynote o LibreOffice.
            let pdf = converted.get(item.path);
            if (!pdf) {
              converting = item.id;
              try {
                pdf = await platform.convertPresentation(item.path);
                converted.set(item.path, pdf);
              } finally {
                converting = null;
              }
            }
            return { kind: "pdf", path: pdf, page: 1 };
          }
        }
    }
  }

  async function prepareItem(item: PlanItem, send = false) {
    if (item.kind === "section" || problem(item)) return;
    try {
      const content = await contentOf(item);
      if (!content && item.kind === "scripture") {
        onerror({ key: "plans.verseNotFound", params: { reference: title(item) } });
        return;
      }
      if (!content || content.kind === "empty") return;
      current = { id: item.id, content };
      if (send) onsend(content);
      else onprepare(content);
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  /** Prepara el siguiente elemento que se pueda mostrar. */
  function prepareNext() {
    const start = current ? presentable.findIndex((i) => i.id === current!.id) + 1 : 0;
    const next = presentable.slice(start).find((i) => !problem(i));
    if (next) prepareItem(next);
  }

  // Cuando lo preparado desde el plan llega a la pantalla, se marca como mostrado.
  $effect(() => {
    const onScreen = live;
    if (!current || !plan) return;
    const c = current.content;
    const same =
      c.kind === "timer"
        ? onScreen.kind === "timer" && onScreen.label === c.label && onScreen.durationMs === c.durationMs
        : screenKey(onScreen) !== null && screenKey(onScreen) === screenKey(c);
    const done = shown[plan.id] ?? [];
    if (same && !done.includes(current.id)) shown[plan.id] = [...done, current.id];
  });

  // --- Editar el plan ---

  function update(change: Partial<Plan>) {
    if (plan) savePlan({ ...plan, ...change });
  }

  function setItems(items: PlanItem[]) {
    update({ items });
  }

  function move(i: number, delta: -1 | 1) {
    if (!plan) return;
    const items = [...plan.items];
    const j = i + delta;
    if (j < 0 || j >= items.length) return;
    [items[i], items[j]] = [items[j], items[i]];
    setItems(items);
  }

  function removeItem(id: string) {
    if (plan) setItems(plan.items.filter((i) => i.id !== id));
  }

  function addSection() {
    if (!plan) return;
    const id = crypto.randomUUID();
    setItems([...plan.items, { id, kind: "section", title: t("plans.newSection") }]);
    editingSection = id;
  }

  function renameSection(id: string, value: string) {
    if (!plan) return;
    editingSection = null;
    const name = value.trim();
    if (!name) return;
    setItems(plan.items.map((i) => (i.id === id && i.kind === "section" ? { ...i, title: name } : i)));
  }

  function addSong(songId: string) {
    if (!plan) return;
    setItems([...plan.items, { id: crypto.randomUUID(), kind: "song", songId }]);
    const song = library.songs.find((s) => s.id === songId);
    notify(t("plans.addedSong", { title: song?.title ?? "", plan: plan.name }));
  }

  async function addMedia() {
    if (!plan) return;
    copying = true;
    try {
      // Se guardan copias en la carpeta de Selah: el plan sigue sirviendo aunque se mueva o borre el original.
      const picked: MediaFile[] = await platform.planPickMedia(t("dialog.title"), t("dialog.filter"));
      // El plan pudo cambiar mientras el diálogo estaba abierto: se añade al que está abierto ahora.
      const target = openPlan.plan;
      if (!target || picked.length === 0) return;
      for (const file of picked) missingFiles.delete(file.path);
      savePlan({
        ...target,
        items: [...target.items, ...picked.map((f): PlanItem => ({ id: crypto.randomUUID(), kind: "media", path: f.path, media: f.kind }))],
      });
      notify(picked.length === 1 ? t("plans.addedMedia", { name: picked[0].name }) : t("plans.addedMediaMany", { count: picked.length }));
    } catch (e) {
      onerror(toMessage(e));
    } finally {
      copying = false;
    }
  }

  /** Añade las citas reconocidas. Las que la versión no tiene (Juan 3:99) no entran y se avisa. */
  async function addScripture() {
    if (!plan) return;
    addingScripture = true;
    try {
      const bible = scriptureBible || null;
      const found: ScriptureItem[] = [];
      const notFound: string[] = [];
      for (const line of scriptureLines) {
        if (!line.reference) continue;
        const { book, chapter, verseStart, verseEnd } = line.reference;
        const item: ScriptureItem = { id: crypto.randomUUID(), kind: "scripture", book, chapter, verseStart, verseEnd, bible };
        const ok = await scriptureContent(item).catch(() => null);
        if (ok) found.push(item);
        else notFound.push(line.text);
      }
      const target = openPlan.plan;
      if (target && found.length > 0) savePlan({ ...target, items: [...target.items, ...found] });
      if (notFound.length > 0) {
        onerror({ key: "plans.versesNotFound", params: { list: notFound.join("; ") } });
        // Quedan escritas las que no entraron, para corregirlas.
        scriptureText = notFound.join("\n");
      } else {
        scriptureText = "";
        adding = null;
      }
      if (found.length > 0) notify(t("plans.addedVerses", { count: found.length }));
    } finally {
      addingScripture = false;
    }
  }

  function newPause(minutes: number) {
    pauseDraft = { id: crypto.randomUUID(), title: t("plans.pauseDefault"), minutes, seconds: 0, background: null, isNew: true };
    adding = null;
  }

  function editPause(item: Pause) {
    const total = Math.round(item.durationMs / 1000);
    pauseDraft = {
      id: item.id,
      title: item.title,
      minutes: Math.floor(total / 60),
      seconds: total % 60,
      background: item.background ?? null,
      isNew: false,
    };
    adding = null;
  }

  let draftMs = $derived(
    pauseDraft
      ? (Math.min(MAX_MINUTES, Math.max(0, Math.floor(pauseDraft.minutes || 0))) * 60 + Math.min(59, Math.max(0, Math.floor(pauseDraft.seconds || 0)))) * 1000
      : 0,
  );

  function savePause() {
    if (!plan || !pauseDraft || draftMs === 0) return;
    const item: PlanItem = {
      id: pauseDraft.id,
      kind: "pause",
      title: pauseDraft.title.trim(),
      durationMs: draftMs,
      background: pauseDraft.background,
    };
    const exists = plan.items.some((i) => i.id === item.id);
    setItems(exists ? plan.items.map((i) => (i.id === item.id ? item : i)) : [...plan.items, item]);
    if (!exists) notify(t("plans.addedPause", { name: item.title || t("plans.pause") }));
    pauseDraft = null;
  }

  async function pickPauseBackground() {
    try {
      const path = await platform.planPickBackground(t("background.dialogTitle"), t("background.dialogFilter"));
      if (path && pauseDraft) {
        missingFiles.delete(path);
        pauseDraft.background = path;
      }
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  // --- Lista de planes ---

  function newPlan() {
    createPlan(t("plans.untitled"), today());
    current = null;
  }

  function open(id: string) {
    choosePlan(id);
    current = null;
    adding = null;
    pauseDraft = null;
    confirmDelete = null;
  }

  function remove(id: string) {
    if (confirmDelete !== id) {
      confirmDelete = id;
      return;
    }
    deletePlan(id);
    confirmDelete = null;
  }

  function countLabel(p: Plan): string {
    return t("plans.itemCount", { count: p.items.filter((i) => i.kind !== "section").length });
  }
</script>

{#snippet planRow(p: Plan)}
  <li>
    <button class="item" onclick={() => open(p.id)}>
      <span class="name">
        {p.name}
        <span class="muted small sub">{formatDate(p.date)} · {countLabel(p)}</span>
      </span>
    </button>
    <button class="btn ghost small-btn" onclick={() => duplicatePlan(p, t("plans.copyName", { name: p.name }))}>{t("plans.duplicate")}</button>
    <button class="icon-btn remove" class:confirm={confirmDelete === p.id} onclick={() => remove(p.id)} aria-label={t("plans.delete", { name: p.name })} title={t("plans.delete", { name: p.name })}>
      {confirmDelete === p.id ? t("songs.confirmDelete") : "✕"}
    </button>
  </li>
{/snippet}

<div class="plan-panel">
  {#if !plan}
    <div class="actions">
      <button class="btn primary" onclick={newPlan}>{t("plans.new")}</button>
      <span class="muted small">{t("plans.intro")}</span>
    </div>
    {#if plans.all.length === 0}
      <p class="muted">{t("plans.empty")}</p>
    {:else}
      {#if upcoming.length > 0}
        <ul class="items">
          {#each upcoming as p (p.id)}{@render planRow(p)}{/each}
        </ul>
      {:else}
        <p class="muted">{t("plans.noUpcoming")}</p>
      {/if}
      {#if past.length > 0}
        <details bind:open={showPast}>
          <summary class="muted">{t("plans.past", { count: past.length })}</summary>
          <ul class="items">
            {#each past as p (p.id)}{@render planRow(p)}{/each}
          </ul>
        </details>
      {/if}
    {/if}
  {:else}
    <div class="head">
      <button class="btn ghost" onclick={() => choosePlan(null)}>← {t("plans.all")}</button>
      <button class="btn primary next" onclick={prepareNext} disabled={presentable.length === 0}>{t("plans.next")} ▶</button>
    </div>

    <div class="meta">
      <input
        class="field plan-name"
        value={plan.name}
        onchange={(e) => update({ name: e.currentTarget.value.trim() || t("plans.untitled") })}
        aria-label={t("plans.name")}
        maxlength="80"
      />
      <input
        class="field"
        type="date"
        value={plan.date ?? ""}
        onchange={(e) => update({ date: e.currentTarget.value || null })}
        aria-label={t("plans.date")}
      />
    </div>
    <details class="notes" open={plan.notes !== ""}>
      <summary class="muted small">{t("plans.notes")}</summary>
      <textarea
        class="field"
        rows="2"
        value={plan.notes}
        onchange={(e) => update({ notes: e.currentTarget.value })}
        placeholder={t("plans.notesPlaceholder")}
      ></textarea>
    </details>

    {#if plan.items.length === 0}
      <p class="muted">{t("plans.emptyPlan")}</p>
    {:else}
      <p class="muted small">{t("plans.hint")}</p>
      <ol class="items">
        {#each plan.items as item, i (item.id)}
          {@const issue = problem(item)}
          {@const done = (shown[plan.id] ?? []).includes(item.id)}
          {#if item.kind === "section"}
            <li class="section">
              {#if editingSection === item.id}
                <!-- svelte-ignore a11y_autofocus -->
                <input
                  class="field section-input"
                  value={item.title}
                  autofocus
                  onfocus={(e) => e.currentTarget.select()}
                  onblur={(e) => renameSection(item.id, e.currentTarget.value)}
                  onkeydown={(e) => {
                    if (e.key === "Enter") e.currentTarget.blur();
                    if (e.key === "Escape") editingSection = null;
                  }}
                  aria-label={t("plans.sectionName")}
                />
              {:else}
                <button class="section-title" onclick={() => (editingSection = item.id)} title={t("plans.renameSection")}>{item.title}</button>
              {/if}
              <button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label={t("songs.moveUp")}>▲</button>
              <button class="icon-btn" onclick={() => move(i, 1)} disabled={i === plan.items.length - 1} aria-label={t("songs.moveDown")}>▼</button>
              <button class="icon-btn remove" onclick={() => removeItem(item.id)} aria-label={t("plans.remove", { name: item.title })}>✕</button>
            </li>
          {:else}
            <li class:chosen={current?.id === item.id} class:broken={issue !== null}>
              <button
                class="item"
                onclick={() => prepareItem(item)}
                ondblclick={() => prepareItem(item, true)}
                disabled={issue !== null}
                title={item.kind === "media" ? item.path : undefined}
              >
                <span class="icon" aria-hidden="true">{icon(item)}</span>
                <span class="name">
                  {title(item)}
                  <span class="small sub" class:muted={!issue}>
                    {#if converting === item.id}{t("media.converting")}{:else}{issue ?? detail(item)}{/if}
                  </span>
                </span>
                {#if done}<span class="done" title={t("plans.shown")} aria-label={t("plans.shown")}>✓</span>{/if}
              </button>
              {#if item.kind === "pause"}
                <button class="icon-btn" onclick={() => editPause(item)} aria-label={t("plans.editPause")} title={t("plans.editPause")}>✎</button>
              {/if}
              <button class="icon-btn" onclick={() => move(i, -1)} disabled={i === 0} aria-label={t("songs.moveUp")}>▲</button>
              <button class="icon-btn" onclick={() => move(i, 1)} disabled={i === plan.items.length - 1} aria-label={t("songs.moveDown")}>▼</button>
              <button class="icon-btn remove" onclick={() => removeItem(item.id)} aria-label={t("plans.remove", { name: title(item) })}>✕</button>
            </li>
          {/if}
        {/each}
      </ol>
    {/if}

    {#if notice}<p class="notice small" role="status">✓ {notice}</p>{/if}

    {#if pauseDraft}
      <form class="box" onsubmit={(e) => { e.preventDefault(); savePause(); }}>
        <strong>{t(pauseDraft.isNew ? "plans.addPause" : "plans.editPause")}</strong>
        <label class="field-row">
          <span class="label">{t("clock.label")}</span>
          <input class="field" bind:value={pauseDraft.title} maxlength="60" placeholder={t("clock.labelPlaceholder")} />
        </label>
        <div class="row">
          <label class="field-row">
            <span class="label">{t("clock.min")}</span>
            <input class="field num-field" type="number" min="0" max={MAX_MINUTES} bind:value={pauseDraft.minutes} />
          </label>
          <span class="colon">:</span>
          <label class="field-row">
            <span class="label">{t("clock.sec")}</span>
            <input class="field num-field" type="number" min="0" max="59" bind:value={pauseDraft.seconds} />
          </label>
        </div>
        <div class="row">
          <span class="label">{t("songs.background")}</span>
          {#if pauseDraft.background}
            <span class="muted small bg-name" title={pauseDraft.background}>{baseName(pauseDraft.background)}</span>
            <button type="button" class="btn ghost" onclick={() => pauseDraft && (pauseDraft.background = null)}>{t("background.remove")}</button>
          {:else}
            <span class="muted small">{t("plans.pauseBackgroundDefault")}</span>
          {/if}
          <button type="button" class="btn" onclick={pickPauseBackground}>{t("background.pick")}</button>
        </div>
        <p class="muted small">{t("plans.pauseHint")}</p>
        <div class="row end">
          <button type="button" class="btn ghost" onclick={() => (pauseDraft = null)}>{t("songs.cancel")}</button>
          <button type="submit" class="btn primary" disabled={draftMs === 0}>{t(pauseDraft.isNew ? "plans.add" : "songs.save")}</button>
        </div>
      </form>
    {:else if adding === "scripture"}
      <form class="box" onsubmit={(e) => { e.preventDefault(); addScripture(); }}>
        <strong>{t("plans.addVerses")}</strong>
        <label class="field-row">
          <span class="label">{t("plans.versesLabel")}</span>
          <!-- svelte-ignore a11y_autofocus -->
          <textarea class="field" rows="3" bind:value={scriptureText} placeholder={t("plans.versesPlaceholder")} autofocus></textarea>
        </label>
        {#if scriptureLines.length > 0}
          <ul class="refs">
            {#each scriptureLines as line, i (i)}
              <li class:bad={!line.reference}>
                {#if line.reference}
                  ✓ {referenceLabel({ id: "", kind: "scripture", ...line.reference, bible: scriptureBible || null }, bibles.all.find((b) => b.id === scriptureBible) ?? currentBibles().bible)}
                {:else}
                  ✕ {line.text} <span class="small">— {t("plans.notUnderstood")}</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
        <label class="field-row">
          <span class="label">{t("plans.version")}</span>
          <select class="field" bind:value={scriptureBible}>
            <option value="">{t("plans.versionConsole", { version: currentBibles().bible?.abbreviation ?? "" })}</option>
            {#each visibleBibles() as b (b.id)}
              <option value={b.id}>{b.abbreviation} · {b.name}</option>
            {/each}
          </select>
        </label>
        <p class="muted small">{t("plans.versesHint")}</p>
        <div class="row end">
          <button type="button" class="btn ghost" onclick={() => (adding = null)}>{t("songs.cancel")}</button>
          <button type="submit" class="btn primary" disabled={scriptureReady === 0 || addingScripture}>{t("plans.addCount", { count: scriptureReady })}</button>
        </div>
      </form>
    {:else if adding === "song"}
      <div class="box">
        <div class="row">
          <!-- svelte-ignore a11y_autofocus -->
          <input class="field grow" bind:value={songFilter} placeholder={t("songs.search")} aria-label={t("songs.search")} autofocus />
          <button class="btn" onclick={() => (adding = null)}>{t("plans.done")}</button>
        </div>
        {#if library.songs.length === 0}
          <p class="muted small">{t("plans.noSongs")}</p>
        {:else}
          <ul class="items picker">
            {#each songMatches as song (song.id)}
              {@const count = plan.items.filter((i) => i.kind === "song" && i.songId === song.id).length}
              <li>
                <button class="item" onclick={() => addSong(song.id)}>
                  <span class="name">{song.title}<span class="muted small sub">{song.singer}</span></span>
                  {#if count > 0}<span class="done" title={t("plans.inPlan")}>✓</span>{/if}
                  <span class="plus" aria-hidden="true">+</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {:else}
      <div class="add">
        <span class="muted small">{t("plans.addLabel")}</span>
        <button class="btn" onclick={addSection}>+ {t("plans.section")}</button>
        <button class="btn" onclick={() => { adding = "song"; songFilter = ""; }}>+ {t("plans.song")}</button>
        <button class="btn" onclick={() => (adding = "scripture")}>+ {t("plans.verses")}</button>
        <button class="btn" onclick={addMedia} disabled={copying}>{copying ? t("plans.copying") : `+ ${t("plans.media")}`}</button>
        <div class="pause-add">
          <button class="btn" onclick={() => newPause(10)}>+ {t("plans.pause")}</button>
          {#each PAUSE_PRESETS as m (m)}
            <button class="chip" onclick={() => newPause(m)} title={t("plans.addPause")}>{t("clock.minutes", { n: m })}</button>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .plan-panel { display: flex; flex-direction: column; gap: 10px; }
  .notice {
    margin: 0;
    padding: 6px 10px;
    border-radius: 8px;
    color: var(--accent);
    background: var(--accent-soft);
    font-weight: 600;
  }
  .actions, .head, .add, .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .head { justify-content: space-between; }
  .meta { display: grid; grid-template-columns: 1fr auto; gap: 8px; }
  .plan-name { font-size: 18px; font-weight: 650; }
  .notes textarea { width: 100%; resize: vertical; margin-top: 6px; box-sizing: border-box; }
  summary { cursor: pointer; }
  .row.end { justify-content: flex-end; }
  .grow { flex: 1; min-width: 0; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
  .field-row { display: flex; flex-direction: column; gap: 4px; }
  .num-field { width: 6em; font-variant-numeric: tabular-nums; }
  .colon { font-size: 20px; font-weight: 700; align-self: flex-end; padding-bottom: 6px; }
  .bg-name { max-width: 14em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .box {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface-2);
  }
  .pause-add { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; }
  .refs { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; font-weight: 600; color: var(--accent); }
  .refs li { border: 0; display: block; }
  .refs li:hover { background: transparent; }
  .refs li.bad { color: var(--live); font-weight: 400; }
  .box textarea { resize: vertical; font-size: 15px; }
  .chip {
    min-height: 32px;
    padding: 0 10px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    cursor: pointer;
  }
  .chip:hover { border-color: var(--accent); }
  .chip:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  .items { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .picker { max-height: 320px; overflow-y: auto; }
  li { display: flex; align-items: center; gap: 2px; border: 1px solid transparent; border-radius: 8px; }
  li:hover { background: var(--surface-2); }
  li.chosen { border-color: var(--accent); background: var(--accent-soft); }
  li.broken .name { color: var(--muted); }
  li.broken .sub { color: var(--live); }
  li.section { margin-top: 8px; border-bottom: 1px solid var(--border); border-radius: 0; }
  li.section:hover { background: transparent; }
  .section-title {
    flex: 1;
    min-width: 0;
    padding: 6px 4px;
    font: inherit;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    text-align: left;
    color: var(--muted);
    background: transparent;
    border: 0;
    cursor: text;
  }
  .section-input { flex: 1; min-width: 0; }
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
  .item:disabled { cursor: default; }
  .icon { width: 1.4em; text-align: center; font-size: 17px; }
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; font-weight: 600; }
  .name, .sub { text-overflow: ellipsis; white-space: nowrap; }
  .sub { font-weight: 400; overflow: hidden; }
  .done { color: var(--accent); font-weight: 700; }
  .plus { color: var(--accent); font-weight: 700; font-size: 18px; }
  .small-btn { min-height: 34px; padding: 0 10px; font-size: 13px; }
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
