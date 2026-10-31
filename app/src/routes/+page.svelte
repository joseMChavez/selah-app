<script lang="ts">
  import { onMount } from "svelte";
  import OutputView, { DEFAULT_STYLE } from "$lib/OutputView.svelte";
  import BiblePanel from "$lib/components/BiblePanel.svelte";
  import MediaPanel from "$lib/components/MediaPanel.svelte";
  import SlideStrip from "$lib/components/SlideStrip.svelte";
  import StylePanel from "$lib/components/StylePanel.svelte";
  import TextPanel from "$lib/components/TextPanel.svelte";
  import { deckPosition, moveDeck } from "$lib/deck";
  import { i18n, locales, setLocale, t, type Locale, type MessageKey } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { pageCount } from "$lib/pdf";
  import { platform, type MonitorInfo, type OutputContent, type OutputState, type OutputStyle } from "$lib/platform";
  import { setTheme, theme, type ThemeChoice } from "$lib/theme.svelte";

  const MONITOR_POLL_MS = 2000;
  const TAB_KEY = "selah.tab";
  const EXPANDED_KEY = "selah.expanded";
  const STYLE_KEY = "selah.outputStyle";

  type Tab = "bible" | "text" | "media";
  // Iconos de trazo simple (24×24), heredan el color del texto.
  const tabs: { id: Tab; label: MessageKey; icon: string }[] = [
    { id: "bible", label: "nav.bible", icon: "M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2V5Zm0 16a2 2 0 0 1 2-2h13v2H6M12 7v6M9.5 9.5h5" },
    { id: "text", label: "nav.text", icon: "M4 6h16M4 10h16M4 14h10M4 18h7" },
    { id: "media", label: "nav.media", icon: "M4 5h16v14H4zM4 15l4.5-4.5 4 4 2.5-2.5L20 17M15.5 8.5h.01" },
  ];
  const themes: { id: ThemeChoice; label: MessageKey }[] = [
    { id: "system", label: "theme.system" },
    { id: "light", label: "theme.light" },
    { id: "dark", label: "theme.dark" },
  ];

  function load<T>(key: string, fallback: T, valid: (v: unknown) => boolean): T {
    try {
      const raw = localStorage.getItem(key);
      if (raw !== null) {
        const value = JSON.parse(raw);
        if (valid(value)) return value;
      }
    } catch {
      // Sin almacenamiento o dato dañado: valor por defecto.
    }
    return fallback;
  }

  function save(key: string, value: unknown) {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // No se recordará, pero sigue funcionando.
    }
  }

  let monitors = $state<MonitorInfo[]>([]);
  let selectedIndex = $state<number | null>(null);
  let output = $state<OutputState>({ open: false, monitorName: null, content: { kind: "empty" }, style: DEFAULT_STYLE });
  let prepared = $state<OutputContent>({ kind: "empty" });
  let error = $state<Message | null>(null);
  let busy = $state(false);
  let pdfPages = $state<Record<string, number>>({});
  let tab = $state<Tab>(load(TAB_KEY, "bible", (v) => tabs.some((x) => x.id === v)));
  /** La sección de contenido ocupa toda la ventana; una barra inferior sigue mostrando preparado y en pantalla. */
  let expanded = $state<boolean>(load(EXPANDED_KEY, false, (v) => typeof v === "boolean"));
  /** Lo que había antes de poner la pantalla en negro, para volver con un clic. */
  let beforeBlack = $state<OutputContent | null>(null);

  let selected = $derived(monitors.find((m) => m.index === selectedIndex) ?? null);
  // Si el monitor de la salida desaparece, el compositor mueve la ventana a otro: hay que avisar.
  let outputMonitorLost = $derived(
    output.open && output.monitorName !== null && !monitors.some((m) => m.name === output.monitorName),
  );
  let preparedPosition = $derived(deckPosition(prepared, pdfPages));
  let livePosition = $derived(deckPosition(output.content, pdfPages));

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function describe(content: OutputContent): string {
    switch (content.kind) {
      case "empty": return t("describe.empty");
      case "black": return t("describe.black");
      case "testPattern": return t("describe.testPattern");
      case "text": return t("describe.text", { n: content.index + 1, total: content.slides.length });
      case "scripture": return t("describe.scripture", { reference: content.verses[content.index]?.reference ?? "", version: content.version });
      case "image": return t("describe.image", { name: baseName(content.path) });
      case "video": return t(content.paused ? "describe.videoPaused" : "describe.video", { name: baseName(content.path) });
      case "pdf": return t("describe.pdf", { name: baseName(content.path), n: content.page });
    }
  }

  async function run(action: () => Promise<unknown>) {
    busy = true;
    error = null;
    try {
      await action();
    } catch (e) {
      error = toMessage(e);
      platform.logError(`comando fallido: ${JSON.stringify(e)}`).catch(() => {});
    } finally {
      busy = false;
    }
  }

  async function ensurePageCount(path: string) {
    if (pdfPages[path]) return;
    try {
      pdfPages[path] = await pageCount(platform.mediaUrl(path));
    } catch (e) {
      error = { key: "error.pdfOpen", params: { name: baseName(path), error: String(e) } };
    }
  }

  function prepare(content: OutputContent) {
    prepared = content;
    if (content.kind === "pdf") ensurePageCount(content.path);
  }

  function chooseTab(id: Tab) {
    tab = id;
    save(TAB_KEY, id);
  }

  function setExpanded(value: boolean) {
    expanded = value;
    save(EXPANDED_KEY, value);
  }

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
    if (content.kind !== "black") beforeBlack = null;
    output = await platform.setOutputContent(content);
  });

  function toggleBlack() {
    if (output.content.kind === "black") {
      if (beforeBlack) send(beforeBlack);
      return;
    }
    const previous = output.content;
    send({ kind: "black" }).then(() => (beforeBlack = previous));
  }

  /** Cambia directamente lo que está en pantalla (n empieza en 1). */
  function goLive(n: number) {
    if (busy) return;
    const next = moveDeck(output.content, n, pdfPages);
    if (next) send(next);
  }

  function goPrepared(n: number) {
    const next = moveDeck(prepared, n, pdfPages);
    if (next) prepared = next;
  }

  function changeStyle(style: OutputStyle) {
    save(STYLE_KEY, style);
    run(async () => {
      output = await platform.setOutputStyle(style);
    });
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && expanded) {
      setExpanded(false);
      return;
    }
    // No interferir mientras se escribe. En los controles de selección sí se navega.
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
    platform.getOutputState().then((state) => {
      output = state;
      // El estilo elegido la última vez se aplica al arrancar.
      const saved = load<OutputStyle | null>(STYLE_KEY, null, (v) => typeof v === "object" && v !== null && "fontScale" in v);
      if (saved) changeStyle({ ...DEFAULT_STYLE, ...saved });
    });
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

<div class="app">
  <header class="topbar">
    <div class="brand">Selah</div>

    <p class="status" class:on={output.open} role="status">
      <span class="dot" aria-hidden="true"></span>
      {output.open ? t("status.live", { monitor: output.monitorName ?? "" }) : t("status.closed")}
    </p>

    <div class="output-controls">
      <label class="inline">
        <span class="muted small">{t("topbar.output")}</span>
        <select class="field compact" bind:value={selectedIndex} disabled={monitors.length === 0}>
          {#if monitors.length === 0}<option value={null}>{t("monitors.none")}</option>{/if}
          {#each monitors as m (m.index)}
            <option value={m.index}>
              {t("monitors.option", { name: m.name, width: m.width, height: m.height })}
              {m.isPrimary ? t("monitors.primary") : ""}
            </option>
          {/each}
        </select>
      </label>
      {#if !output.open}
        <button class="btn primary" onclick={openOutput} disabled={busy || selected === null}>{t("topbar.open")}</button>
      {:else}
        {#if selected && selected.name !== output.monitorName}
          <button class="btn" onclick={openOutput} disabled={busy}>{t("topbar.move")}</button>
        {/if}
        <button class="btn" onclick={closeOutput} disabled={busy}>{t("topbar.close")}</button>
      {/if}
    </div>

    <div class="prefs">
      <label class="inline">
        <span class="muted small">{t("settings.language")}</span>
        <select class="field compact" value={i18n.locale} onchange={(e) => setLocale(e.currentTarget.value as Locale)}>
          {#each Object.entries(locales) as [code, locale] (code)}
            <option value={code}>{locale.name}</option>
          {/each}
        </select>
      </label>
      <label class="inline">
        <span class="muted small">{t("settings.theme")}</span>
        <select class="field compact" value={theme.choice} onchange={(e) => setTheme(e.currentTarget.value as ThemeChoice)}>
          {#each themes as th (th.id)}
            <option value={th.id}>{t(th.label)}</option>
          {/each}
        </select>
      </label>
    </div>
  </header>

  {#if outputMonitorLost || error}
    <div class="alerts">
      {#if outputMonitorLost}
        <p class="alert" role="alert">{t("monitors.lost", { monitor: output.monitorName ?? "" })}</p>
      {/if}
      {#if error}
        <p class="alert" role="alert">
          <span>{t(error.key, error.params)}</span>
          <button class="btn ghost" onclick={() => (error = null)} aria-label={t("error.dismiss")}>✕</button>
        </p>
      {/if}
    </div>
  {/if}

  <div class="body">
  <nav class="sidebar" aria-label={t("sources.title")}>
    {#each tabs as tb (tb.id)}
      <button class="nav-item" aria-current={tab === tb.id ? "page" : undefined} onclick={() => chooseTab(tb.id)}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d={tb.icon} /></svg>
        <span>{t(tb.label)}</span>
      </button>
    {/each}
  </nav>

  <main class="workspace" class:expanded>
    <section class="card sources" aria-labelledby="sources-title">
      <div class="card-head">
        <h2 id="sources-title">{t(tabs.find((x) => x.id === tab)!.label)}</h2>
        <button class="btn ghost" onclick={() => setExpanded(!expanded)} title={expanded ? undefined : t("layout.expandHint")}>
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d={expanded ? "M9 4v5H4M15 4v5h5M9 20v-5H4M15 20v-5h5" : "M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"} />
          </svg>
          {expanded ? t("layout.collapse") : t("layout.expand")}
        </button>
      </div>
      <!-- Los paneles se ocultan en vez de destruirse, para no perder la búsqueda ni el texto al cambiar de pestaña. -->
      <div class="card-body" hidden={tab !== "bible"}>
        <BiblePanel
          {prepared}
          live={output.content}
          onprepare={prepare}
          onsend={(content) => {
            prepare(content);
            send(content);
          }}
          onerror={(m) => (error = m)}
        />
      </div>
      <div class="card-body" hidden={tab !== "text"}>
        <TextPanel onprepare={prepare} />
      </div>
      <div class="card-body" hidden={tab !== "media"}>
        <MediaPanel {prepared} onprepare={prepare} onerror={(m) => (error = m)} />
      </div>
    </section>

    <section class="card prepared" aria-labelledby="prepared-title">
      <div class="card-head">
        <h2 id="prepared-title">{t("prepared.title")}</h2>
        <span class="muted small">{t("prepared.tag")}</span>
      </div>
      <div class="card-body stack">
        <div class="screen"><OutputView content={prepared} style={output.style} preview /></div>
        {#if prepared.kind === "empty"}
          <p class="muted">{t("prepared.empty")}</p>
        {/if}
        {#if preparedPosition}
          <div class="nav-row">
            <button class="btn" onclick={() => goPrepared(preparedPosition!.n - 1)} disabled={preparedPosition.n <= 1}>◀</button>
            <span class="position">{t(preparedPosition.label, { n: preparedPosition.n, total: preparedPosition.total ?? "…" })}</span>
            <button class="btn" onclick={() => goPrepared(preparedPosition!.n + 1)} disabled={preparedPosition.total !== null && preparedPosition.n >= preparedPosition.total}>▶</button>
          </div>
        {/if}
        <SlideStrip content={prepared} style={output.style} tone="prepared" onselect={goPrepared} />
        <button class="btn primary big send" onclick={() => send(prepared)} disabled={busy || prepared.kind === "empty"}>
          {t("prepared.send")} →
        </button>
      </div>
    </section>

    <section class="card live" aria-labelledby="live-title">
      <div class="card-head">
        <h2 id="live-title">{t("live.title")}</h2>
        <span class="badge" class:on={output.open}>{output.open ? t("live.badge") : t("live.closedBadge")}</span>
      </div>
      <div class="card-body stack">
        <div class="screen live"><OutputView content={output.content} style={output.style} preview /></div>
        <p class="muted small">{t("live.tag")}: <strong>{describe(output.content)}</strong></p>
        {#if !output.open}<p class="muted small">{t("live.closedHint")}</p>{/if}

        {#if livePosition}
          <div class="nav-row">
            <button class="btn big" onclick={() => goLive(livePosition!.n - 1)} disabled={busy || livePosition.n <= 1}>{t("live.previous")}</button>
            <span class="position">{t(livePosition.label, { n: livePosition.n, total: livePosition.total ?? "…" })}</span>
            <button class="btn big" onclick={() => goLive(livePosition!.n + 1)} disabled={busy || (livePosition.total !== null && livePosition.n >= livePosition.total)}>{t("live.next")}</button>
          </div>
          <SlideStrip content={output.content} style={output.style} tone="live" disabled={busy} onselect={goLive} />
          <p class="muted small">{t("live.keysHint")}</p>
        {/if}

        <div class="quick">
          {#if output.content.kind === "video"}
            <button class="btn" onclick={() => output.content.kind === "video" && send({ ...output.content, paused: !output.content.paused })} disabled={busy}>
              {t(output.content.paused ? "live.resume" : "live.pause")}
            </button>
          {/if}
          {#if output.content.kind === "black" && beforeBlack}
            <button class="btn danger" onclick={toggleBlack} disabled={busy}>{t("live.unblack")}</button>
          {:else}
            <button class="btn" onclick={toggleBlack} disabled={busy || output.content.kind === "black"}>{t("live.black")}</button>
          {/if}
          <button class="btn" onclick={() => send({ kind: "empty" })} disabled={busy || output.content.kind === "empty"}>{t("live.clear")}</button>
        </div>

        <details class="style">
          <summary>{t("style.title")}</summary>
          <StylePanel style={output.style} onchange={changeStyle} />
          <button class="btn ghost small" onclick={() => send({ kind: "testPattern" })} disabled={busy}>{t("style.testPattern")}</button>
        </details>
      </div>
    </section>
  </main>
  </div>

  {#if expanded}
    <!-- Modo ampliado: lo esencial de «Preparado» y «En pantalla» sigue a mano. -->
    <footer class="dock">
      <div class="dock-part">
        <div class="mini screen"><OutputView content={prepared} style={output.style} preview /></div>
        <div class="dock-info">
          <span class="dock-title">{t("prepared.title")}</span>
          {#if preparedPosition}
            <span class="muted small">{t(preparedPosition.label, { n: preparedPosition.n, total: preparedPosition.total ?? "…" })}</span>
          {/if}
        </div>
        <button class="btn primary big" onclick={() => send(prepared)} disabled={busy || prepared.kind === "empty"}>
          {t("prepared.send")} →
        </button>
      </div>
      <div class="dock-part">
        <div class="mini screen live"><OutputView content={output.content} style={output.style} preview /></div>
        <div class="dock-info">
          <span class="badge" class:on={output.open}>{output.open ? t("live.badge") : t("live.closedBadge")}</span>
          <span class="small"><strong>{describe(output.content)}</strong></span>
        </div>
        {#if livePosition}
          <button class="btn big" onclick={() => goLive(livePosition!.n - 1)} disabled={busy || livePosition.n <= 1} aria-label={t("live.previous")}>◀</button>
          <button class="btn big" onclick={() => goLive(livePosition!.n + 1)} disabled={busy || (livePosition.total !== null && livePosition.n >= livePosition.total)} aria-label={t("live.next")}>▶</button>
        {/if}
        {#if output.content.kind === "black" && beforeBlack}
          <button class="btn danger big" onclick={toggleBlack} disabled={busy}>{t("live.unblack")}</button>
        {:else}
          <button class="btn big" onclick={toggleBlack} disabled={busy || output.content.kind === "black"}>{t("live.black")}</button>
        {/if}
      </div>
    </footer>
  {/if}
</div>

<style>
  .app { height: 100vh; display: flex; flex-direction: column; }

  .topbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px 18px;
    padding: 10px 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }
  .brand { font-size: 19px; font-weight: 750; letter-spacing: 0.01em; }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    font-size: 14px;
    font-weight: 600;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--muted);
  }
  .status .dot { width: 9px; height: 9px; border-radius: 50%; background: var(--muted); }
  .status.on { background: var(--live-soft); color: var(--live); }
  .status.on .dot { background: var(--live); box-shadow: 0 0 0 3px rgb(239 107 99 / 0.25); }
  .output-controls, .prefs { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .prefs { margin-left: auto; }
  .inline { display: flex; align-items: center; gap: 6px; }
  .field.compact { width: auto; min-height: 36px; padding: 4px 8px; }

  .alerts { padding: 10px 16px 0; display: flex; flex-direction: column; gap: 6px; }
  .alert {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 8px 8px 8px 14px;
    border-radius: 8px;
    background: var(--warn-bg);
    color: var(--warn-text);
  }
  .alert .btn { min-height: 30px; padding: 0 10px; color: inherit; border-color: transparent; }

  .body { flex: 1; min-height: 0; display: flex; }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px 8px;
    background: var(--surface);
    border-right: 1px solid var(--border);
  }
  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 76px;
    padding: 10px 4px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
    background: none;
    border: 0;
    border-radius: 10px;
    cursor: pointer;
  }
  .nav-item:hover { background: var(--surface-2); color: var(--text); }
  .nav-item[aria-current="page"] { background: var(--accent-soft); color: var(--text); }
  .nav-item[aria-current="page"] svg { stroke: var(--accent); }
  svg { width: 26px; height: 26px; fill: none; stroke: currentColor; stroke-width: 1.7; stroke-linecap: round; stroke-linejoin: round; }
  svg.icon { width: 18px; height: 18px; }

  .workspace {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: grid;
    gap: 14px;
    padding: 14px 16px 16px;
    grid-template-columns: minmax(340px, 0.95fr) minmax(0, 1fr) minmax(0, 1fr);
    grid-template-areas: "sources prepared live";
  }
  .sources { grid-area: sources; }
  .prepared { grid-area: prepared; }
  .live { grid-area: live; }
  .card { display: flex; flex-direction: column; min-height: 0; }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .card-head h2 { font-size: 16px; }
  .card-body { flex: 1; min-height: 0; overflow: auto; padding: 14px; }
  .card-body[hidden] { display: none; }
  .stack { display: flex; flex-direction: column; gap: 12px; }

  .nav-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .position { font-weight: 600; font-variant-numeric: tabular-nums; text-align: center; }
  .send { width: 100%; margin-top: auto; }
  .quick { display: flex; flex-wrap: wrap; gap: 8px; }

  .badge {
    font-size: 12px;
    font-weight: 750;
    letter-spacing: 0.06em;
    padding: 3px 8px;
    border-radius: 5px;
    background: var(--surface-2);
    color: var(--muted);
  }
  .badge.on { background: var(--live); color: #fff; }

  .workspace.expanded { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr); grid-template-areas: "sources"; }
  .workspace.expanded .prepared,
  .workspace.expanded .live { display: none; }
  .workspace.expanded .sources { position: static; max-height: none; }

  .dock {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 24px;
    padding: 10px 16px;
    background: var(--surface);
    border-top: 1px solid var(--border);
  }
  .dock-part { display: flex; align-items: center; gap: 12px; flex: 1 1 420px; min-width: 0; }
  .dock .mini { width: 150px; flex: none; }
  .dock-info { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1; }
  .dock-title { font-weight: 650; }

  .style { border-top: 1px solid var(--border); padding-top: 10px; }
  .style summary { cursor: pointer; font-weight: 650; padding: 4px 0 10px; }
  .style :global(.btn.small) { margin-top: 10px; font-size: 13px; min-height: 32px; }

  /* Ventana mediana: «Preparado» y «En pantalla» se apilan junto al contenido. */
  @media (max-width: 1280px) {
    .workspace {
      grid-template-columns: minmax(320px, 1fr) minmax(0, 1.1fr);
      grid-template-rows: auto auto;
      grid-template-areas: "sources prepared" "sources live";
      overflow: auto;
    }
    .workspace .card { min-height: auto; }
    .sources { min-height: 0; max-height: calc(100vh - 110px); position: sticky; top: 0; }
  }

  /* Ventana estrecha: una columna; la página entera se desplaza. */
  @media (max-width: 860px) {
    .app { height: auto; min-height: 100vh; }
    .workspace {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "sources" "prepared" "live";
      overflow: visible;
      padding: 12px;
    }
    .sources { position: static; max-height: none; }
    .card-body { overflow: visible; }
    .body { flex-direction: column; }
    .sidebar { flex-direction: row; justify-content: center; padding: 6px; border-right: 0; border-bottom: 1px solid var(--border); }
    .nav-item { flex-direction: row; width: auto; padding: 8px 14px; }
    .dock { position: sticky; bottom: 0; }
    .prefs { margin-left: 0; }
  }
</style>
