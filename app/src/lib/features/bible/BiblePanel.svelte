<script lang="ts">
  // Navegación de la Biblia en tres pasos: libro → capítulo → versículo.
  // El buscador es un atajo: «Juan 3:16», «Sal 23» o palabras.
  import { onMount, tick, untrack } from "svelte";
  import { bookName } from "$lib/features/bible/books";
  import { parseQuery } from "$lib/features/bible/reference";
  import { locales, t, type Locale } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/core/messages";
  import { bibles, currentBibles, importBibles as importFromDisk, loadBibles, visibleBibles } from "$lib/features/bible/bibles.svelte";
  import { bookLanguage, buildScripture } from "$lib/features/bible/scripture";
  import { platform, type BibleInfo, type OutputContent, type SearchHit, type Verse } from "$lib/platform";
  import { setSetting, settings } from "$lib/core/settings.svelte";

  let {
    prepared,
    live,
    onprepare,
    onsend,
    onerror,
  }: {
    prepared: OutputContent;
    live: OutputContent;
    onprepare: (content: OutputContent) => void;
    /** Doble clic en un versículo: prepararlo y mostrarlo en pantalla en un solo gesto. */
    onsend: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  const FIRST_NEW_TESTAMENT_BOOK = 40;

  type View =
    | { step: "books" }
    | { step: "chapters"; book: number }
    | {
        step: "verses";
        book: number;
        chapter: number;
        verses: Verse[];
        /** El mismo capítulo en la segunda versión (null si no hay o no lo tiene). */
        second: Verse[] | null;
        range: [number, number] | null;
      }
    | { step: "search"; query: string; hits: SearchHit[] }
    | { step: "notFound"; reference: string };

  /** Las versiones que se pueden elegir (sin las ocultas en Ajustes). */
  let available = $derived(visibleBibles());
  // La versión elegida se guarda en los ajustes; si ya no existe, se usa la primera.
  let current = $derived(currentBibles());
  let bibleId = $derived(current.bible?.id ?? null);
  let query = $state("");
  let view = $state<View>({ step: "books" });
  let input = $state<HTMLInputElement>();
  let verseList = $state<HTMLOListElement>();

  let bible = $derived(current.bible);
  /** Segunda versión que se muestra debajo de la primera; null = solo una. */
  let secondBible = $derived(current.second);
  // Los nombres de libro siguen el idioma de la Biblia, no el de la interfaz.
  let language = $derived(bookLanguage(bible));
  let parsed = $derived(parseQuery(query, language));
  let suggestions = $derived(query.trim() && !parsed.reference ? parsed.books.slice(0, 6) : []);
  let preparedRef = $derived(prepared.kind === "scripture" ? prepared.verses[prepared.index]?.reference : null);
  let liveRef = $derived(live.kind === "scripture" ? live.verses[live.index]?.reference : null);
  let groups = $derived(groupByLanguage(available));
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

  /** Los botones de versión funcionan como interruptores: una marcada = una versión; dos = vista doble.
   *  La primera marcada va arriba y la segunda abajo. Siempre queda al menos una. */
  function toggleVersion(id: string) {
    if (id === bibleId) {
      // Se desmarca la de arriba: la de abajo sube.
      if (secondBible) {
        setSetting("bible", secondBible.id);
        setSetting("bible2", null);
      }
    } else if (id === secondBible?.id) {
      setSetting("bible2", null);
    } else if (bibleId === null) {
      setSetting("bible", id);
    } else {
      // Con dos ya marcadas, la nueva sustituye a la de abajo.
      if (settings.bible !== bibleId) setSetting("bible", bibleId);
      setSetting("bible2", id);
    }
  }

  function versionTooltip(b: BibleInfo): string {
    return `${b.name}\n${b.copyright || t("bible.noCopyright")}`;
  }

  // Cambie desde aquí o desde Ajustes (también al restablecer), lo que se está viendo se recarga.
  $effect(() => {
    void [bible?.id, secondBible?.id];
    untrack(reload);
  });

  // Al cambiar de versión se recarga lo que se estaba viendo, para no mezclar textos de dos versiones.
  // Si había un versículo de este capítulo preparado, se vuelve a preparar con las versiones nuevas.
  function reload() {
    if (view.step === "verses") openChapter(view.book, view.chapter, view.range, preparedVerseHere());
    else if (view.step === "search") search(view.query);
  }

  /** Número del versículo preparado si es de este capítulo. La referencia cambia de idioma con la versión,
   *  así que se compara el «capítulo:versículo» del final. */
  function preparedVerseHere(): number | undefined {
    if (view.step !== "verses" || prepared.kind !== "scripture") return undefined;
    const match = prepared.verses[prepared.index]?.reference.match(/(\d+):(\d+)$/);
    if (!match || Number(match[1]) !== view.chapter) return undefined;
    const n = Number(match[2]);
    return view.verses.some((v) => v.number === n) ? n : undefined;
  }

  async function importBibles() {
    try {
      const imported = await importFromDisk(t("bible.dialogTitle"), t("bible.dialogFilter"));
      if (imported.length > 0) setSetting("bible", imported[0].id);
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  function refText(book: number, chapter: number, verse?: number) {
    return `${bookName(book, language)} ${chapter}${verse ? `:${verse}` : ""}`;
  }

  function scripture(book: number, chapter: number, verses: Verse[], start: number, second: Verse[] | null): OutputContent | null {
    return bible ? buildScripture(bible, secondBible, book, chapter, verses, start, second) : null;
  }

  function prepareFrom(book: number, chapter: number, verses: Verse[], start: number, second: Verse[] | null) {
    const content = scripture(book, chapter, verses, start, second);
    if (content) onprepare(content);
  }

  /** El capítulo en la segunda versión; null si no hay segunda o ella no lo tiene. */
  async function secondChapter(book: number, chapter: number): Promise<Verse[] | null> {
    if (!secondBible) return null;
    return platform.getChapter(secondBible.id, book, chapter).catch(() => null);
  }

  async function scrollToVerse(n: number) {
    await tick();
    verseList?.querySelector<HTMLElement>(`[data-verse="${n}"]`)?.scrollIntoView({ block: "nearest" });
  }

  async function openChapter(book: number, chapter: number, range: [number, number] | null, prepareVerse?: number) {
    if (!bible) return;
    try {
      const [verses, second] = await Promise.all([platform.getChapter(bible.id, book, chapter), secondChapter(book, chapter)]);
      view = { step: "verses", book, chapter, verses, second, range };
      if (range) {
        const inRange = verses.filter((v) => v.number >= range[0] && v.number <= range[1]);
        if (inRange.length === 0) {
          view = { step: "notFound", reference: `${refText(book, chapter)}:${range[0]}` };
          return;
        }
        const start = prepareVerse && inRange.some((v) => v.number === prepareVerse) ? prepareVerse : inRange[0].number;
        prepareFrom(book, chapter, inRange, start, second);
        scrollToVerse(start);
      } else if (prepareVerse) {
        prepareFrom(book, chapter, verses, prepareVerse, second);
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

  /** La serie que recorre un versículo: dentro del rango buscado, solo el rango; fuera, el capítulo entero. */
  function verseContent(verse: Verse): OutputContent | null {
    if (view.step !== "verses") return null;
    const { book, chapter, verses, second, range } = view;
    if (range && verse.number >= range[0] && verse.number <= range[1]) {
      return scripture(book, chapter, verses.filter((v) => v.number >= range[0] && v.number <= range[1]), verse.number, second);
    }
    view = { ...view, range: null };
    return scripture(book, chapter, verses, verse.number, second);
  }

  // Sin desplazar la lista: si se moviera entre los dos clics, el doble clic caería en otro versículo.
  function clickVerse(verse: Verse) {
    const content = verseContent(verse);
    if (content) onprepare(content);
  }

  function doubleClickVerse(verse: Verse) {
    const content = verseContent(verse);
    if (content) onsend(content);
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
    loadBibles().catch((e) => onerror(toMessage(e)));
  });
</script>

<div class="bible">
  {#if bibles.loading}
    <p class="muted">{t("bible.loading")}</p>
  {:else if available.length === 0}
    <div class="empty">
      <p>{t("bible.empty")}</p>
      <button class="btn primary" onclick={importBibles}>{t("bible.importFirst")}</button>
    </div>
  {:else}
    <div class="top">
      <div class="versions" role="group" aria-label={t("bible.version")}>
        {#each groups as group (group.label)}
          {#if groups.length > 1}<span class="lang">{group.label}</span>{/if}
          {#each group.items as b (b.id)}
            {@const order = b.id === bibleId ? 1 : b.id === secondBible?.id ? 2 : 0}
            <button
              class="version-chip"
              class:no-copyright={!b.copyright}
              aria-pressed={order > 0}
              title={versionTooltip(b)}
              onclick={() => toggleVersion(b.id)}
            >
              {#if order}<span class="order" aria-label={t(order === 1 ? "bible.top" : "bible.bottom")}>{order}</span>{/if}
              {b.abbreviation}
              {#if !b.copyright}<span class="flag" aria-hidden="true">!</span>{/if}
            </button>
          {/each}
        {/each}
        <button class="version-chip add" onclick={importBibles} title={t("bible.import")} aria-label={t("bible.import")}>+</button>
      </div>
      <p class="muted small">{t(secondBible ? "bible.dualOn" : "bible.versionsHint")}</p>
      {#each [bible, secondBible].filter((b) => b && !b.copyright) as b (b!.id)}
        <p class="no-copyright-note">{b!.abbreviation}: {t("bible.noCopyright")}</p>
      {/each}

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
          <h3>
            {refText(view.book, view.chapter)}
            {#if secondBible}<span class="dual" title={t("bible.dualHint")}>{bible?.abbreviation} + {secondBible.abbreviation}</span>{/if}
          </h3>
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
              ondblclick={() => doubleClickVerse(verse)}
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
                ondblclick={() => doubleClickVerse(verse)}
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
  .search { display: flex; gap: 8px; align-items: flex-end; }
  .search .field { min-height: 44px; font-size: 16px; }
  .versions { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
  .lang { width: 100%; font-size: 12px; font-weight: 650; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; }
  .lang:not(:first-child) { margin-top: 4px; }
  .version-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 38px;
    padding: 0 12px;
    font: inherit;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    cursor: pointer;
  }
  .version-chip:hover { border-color: var(--accent); }
  .version-chip[aria-pressed="true"] { background: var(--accent); border-color: var(--accent); color: var(--accent-text); }
  .version-chip.add { min-width: 38px; justify-content: center; padding: 0; font-size: 18px; font-weight: 500; color: var(--muted); }
  .order {
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    font-size: 11px;
    border-radius: 50%;
    background: var(--accent-text);
    color: var(--accent);
  }
  .flag { color: var(--live); }
  .no-copyright-note { font-size: 12.5px; padding: 6px 10px; border-left: 3px solid var(--live); background: var(--surface-2); border-radius: 0 6px 6px 0; }
  .version-chip[aria-pressed="true"] .flag { color: inherit; }
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
  .chapter-head h3 { font-size: 18px; display: flex; align-items: center; gap: 8px; }
  .dual {
    font-size: 12px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
  }
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
