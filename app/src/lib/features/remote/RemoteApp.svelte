<script lang="ts">
  // Página del control remoto, en el navegador del teléfono. No usa Tauri: habla con el servidor de Selah
  // por WebSocket (órdenes y estado) y por /api/chapter (versículos). La consola aplica cada orden.
  import { onMount, tick, untrack } from "svelte";
  import { bookName, type BookLanguage } from "$lib/features/bible/books";
  import { parseQuery } from "$lib/features/bible/reference";
  import { i18n, t, type MessageKey } from "$lib/i18n/index.svelte";
  import type { Verse } from "$lib/platform";
  import type { RemoteCommand, RemoteStatus, ServerMessage } from "./protocol";
  import BoardPad from "./BoardPad.svelte";
  import { applyMessage, applyOp, emptyBoard, type Board, type BoardOp } from "./board";

  const KEY_STORAGE = "selah-remote-key";
  const RETRY_MS = [1000, 2000, 3000, 5000];
  const FIRST_NEW_TESTAMENT_BOOK = 40;
  const UNDO_MS = 6000;
  const THUMB_RETRIES = 4;
  /** Versículos a cada lado del que está en pantalla con la lista plegada. */
  const AROUND = 2;

  type Connection = "connecting" | "open" | "offline" | "rejected" | "noKey";
  type Step =
    | { kind: "books" }
    | { kind: "chapters"; book: number }
    | { kind: "verses"; bible: string; book: number; chapter: number; verses: Verse[] };

  const key = readKey();
  let connection = $state<Connection>(key ? "connecting" : "noKey");
  let status = $state<RemoteStatus | null>(null);
  let step = $state<Step>({ kind: "books" });
  let query = $state("");
  let problem = $state<string | null>(null);
  /** Lo que espera el «¿Seguro?»: un versículo o algo de «Para mostrar». */
  let asking = $state<{ title: string; text: string; command: RemoteCommand } | null>(null);
  let verseList = $state<HTMLOListElement>();
  /** El capítulo entero a la vista; plegada, solo lo que rodea al versículo en pantalla. */
  let expanded = $state(false);
  /** Cada capítulo pedido lleva su número: si llega tarde (ya se pidió otro), no se muestra. */
  let chapterRequest = 0;
  /** El capítulo pedido que aún no llegó: cuenta como abierto (si no, otro estado lo pediría otra vez). */
  let loadingChapter: { bible: string; book: number; chapter: number } | null = null;
  /** El capítulo en pantalla que la lista ya siguió (Biblia|libro|capítulo). */
  let followed: string | null = null;
  let stripList = $state<HTMLOListElement>();
  /** Tras saltar desde la tira: para volver a donde se estaba con un toque (deshacer en vez de preguntar). */
  let undo = $state<{ from: number; to: number; strip: string } | null>(null);
  let undoTimer: ReturnType<typeof setTimeout> | undefined;
  /** Reintentos de cada miniatura que aún no estaba lista (la consola las dibuja poco a poco). */
  let thumbTries = $state<Record<number, number>>({});
  /** La pizarra (D-021). Los trazos no son reactivos (miles de números): `boardVersion` avisa de cada cambio. */
  let board = $state.raw<Board>(emptyBoard());
  let boardVersion = $state(0);
  let drawing = $state(false);
  /** Trazos dibujados aquí: ya están completos en este teléfono, su eco del servidor llegaría con retraso. */
  const ownStrokes = new Set<string>();

  /** Abierta como app instalada (pantalla de inicio): ya ocupa toda la pantalla. */
  const installed = matchMedia("(display-mode: fullscreen), (display-mode: standalone)").matches || (navigator as { standalone?: boolean }).standalone === true;
  /** Android permite pantalla completa desde el navegador; el iPhone no (allí se usa «Agregar a inicio»). */
  const canFullscreen = !installed && document.fullscreenEnabled;
  let fullscreen = $state(false);

  let socket: WebSocket | null = null;
  let retries = 0;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;

  let bible = $derived(status?.bible ?? null);
  let language = $derived<BookLanguage>(bible?.language && ["es", "en", "it"].includes(bible.language) ? (bible.language as BookLanguage) : i18n.locale);
  let position = $derived(status?.position ?? null);
  let canPrevious = $derived(position !== null && position.n > 1);
  let canNext = $derived(position !== null && (position.total === null || position.n < position.total));
  let strip = $derived(status?.strip ?? null);
  /** Identifica lo que se puede elegir: si cambia (otro PDF, otra letra), lo de «Volver» ya no vale. */
  let stripId = $derived(strip ? (strip.kind === "pages" ? strip.doc : strip.labels.join("\u0001")) : "");
  let oldTestament = $derived(bible?.books.filter((b) => b.number < FIRST_NEW_TESTAMENT_BOOK) ?? []);
  let newTestament = $derived(bible?.books.filter((b) => b.number >= FIRST_NEW_TESTAMENT_BOOK) ?? []);

  /** La clave llega en el QR (?key=…) y se recuerda, para volver a abrir la página sin escanear. */
  function readKey(): string | null {
    const fromUrl = new URLSearchParams(location.search).get("key");
    try {
      if (fromUrl) localStorage.setItem(KEY_STORAGE, fromUrl);
      return fromUrl ?? localStorage.getItem(KEY_STORAGE);
    } catch {
      return fromUrl;
    }
  }

  function toggleFullscreen() {
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
    else document.documentElement.requestFullscreen({ navigationUI: "hide" }).catch(() => {});
  }

  function connect() {
    if (!key) return;
    clearTimeout(retryTimer);
    socket?.close();
    let opened = false;
    const ws = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws?key=${encodeURIComponent(key)}`);
    socket = ws;
    if (connection !== "open") connection = "connecting";
    ws.onopen = () => {
      opened = true;
      retries = 0;
      connection = "open";
    };
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as ServerMessage;
        if (message.type === "status") {
          status = message.status;
        } else if (!(message.type === "op" && message.op.type === "draw" && ownStrokes.has(message.op.stroke))) {
          board = applyMessage(board, message);
          boardVersion++;
        }
      } catch {
        // Un mensaje que no se entiende (versión más nueva de Selah) se ignora.
      }
    };
    ws.onclose = async () => {
      if (socket !== ws) return;
      socket = null;
      // Si nunca llegó a abrir, puede que la clave ya no valga: el servidor lo dice con un 401.
      if (!opened && (await keyRejected())) {
        connection = "rejected";
        return;
      }
      connection = "offline";
      retryTimer = setTimeout(connect, RETRY_MS[Math.min(retries++, RETRY_MS.length - 1)]);
    };
  }

  async function keyRejected(): Promise<boolean> {
    try {
      const response = await fetch(`/api/chapter?key=${encodeURIComponent(key ?? "")}&bible=-&book=1&chapter=1`);
      return response.status === 401;
    } catch {
      return false;
    }
  }

  function command(c: RemoteCommand) {
    if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(c));
    if (c.type !== "goTo") clearUndo();
  }

  /** Un cambio en la pizarra: se ve aquí enseguida y se manda al servidor, que lo reparte. */
  function act(op: BoardOp) {
    if (socket?.readyState !== WebSocket.OPEN) return;
    if (op.type === "draw") ownStrokes.add(op.stroke);
    applyOp(board, op);
    boardVersion++;
    socket.send(JSON.stringify(op));
  }

  function sheetNumber(id: string): number {
    void boardVersion;
    return board.sheets.findIndex((s) => s.id === id) + 1 || 1;
  }

  function clearUndo() {
    clearTimeout(undoTimer);
    undo = null;
  }

  /** Saltar desde la tira: va directo (como Anterior/Siguiente) y deja unos segundos para volver. */
  function jump(n: number) {
    if (!position || n === position.n) return;
    const from = undo?.strip === stripId ? undo.from : position.n;
    command({ type: "goTo", n });
    clearTimeout(undoTimer);
    undo = from === n ? null : { from, to: n, strip: stripId };
    undoTimer = setTimeout(clearUndo, UNDO_MS);
  }

  function goBack() {
    if (!undo) return;
    const from = undo.from;
    clearUndo();
    command({ type: "goTo", n: from });
  }

  function thumbUrl(doc: string, n: number): string {
    const tries = thumbTries[n] ?? 0;
    return `/api/thumb?${new URLSearchParams({ key: key ?? "", doc, n: String(n), ...(tries ? { r: String(tries) } : {}) })}`;
  }

  /** La consola aún la está dibujando: se vuelve a pedir en un momento, pocas veces. */
  function thumbFailed(n: number) {
    const tries = thumbTries[n] ?? 0;
    if (tries < THUMB_RETRIES) setTimeout(() => (thumbTries[n] = tries + 1), 1000);
  }

  function chapterTitle(book: number, chapter?: number): string {
    return chapter ? `${bookName(book, language)} ${chapter}` : bookName(book, language);
  }

  /** Abre un capítulo de la versión elegida. `quiet`: lo abre la página sola (sigue a la pantalla o a otra versión),
   *  sin mover la vista; si esa versión no lo tiene, vuelve a los libros sin avisar. */
  async function openChapter(book: number, chapter: number, verse?: number, quiet = false) {
    if (!bible || !key) return;
    const bibleId = bible.id;
    const request = ++chapterRequest;
    loadingChapter = { bible: bibleId, book, chapter };
    problem = null;
    try {
      const params = new URLSearchParams({ key, bible: bibleId, book: String(book), chapter: String(chapter) });
      const response = await fetch(`/api/chapter?${params}`);
      if (request !== chapterRequest) return;
      if (!response.ok) {
        if (quiet) step = { kind: "books" };
        else problem = t("remote.notInBible", { reference: chapterTitle(book, chapter) });
        return;
      }
      const verses = (await response.json()) as Verse[];
      if (request !== chapterRequest) return;
      expanded = false;
      step = { kind: "verses", bible: bibleId, book, chapter, verses };
      if (verse) {
        const found = verses.find((v) => v.number === verse);
        if (!found) {
          problem = t("remote.notInBible", { reference: `${chapterTitle(book, chapter)}:${verse}` });
          return;
        }
        await tick();
        verseList?.querySelector(`[data-verse="${verse}"]`)?.scrollIntoView({ block: "center" });
        ask(book, chapter, found);
      } else if (!quiet) {
        window.scrollTo({ top: 0 });
      }
    } catch {
      if (request === chapterRequest) problem = t("remote.offline");
    } finally {
      if (request === chapterRequest) loadingChapter = null;
    }
  }

  // La lista sigue a lo que se lee: si llega a pantalla otro capítulo (o se vuelve a uno de otra versión, y la
  // consola cambia a esa versión), se abre ese. Si cambia la versión, el capítulo abierto se recarga en la nueva.
  // Mientras se elige capítulo de un libro no se interrumpe.
  $effect(() => {
    if (!bible) return;
    const live = status?.live;
    const place = live?.kind === "scripture" && live.bible === bible.id ? live.place : null;
    const liveKey = place && `${bible.id}|${place.book}|${place.chapter}`;
    const open = loadingChapter ?? (step.kind === "verses" ? step : null);
    let target: { book: number; chapter: number } | null = null;
    if (place && liveKey !== followed && step.kind !== "chapters") target = place;
    else if (open && open.bible !== bible.id) target = open;
    if (liveKey) followed = liveKey;
    if (!target) return;
    if (open && open.bible === bible.id && open.book === target.book && open.chapter === target.chapter) return;
    const { book, chapter } = target;
    untrack(() => openChapter(book, chapter, undefined, true));
  });

  /** El versículo en pantalla, si es de este capítulo y versión. */
  let liveVerse = $derived.by(() => {
    const live = status?.live;
    if (step.kind !== "verses" || live?.kind !== "scripture" || !live.place || live.bible !== step.bible) return null;
    return live.place.book === step.book && live.place.chapter === step.chapter ? live.place.verse : null;
  });
  let shownVerses = $derived(
    step.kind !== "verses" ? [] : liveVerse !== null && !expanded ? step.verses.filter((v) => Math.abs(v.number - liveVerse!) <= AROUND) : step.verses,
  );

  async function toggleChapter() {
    expanded = !expanded;
    if (!expanded || liveVerse === null) return;
    // Al desplegar, el que está en pantalla sigue a la vista.
    await tick();
    verseList?.querySelector(`[data-verse="${liveVerse}"]`)?.scrollIntoView({ block: "center" });
  }

  function search(event: SubmitEvent) {
    event.preventDefault();
    const reference = parseQuery(query, language).reference;
    if (!reference) {
      problem = t("remote.notInBible", { reference: query.trim() });
      return;
    }
    openChapter(reference.book, reference.chapter, reference.verseStart ?? undefined);
  }

  /** Mostrar algo nuevo: se pregunta si la consola pregunta; si no, va directo. */
  function confirmThen(title: string, text: string, c: RemoteCommand) {
    if (status?.confirmSend) asking = { title, text, command: c };
    else command(c);
  }

  function ask(book: number, chapter: number, verse: Verse) {
    confirmThen(`${chapterTitle(book, chapter)}:${verse.number}`, verse.text, { type: "scripture", book, chapter, verse: verse.number });
  }

  function confirmed() {
    if (asking) command(asking.command);
    asking = null;
  }

  /** La consola cambia de versión; lo abierto se recarga en la nueva al llegar su estado (el $effect de arriba). */
  function chooseBible(id: string) {
    if (id !== bible?.id) command({ type: "setBible", id });
  }

  /** Una línea con lo que está en pantalla, para la barra de abajo. */
  function summary(): string {
    const live = status?.live;
    switch (live?.kind) {
      case undefined: return t("remote.waiting");
      case "scripture": return live.reference;
      case "pdf": return live.name;
      case "text": return live.text.split("\n")[0];
      case "other": return live.description;
      case "black": return t("describe.black");
      case "empty": return t("describe.empty");
      case "board": return t("remote.board.sheet", { n: sheetNumber(live.sheet) });
    }
  }

  // Otro documento: sus miniaturas empiezan de cero. Y si lo elegible cambió, «Volver» ya no aplica.
  $effect(() => {
    void stripId;
    thumbTries = {};
    if (undo && undo.strip !== stripId) clearUndo();
  });

  // La posición actual a la vista en la tira, sin mover la página (scrollIntoView la desplazaría).
  $effect(() => {
    void position?.n;
    void stripId;
    tick().then(() => {
      const current = stripList?.querySelector<HTMLElement>("[aria-current='true']");
      if (!stripList || !current) return;
      const left = current.offsetLeft - (stripList.clientWidth - current.offsetWidth) / 2;
      stripList.scrollTo({ left: Math.max(0, left), behavior: "smooth" });
    });
  });

  /** La referencia que está en pantalla, para resaltar su versículo en la lista. */
  let liveReference = $derived(status?.live.kind === "scripture" ? status.live.reference : null);

  /** «Página 12», «Diapositiva 3»…: para el aviso de «Volver». */
  function stepName(n: number): string {
    return t(`remote.step.${position?.unit ?? "slide"}` as MessageKey, { n });
  }

  function positionLabelFor(n: number, total: number): string {
    return t(`position.${position?.unit ?? "slide"}` as MessageKey, { n, total });
  }

  function positionLabel(): string {
    if (!position) return "";
    const key = `position.${position.unit}` as MessageKey;
    return t(key, { n: position.n, total: position.total ?? "…" });
  }

  onMount(() => {
    // Tocar rápido los botones no debe acercar la página (doble toque), ni enfocar un campo (iPhone).
    document.querySelector('meta[name="viewport"]')?.setAttribute("content", "width=device-width, initial-scale=1, maximum-scale=1, viewport-fit=cover");
    const onFullscreen = () => (fullscreen = document.fullscreenElement !== null);
    document.addEventListener("fullscreenchange", onFullscreen);
    // El navegador solo deja pasar a pantalla completa tras un toque: el primero lo hace.
    const firstTouch = () => {
      if (canFullscreen && !document.fullscreenElement) toggleFullscreen();
    };
    if (canFullscreen) document.addEventListener("pointerup", firstTouch, { once: true });
    connect();
    // El teléfono duerme la página al apagar la pantalla: al volver, se reconecta enseguida.
    const wake = () => {
      if (document.visibilityState === "visible" && !socket && connection !== "rejected") connect();
    };
    document.addEventListener("visibilitychange", wake);
    return () => {
      document.removeEventListener("visibilitychange", wake);
      document.removeEventListener("fullscreenchange", onFullscreen);
      document.removeEventListener("pointerup", firstTouch);
      clearTimeout(retryTimer);
      socket?.close();
    };
  });
</script>

<svelte:head>
  <title>Selah · {t("remote.title")}</title>
  <!-- Instalable en la pantalla de inicio. El manifiesto lleva la clave para que la app abra ya vinculada. -->
  <link rel="manifest" href={`/remote.webmanifest${key ? `?key=${encodeURIComponent(key)}` : ""}`} />
  <link rel="apple-touch-icon" href="/remote/apple-touch-icon.png" />
  <meta name="mobile-web-app-capable" content="yes" />
  <meta name="apple-mobile-web-app-capable" content="yes" />
  <meta name="apple-mobile-web-app-title" content="Selah" />
  <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
  <meta name="theme-color" content="#1d3a78" />
</svelte:head>

<div class="remote">
  <header>
    <div class="brand"><img src="/favicon.svg" alt="" />Selah</div>
    <div class="head-right">
      <span class="conn" class:ok={connection === "open"}>
        <span class="dot" aria-hidden="true"></span>
        {connection === "open" ? t("remote.connected") : connection === "connecting" ? t("remote.connecting") : ""}
      </span>
      {#if canFullscreen}
        <button class="btn ghost icon" onclick={toggleFullscreen} aria-label={t(fullscreen ? "remote.exitFullscreen" : "remote.fullscreen")}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d={fullscreen ? "M9 4v5H4M15 4v5h5M9 20v-5H4M15 20v-5h5" : "M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"} />
          </svg>
        </button>
      {/if}
    </div>
  </header>

  {#if connection === "noKey" || connection === "rejected"}
    <p class="notice">{t(connection === "noKey" ? "remote.noKey" : "remote.rejected")}</p>
  {:else}
    {#if connection === "offline"}<p class="notice">{t("remote.offline")}</p>{/if}

    <section class="card live" aria-live="polite">
      <div class="live-head">
        <h2>{t("remote.onScreen")}</h2>
        {#if position}<span class="muted small">{positionLabel()}</span>{/if}
        <!-- Dibujar sobre lo que está en pantalla, o en una hoja en blanco (D-021). -->
        <button class="btn board-open" onclick={() => (drawing = true)} disabled={connection !== "open" || !status}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20l1-4L16 5l3 3L8 19l-4 1ZM14 7l3 3" /></svg>
          {t("remote.board.title")}
        </button>
      </div>
      {#if !status}
        <p class="muted">{t("remote.waiting")}</p>
      {:else}
        {@const live = status.live}
        {#if live.kind === "scripture"}
          <p class="ref">{live.reference} <span class="muted small">{live.version}</span></p>
          <p class="text">{live.text}</p>
        {:else if live.kind === "pdf"}
          <!-- Espejo de la pantalla: la página que ve la congregación. -->
          <div class="screen">
            {#if strip?.kind === "pages" && position}
              {#key `${strip.doc}-${position.n}-${thumbTries[position.n] ?? 0}`}
                <img src={thumbUrl(strip.doc, position.n)} alt={positionLabel()} onerror={() => position && thumbFailed(position.n)} />
              {/key}
            {/if}
            <span class="screen-num" aria-hidden="true">{position?.n ?? ""}</span>
          </div>
          <p class="ref small">{t("remote.presentation", { name: live.name })}</p>
        {:else if live.kind === "text"}
          <p class="text">{live.text}</p>
        {:else if live.kind === "other"}
          <p class="ref">{live.description}</p>
        {:else if live.kind === "black"}
          <div class="screen black"><span>{t("remote.black")}</span></div>
        {:else if live.kind === "board"}
          <p class="ref">{t("remote.board.sheet", { n: sheetNumber(live.sheet) })}</p>
        {:else}
          <p class="muted">{t("describe.empty")}</p>
        {/if}
        {#if strip && position}
          <!-- Para saltar a otra página, diapositiva o versículo: va directo, con «Volver» por si fue sin querer. -->
          <ol class="strip" class:pages={strip.kind === "pages"} bind:this={stripList} aria-label={t("remote.choose")}>
            {#if strip.kind === "pages"}
              {#each { length: strip.total } as _, i (i)}
                {@const n = i + 1}
                <li>
                  <button class="thumb" aria-current={n === position.n} aria-label={t("position.page", { n, total: strip.total })} disabled={connection !== "open"} onclick={() => jump(n)}>
                    <span class="frame">
                      {#key thumbTries[n] ?? 0}
                        <img src={thumbUrl(strip.doc, n)} alt="" loading="lazy" onerror={() => thumbFailed(n)} />
                      {/key}
                    </span>
                    <span class="num">{n}</span>
                  </button>
                </li>
              {/each}
            {:else}
              {#each strip.labels as label, i (i)}
                {@const n = i + 1}
                <li>
                  <button class="chip" aria-current={n === position.n} aria-label={positionLabelFor(n, strip.labels.length)} disabled={connection !== "open"} onclick={() => jump(n)}>
                    {#if position.unit === "verse"}<strong>{label}</strong>{:else}<span class="num">{n}</span><span class="chip-text">{label}</span>{/if}
                  </button>
                </li>
              {/each}
            {/if}
          </ol>
        {/if}
        {#if !status.open}<p class="muted small">{t("remote.outputClosed")}</p>{/if}
      {/if}
    </section>

    {#if status?.plan && status.plan.items.length > 0}
      <section class="card">
        <h2>{t("remote.plan", { name: status.plan.name })}</h2>
        <div class="items">
          {#each status.plan.items as item (item.id)}
            {#if item.kind === "section"}
              <h3 class="plan-section">{item.title}</h3>
            {:else}
              <button
                class="btn item plan-item"
                class:current={item.current}
                onclick={() => confirmThen(item.title, item.detail, { type: "planItem", id: item.id })}
                disabled={connection !== "open" || !item.available}
              >
                <span class="icon" aria-hidden="true">{item.icon}</span>
                <span class="label">
                  {item.title}
                  <span class="muted small sub" class:bad={!item.available}>{item.detail}</span>
                </span>
                {#if item.shown}<span class="check" aria-label={t("plans.shown")}>✓</span>{/if}
              </button>
            {/if}
          {/each}
        </div>
      </section>
    {/if}

    {#if status && status.items.length > 0}
      <section class="card">
        <h2>{t("remote.items")}</h2>
        <div class="items">
          {#each status.items as item (item.key)}
            <button class="btn item" onclick={() => confirmThen(item.label, "", { type: "show", key: item.key })} disabled={connection !== "open"}>
              {#if item.prepared}<span class="tag">{t("prepared.title")}</span>{/if}
              <span class="label">{item.label}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="card bible">
      {#if status && status.bibles.length > 1 && bible}
        <!-- Como en la consola: un botón por versión, marcada la elegida (cambia sola si vuelve a pantalla un
             versículo de otra). -->
        <div class="versions" role="group" aria-label={t("remote.version")}>
          {#each status.bibles as b (b.id)}
            <button class="version-chip" aria-pressed={b.id === bible.id} title={b.name} disabled={connection !== "open"} onclick={() => chooseBible(b.id)}>
              {b.abbreviation}
            </button>
          {/each}
        </div>
      {/if}
      <form class="search" onsubmit={search}>
        <input class="field" bind:value={query} placeholder={t("remote.searchPlaceholder")} enterkeyhint="go" autocomplete="off" disabled={!bible} />
        <button class="btn primary" type="submit" disabled={!bible || !query.trim()}>{t("remote.show")}</button>
      </form>
      {#if problem}<p class="notice small">{problem}</p>{/if}

      {#if !bible}
        <p class="muted">{status ? t("remote.noBible") : t("remote.waiting")}</p>
      {:else if step.kind === "books"}
        <h2>{t("bible.oldTestament")} <span class="muted small">{bible.abbreviation}</span></h2>
        <div class="grid books">
          {#each oldTestament as b (b.number)}
            <button class="btn" onclick={() => (step = { kind: "chapters", book: b.number })}>{bookName(b.number, language)}</button>
          {/each}
        </div>
        <h2>{t("bible.newTestament")}</h2>
        <div class="grid books">
          {#each newTestament as b (b.number)}
            <button class="btn" onclick={() => (step = { kind: "chapters", book: b.number })}>{bookName(b.number, language)}</button>
          {/each}
        </div>
      {:else if step.kind === "chapters"}
        {@const book = step.book}
        <div class="step-head">
          <button class="btn ghost" onclick={() => (step = { kind: "books" })}>{t("remote.back")}</button>
          <h2>{chapterTitle(book)}</h2>
        </div>
        <div class="grid numbers">
          {#each { length: bible.books.find((b) => b.number === book)?.chapters ?? 0 } as _, i (i)}
            <button class="btn" onclick={() => openChapter(book, i + 1)}>{i + 1}</button>
          {/each}
        </div>
      {:else}
        {@const { book, chapter, verses } = step}
        {@const folded = liveVerse !== null && shownVerses.length < verses.length}
        <div class="step-head">
          <button class="btn ghost" onclick={() => (step = { kind: "chapters", book })}>{t("remote.back")}</button>
          <h2>{chapterTitle(book, chapter)}</h2>
        </div>
        <ol class="verses" bind:this={verseList}>
          {#each shownVerses as v (v.number)}
            <li data-verse={v.number} class:on-screen={liveVerse === v.number || liveReference === `${chapterTitle(book, chapter)}:${v.number}`}>
              <button onclick={() => ask(book, chapter, v)}><strong>{v.number}</strong> {v.text}</button>
            </li>
          {/each}
        </ol>
        {#if folded || (expanded && liveVerse !== null)}
          <button class="btn ghost more" onclick={toggleChapter}>
            {folded ? t("remote.wholeChapter", { n: verses.length }) : t("remote.lessChapter")}
          </button>
        {/if}
      {/if}
    </section>

    <!-- A mano del pulgar: siempre visible abajo. -->
    <nav class="controls">
      {#if undo}
        <div class="undo" role="status">
          <span>{stepName(undo.to)}</span>
          <button class="btn" onclick={goBack} disabled={connection !== "open"}>{t("remote.undo", { step: stepName(undo.from) })}</button>
        </div>
      {/if}
      <p class="now"><span class="dot" aria-hidden="true"></span><strong>{summary()}</strong>{#if position}<span class="muted"> · {positionLabel()}</span>{/if}</p>
      <button class="btn big" onclick={() => command({ type: "previous" })} disabled={connection !== "open" || !canPrevious}>{t("live.previous")}</button>
      <button class="btn big" onclick={() => command({ type: "next" })} disabled={connection !== "open" || !canNext}>{t("live.next")}</button>
      {#if status?.canUnblack}
        <button class="btn danger" onclick={() => command({ type: "black" })} disabled={connection !== "open"}>{t("live.unblack")}</button>
      {:else}
        <button class="btn" onclick={() => command({ type: "black" })} disabled={connection !== "open" || status?.live.kind === "black"}>{t("live.black")}</button>
      {/if}
      <button class="btn" onclick={() => command({ type: "clear" })} disabled={connection !== "open" || status?.live.kind === "empty"}>{t("live.clear")}</button>
    </nav>
  {/if}
</div>

{#if drawing && status}
  <BoardPad
    {status}
    {board}
    version={boardVersion}
    online={connection === "open"}
    summary={summary()}
    thumbUrl={(doc, n) => `/api/thumb?${new URLSearchParams({ key: key ?? "", doc, n: String(n) })}`}
    {act}
    {command}
    onclose={() => (drawing = false)}
  />
{/if}

{#if asking}
  {@const request = asking}
  <div class="backdrop" role="presentation" onclick={() => (asking = null)}>
    <div class="confirm card" role="alertdialog" aria-modal="true" aria-labelledby="remote-confirm" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <h2 id="remote-confirm">{t("confirm.title")}</h2>
      <p class="ref">{request.title}</p>
      {#if request.text}<p class="text small">{request.text}</p>{/if}
      <div class="actions">
        <button class="btn big" onclick={() => (asking = null)}>{t("confirm.cancel")}</button>
        <button class="btn primary big" onclick={confirmed}>{t("confirm.send")}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .remote {
    max-width: 640px;
    margin: 0 auto;
    padding: 12px 16px calc(190px + env(safe-area-inset-bottom));
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 100vh;
    min-height: 100dvh;
    padding-top: calc(12px + env(safe-area-inset-top));
    background: var(--bg);
    /* Sin zoom por doble toque: los botones se pulsan rápido y seguido. */
    touch-action: manipulation;
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
  }
  .remote :global(button), .remote :global(select), .remote :global(input) { touch-action: manipulation; }
  .head-right { display: flex; align-items: center; gap: 6px; }
  .btn.icon { min-height: 40px; width: 40px; padding: 0; justify-content: center; }
  .btn.icon svg { width: 20px; height: 20px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
  header { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .brand { display: flex; align-items: center; gap: 8px; font-size: 18px; font-weight: 750; }
  .brand img { width: 28px; height: 28px; }
  .conn { display: inline-flex; align-items: center; gap: 6px; font-size: 14px; color: var(--muted); }
  .conn .dot { width: 9px; height: 9px; border-radius: 50%; background: var(--muted); }
  .conn.ok { color: var(--ok); }
  .conn.ok .dot { background: var(--ok); }
  .notice { padding: 10px 12px; border-radius: 8px; background: var(--warn-bg); color: var(--warn-text); }
  .card { padding: 14px; display: flex; flex-direction: column; gap: 10px; }
  h2 { font-size: 15px; margin: 0; }
  .live-head { display: flex; align-items: center; gap: 8px; }
  .live-head h2 { flex: 1; }
  .board-open { min-height: 40px; gap: 6px; padding: 0 12px; font-size: 14px; }
  .board-open svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
  .ref { font-weight: 700; margin: 0; }
  .text { margin: 0; line-height: 1.45; white-space: pre-line; }
  .search { display: flex; gap: 8px; }
  .versions { display: flex; flex-wrap: wrap; gap: 6px; }
  .version-chip {
    min-height: 40px;
    min-width: 52px;
    padding: 0 12px;
    font: inherit;
    font-size: 14px;
    font-weight: 700;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    cursor: pointer;
  }
  .version-chip[aria-pressed="true"] { background: var(--accent); border-color: var(--accent); color: var(--accent-text); }
  .version-chip:disabled { opacity: 0.55; }
  .more { align-self: center; }
  .items { display: flex; flex-direction: column; gap: 6px; }
  .item { justify-content: flex-start; gap: 8px; min-height: 52px; text-align: left; }
  .item .label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .plan-section { margin: 8px 0 0; font-size: 12px; font-weight: 750; letter-spacing: 0.05em; text-transform: uppercase; color: var(--muted); }
  .plan-item .label { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .plan-item .sub { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 400; }
  .plan-item .sub.bad { color: var(--live); }
  .plan-item.current { border-color: var(--accent); background: var(--accent-soft); }
  .plan-item .icon { flex: none; width: 1.4em; text-align: center; }
  .plan-item .check { flex: none; color: var(--accent); font-weight: 750; }
  .tag { flex: none; font-size: 12px; font-weight: 750; padding: 2px 6px; border-radius: 5px; background: var(--accent-soft); color: var(--text); }
  .search .field { flex: 1; min-width: 0; font-size: 16px; }
  .grid { display: grid; gap: 6px; }
  .books { grid-template-columns: repeat(auto-fill, minmax(118px, 1fr)); }
  .books .btn { font-size: 14px; padding: 0 8px; justify-content: flex-start; }
  .numbers { grid-template-columns: repeat(auto-fill, minmax(56px, 1fr)); }
  .numbers .btn { min-height: 48px; font-variant-numeric: tabular-nums; }
  .step-head { display: flex; align-items: center; gap: 10px; }
  .verses { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .verses button {
    width: 100%;
    text-align: left;
    font: inherit;
    line-height: 1.45;
    padding: 10px 12px;
    color: var(--text);
    background: var(--surface-2);
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
  }
  .verses button:active { border-color: var(--accent); background: var(--accent-soft); }
  .verses .on-screen button { border-color: var(--live); background: var(--live-soft); }
  .controls {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 10px 16px calc(10px + env(safe-area-inset-bottom));
    background: var(--surface);
    border-top: 1px solid var(--border);
  }
  .controls .btn.big { min-height: 64px; font-size: 18px; }
  .now { grid-column: 1 / -1; margin: 0; display: flex; align-items: center; gap: 6px; min-width: 0; font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .now strong { overflow: hidden; text-overflow: ellipsis; }
  .now .dot { flex: none; width: 9px; height: 9px; border-radius: 50%; background: var(--live); }
  .backdrop { position: fixed; inset: 0; z-index: 10; display: grid; place-items: center; padding: 16px; background: rgb(0 0 0 / 50%); }
  .confirm { width: min(460px, 100%); max-height: 80vh; overflow: auto; }
  /* Espejo de la pantalla: 16:9 desde el principio para que nada salte mientras llega la imagen. */
  .screen {
    position: relative;
    aspect-ratio: 16 / 9;
    max-height: 32vh;
    width: 100%;
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: 8px;
    background: #000;
    outline: 2px solid var(--live);
    outline-offset: -2px;
  }
  .screen img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; }
  .screen-num { color: rgb(255 255 255 / 45%); font-size: 28px; font-weight: 750; font-variant-numeric: tabular-nums; }
  .screen.black { outline-color: var(--border); }
  .screen.black span { color: rgb(255 255 255 / 70%); font-size: 14px; font-weight: 650; }
  .strip {
    list-style: none;
    margin: 0 -14px;
    padding: 2px 14px 6px;
    display: flex;
    gap: 8px;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
  }
  .strip::-webkit-scrollbar { display: none; }
  .strip li { flex: none; scroll-snap-align: center; }
  .thumb, .chip {
    font: inherit;
    color: var(--text);
    background: var(--surface-2);
    border: 2px solid transparent;
    border-radius: 8px;
    cursor: pointer;
  }
  .thumb { display: flex; flex-direction: column; align-items: stretch; gap: 4px; padding: 3px; width: 112px; }
  .frame { position: relative; aspect-ratio: 16 / 9; border-radius: 5px; overflow: hidden; background: #000; }
  .frame img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; }
  .thumb .num { font-size: 13px; font-weight: 650; text-align: center; font-variant-numeric: tabular-nums; }
  .chip { min-height: 48px; max-width: 200px; padding: 6px 12px; display: flex; align-items: center; gap: 8px; text-align: left; }
  .chip strong { min-width: 1.6em; text-align: center; font-variant-numeric: tabular-nums; }
  .chip .num { flex: none; font-size: 13px; font-weight: 750; color: var(--muted); font-variant-numeric: tabular-nums; }
  .chip-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; }
  /* La posición actual: borde rojo de «en vivo» y número destacado (no solo color). */
  .thumb[aria-current="true"], .chip[aria-current="true"] { border-color: var(--live); background: var(--live-soft); }
  .thumb[aria-current="true"] .num, .chip[aria-current="true"] .num { color: var(--live); font-weight: 800; }
  .thumb:disabled, .chip:disabled { opacity: 0.55; }
  .undo {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 6px 6px 6px 12px;
    border-radius: 8px;
    background: var(--text);
    color: var(--bg);
    font-size: 14px;
    font-weight: 650;
  }
  .undo .btn { min-height: 40px; }
  .actions { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
</style>
