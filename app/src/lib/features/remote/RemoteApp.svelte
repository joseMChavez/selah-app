<script lang="ts">
  // Página del control remoto, en el navegador del teléfono. No usa Tauri: habla con el servidor de Selah
  // por WebSocket (órdenes y estado) y por /api/chapter (versículos). La consola aplica cada orden.
  import { onMount, tick } from "svelte";
  import { bookName, type BookLanguage } from "$lib/features/bible/books";
  import { parseQuery } from "$lib/features/bible/reference";
  import { i18n, t, type MessageKey } from "$lib/i18n/index.svelte";
  import type { Verse } from "$lib/platform";
  import type { RemoteCommand, RemoteStatus, ServerMessage } from "./protocol";

  const KEY_STORAGE = "selah-remote-key";
  const RETRY_MS = [1000, 2000, 3000, 5000];
  const FIRST_NEW_TESTAMENT_BOOK = 40;

  type Connection = "connecting" | "open" | "offline" | "rejected" | "noKey";
  type Step =
    | { kind: "books" }
    | { kind: "chapters"; book: number }
    | { kind: "verses"; book: number; chapter: number; verses: Verse[] };

  const key = readKey();
  let connection = $state<Connection>(key ? "connecting" : "noKey");
  let status = $state<RemoteStatus | null>(null);
  let step = $state<Step>({ kind: "books" });
  let query = $state("");
  let problem = $state<string | null>(null);
  /** Lo que espera el «¿Seguro?»: un versículo o algo de «Para mostrar». */
  let asking = $state<{ title: string; text: string; command: RemoteCommand } | null>(null);
  let verseList = $state<HTMLOListElement>();

  let socket: WebSocket | null = null;
  let retries = 0;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;

  let bible = $derived(status?.bible ?? null);
  let language = $derived<BookLanguage>(bible?.language && ["es", "en", "it"].includes(bible.language) ? (bible.language as BookLanguage) : i18n.locale);
  let position = $derived(status?.position ?? null);
  let canPrevious = $derived(position !== null && position.n > 1);
  let canNext = $derived(position !== null && (position.total === null || position.n < position.total));
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
        if (message.type === "status") status = message.status;
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
  }

  function chapterTitle(book: number, chapter?: number): string {
    return chapter ? `${bookName(book, language)} ${chapter}` : bookName(book, language);
  }

  async function openChapter(book: number, chapter: number, verse?: number) {
    if (!bible || !key) return;
    problem = null;
    try {
      const params = new URLSearchParams({ key, bible: bible.id, book: String(book), chapter: String(chapter) });
      const response = await fetch(`/api/chapter?${params}`);
      if (!response.ok) {
        problem = t("remote.notInBible", { reference: chapterTitle(book, chapter) });
        return;
      }
      const verses = (await response.json()) as Verse[];
      step = { kind: "verses", book, chapter, verses };
      if (verse) {
        const found = verses.find((v) => v.number === verse);
        if (!found) {
          problem = t("remote.notInBible", { reference: `${chapterTitle(book, chapter)}:${verse}` });
          return;
        }
        await tick();
        verseList?.querySelector(`[data-verse="${verse}"]`)?.scrollIntoView({ block: "center" });
        ask(book, chapter, found);
      } else {
        window.scrollTo({ top: 0 });
      }
    } catch {
      problem = t("remote.offline");
    }
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

  function chooseBible(id: string) {
    command({ type: "setBible", id });
    // Lo abierto era de la otra versión: se vuelve a los libros (los números de capítulo pueden cambiar).
    step = { kind: "books" };
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
    }
  }

  /** La referencia que está en pantalla, para resaltar su versículo en la lista. */
  let liveReference = $derived(status?.live.kind === "scripture" ? status.live.reference : null);

  function positionLabel(): string {
    if (!position) return "";
    const key = `position.${position.unit}` as MessageKey;
    return t(key, { n: position.n, total: position.total ?? "…" });
  }

  onMount(() => {
    connect();
    // El teléfono duerme la página al apagar la pantalla: al volver, se reconecta enseguida.
    const wake = () => {
      if (document.visibilityState === "visible" && !socket && connection !== "rejected") connect();
    };
    document.addEventListener("visibilitychange", wake);
    return () => {
      document.removeEventListener("visibilitychange", wake);
      clearTimeout(retryTimer);
      socket?.close();
    };
  });
</script>

<svelte:head>
  <title>Selah · {t("remote.title")}</title>
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
</svelte:head>

<div class="remote">
  <header>
    <div class="brand"><img src="/favicon.svg" alt="" />Selah</div>
    <span class="conn" class:ok={connection === "open"}>
      <span class="dot" aria-hidden="true"></span>
      {connection === "open" ? t("remote.connected") : connection === "connecting" ? t("remote.connecting") : ""}
    </span>
  </header>

  {#if connection === "noKey" || connection === "rejected"}
    <p class="notice">{t(connection === "noKey" ? "remote.noKey" : "remote.rejected")}</p>
  {:else}
    {#if connection === "offline"}<p class="notice">{t("remote.offline")}</p>{/if}

    <section class="card live" aria-live="polite">
      <div class="live-head">
        <h2>{t("remote.onScreen")}</h2>
        {#if position}<span class="muted small">{positionLabel()}</span>{/if}
      </div>
      {#if !status}
        <p class="muted">{t("remote.waiting")}</p>
      {:else}
        {@const live = status.live}
        {#if live.kind === "scripture"}
          <p class="ref">{live.reference} <span class="muted small">{live.version}</span></p>
          <p class="text">{live.text}</p>
        {:else if live.kind === "pdf"}
          <p class="ref">{t("remote.presentation", { name: live.name })}</p>
        {:else if live.kind === "text"}
          <p class="text">{live.text}</p>
        {:else if live.kind === "other"}
          <p class="ref">{live.description}</p>
        {:else}
          <p class="muted">{t(live.kind === "black" ? "describe.black" : "describe.empty")}</p>
        {/if}
        {#if !status.open}<p class="muted small">{t("remote.outputClosed")}</p>{/if}
      {/if}
    </section>

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
        <label class="version">
          <span class="muted small">{t("remote.version")}</span>
          <select class="field" value={bible.id} onchange={(e) => chooseBible(e.currentTarget.value)} disabled={connection !== "open"}>
            {#each status.bibles as b (b.id)}
              <option value={b.id}>{b.abbreviation} · {b.name}</option>
            {/each}
          </select>
        </label>
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
        <div class="step-head">
          <button class="btn ghost" onclick={() => (step = { kind: "chapters", book })}>{t("remote.back")}</button>
          <h2>{chapterTitle(book, chapter)}</h2>
        </div>
        <ol class="verses" bind:this={verseList}>
          {#each verses as v (v.number)}
            <li data-verse={v.number} class:on-screen={liveReference === `${chapterTitle(book, chapter)}:${v.number}`}>
              <button onclick={() => ask(book, chapter, v)}><strong>{v.number}</strong> {v.text}</button>
            </li>
          {/each}
        </ol>
      {/if}
    </section>

    <!-- A mano del pulgar: siempre visible abajo. -->
    <nav class="controls">
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
    background: var(--bg);
  }
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
  .live-head { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; }
  .ref { font-weight: 700; margin: 0; }
  .text { margin: 0; line-height: 1.45; }
  .search { display: flex; gap: 8px; }
  .version { display: flex; flex-direction: column; gap: 4px; }
  .version .field { font-size: 16px; }
  .items { display: flex; flex-direction: column; gap: 6px; }
  .item { justify-content: flex-start; gap: 8px; min-height: 52px; text-align: left; }
  .item .label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
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
  .actions { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
</style>
