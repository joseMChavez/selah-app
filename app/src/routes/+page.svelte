<script lang="ts">
  import { onMount } from "svelte";
  import OutputView from "$lib/OutputView.svelte";
  import { platform, type MonitorInfo, type OutputContent, type OutputState } from "$lib/platform";

  const MONITOR_POLL_MS = 2000;

  let monitors = $state<MonitorInfo[]>([]);
  let selectedIndex = $state<number | null>(null);
  let output = $state<OutputState>({ open: false, monitorName: null, content: { kind: "empty" } });
  let draft = $state("Porque donde están dos o tres congregados en mi nombre,\nallí estoy yo en medio de ellos.\n— Mateo 18:20");
  let error = $state("");
  let busy = $state(false);

  let prepared = $derived<OutputContent>(draft.trim() ? { kind: "text", text: draft } : { kind: "empty" });
  let selected = $derived(monitors.find((m) => m.index === selectedIndex) ?? null);
  // Si el monitor de la salida desaparece, el compositor mueve la ventana a otro: hay que avisar.
  let outputMonitorLost = $derived(
    output.open && output.monitorName !== null && !monitors.some((m) => m.name === output.monitorName),
  );

  function describe(content: OutputContent): string {
    switch (content.kind) {
      case "empty": return "Nada (fondo negro sin contenido)";
      case "black": return "Pantalla en negro";
      case "testPattern": return "Patrón de prueba";
      case "text": return "Texto";
    }
  }

  async function run(action: () => Promise<unknown>) {
    busy = true;
    error = "";
    try {
      await action();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function refreshMonitors() {
    try {
      monitors = await platform.listMonitors();
    } catch (e) {
      error = `No se pudieron leer los monitores: ${e}`;
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
  const send = (content: OutputContent) => run(async () => {
    output = await platform.setOutputContent(content);
  });

  onMount(() => {
    refreshMonitors();
    platform.getOutputState().then((state) => (output = state));
    const unlisten = platform.onOutputState((state) => (output = state));
    const timer = setInterval(refreshMonitors, MONITOR_POLL_MS);
    return () => {
      clearInterval(timer);
      unlisten.then((fn) => fn());
    };
  });
</script>

<main>
  <header>
    <h1>Selah · Proyección</h1>
    <p class="status" class:live={output.open} role="status">
      {#if output.open}
        ● Salida abierta en <strong>{output.monitorName}</strong> · En pantalla: {describe(output.content)}
      {:else}
        ○ Salida cerrada
      {/if}
    </p>
  </header>

  {#if outputMonitorLost}
    <p class="alert" role="alert">
      El monitor «{output.monitorName}» ya no está conectado. La salida puede haberse movido a otra pantalla.
      Elige un monitor y pulsa «Abrir salida» de nuevo.
    </p>
  {/if}
  {#if error}<p class="alert" role="alert">{error}</p>{/if}

  <section>
    <h2>Monitor de salida</h2>
    {#if monitors.length === 0}
      <p>No se detectó ningún monitor.</p>
    {:else}
      <ul class="monitors">
        {#each monitors as m (m.index)}
          <li>
            <label>
              <input type="radio" name="monitor" value={m.index} bind:group={selectedIndex} />
              <span>
                <strong>{m.name}</strong>
                {m.width}×{m.height} · escala {m.scale} · posición {m.x},{m.y}
                {#if m.isPrimary}· principal{/if}
                {#if output.open && m.name === output.monitorName}· <em>salida aquí</em>{/if}
              </span>
            </label>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="actions">
      <button onclick={openOutput} disabled={busy || selected === null}>
        {output.open ? "Mover salida a" : "Abrir salida en"} {selected?.name ?? "…"}
      </button>
      <button onclick={closeOutput} disabled={busy || !output.open}>Cerrar salida</button>
    </div>
  </section>

  <div class="threads">
    <section>
      <h2>Preparado <span class="tag">vista previa, no visible</span></h2>
      <textarea bind:value={draft} rows="4" aria-label="Texto preparado"></textarea>
      <div class="screen"><OutputView content={prepared} /></div>
      <div class="actions">
        <button class="primary" onclick={() => send(prepared)} disabled={busy}>Enviar a pantalla</button>
      </div>
    </section>

    <section>
      <h2>En pantalla <span class="tag live">lo que ve la congregación</span></h2>
      <div class="screen live"><OutputView content={output.content} /></div>
      <div class="actions">
        <button onclick={() => send({ kind: "black" })} disabled={busy}>Pantalla en negro</button>
        <button onclick={() => send({ kind: "testPattern" })} disabled={busy}>Patrón de prueba</button>
        <button onclick={() => send({ kind: "empty" })} disabled={busy}>Quitar contenido</button>
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
