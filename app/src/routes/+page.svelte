<script lang="ts">
  import { onMount } from "svelte";
  import OutputView from "$lib/OutputView.svelte";
  import { i18n, locales, setLocale, t, type Locale, type MessageKey } from "$lib/i18n/index.svelte";
  import { pageCount } from "$lib/pdf";
  import {
    platform,
    type CommandError,
    type MediaFile,
    type MonitorInfo,
    type OutputContent,
    type OutputState,
  } from "$lib/platform";
  import { splitSlides } from "$lib/slides";

  const MONITOR_POLL_MS = 2000;

  type Message = { key: MessageKey; params?: Record<string, string | number> };

  let monitors = $state<MonitorInfo[]>([]);
  let selectedIndex = $state<number | null>(null);
  let output = $state<OutputState>({ open: false, monitorName: null, content: { kind: "empty" } });
  let draft = $state(t("prepared.sample"));
  // Se guarda la clave y no el texto, para que el aviso cambie de idioma junto con la interfaz.
  let error = $state<Message | null>(null);
  let busy = $state(false);
  let media = $state<MediaFile[]>([]);
  /** null = se prepara el texto; si no, el medio elegido de la lista. */
  let preparedMedia = $state<MediaFile | null>(null);
  let preparedSlide = $state(0);
  let preparedPage = $state(1);
  let pageCounts = $state<Record<string, number>>({});

  let slides = $derived(splitSlides(draft));
  let slideIndex = $derived(Math.min(preparedSlide, Math.max(slides.length - 1, 0)));
  let prepared = $derived<OutputContent>(toContent(preparedMedia, preparedPage, slides, slideIndex));
  let selected = $derived(monitors.find((m) => m.index === selectedIndex) ?? null);
  // Si el monitor de la salida desaparece, el compositor mueve la ventana a otro: hay que avisar.
  let outputMonitorLost = $derived(
    output.open && output.monitorName !== null && !monitors.some((m) => m.name === output.monitorName),
  );
  /** Posición navegable de lo que está en pantalla: diapositiva de texto o página de PDF (1 = primera). */
  let livePosition = $derived(position(output.content));

  function toContent(file: MediaFile | null, page: number, deck: string[], index: number): OutputContent {
    if (!file) return deck.length > 0 ? { kind: "text", slides: deck, index } : { kind: "empty" };
    switch (file.kind) {
      case "image": return { kind: "image", path: file.path };
      case "video": return { kind: "video", path: file.path, paused: false };
      case "pdf": return { kind: "pdf", path: file.path, page };
    }
  }

  function position(content: OutputContent): { n: number; total: number | null; label: MessageKey } | null {
    if (content.kind === "text") return { n: content.index + 1, total: content.slides.length, label: "position.slide" };
    if (content.kind === "pdf") return { n: content.page, total: pageCounts[content.path] ?? null, label: "position.page" };
    return null;
  }

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function describe(content: OutputContent): string {
    switch (content.kind) {
      case "empty": return t("describe.empty");
      case "black": return t("describe.black");
      case "testPattern": return t("describe.testPattern");
      case "text": return t("describe.text", { n: content.index + 1, total: content.slides.length });
      case "image": return t("describe.image", { name: baseName(content.path) });
      case "video": return t(content.paused ? "describe.videoPaused" : "describe.video", { name: baseName(content.path) });
      case "pdf": return t("describe.pdf", { name: baseName(content.path), n: content.page });
    }
  }

  function toMessage(e: unknown): Message {
    const err = e as Partial<CommandError> | null;
    switch (err?.code) {
      case "monitorDisconnected": return { key: "error.monitorDisconnected" };
      case "fileMissing": return { key: "error.fileMissing", params: { path: (err as { path: string }).path } };
      case "invalidContent": return { key: "error.invalidContent" };
      case "internal": return { key: "error.internal", params: { message: (err as { message: string }).message } };
      default: return { key: "error.internal", params: { message: String(e) } };
    }
  }

  async function run(action: () => Promise<unknown>) {
    busy = true;
    error = null;
    try {
      await action();
    } catch (e) {
      error = toMessage(e);
    } finally {
      busy = false;
    }
  }

  async function ensurePageCount(path: string) {
    if (pageCounts[path]) return;
    try {
      pageCounts[path] = await pageCount(platform.mediaUrl(path));
    } catch (e) {
      error = { key: "error.pdfOpen", params: { name: baseName(path), error: String(e) } };
    }
  }

  function prepare(file: MediaFile | null) {
    preparedMedia = file;
    preparedPage = 1;
    if (file?.kind === "pdf") ensurePageCount(file.path);
  }

  const addMedia = () => run(async () => {
    const picked = await platform.pickMedia(t("dialog.title"), t("dialog.filter"));
    const known = new Set(media.map((m) => m.path));
    media = [...media, ...picked.filter((m) => !known.has(m.path))];
    if (picked.length > 0) prepare(picked[0]);
  });

  async function refreshMonitors() {
    try {
      monitors = await platform.listMonitors();
    } catch (e) {
      error = { key: "error.monitors", params: { error: String(e) } };
      return;
    }
    if (selectedIndex === null || !monitors.some((m) => m.index === selectedIndex)) {
      // Por defecto, un monitor que no sea el principal: suele ser el proyector.
      selectedIndex = (monitors.find((m) => !m.isPrimary && m.index > 0) ?? monitors.at(-1))?.index ?? null;
    }
  }

  const openOutput = () => run(async () => {
    if (selectedIndex !== null) output = await platform.openOutput(selectedIndex);
  });
  const closeOutput = () => run(() => platform.closeOutput());
  function isTyping(el: Element | null): boolean {
    if (!el) return false;
    if (el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) return true;
    if (el instanceof HTMLInputElement) return !["radio", "checkbox", "button", "submit"].includes(el.type);
    return (el as HTMLElement).isContentEditable;
  }

  const send = (content: OutputContent) => run(async () => {
    // En WebKit hacer clic en un botón no le quita el foco al cuadro de texto; sin esto las flechas no navegarían.
    if (isTyping(document.activeElement)) (document.activeElement as HTMLElement).blur();
    output = await platform.setOutputContent(content);
  });

  /** Cambia directamente lo que está en pantalla (n empieza en 1). Es una acción explícita del panel «En pantalla». */
  function goLive(n: number) {
    const content = output.content;
    if (busy || !livePosition || n < 1 || n === livePosition.n) return;
    if (livePosition.total !== null && n > livePosition.total) return;
    if (content.kind === "text") send({ ...content, index: n - 1 });
    else if (content.kind === "pdf") send({ ...content, page: n });
  }

  function onKeydown(event: KeyboardEvent) {
    // No interferir mientras se escribe. En los botones de monitor sí se navega: si no, las flechas cambiarían el monitor.
    if (isTyping(event.target as Element) || !livePosition) return;
    if (event.key === "ArrowRight" || event.key === "PageDown") {
      event.preventDefault();
      goLive(livePosition.n + 1);
    } else if (event.key === "ArrowLeft" || event.key === "PageUp") {
      event.preventDefault();
      goLive(livePosition.n - 1);
    }
  }

  $effect(() => {
    if (output.content.kind === "pdf") ensurePageCount(output.content.path);
  });

  onMount(() => {
    refreshMonitors();
    platform.getOutputState().then((state) => (output = state));
    const unlisten = platform.onOutputState((state) => (output = state));
    const unlistenError = platform.onOutputError((message) => (error = { key: "error.fromOutput", params: { message } }));
    const timer = setInterval(refreshMonitors, MONITOR_POLL_MS);
    return () => {
      clearInterval(timer);
      unlisten.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<main>
  <header>
    <h1>{t("app.title")}</h1>
    <p class="status" class:live={output.open} role="status">
      {#if output.open}
        ● {t("status.openOn")} <strong>{output.monitorName}</strong> · {t("status.onScreen")}: {describe(output.content)}
      {:else}
        ○ {t("status.closed")}
      {/if}
    </p>
    <label class="language">
      {t("settings.language")}
      <select value={i18n.locale} onchange={(e) => setLocale(e.currentTarget.value as Locale)}>
        {#each Object.entries(locales) as [code, locale] (code)}
          <option value={code}>{locale.name}</option>
        {/each}
      </select>
    </label>
  </header>

  {#if outputMonitorLost}
    <p class="alert" role="alert">{t("monitors.lost", { monitor: output.monitorName ?? "" })}</p>
  {/if}
  {#if error}<p class="alert" role="alert">{t(error.key, error.params)}</p>{/if}

  <section>
    <h2>{t("monitors.title")}</h2>
    {#if monitors.length === 0}
      <p>{t("monitors.none")}</p>
    {:else}
      <ul class="monitors">
        {#each monitors as m (m.index)}
          <li>
            <label>
              <input type="radio" name="monitor" value={m.index} bind:group={selectedIndex} />
              <span>
                <strong>{m.name}</strong>
                {t("monitors.details", { width: m.width, height: m.height, scale: m.scale, x: m.x, y: m.y })}
                {#if m.isPrimary}· {t("monitors.primary")}{/if}
                {#if output.open && m.name === output.monitorName}· <em>{t("monitors.outputHere")}</em>{/if}
              </span>
            </label>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="actions">
      <button onclick={openOutput} disabled={busy || selected === null}>
        {t(output.open ? "output.moveTo" : "output.openOn", { monitor: selected?.name ?? "…" })}
      </button>
      <button onclick={closeOutput} disabled={busy || !output.open}>{t("output.close")}</button>
    </div>
  </section>

  <div class="threads">
    <section>
      <h2>{t("prepared.title")} <span class="tag">{t("prepared.tag")}</span></h2>
      <ul class="media">
        <li>
          <button class:chosen={preparedMedia === null} onclick={() => prepare(null)}>{t("prepared.text")}</button>
        </li>
        {#each media as m (m.path)}
          <li>
            <button class:chosen={preparedMedia?.path === m.path} onclick={() => prepare(m)} title={m.path}>
              {t(`kind.${m.kind}`)} · {m.name}
            </button>
          </li>
        {/each}
        <li><button onclick={addMedia} disabled={busy}>{t("prepared.addFiles")}</button></li>
      </ul>
      {#if preparedMedia === null}
        <textarea bind:value={draft} rows="6" aria-label={t("prepared.textLabel")}></textarea>
        <p class="hint">{t("prepared.slidesHint")}</p>
        {#if slides.length > 1}
          <ol class="slides">
            {#each slides as slide, i (i)}
              <li>
                <button class="thumb" class:chosen={i === slideIndex} onclick={() => (preparedSlide = i)} aria-label={t("position.slide", { n: i + 1, total: slides.length })}>
                  <span class="mini"><OutputView content={{ kind: "text", slides, index: i }} preview /></span>
                  <span class="num">{i + 1}</span>
                </button>
              </li>
            {/each}
          </ol>
        {/if}
      {/if}
      <div class="screen"><OutputView content={prepared} preview /></div>
      <div class="actions">
        <button class="primary" onclick={() => send(prepared)} disabled={busy}>{t("prepared.send")}</button>
        {#if preparedMedia === null && slides.length > 1}
          <button onclick={() => (preparedSlide = Math.max(slideIndex - 1, 0))} disabled={slideIndex <= 0}>◀</button>
          <span class="pages">{t("position.slide", { n: slideIndex + 1, total: slides.length })}</span>
          <button onclick={() => (preparedSlide = Math.min(slideIndex + 1, slides.length - 1))} disabled={slideIndex >= slides.length - 1}>▶</button>
        {:else if preparedMedia?.kind === "pdf"}
          <button onclick={() => (preparedPage = Math.max(preparedPage - 1, 1))} disabled={preparedPage <= 1}>◀</button>
          <span class="pages">{t("position.page", { n: preparedPage, total: pageCounts[preparedMedia.path] ?? "…" })}</span>
          <button
            onclick={() => (preparedPage = Math.min(preparedPage + 1, pageCounts[preparedMedia!.path] ?? preparedPage))}
            disabled={preparedPage >= (pageCounts[preparedMedia.path] ?? 1)}>▶</button>
        {/if}
      </div>
    </section>

    <section>
      <h2>{t("live.title")} <span class="tag live">{t("live.tag")}</span></h2>
      <div class="screen live"><OutputView content={output.content} preview /></div>
      {#if livePosition}
        <div class="actions">
          <button onclick={() => goLive(livePosition!.n - 1)} disabled={busy || livePosition.n <= 1}>{t("live.previous")}</button>
          <span class="pages">{t(livePosition.label, { n: livePosition.n, total: livePosition.total ?? "…" })}</span>
          <button onclick={() => goLive(livePosition!.n + 1)} disabled={busy || (livePosition.total !== null && livePosition.n >= livePosition.total)}>{t("live.next")}</button>
        </div>
        <p class="hint">{t("live.keysHint")}{output.content.kind === "text" && output.content.slides.length > 1 ? ` ${t("live.jumpHint")}` : ""}</p>
        {#if output.content.kind === "text" && output.content.slides.length > 1}
          <ol class="slides">
            {#each output.content.slides as slide, i (i)}
              <li>
                <button class="thumb" class:on-air={i === output.content.index} onclick={() => goLive(i + 1)} disabled={busy} aria-label={t("position.slide", { n: i + 1, total: output.content.slides.length })}>
                  <span class="mini"><OutputView content={{ kind: "text", slides: output.content.slides, index: i }} preview /></span>
                  <span class="num">{i + 1}</span>
                </button>
              </li>
            {/each}
          </ol>
        {/if}
      {:else if output.content.kind === "video"}
        <div class="actions">
          <button onclick={() => output.content.kind === "video" && send({ ...output.content, paused: !output.content.paused })} disabled={busy}>
            {t(output.content.paused ? "live.resume" : "live.pause")}
          </button>
        </div>
      {/if}
      <div class="actions">
        <button onclick={() => send({ kind: "black" })} disabled={busy}>{t("live.black")}</button>
        <button onclick={() => send({ kind: "testPattern" })} disabled={busy}>{t("live.testPattern")}</button>
        <button onclick={() => send({ kind: "empty" })} disabled={busy}>{t("live.clear")}</button>
      </div>
    </section>
  </div>
</main>

<style>
  :root {
    --bg: #f4f4f5;
    --panel: #ffffff;
    --text: #18181b;
    --muted: #52525b;
    --border: #d4d4d8;
    --accent: #2563eb;
    --live: #dc2626;
    --warn-bg: #fef3c7;
    --warn-text: #78350f;
    color-scheme: light dark;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      --bg: #18181b;
      --panel: #27272a;
      --text: #f4f4f5;
      --muted: #a1a1aa;
      --border: #3f3f46;
      --accent: #3b82f6;
      --live: #f87171;
      --warn-bg: #451a03;
      --warn-text: #fde68a;
    }
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--text);
    font: 15px/1.4 system-ui, sans-serif;
  }
  main {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }
  h1 { margin: 0; font-size: 20px; }
  h2 { margin: 0 0 10px; font-size: 16px; }
  .status { margin: 0; color: var(--muted); }
  .status.live { color: var(--live); }
  .alert {
    margin: 0;
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--warn-bg);
    color: var(--warn-text);
  }
  section {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
  }
  .threads {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 14px;
  }
  .monitors { list-style: none; margin: 0 0 10px; padding: 0; display: grid; gap: 6px; }
  .monitors label { display: flex; gap: 8px; align-items: center; cursor: pointer; }
  .media { list-style: none; margin: 0 0 10px; padding: 0; display: flex; flex-wrap: wrap; gap: 6px; }
  .media button { max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .media button.chosen { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); }
  .pages { align-self: center; color: var(--muted); }
  .hint { margin: 0 0 10px; font-size: 13px; color: var(--muted); }
  .language { display: flex; align-items: center; gap: 6px; color: var(--muted); }
  .language select { font: inherit; padding: 4px 6px; border-radius: 6px; border: 1px solid var(--border); background: var(--bg); color: var(--text); }
  .slides {
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }
  button.thumb { position: relative; width: 100%; height: auto; min-height: 0; padding: 3px; }
  .thumb .mini { display: block; aspect-ratio: 16 / 9; overflow: hidden; border-radius: 3px; pointer-events: none; }
  .thumb .num { position: absolute; left: 6px; bottom: 5px; padding: 0 5px; font-size: 12px; border-radius: 3px; background: rgb(0 0 0 / 0.6); color: #fff; }
  button.thumb.chosen { border-color: var(--accent); box-shadow: inset 0 0 0 2px var(--accent); }
  button.thumb.on-air { border-color: var(--live); box-shadow: inset 0 0 0 2px var(--live); }
  .tag { font-weight: normal; font-size: 13px; color: var(--muted); }
  .tag.live { color: var(--live); }
  textarea {
    box-sizing: border-box;
    width: 100%;
    margin-bottom: 10px;
    padding: 8px;
    font: inherit;
    color: inherit;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .screen {
    aspect-ratio: 16 / 9;
    border: 2px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
  }
  .screen.live { border-color: var(--live); }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
  button {
    min-height: 38px;
    padding: 0 14px;
    font: inherit;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
  }
  button.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
  button:disabled { opacity: 0.5; cursor: default; }
</style>
