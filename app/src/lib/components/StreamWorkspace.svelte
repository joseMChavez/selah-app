<script lang="ts">
  // Hilo de Transmisión: controla OBS. Es independiente de la proyección (regla de los dos hilos):
  // lo único que cruza es «Copiar lo que está en la pantalla del templo», y es una copia fija.
  import { onMount } from "svelte";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { connect, detect, disconnect, formatDuration, launchAndConnect, obs, SCENES } from "$lib/obs.svelte";
  import { platform, type ObsCamera, type OutputContent } from "$lib/platform";

  let { projectionLive, onerror }: { projectionLive: OutputContent; onerror: (message: Message) => void } = $props();

  const PREVIEW_MS = 1000;
  const TEXT_SOURCE_KEY = "selah.obsTextSource";
  const SELAH_TEXT_SOURCE = "Selah · Texto";
  const STOP_CONFIRM_MS = 4000;

  const selahLayouts: { scene: string; label: MessageKey }[] = [
    { scene: SCENES.camera, label: "stream.layout.camera" },
    { scene: SCENES.cameraText, label: "stream.layout.cameraText" },
    { scene: SCENES.wait, label: "stream.layout.wait" },
  ];

  let busy = $state(false);
  let preview = $state<string | null>(null);
  let text = $state("");
  let credit = $state("");
  let note = $state("");
  let cameras = $state<ObsCamera[]>([]);
  let server = $state("");
  let key = $state("");
  let confirmStop = $state(false);
  let wasConnected = $state(false);
  let textSource = $state<string | null>(loadTextSource());

  let status = $derived(obs.status);
  let streaming = $derived(status?.streaming.active ?? false);
  let program = $derived(status?.programScene ?? null);
  // Las escenas de Selah con su nombre traducido; el resto, tal como se llaman en OBS.
  let selahScenes = $derived(selahLayouts.filter((l) => status?.scenes.includes(l.scene)));
  let otherScenes = $derived((status?.scenes ?? []).filter((s) => !selahLayouts.some((l) => l.scene === s)));
  let sources = $derived(status?.textSources ?? []);
  let source = $derived(
    textSource && sources.includes(textSource)
      ? textSource
      : sources.includes(SELAH_TEXT_SOURCE)
        ? SELAH_TEXT_SOURCE
        : (sources[0] ?? null),
  );

  function loadTextSource(): string | null {
    try {
      return localStorage.getItem(TEXT_SOURCE_KEY);
    } catch {
      return null;
    }
  }

  function chooseTextSource(name: string) {
    textSource = name;
    try {
      localStorage.setItem(TEXT_SOURCE_KEY, name);
    } catch {
      // Se usará la de por defecto la próxima vez.
    }
  }

  async function run(action: () => Promise<unknown>) {
    busy = true;
    try {
      await action();
    } catch (e) {
      onerror(toMessage(e));
    } finally {
      busy = false;
    }
  }

  const enable = () => run(async () => {
    await platform.obsEnableWebsocket();
    await detect();
  });
  const launch = () => run(launchAndConnect);
  const connectNow = () => run(connect);
  const createScenes = () => run(async () => {
    await platform.obsSetup(t("stream.waitMessage"));
    await loadCameras();
  });
  const setScene = (scene: string) => run(() => platform.obsSetScene(scene));

  async function loadCameras() {
    cameras = await platform.obsCameras().catch(() => []);
  }

  function copyFromProjection() {
    note = "";
    const c = projectionLive;
    if (c.kind === "scripture") {
      const verse = c.verses[c.index];
      text = `${verse.text}\n${verse.reference} · ${c.version}`;
      credit = c.copyright;
    } else if (c.kind === "text") {
      text = c.slides[c.index] ?? "";
      credit = "";
    } else {
      note = t("stream.text.nothingToCopy");
    }
  }

  const showText = () => run(async () => {
    if (!source) return;
    note = "";
    const shown = await platform.obsShowText(source, text, credit);
    if (shown.kind === "switched") note = t("stream.text.switched", { scene: shown.scene });
    else if (shown.kind === "notInScene") note = t("stream.text.notInScene");
  });
  const hideText = () => run(async () => {
    if (source) await platform.obsHideText(source);
  });

  const saveDestination = () => run(async () => {
    await platform.obsSetDestination(server.trim(), key.trim());
    key = "";
  });
  const start = () => run(() => platform.obsStartStream());

  // Finalizar pide un segundo clic: es la única acción de esta vista que no se puede deshacer.
  function stop() {
    if (!confirmStop) {
      confirmStop = true;
      setTimeout(() => (confirmStop = false), STOP_CONFIRM_MS);
      return;
    }
    confirmStop = false;
    run(() => platform.obsStopStream());
  }

  // Si se pierde la conexión, se avisa una vez.
  $effect(() => {
    if (obs.connected) wasConnected = true;
  });

  // Rellena el servidor con lo que ya tiene OBS, la primera vez que se conoce.
  $effect(() => {
    if (!server && status?.destination?.server) server = status.destination.server;
  });

  $effect(() => {
    if (status?.selahReady && cameras.length === 0) loadCameras();
  });

  onMount(() => {
    if (!obs.connected) detect().catch(() => {});
    // La vista previa solo se pide mientras esta vista está abierta.
    const timer = setInterval(async () => {
      if (!obs.connected) return;
      preview = await platform.obsPreview().catch(() => null);
    }, PREVIEW_MS);
    return () => clearInterval(timer);
  });
</script>

<div class="stream">
  <p class="independent muted small">{t("stream.independent")}</p>

  {#if !obs.connected || !status}
    <!-- Asistente: un paso cada vez, con una sola acción principal. -->
    <section class="card setup">
      <div class="card-head"><h2>{t("stream.setup.title")}</h2></div>
      <div class="card-body stack">
        {#if wasConnected && !obs.connected}<p class="alert">{t("stream.lost")}</p>{/if}
        <p>{t("stream.setup.intro")}</p>
        {#if !obs.detection}
          <p class="muted">…</p>
        {:else if !obs.detection.installed}
          <p>{t("stream.setup.notInstalled")}</p>
        {:else if !obs.detection.websocketEnabled}
          <p>{t("stream.setup.enableText")}</p>
          {#if obs.detection.running}<p class="alert">{t("stream.setup.closeFirst")}</p>{/if}
          <button class="btn primary big" onclick={enable} disabled={busy}>{t("stream.setup.enable")}</button>
        {:else if obs.detection.running}
          <p>{t("stream.setup.runningText")}</p>
          <button class="btn primary big" onclick={connectNow} disabled={busy}>{t("stream.setup.connect")}</button>
        {:else}
          <button class="btn primary big" onclick={launch} disabled={busy || obs.busy}>
            {obs.busy ? t("stream.setup.launching") : t("stream.setup.launch")}
          </button>
        {/if}
      </div>
    </section>
  {:else}
    <div class="grid">
      <section class="card">
        <div class="card-head">
          <h2>{t("stream.preview")}</h2>
          <span class="badge" class:on={streaming} class:warn={status.streaming.reconnecting}>
            {status.streaming.reconnecting ? t("stream.pill.reconnecting") : streaming ? t("stream.onAir") : t("stream.offAir")}
          </span>
        </div>
        <div class="card-body stack">
          <div class="screen" class:live={streaming}>
            {#if preview}
              <img src={preview} alt={t("stream.preview")} />
            {:else}
              <p class="empty muted">{t("stream.previewEmpty")}</p>
            {/if}
          </div>

          <h3>{t("stream.layouts")}</h3>
          {#if selahScenes.length > 0}
            <p class="group muted small">{t("stream.scenes.selah")}</p>
            <div class="layouts">
              {#each selahScenes as layout (layout.scene)}
                <button class="layout" class:current={program === layout.scene} onclick={() => setScene(layout.scene)} disabled={busy}>
                  <span>{t(layout.label)}</span>
                  {#if program === layout.scene}<span class="small">● {t("stream.layout.current")}</span>{/if}
                </button>
              {/each}
            </div>
          {/if}
          {#if otherScenes.length > 0}
            <p class="group muted small">{t("stream.scenes.obs")}</p>
            <div class="layouts">
              {#each otherScenes as scene (scene)}
                <button class="layout" class:current={program === scene} onclick={() => setScene(scene)} disabled={busy}>
                  <span>{scene}</span>
                  {#if program === scene}<span class="small">● {t("stream.layout.current")}</span>{/if}
                </button>
              {/each}
            </div>
          {/if}
          {#if !status.selahReady}
            <div class="optional">
              <p class="muted small">{t("stream.setup.scenesHint")}</p>
              <button class="btn" onclick={createScenes} disabled={busy}>{t("stream.setup.scenesOptional")}</button>
            </div>
          {/if}

          {#if cameras.length > 0}
            <label class="field-row">
              <span class="label">{t("stream.camera")}</span>
              <select class="field" onchange={(e) => run(() => platform.obsSetCamera(e.currentTarget.value))}>
                {#each cameras as cam (cam.id)}<option value={cam.id}>{cam.name}</option>{/each}
              </select>
            </label>
          {:else if !status.hasCamera}
            <p class="muted small">{t("stream.cameraNone")}</p>
          {/if}
        </div>
      </section>

      <div class="side">
        <section class="card">
          <div class="card-head"><h2>{t("stream.text.title")}</h2></div>
          <div class="card-body stack">
            <button class="btn" onclick={copyFromProjection}>{t("stream.text.copyLive")}</button>
            <p class="muted small">{t("stream.text.copyHint")}</p>
            {#if note}<p class="muted small">{note}</p>{/if}
            {#if sources.length === 0}
              <p class="muted small">{t("stream.text.noSources")}</p>
            {:else}
              <label class="field-row">
                <span class="label">{t("stream.text.source")}</span>
                <select class="field" value={source} onchange={(e) => chooseTextSource(e.currentTarget.value)}>
                  {#each sources as name (name)}<option value={name}>{name}</option>{/each}
                </select>
              </label>
            {/if}
            <textarea class="field" rows="4" bind:value={text} placeholder={t("stream.text.placeholder")}></textarea>
            {#if credit}
              <p class="credit small">
                {t("stream.text.credit", { credit })}
                <button class="btn ghost tiny" onclick={() => (credit = "")} aria-label="✕">✕</button>
              </p>
            {/if}
            <div class="row">
              <button class="btn primary" onclick={showText} disabled={busy || !source || !text.trim()}>{t("stream.text.show")}</button>
              <button class="btn" onclick={hideText} disabled={busy || !source}>{t("stream.text.hide")}</button>
            </div>
          </div>
        </section>

        <section class="card">
          <div class="card-head"><h2>{t("stream.destination")}</h2></div>
          <div class="card-body stack">
            <label class="field-row">
              <span class="label">{t("stream.server")}</span>
              <input class="field" bind:value={server} placeholder="rtmp://a.rtmp.youtube.com/live2" spellcheck="false" />
            </label>
            <label class="field-row">
              <span class="label">{t("stream.key")}</span>
              <input class="field" type="password" bind:value={key} autocomplete="off" />
            </label>
            {#if status.destination?.hasKey}<p class="muted small">{t("stream.keySaved")}</p>{/if}
            <button class="btn" onclick={saveDestination} disabled={busy || !server.trim()}>{t("stream.saveDestination")}</button>

            {#if streaming}
              <p class="stats small">
                {t("stream.stats", {
                  time: formatDuration(status.streaming.durationMs),
                  congestion: Math.round(status.streaming.congestion * 100),
                  skipped: status.streaming.skippedFrames,
                })}
              </p>
              <button class="btn danger big" onclick={stop} disabled={busy}>
                {confirmStop ? t("stream.stopConfirm") : t("stream.stop")}
              </button>
            {:else}
              {#if !status.destination?.server}<p class="muted small">{t("stream.noDestination")}</p>{/if}
              <button class="btn primary big" onclick={start} disabled={busy || !status.destination?.server}>{t("stream.start")}</button>
            {/if}
            <p class="muted small">{t("stream.obsKeeps")}</p>
            <button class="btn ghost small-btn" onclick={() => run(disconnect)} disabled={busy}>{t("stream.disconnect")}</button>
          </div>
        </section>
      </div>
    </div>
  {/if}
</div>

<style>
  .stream { display: flex; flex-direction: column; gap: 12px; min-height: 0; height: 100%; overflow: auto; }
  .independent { padding: 0 2px; }
  .setup { max-width: 640px; }
  .grid { display: grid; gap: 14px; grid-template-columns: minmax(0, 1.4fr) minmax(320px, 1fr); align-items: start; }
  .side { display: flex; flex-direction: column; gap: 14px; }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .card-head h2 { font-size: 16px; }
  .card-body { padding: 14px; }
  .stack { display: flex; flex-direction: column; gap: 10px; }
  h3 { font-size: 14px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; }
  .screen img { width: 100%; height: 100%; object-fit: contain; display: block; }
  .empty { position: absolute; inset: 0; display: grid; place-items: center; color: #9aa0a6; }
  .group { margin-top: 2px; }
  .optional { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; padding-top: 6px; border-top: 1px dashed var(--border); }
  .layouts { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 8px; }
  .layout {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-height: 56px;
    padding: 8px 12px;
    font: inherit;
    font-weight: 600;
    text-align: left;
    color: var(--text);
    background: var(--surface-2);
    border: 2px solid var(--border);
    border-radius: 10px;
    cursor: pointer;
  }
  .layout.current { border-color: var(--live); background: var(--live-soft); }
  .layout.current .small { color: var(--live); }
  .layout:disabled { opacity: 0.6; cursor: default; }
  .field-row { display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 13px; font-weight: 600; color: var(--muted); }
  textarea { resize: vertical; font-size: 15px; line-height: 1.45; }
  .row { display: flex; flex-wrap: wrap; gap: 8px; }
  .credit { display: flex; align-items: center; gap: 6px; color: var(--muted); }
  .btn.tiny { min-height: 26px; padding: 0 8px; }
  .btn.small-btn { align-self: flex-start; min-height: 32px; font-size: 13px; }
  .stats { font-variant-numeric: tabular-nums; }
  .alert { padding: 8px 12px; border-radius: 8px; background: var(--warn-bg); color: var(--warn-text); }
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
  .badge.warn { background: var(--warn-bg); color: var(--warn-text); }
  @media (max-width: 1100px) {
    .grid { grid-template-columns: minmax(0, 1fr); }
  }
</style>
