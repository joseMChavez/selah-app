<script lang="ts">
  // Ajustes guardados en la base local. Cada sección se puede cambiar aquí y volver a su estado inicial;
  // lo que borra datos del usuario (las letras, todo) pide confirmación con un segundo clic.
  import { onMount } from "svelte";
  import StylePanel from "$lib/components/StylePanel.svelte";
  import { DEFAULT_STYLE } from "$lib/OutputView.svelte";
  import { locales, setLocale, t, type Locale, type MessageKey } from "$lib/i18n/index.svelte";
  import { library, resetLibrary } from "$lib/library.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { obs, waitMessage } from "$lib/obs.svelte";
  import { platform, type BibleInfo, type OutputStyle } from "$lib/platform";
  import { resetSettings, SECTIONS, setSetting, settings } from "$lib/settings.svelte";
  import { setTheme, type ThemeChoice } from "$lib/theme.svelte";

  let {
    style,
    onstylechange,
    onerror,
  }: {
    style: OutputStyle;
    onstylechange: (style: OutputStyle) => void;
    onerror: (message: Message) => void;
  } = $props();

  const CONFIRM_MS = 4000;
  const themes: { id: ThemeChoice; label: MessageKey }[] = [
    { id: "system", label: "theme.system" },
    { id: "light", label: "theme.light" },
    { id: "dark", label: "theme.dark" },
  ];

  type Section = keyof typeof SECTIONS | "songs" | "all";

  let bibles = $state<BibleInfo[]>([]);
  let path = $state<string | null | undefined>(undefined);
  let waitDraft = $state(settings.obsWaitMessage);
  let confirming = $state<Section | null>(null);
  let done = $state<Section | null>(null);
  let confirmTimer: ReturnType<typeof setTimeout> | undefined;

  // Si la base cambia por fuera (restablecer), el borrador del mensaje la sigue.
  $effect(() => {
    waitDraft = settings.obsWaitMessage;
  });

  async function run(action: () => Promise<unknown>, section: Section) {
    try {
      await action();
      done = section;
      setTimeout(() => done === section && (done = null), CONFIRM_MS);
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  /** Para lo que borra datos: el primer clic pide confirmar, el segundo lo hace. */
  function confirmed(section: Section): boolean {
    clearTimeout(confirmTimer);
    if (confirming === section) {
      confirming = null;
      return true;
    }
    confirming = section;
    confirmTimer = setTimeout(() => (confirming = null), CONFIRM_MS);
    return false;
  }

  const resetSection = (section: keyof typeof SECTIONS) =>
    run(async () => {
      await resetSettings(SECTIONS[section]);
      // El estilo vive también en la salida: hay que devolverle el inicial.
      if (section === "projection") await platform.setOutputStyle(DEFAULT_STYLE);
    }, section);

  function resetSongs() {
    if (confirmed("songs")) run(resetLibrary, "songs");
  }

  function resetAll() {
    if (!confirmed("all")) return;
    run(async () => {
      await resetSettings();
      await resetLibrary();
      await platform.setOutputStyle(DEFAULT_STYLE);
    }, "all");
  }

  const saveWaitMessage = () =>
    run(async () => {
      setSetting("obsWaitMessage", waitDraft.trim());
      // Si OBS ya tiene los diseños de Selah, el nuevo texto se ve en la escena «Espera» al momento.
      if (obs.connected && obs.status?.selahReady) await platform.obsSetup(waitMessage());
    }, "stream");

  onMount(() => {
    platform.listBibles().then((list) => (bibles = list)).catch(() => {});
    platform.storePath().then((p) => (path = p)).catch(() => (path = null));
  });
</script>

{#snippet resetButton(section: keyof typeof SECTIONS, label: MessageKey)}
  <div class="foot">
    {#if done === section}<span class="ok small">{t("settings.restored")}</span>{/if}
    <button class="btn ghost" onclick={() => resetSection(section)}>{t(label)}</button>
  </div>
{/snippet}

<div class="settings">
  <header>
    <h1>{t("settings.title")}</h1>
    <p class="muted">{t("settings.intro")}</p>
  </header>

  <section class="card">
    <h2>{t("settings.interface")}</h2>
    <div class="row">
      <label for="set-locale">{t("settings.language")}</label>
      <select id="set-locale" class="field compact" value={settings.locale ?? ""} onchange={(e) => setLocale((e.currentTarget.value || null) as Locale | null)}>
        <option value="">{t("settings.systemLanguage")}</option>
        {#each Object.entries(locales) as [code, locale] (code)}
          <option value={code}>{locale.name}</option>
        {/each}
      </select>
    </div>
    <div class="row">
      <span class="label">{t("settings.theme")}</span>
      <div class="segmented">
        {#each themes as th (th.id)}
          <button aria-pressed={settings.theme === th.id} onclick={() => setTheme(th.id)}>{t(th.label)}</button>
        {/each}
      </div>
    </div>
    {@render resetButton("interface", "settings.resetInterface")}
  </section>

  <section class="card">
    <h2>{t("settings.projection")}</h2>
    <StylePanel {style} onchange={onstylechange} {onerror} />
    {@render resetButton("projection", "settings.resetProjection")}
  </section>

  <section class="card">
    <h2>{t("nav.bible")}</h2>
    <div class="row">
      <label for="set-bible">{t("bible.version")}</label>
      <select id="set-bible" class="field compact" value={settings.bible ?? ""} onchange={(e) => setSetting("bible", e.currentTarget.value || null)}>
        <option value="">{t("settings.bibleFirst")}</option>
        {#each bibles as b (b.id)}<option value={b.id}>{b.abbreviation} — {b.name}</option>{/each}
      </select>
    </div>
    <div class="row">
      <label for="set-bible2">{t("bible.secondVersion")}</label>
      <select id="set-bible2" class="field compact" value={settings.bible2 ?? ""} onchange={(e) => setSetting("bible2", e.currentTarget.value || null)}>
        <option value="">{t("bible.secondNone")}</option>
        {#each bibles as b (b.id)}<option value={b.id}>{b.abbreviation} — {b.name}</option>{/each}
      </select>
    </div>
    {@render resetButton("bible", "settings.resetBible")}
  </section>

  <section class="card">
    <h2>{t("settings.songs")}</h2>
    <p>{t("settings.songsCount", { songs: library.songs.length, setlist: library.setlist.length })}</p>
    <div class="foot">
      {#if done === "songs"}<span class="ok small">{t("settings.restored")}</span>{/if}
      <button class="btn" class:danger={confirming === "songs"} onclick={resetSongs} disabled={library.songs.length === 0 && library.setlist.length === 0}>
        {confirming === "songs" ? t("settings.confirm") : t("settings.resetSongs")}
      </button>
    </div>
  </section>

  <section class="card">
    <h2>{t("settings.stream")}</h2>
    <label class="check">
      <input type="checkbox" checked={settings.obsAutoConnect} onchange={(e) => setSetting("obsAutoConnect", e.currentTarget.checked)} />
      {t("settings.obsAutoConnect")}
    </label>
    <form class="stack" onsubmit={(e) => { e.preventDefault(); saveWaitMessage(); }}>
      <label class="label" for="set-wait">{t("settings.obsWaitMessage")}</label>
      <div class="inline">
        <input id="set-wait" class="field" bind:value={waitDraft} placeholder={t("stream.waitMessage")} />
        <button class="btn" type="submit" disabled={waitDraft.trim() === settings.obsWaitMessage}>{t("songs.save")}</button>
      </div>
      <p class="muted small">{t("settings.obsWaitHint")}</p>
    </form>
    <div class="row">
      <span class="label">{t("stream.text.source")}</span>
      <span class="muted">{settings.obsTextSource ?? t("settings.automatic")}</span>
    </div>
    <p class="muted small">{t("settings.obsKeepsDestination")}</p>
    {@render resetButton("stream", "settings.resetStream")}
  </section>

  <section class="card danger-zone">
    <h2>{t("settings.storage")}</h2>
    {#if path === null}
      <p class="warn">{t("settings.inMemory")}</p>
    {:else if path}
      <p class="muted small">{t("settings.storedAt")} <code>{path}</code></p>
    {/if}
    <p class="muted small">{t("settings.resetAllHint")}</p>
    <div class="foot">
      {#if done === "all"}<span class="ok small">{t("settings.restored")}</span>{/if}
      <button class="btn" class:danger={confirming === "all"} onclick={resetAll}>
        {confirming === "all" ? t("settings.confirm") : t("settings.resetAll")}
      </button>
    </div>
  </section>
</div>

<style>
  .settings { max-width: 760px; margin: 0 auto; display: flex; flex-direction: column; gap: 14px; }
  header h1 { font-size: 22px; margin-bottom: 4px; }
  .card { padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  h2 { font-size: 16px; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
  .row label, .label { font-size: 14px; font-weight: 600; }
  .field.compact { width: auto; max-width: 100%; min-height: 36px; padding: 4px 8px; }
  .foot { display: flex; align-items: center; justify-content: flex-end; gap: 10px; border-top: 1px solid var(--border); padding-top: 10px; }
  .ok { color: var(--ok); font-weight: 600; }
  .check { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .stack { display: flex; flex-direction: column; gap: 6px; }
  .inline { display: flex; gap: 8px; }
  .inline .field { flex: 1; min-width: 0; }
  .warn { padding: 8px 12px; border-radius: 8px; background: var(--warn-bg); color: var(--warn-text); }
  code { font-size: 12.5px; overflow-wrap: anywhere; }
  .danger-zone { border-color: var(--live-soft); }
</style>
