<script lang="ts">
  // Navegación de la Biblia en tres pasos: libro → capítulo → versículo.
  // El buscador es un atajo: «Juan 3:16», «Sal 23» o palabras.
  import { onMount, tick } from "svelte";
  import { bookName, type BookLanguage } from "$lib/bible/books";
  import { parseQuery } from "$lib/bible/reference";
  import { i18n, locales, t, type Locale } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { platform, type BibleInfo, type OutputContent, type SearchHit, type Verse } from "$lib/platform";

  let {
    prepared,
    live,
    onprepare,
    onerror,
  }: {
    prepared: OutputContent;
    live: OutputContent;
    onprepare: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  const STORAGE_KEY = "selah.bible";
  const FIRST_NEW_TESTAMENT_BOOK = 40;

  type View =
    | { step: "books" }
    | { step: "chapters"; book: number }
    | { step: "verses"; book: number; chapter: number; verses: Verse[]; range: [number, number] | null }
    | { step: "search"; query: string; hits: SearchHit[] }
    | { step: "notFound"; reference: string };

  let bibles = $state<BibleInfo[]>([]);
  let loading = $state(true);
  let bibleId = $state<string | null>(null);
  let query = $state("");
  let view = $state<View>({ step: "books" });
  let input = $state<HTMLInputElement>();
  let verseList = $state<HTMLOListElement>();

  let bible = $derived(bibles.find((b) => b.id === bibleId) ?? null);
  // Los nombres de libro siguen el idioma de la Biblia, no el de la interfaz.
  let language = $derived<BookLanguage>(
    bible?.language && bible.language in locales ? (bible.language as BookLanguage) : i18n.locale,
  );
  let parsed = $derived(parseQuery(query, language));
  let suggestions = $derived(query.trim() && !parsed.reference ? parsed.books.slice(0, 6) : []);
  let preparedRef = $derived(prepared.kind === "scripture" ? prepared.verses[prepared.index]?.reference : null);
  let liveRef = $derived(live.kind === "scripture" ? live.verses[live.index]?.reference : null);
  let groups = $derived(groupByLanguage(bibles));
  let oldTestament = $derived(bible?.books.filter((b) => b.number < FIRST_NEW_TESTAMENT_BOOK) ?? []);
  let newTestament = $derived(bible?.books.filter((b) => b.number >= FIRST_NEW_TESTAMENT_BOOK) ?? []);

  function groupByLanguage(list: BibleInfo[]) {
    const map = new Map<string, BibleInfo[]>();
    for (const b of list) {
      const key = b.language ?? "";
      map.set(key, [...(map.get(key) ?? []), b]);
    }
    return [...map.entries()].map(([code, items]) => ({
      label: code in locales ? locales[code as Locale].name : t("bible.otherLanguage"),
      items,
    }));
  }

  function remember(id: string) {
    try {
      localStorage.setItem(STORAGE_KEY, id);
    } catch {
      // Sin almacenamiento: se elige la primera la próxima vez.
    }
  }

  function selectBible(id: string) {
    bibleId = id;
    remember(id);
    // Al cambiar de versión se recarga lo que se estaba viendo, para no mezclar textos de dos versiones.
    if (view.step === "verses") openChapter(view.book, view.chapter, view.range);
    else if (view.step === "search") search(view.query);
  }

  async function loadBibles(preferred?: string) {
    try {
      bibles = await platform.listBibles();
    } catch (e) {
      onerror(toMessage(e));
    }
    loading = false;
    let saved: string | null = null;
    try {
      saved = localStorage.getItem(STORAGE_KEY);
    } catch {
      // Ignorado.
    }
    const pick = [preferred, bibleId, saved].find((id) => id && bibles.some((b) => b.id === id));
    bibleId = pick ?? bibles[0]?.id ?? null;
  }

  async function importBibles() {
    try {
      const imported = await platform.importBibles(t("bible.dialogTitle"), t("bible.dialogFilter"));
      if (imported.length > 0) {
        await loadBibles(imported[0].id);
        remember(imported[0].id);
      }
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  function refText(book: number, chapter: number, verse?: number) {
    return `${bookName(book, language)} ${chapter}${verse ? `:${verse}` : ""}`;
  }

  function prepareFrom(book: number, chapter: number, verses: Verse[], start: number) {
    if (!bible) return;
    const index = verses.findIndex((v) => v.number === start);
    onprepare({
      kind: "scripture",
      verses: verses.map((v) => ({ reference: refText(book, chapter, v.number), text: v.text })),
      index: Math.max(index, 0),
      version: bible.abbreviation,
      copyright: bible.copyright,
    });
  }

  async function scrollToVerse(n: number) {
    await tick();
    verseList?.querySelector<HTMLElement>(`[data-verse="${n}"]`)?.scrollIntoView({ block: "nearest" });
  }

  async function openChapter(book: number, chapter: number, range: [number, number] | null, prepareVerse?: number) {
    if (!bible) return;
    try {
      const verses = await platform.getChapter(bible.id, book, chapter);
      view = { step: "verses", book, chapter, verses, range };
      if (range) {
        const inRange = verses.filter((v) => v.number >= range[0] && v.number <= range[1]);
        if (inRange.length === 0) {
          view = { step: "notFound", reference: `${refText(book, chapter)}:${range[0]}` };
          return;
        }
        prepareFrom(book, chapter, inRange, inRange[0].number);
        scrollToVerse(inRange[0].number);
      } else if (prepareVerse) {
        prepareFrom(book, chapter, verses, prepareVerse);
        scrollToVerse(prepareVerse);
      }
    } catch (e) {
      // Un capítulo que no existe en esta versión llega como bibleNotFound.
      if ((e as { code?: string })?.code === "bibleNotFound") {
        view = { step: "notFound", reference: refText(book, chapter) };
      } else {
        onerror(toMessage(e));
      }
    }
  }

  function openBook(book: number) {
    const chapters = bible?.books.find((b) => b.number === book)?.chapters ?? 0;
    // Un libro de un solo capítulo (Judas, Filemón…) pasa directo a los versículos.
    if (chapters === 1) openChapter(book, 1, null);
    else view = { step: "chapters", book };
  }

  async function search(text: string) {
    if (!bible) return;
    try {
      view = { step: "search", query: text, hits: await platform.searchBible(bible.id, text) };
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const ref = parsed.reference;
    if (ref) {
      openChapter(ref.book, ref.chapter, ref.verseStart ? [ref.verseStart, ref.verseEnd ?? ref.verseStart] : null);
    } else if (parsed.books.length > 0) {
      // Solo un libro escrito: se abre para elegir capítulo.
      openBook(parsed.books[0]);
    } else if (query.trim()) {
      search(query.trim());
    }
  }

  async function pickBook(book: number) {
    query = "";
    openBook(book);
    await tick();
  }

  function clickVerse(verse: Verse) {
    if (view.step !== "verses") return;
    const { book, chapter, verses, range } = view;
    // Dentro del rango buscado se recorre solo el rango; fuera de él, el capítulo entero.
    if (range && verse.number >= range[0] && verse.number <= range[1]) {
      prepareFrom(book, chapter, verses.filter((v) => v.number >= range[0] && v.number <= range[1]), verse.number);
    } else {
      view = { ...view, range: null };
      prepareFrom(book, chapter, verses, verse.number);
    }
    scrollToVerse(verse.number);
  }

  function adjacentChapter(delta: 1 | -1): [number, number] | null {
    const current = view;
    if (current.step !== "verses" || !bible) return null;
    const books = bible.books;
    const i = books.findIndex((b) => b.number === current.book);
    if (i < 0) return null;
    const chapter = current.chapter + delta;
    if (chapter >= 1 && chapter <= books[i].chapters) return [current.book, chapter];
    const next = books[i + delta];
    if (!next) return null;
    return [next.number, delta === 1 ? 1 : next.chapters];
  }

  function goChapter(delta: 1 | -1) {
    const target = adjacentChapter(delta);
    if (target) openChapter(target[0], target[1], null);
  }

  function chapterCount(book: number) {
    return bible?.books.find((b) => b.number === book)?.chapters ?? 0;
  }

  onMount(() => {
    loadBibles();
  });
</script>

<div class="bible">
  {#if loading}
    <p class="muted">{t("bible.loading")}</p>
  {:else if bibles.length === 0}
    <div class="empty">
      <p>{t("bible.empty")}</p>
      <button class="btn primary" onclick={importBibles}>{t("bible.importFirst")}</button>
    </div>
  {:else}
    <div class="top">
      <div class="version">
        <label class="grow">
          <span class="label">{t("bible.version")}</span>
          <select class="field" value={bibleId} onchange={(e) => selectBible(e.currentTarget.value)}>
            {#each groups as group (group.label)}
              <optgroup label={group.label}>
                {#each group.items as b (b.id)}
                  <option value={b.id}>{b.abbreviation} — {b.name}</option>
                {/each}
              </optgroup>
            {/each}
          </select>
        </label>
        <button class="btn" onclick={importBibles}>{t("bible.import")}</button>
      </div>

      {#if bible}
        <p class="copyright" class:missing={!bible.copyright}>
          {bible.copyright ? bible.copyright : t("bible.noCopyright")}
        </p>
      {/if}

      <form class="search" onsubmit={submit}>
        <input
          class="field"
          bind:this={input}
          bind:value={query}
          placeholder={t("bible.searchPlaceholder")}
          aria-label={t("bible.searchLabel")}
          autocomplete="off"
          spellcheck="false"
        />
        <button class="btn primary" type="submit" disabled={!query.trim()}>{t("bible.search")}</button>
      </form>

      {#if suggestions.length > 0}
        <div class="suggestions" aria-label={t("bible.whichBook")}>
          {#each suggestions as book (book)}
            <button class="chip" onclick={() => pickBook(book)}>{bookName(book, language)}</button>
          {/each}
        </div>
      {/if}

      <nav class="crumbs" aria-label={t("bible.books")}>
        <button class="crumb" class:current={view.step === "books"} onclick={() => (view = { step: "books" })}>
          {t("bible.books")}
        </button>
        {#if view.step === "chapters" || view.step === "verses"}
          <span aria-hidden="true">›</span>
          <button
            class="crumb"
            class:current={view.step === "chapters"}
            onclick={() => view.step === "verses" && chapterCount(view.book) > 1 && (view = { step: "chapters", book: view.book })}
          >
            {bookName(view.book, language)}
          </button>
        {/if}
        {#if view.step === "verses"}
          <span aria-hidden="true">›</span>
          <span class="crumb current">{t("bible.chapterShort", { n: view.chapter })}</span>
        {/if}
      </nav>
    </div>

    <div class="results">
      {#if view.step === "books"}
        {#each [{ title: "bible.oldTestament", books: oldTestament }, { title: "bible.newTestament", books: newTestament }] as const as section (section.title)}
          {#if section.books.length > 0}
            <h3 class="section">{t(section.title)}</h3>
            <div class="grid books">
              {#each section.books as b (b.number)}
                <button class="cell book" onclick={() => openBook(b.number)}>{bookName(b.number, language)}</button>
              {/each}
            </div>
          {/if}
        {/each}
      {:else if view.step === "chapters"}
        <h3 class="section">{bookName(view.book, language)} · {t("bible.chooseChapter")}</h3>
        <div class="grid numbers">
          {#each { length: chapterCount(view.book) } as _, i (i)}
            <button class="cell number" onclick={() => view.step === "chapters" && openChapter(view.book, i + 1, null)}>{i + 1}</button>
          {/each}
        </div>
      {:else if view.step === "verses"}
        <div class="chapter-head">
          <h3>{refText(view.book, view.chapter)}</h3>
          <div class="nav">
            <button class="btn ghost" onclick={() => goChapter(-1)} disabled={!adjacentChapter(-1)}>{t("bible.prevChapter")}</button>
            <button class="btn ghost" onclick={() => goChapter(1)} disabled={!adjacentChapter(1)}>{t("bible.nextChapter")}</button>
          </div>
        </div>
        <p class="muted small">{t("bible.chooseVerse")} — {t("bible.hint")}</p>
        <div class="grid numbers compact">
          {#each view.verses as verse (verse.number)}
            {@const ref = refText(view.book, view.chapter, verse.number)}
            <button
              class="cell number"
              class:prepared={ref === preparedRef}
              class:live={ref === liveRef}
              onclick={() => clickVerse(verse)}
            >{verse.number}</button>
          {/each}
        </div>
        <ol class="verses" bind:this={verseList}>
          {#each view.verses as verse (verse.number)}
            {@const ref = refText(view.book, view.chapter, verse.number)}
            <li data-verse={verse.number}>
              <button
                class="verse"
                class:in-range={view.range !== null && verse.number >= view.range[0] && verse.number <= view.range[1]}
                class:prepared={ref === preparedRef}
                class:live={ref === liveRef}
                onclick={() => clickVerse(verse)}
              >
                <span class="num">{verse.number}</span>
                <span class="vtext">{verse.text}</span>
                {#if ref === liveRef}<span class="tag live">{t("live.badge")}</span>{/if}
              </button>
            </li>
          {/each}
        </ol>
      {:else if view.step === "search"}
        <p class="muted small">
          {#if view.hits.length === 0}
            {t("bible.noResults", { query: view.query })}
          {:else}
            {t(view.hits.length >= 200 ? "bible.resultsMax" : "bible.results", { count: view.hits.length, query: view.query })}
          {/if}
        </p>
        <ol class="verses">
          {#each view.hits as hit (`${hit.book}-${hit.chapter}-${hit.verse}`)}
            <li>
              <button class="verse hit" onclick={() => openChapter(hit.book, hit.chapter, null, hit.verse)}>
                <span class="ref">{refText(hit.book, hit.chapter, hit.verse)}</span>
                <span class="vtext">{hit.text}</span>
              </button>
            </li>
          {/each}
        </ol>
      {:else if view.step === "notFound"}
        <p class="muted">{t("bible.notFound", { reference: view.reference })}</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .bible { display: flex; flex-direction: column; gap: 12px; min-height: 0; height: 100%; }
  .top { display: flex; flex-direction: column; gap: 10px; }
  .empty { display: flex; flex-direction: column; gap: 12px; align-items: flex-start; padding: 8px 0; }
  .version, .search { display: flex; gap: 8px; align-items: flex-end; }
  .search .field { min-height: 44px; font-size: 16px; }
  .grow { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
  .copyright {
    font-size: 12.5px;
    line-height: 1.4;
    color: var(--muted);
    padding: 8px 10px;
    border-left: 3px solid var(--accent);
    background: var(--surface-2);
    border-radius: 0 6px 6px 0;
  }
  .copyright.missing { border-left-color: var(--live); color: var(--text); }
  .suggestions { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip {
    min-height: 32px;
    padding: 0 12px;
    font: inherit;
    font-size: 14px;
    color: var(--text);
    background: var(--accent-soft);
    border: 1px solid transparent;
    border-radius: 999px;
    cursor: pointer;
  }
  .chip:hover { border-color: var(--accent); }

  .crumbs { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; color: var(--muted); }
  .crumb {
    padding: 4px 8px;
    font: inherit;
    font-weight: 600;
    color: var(--accent);
    background: none;
    border: 0;
    border-radius: 6px;
    cursor: pointer;
  }
  .crumb:hover { background: var(--surface-2); }
  .crumb.current { color: var(--text); cursor: default; }

  .results { display: flex; flex-direction: column; gap: 10px; min-height: 0; overflow: auto; flex: 1; padding-right: 2px; }
  .section { font-size: 14px; font-weight: 650; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; }
  .grid { display: grid; gap: 6px; }
  /* Las rejillas ganan columnas solas al ampliar el panel. */
  .grid.books { grid-template-columns: repeat(auto-fill, minmax(128px, 1fr)); }
  .grid.numbers { grid-template-columns: repeat(auto-fill, minmax(52px, 1fr)); }
  .grid.numbers.compact { grid-template-columns: repeat(auto-fill, minmax(44px, 1fr)); }
  .cell {
    min-height: 44px;
    padding: 6px 8px;
    font: inherit;
    font-weight: 550;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cell:hover { border-color: var(--accent); background: var(--accent-soft); }
  .cell.book { text-align: left; }
  .cell.number { font-variant-numeric: tabular-nums; font-size: 16px; }
  .compact .cell.number { min-height: 38px; font-size: 15px; }
  .cell.prepared { border-color: var(--accent); background: var(--accent-soft); box-shadow: inset 0 0 0 1px var(--accent); }
  .cell.live { border-color: var(--live); background: var(--live-soft); box-shadow: inset 0 0 0 1px var(--live); }

  .chapter-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex-wrap: wrap; }
  .chapter-head h3 { font-size: 18px; }
  .nav { display: flex; gap: 4px; }
  .verses { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .verse {
    width: 100%;
    display: flex;
    gap: 10px;
    align-items: baseline;
    padding: 8px 10px;
    font: inherit;
    font-size: 15.5px;
    line-height: 1.5;
    text-align: left;
    color: var(--text);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
  }
  .verse:hover { background: var(--surface-2); }
  .verse.in-range { background: var(--surface-2); }
  .verse.prepared { border-color: var(--accent); background: var(--accent-soft); }
  .verse.live { border-color: var(--live); }
  .num { min-width: 1.8em; font-weight: 700; color: var(--accent); font-variant-numeric: tabular-nums; }
  .ref { min-width: 7.5em; font-weight: 650; color: var(--accent); }
  .vtext { flex: 1; }
  .tag.live {
    align-self: center;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--live);
    color: #fff;
  }
</style>
