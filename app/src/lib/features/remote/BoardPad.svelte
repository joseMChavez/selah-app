<script lang="ts">
  // La pizarra en el teléfono (D-021): se dibuja con el dedo sobre lo que está en pantalla (una página, un versículo,
  // una diapositiva…) o sobre hojas en blanco, y se ve en el proyector mientras se dibuja.
  // Los trazos se mandan en trozos pequeños (unas 25 veces por segundo); la pizarra la guarda el backend.
  import { onMount } from "svelte";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import {
    COLORS,
    COORD,
    drawLayer,
    inkColor,
    MAX_BATCH,
    MAX_SHEETS,
    newId,
    PAPER_COLOR,
    redrawOps,
    SIZES,
    strokesAt,
    strokeWidth,
    type Board,
    type BoardOp,
    type InkColor,
    type InkSize,
    type InkTool,
    type Stroke,
  } from "./board";
  import type { RemoteCommand, RemoteStatus } from "./protocol";

  let {
    status,
    board,
    version,
    online,
    summary,
    thumbUrl,
    act,
    command,
    onclose,
  }: {
    status: RemoteStatus;
    board: Board;
    /** Cambia con cada cambio de la pizarra (los trazos no son reactivos: son miles de números). */
    version: number;
    online: boolean;
    /** Lo que está en pantalla, en pocas palabras. */
    summary: string;
    thumbUrl: (doc: string, n: number) => string;
    /** Aplica un cambio aquí y lo manda al servidor. */
    act: (op: BoardOp) => void;
    command: (c: RemoteCommand) => void;
    onclose: () => void;
  } = $props();

  /** Cada cuánto se manda lo dibujado mientras se dibuja. */
  const FLUSH_MS = 40;
  /** Distancia mínima entre puntos, en píxeles del teléfono: menos es ruido y más datos. */
  const MIN_STEP = 1.5;
  /** Alcance del borrador alrededor del dedo, en píxeles. */
  const ERASER_RADIUS = 14;
  /** Un trazo larguísimo se corta aquí (igual que MAX_STROKE en board.rs). */
  const MAX_POINTS = 20_000;
  const WIPE_CONFIRM_MS = 3000;
  const ICONS = {
    close: "M6 6l12 12M18 6 6 18",
    pen: "M4 20l1-4L16 5l3 3L8 19l-4 1ZM14 7l3 3",
    marker: "M5 19h6M9 14l-3 3 2 2 3-3M9 14l7-9 4 3-7 9-4-3Z",
    eraser: "M15 4l5 5-10 10H6l-3-3L15 4ZM9 19h11M11 8l5 5",
    undo: "M9 14 4 9l5-5M4 9h10a6 6 0 0 1 0 12h-3",
    trash: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
  };

  type Tool = InkTool | "eraser";
  /** Lo que se puede deshacer: un trazo que se dibujó, o trazos que se borraron (con el borrador o «Borrar todo»). */
  type Undo = { layer: string; added: string | null; removed: Stroke[] };

  let tool = $state<Tool>("pen");
  let color = $state<InkColor>("red");
  let size = $state<InkSize>("medium");
  let undos = $state<Undo[]>([]);
  let wipeArmed = $state(false);
  let wipeTimer: ReturnType<typeof setTimeout> | undefined;

  let ink = $state<HTMLCanvasElement>();
  let under = $state<HTMLCanvasElement>();
  let width = $state(0);
  let height = $state(0);

  /** El trazo que se está dibujando: va por delante de lo ya enviado. */
  let current: { layer: string; stroke: Stroke; sent: number; pointer: number } | null = null;
  /** El borrado en curso: lo quitado en un solo gesto se deshace junto. */
  let erasing: { layer: string; removed: Stroke[]; pointer: number } | null = null;
  let flushTimer: ReturnType<typeof setInterval> | undefined;
  let frame = 0;

  let canvas = $derived(status.canvas);
  let layer = $derived(canvas?.layer ?? null);
  let sheetId = $derived(canvas?.under.kind === "sheet" ? canvas.under.id : null);
  let sheets = $derived.by(() => {
    void version;
    return board.sheets.map((s) => ({ ...s }));
  });
  let paper = $derived(sheetId ? (sheets.find((s) => s.id === sheetId)?.paper ?? "black") : null);
  /** Cómo es la pantalla del proyector, medida por la ventana de salida. */
  let measured = $derived.by(() => {
    void version;
    return board.screen;
  });
  let aspect = $derived(measured && measured.aspect > 0.5 && measured.aspect < 4 ? measured.aspect : 16 / 9);
  /** Las palabras medidas en la pantalla, si son de lo que está ahora. */
  let words = $derived(measured && layer && measured.layer === layer && measured.words.length > 0 ? measured.words : null);
  let canUndo = $derived(undos.some((u) => u.layer === layer));
  let hasStrokes = $derived.by(() => {
    void version;
    return layer !== null && (board.layers[layer]?.length ?? 0) > 0;
  });
  /** Para volver a lo que había antes de la hoja: lo último que estuvo en pantalla. */
  let back = $derived(sheetId ? (status.items.find((i) => !i.prepared) ?? null) : null);

  function sheetName(id: string): string {
    return t("remote.board.sheet", { n: sheets.findIndex((s) => s.id === id) + 1 });
  }

  function chooseColor(c: InkColor) {
    color = c;
    if (tool === "eraser") tool = "pen";
    // En una hoja, el fondo cambia para que la tinta elegida se vea: negro sobre blanco, blanco sobre negro.
    if (sheetId && paper === "black" && c === "black") act({ type: "setPaper", id: sheetId, paper: "white" });
    if (sheetId && paper === "white" && c === "white") act({ type: "setPaper", id: sheetId, paper: "black" });
  }

  /** El color de la muestra: lo que se verá al dibujar. El blanco y el negro se muestran tal cual se eligen. */
  function swatch(c: InkColor): string {
    return c === "white" ? "#ffffff" : c === "black" ? "#111111" : inkColor(c, paper);
  }

  function newSheet() {
    if (sheets.length >= MAX_SHEETS) return;
    const id = newId();
    act({ type: "addSheet", id });
    command({ type: "showSheet", id });
    // Hoja negra: se empieza con la pluma y tinta blanca, lo que mejor se ve proyectado.
    color = "white";
    tool = "pen";
  }

  function strokes(): Stroke[] {
    return (layer && board.layers[layer]) || [];
  }

  // ---- Dibujar ----

  function position(event: PointerEvent): [number, number] {
    const rect = ink!.getBoundingClientRect();
    return [event.clientX - rect.left, event.clientY - rect.top];
  }

  function toCoord(px: number, size: number): number {
    return Math.max(-1000, Math.min(COORD + 1000, Math.round((px / size) * COORD)));
  }

  function down(event: PointerEvent) {
    if (!event.isPrimary || !layer || !online || !ink || current || erasing) return;
    event.preventDefault();
    ink.setPointerCapture(event.pointerId);
    const [x, y] = position(event);
    if (tool === "eraser") {
      erasing = { layer, removed: [], pointer: event.pointerId };
      eraseAt(x, y);
      return;
    }
    const stroke: Stroke = { id: newId(), tool, color, size, points: [toCoord(x, width), toCoord(y, height)] };
    current = { layer, stroke, sent: 0, pointer: event.pointerId };
    flush();
    clearInterval(flushTimer);
    flushTimer = setInterval(flush, FLUSH_MS);
    redraw();
  }

  function move(event: PointerEvent) {
    if (erasing?.pointer === event.pointerId) {
      for (const e of event.getCoalescedEvents?.() ?? [event]) eraseAt(...position(e));
      return;
    }
    if (current?.pointer !== event.pointerId) return;
    const points = current.stroke.points;
    for (const e of event.getCoalescedEvents?.() ?? [event]) {
      if (points.length >= MAX_POINTS) break;
      const [x, y] = position(e);
      const lx = (points[points.length - 2] / COORD) * width;
      const ly = (points[points.length - 1] / COORD) * height;
      if (Math.hypot(x - lx, y - ly) < MIN_STEP) continue;
      points.push(toCoord(x, width), toCoord(y, height));
    }
    redraw();
  }

  function up(event: PointerEvent) {
    if (erasing?.pointer === event.pointerId) {
      if (erasing.removed.length > 0) remember({ layer: erasing.layer, added: null, removed: erasing.removed });
      erasing = null;
      return;
    }
    if (current?.pointer !== event.pointerId) return;
    flush();
    clearInterval(flushTimer);
    remember({ layer: current.layer, added: current.stroke.id, removed: [] });
    current = null;
    redraw();
  }

  /** Manda lo dibujado desde el último envío. El primer envío crea el trazo (at = 0). */
  function flush() {
    if (!current) return;
    const { stroke } = current;
    while (current.sent < stroke.points.length || current.sent === 0) {
      const points = stroke.points.slice(current.sent, current.sent + MAX_BATCH);
      act({ type: "draw", layer: current.layer, stroke: stroke.id, tool: stroke.tool, color: stroke.color, size: stroke.size, at: current.sent, points });
      current.sent += points.length;
      if (points.length === 0) break;
    }
  }

  function eraseAt(x: number, y: number) {
    if (!erasing) return;
    for (const stroke of strokesAt(strokes(), x, y, width, height, ERASER_RADIUS)) {
      erasing.removed.push({ ...stroke, points: [...stroke.points] });
      act({ type: "erase", layer: erasing.layer, stroke: stroke.id });
    }
  }

  function remember(entry: Undo) {
    undos.push(entry);
    // Solo lo último: deshacer es para el error de cada momento.
    if (undos.length > 50) undos.shift();
  }

  function undo() {
    const index = undos.findLastIndex((u) => u.layer === layer);
    if (index < 0) return;
    const [entry] = undos.splice(index, 1);
    if (entry.added) act({ type: "erase", layer: entry.layer, stroke: entry.added });
    for (const stroke of entry.removed) for (const op of redrawOps(entry.layer, stroke)) act(op);
  }

  /** «Borrar todo» pregunta con un segundo toque: no se puede recuperar salvo con Deshacer. */
  function wipe() {
    if (!layer) return;
    if (!wipeArmed) {
      wipeArmed = true;
      clearTimeout(wipeTimer);
      wipeTimer = setTimeout(() => (wipeArmed = false), WIPE_CONFIRM_MS);
      return;
    }
    wipeArmed = false;
    clearTimeout(wipeTimer);
    const removed = strokes().map((s) => ({ ...s, points: [...s.points] }));
    if (removed.length > 0) remember({ layer, added: null, removed });
    act({ type: "wipe", layer });
  }

  // ---- Pintar ----

  function redraw() {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(paint);
  }

  function sized(c: HTMLCanvasElement): CanvasRenderingContext2D | null {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(width * dpr);
    const h = Math.round(height * dpr);
    if (c.width !== w) c.width = w;
    if (c.height !== h) c.height = h;
    return c.getContext("2d");
  }

  function paint() {
    if (!ink || width === 0 || height === 0) return;
    const ctx = sized(ink);
    if (!ctx) return;
    const list = current && current.layer === layer ? [...strokes().filter((s) => s.id !== current!.stroke.id), current.stroke] : strokes();
    drawLayer(ctx, list, ink.width, ink.height, paper);
  }

  /** El texto de la pantalla, cada palabra donde quedó en el proyector: así se subraya justo encima. */
  function paintWords() {
    if (!under || !words || !measured || width === 0 || height === 0) return;
    const ctx = sized(under);
    if (!ctx) return;
    const w = under.width;
    const h = under.height;
    ctx.fillStyle = measured.paper;
    ctx.fillRect(0, 0, w, h);
    ctx.textBaseline = "alphabetic";
    for (const word of words) {
      const x = (word.x / COORD) * w;
      const y = (word.y / COORD) * h;
      const ww = (word.w / COORD) * w;
      const wh = (word.h / COORD) * h;
      ctx.font = `${word.k === "ref" ? 650 : 500} ${wh / 1.2}px system-ui, sans-serif`;
      ctx.fillStyle = word.k === "ref" ? "#f2d58a" : word.k === "second" ? "rgb(245 245 240 / 0.88)" : "#f5f5f0";
      const natural = ctx.measureText(word.t).width;
      ctx.save();
      ctx.translate(x, y + wh * 0.8);
      // Otro tipo de letra que en el proyector: se estira o encoge para ocupar el mismo sitio.
      if (natural > 0) ctx.scale(ww / natural, 1);
      ctx.fillText(word.t, 0, 0);
      ctx.restore();
    }
  }

  $effect(() => {
    void [version, layer, paper, width, height, ink];
    redraw();
  });

  $effect(() => {
    void [words, width, height, under];
    paintWords();
  });

  // Si cambia lo que está en pantalla a mitad de un trazo, el trazo termina donde estaba.
  $effect(() => {
    void layer;
    return () => {
      if (current) up(new PointerEvent("pointerup", { pointerId: current.pointer }));
      erasing = null;
    };
  });

  onMount(() => {
    // Mientras se dibuja, la página no se desplaza.
    const root = document.documentElement;
    const overflow = root.style.overflow;
    root.style.overflow = "hidden";
    // En horizontal el lienzo es más grande. Solo se puede fijar a pantalla completa (Android); si no, no pasa nada.
    const orientation = globalThis.screen?.orientation as (ScreenOrientation & { lock?: (o: string) => Promise<void> }) | undefined;
    if (document.fullscreenElement) orientation?.lock?.("landscape").catch(() => {});
    return () => {
      root.style.overflow = overflow;
      clearInterval(flushTimer);
      clearTimeout(wipeTimer);
      cancelAnimationFrame(frame);
      if (document.fullscreenElement) orientation?.unlock?.();
    };
  });
</script>

<div class="pad" role="dialog" aria-modal="true" aria-label={t("remote.board.title")}>
  <header>
    <button class="tool close" onclick={onclose} aria-label={t("remote.board.close")}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.close} /></svg>
    </button>
    <ol class="targets" aria-label={t("remote.board.where")}>
      {#if canvas && !sheetId}
        <li><span class="target" aria-current="true"><span class="dot" aria-hidden="true"></span>{summary}</span></li>
      {/if}
      {#if back}
        <li><button class="target" onclick={() => command({ type: "show", key: back.key })} disabled={!online}>↩ {back.label}</button></li>
      {/if}
      {#each sheets as sheet (sheet.id)}
        <li>
          <button class="target" aria-current={sheet.id === sheetId} onclick={() => sheet.id !== sheetId && command({ type: "showSheet", id: sheet.id })} disabled={!online}>
            {#if sheet.id === sheetId}<span class="dot" aria-hidden="true"></span>{/if}{sheetName(sheet.id)}
          </button>
        </li>
      {/each}
      <li>
        <button class="target add" onclick={newSheet} disabled={!online || sheets.length >= MAX_SHEETS}>{t("remote.board.newSheet")}</button>
      </li>
    </ol>
  </header>

  <div class="stage">
    {#if canvas}
      <div
        class="frame"
        style:--aspect={aspect}
        style:background={paper ? PAPER_COLOR[paper] : (measured?.paper ?? "#000")}
        bind:clientWidth={width}
        bind:clientHeight={height}
      >
        {#if canvas.under.kind === "page"}
          {#key `${canvas.under.doc}-${canvas.under.n}`}
            <img class="under" src={thumbUrl(canvas.under.doc, canvas.under.n)} alt="" />
          {/key}
        {:else if canvas.under.kind === "text"}
          {#if words}
            <canvas class="under" bind:this={under}></canvas>
          {:else}
            <!-- Sin la medida del proyector (salida cerrada): el texto, pero no exactamente donde está allí. -->
            <p class="approx">{canvas.under.text}</p>
          {/if}
        {:else if canvas.under.kind === "other"}
          <p class="approx muted">{canvas.under.description}</p>
        {/if}
        <canvas
          class="ink"
          class:erasing={tool === "eraser"}
          bind:this={ink}
          onpointerdown={down}
          onpointermove={move}
          onpointerup={up}
          onpointercancel={up}
        ></canvas>
      </div>
      <p class="turn">{t("remote.board.turn")}</p>
    {:else}
      <div class="empty">
        <p>{t(status.live.kind === "black" ? "remote.black" : "remote.board.nothing")}</p>
        <button class="big-add" onclick={newSheet} disabled={!online || sheets.length >= MAX_SHEETS}>{t("remote.board.newSheet")}</button>
      </div>
    {/if}
  </div>

  <div class="tools">
    <div class="group" role="radiogroup" aria-label={t("remote.board.tool")}>
      {#each [["pen", "remote.board.pen"], ["marker", "remote.board.marker"], ["eraser", "remote.board.eraser"]] as [id, label] (id)}
        <button class="tool" role="radio" aria-checked={tool === id} aria-label={t(label as MessageKey)} onclick={() => (tool = id as Tool)}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS[id as Tool]} /></svg>
        </button>
      {/each}
    </div>
    <div class="group" role="radiogroup" aria-label={t("remote.board.color")}>
      {#each COLORS as c (c)}
        <button
          class="swatch"
          role="radio"
          aria-checked={tool !== "eraser" && color === c}
          aria-label={t(`remote.board.color.${c}` as MessageKey)}
          style:--ink={swatch(c)}
          onclick={() => chooseColor(c)}
        ></button>
      {/each}
    </div>
    <div class="group" role="radiogroup" aria-label={t("remote.board.size")}>
      {#each SIZES as s (s)}
        <button class="tool" role="radio" aria-checked={size === s} aria-label={t(`remote.board.size.${s}` as MessageKey)} onclick={() => (size = s)}>
          <span class="dot-size" style:--d="{Math.min(18, strokeWidth(tool === "marker" ? "marker" : "pen", s, 220))}px"></span>
        </button>
      {/each}
    </div>
    <div class="group">
      <button class="tool" onclick={undo} disabled={!canUndo || !online} aria-label={t("remote.board.undo")}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.undo} /></svg>
      </button>
      <button class="tool wipe" class:armed={wipeArmed} onclick={wipe} disabled={!hasStrokes || !online} aria-label={t("remote.board.wipe")}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.trash} /></svg>
        {#if wipeArmed}<span>{t("remote.board.wipeConfirm")}</span>{/if}
      </button>
    </div>
  </div>
</div>

<style>
  /* Pantalla propia, oscura como un lienzo: lo que importa es el dibujo. */
  .pad {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: grid;
    grid-template-rows: auto 1fr auto;
    gap: 8px;
    padding: calc(8px + env(safe-area-inset-top)) calc(8px + env(safe-area-inset-right)) calc(8px + env(safe-area-inset-bottom)) calc(8px + env(safe-area-inset-left));
    background: #17181c;
    color: #f2f2f2;
    touch-action: none;
    -webkit-user-select: none;
    user-select: none;
  }
  header { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .targets {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1;
    min-width: 0;
    display: flex;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
    touch-action: pan-x;
  }
  .targets::-webkit-scrollbar { display: none; }
  .targets li { flex: none; }
  .target {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 220px;
    min-height: 44px;
    padding: 0 12px;
    border-radius: 22px;
    border: 1.5px solid #3a3c44;
    background: #24262c;
    color: inherit;
    font: inherit;
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .target[aria-current="true"] { border-color: #ff453a; background: rgb(255 69 58 / 0.16); }
  .target.add { border-style: dashed; }
  .target:disabled { opacity: 0.5; }
  .dot { flex: none; width: 8px; height: 8px; border-radius: 50%; background: #ff453a; }
  .stage {
    container-type: size;
    min-height: 0;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 10px;
  }
  .turn { margin: 0; font-size: 13px; color: rgb(255 255 255 / 0.5); text-align: center; }
  @media (orientation: landscape) {
    .turn { display: none; }
    .frame { width: min(100cqw, calc(100cqh * var(--aspect))); }
  }
  /* Con la forma de la pantalla del proyector: lo dibujado cae en el mismo sitio. */
  .frame {
    position: relative;
    width: min(100cqw, calc((100cqh - 40px) * var(--aspect)));
    aspect-ratio: var(--aspect);
    overflow: hidden;
    border-radius: 4px;
    outline: 2px solid #ff453a;
    outline-offset: 1px;
  }
  .under, .ink { position: absolute; inset: 0; width: 100%; height: 100%; }
  img.under { object-fit: contain; }
  .ink { touch-action: none; cursor: crosshair; }
  .approx {
    position: absolute;
    inset: 6%;
    margin: 0;
    display: grid;
    place-items: center;
    text-align: center;
    white-space: pre-line;
    font-size: clamp(12px, 4cqh, 22px);
    line-height: 1.3;
    color: #f5f5f0;
    overflow: hidden;
  }
  .approx.muted { color: rgb(255 255 255 / 0.55); }
  .empty { display: grid; justify-items: center; gap: 14px; text-align: center; padding: 16px; color: rgb(255 255 255 / 0.75); }
  .empty p { margin: 0; max-width: 32ch; }
  .big-add {
    min-height: 56px;
    padding: 0 24px;
    border-radius: 28px;
    border: 0;
    background: #f2f2f2;
    color: #111;
    font: inherit;
    font-size: 17px;
    font-weight: 700;
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 6px 14px;
  }
  .group { display: flex; align-items: center; gap: 4px; }
  .tool {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-width: 44px;
    height: 44px;
    padding: 0 8px;
    border-radius: 10px;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    font-size: 14px;
    font-weight: 700;
  }
  .tool svg { width: 24px; height: 24px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
  .tool[aria-checked="true"] { background: #f2f2f2; color: #111; }
  .tool:disabled { opacity: 0.35; }
  .tool.wipe.armed { background: #ff453a; color: #fff; }
  .close { background: #24262c; }
  .swatch {
    width: 36px;
    height: 36px;
    margin: 4px 1px;
    border-radius: 50%;
    border: 2px solid #3a3c44;
    background: var(--ink);
  }
  /* El color elegido: un anillo claro alrededor, no solo el color. */
  .swatch[aria-checked="true"] { box-shadow: 0 0 0 3px #17181c, 0 0 0 5px #f2f2f2; border-color: #17181c; }
  .dot-size { width: var(--d); height: var(--d); min-width: 4px; min-height: 4px; border-radius: 50%; background: currentColor; }
  /* En horizontal, las herramientas van a la derecha y el lienzo ocupa todo el alto. */
  @media (orientation: landscape) and (max-height: 560px) {
    .pad { grid-template-columns: 1fr auto; grid-template-rows: auto 1fr; }
    header { grid-column: 1 / -1; }
    .tools { grid-row: 2; grid-column: 2; flex-direction: column; flex-wrap: nowrap; justify-content: flex-start; overflow-y: auto; gap: 8px; touch-action: pan-y; }
    .group { flex-wrap: wrap; justify-content: center; max-width: 148px; }
    .swatch { width: 32px; height: 32px; }
  }
</style>
