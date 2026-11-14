<script lang="ts">
  // Ajustes guardados en la base local. Cada sección se puede cambiar aquí y volver a su estado inicial;
  // lo que borra datos del usuario (las letras, todo) pide confirmación con un segundo clic.
  import { onMount } from "svelte";
  import { bibles, importBibles, loadBibles, removeAllBibles, removeBible, showBible } from "$lib/features/bible/bibles.svelte";
  import OfficeDownload from "$lib/features/presentations/OfficeDownload.svelte";
  import RemoteSettings from "$lib/features/remote/RemoteSettings.svelte";
  import StylePanel from "$lib/features/output/StylePanel.svelte";
  import { DEFAULT_STYLE } from "$lib/features/output/OutputView.svelte";
  import { locales, setLocale, t, type Locale, type MessageKey } from "$lib/i18n/index.svelte";
  import { library, resetLibrary } from "$lib/features/songs/library.svelte";
  import { plans, resetPlans } from "$lib/features/plans/plans.svelte";
  import { toMessage, type Message } from "$lib/core/messages";
  import { obs, waitMessage } from "$lib/features/stream/obs.svelte";
  import { loadOffice, office, removeOffice } from "$lib/features/presentations/office.svelte";
  import { platform, type BibleInfo, type OutputStyle } from "$lib/platform";
  import { resetSettings, SECTIONS, setSetting, settings } from "$lib/core/settings.svelte";
  import { setTheme, type ThemeChoice } from "$lib/core/theme.svelte";

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

  /** Sección, o «bible:<id>» para quitar una versión concreta. */
  type Section = keyof typeof SECTIONS | "songs" | "plans" | "all" | "bibles" | "office" | `bible:${string}`;

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

  function removeVersion(b: BibleInfo) {
    // Ocultar una incluida se deshace con un clic; borrar una importada, no: pide confirmar.
    if (b.builtin || confirmed(`bible:${b.id}`)) run(() => removeBible(b), "bible");
  }

  function removeAll() {
    if (confirmed("bibles")) run(removeAllBibles, "bible");
  }

  const importMore = () => run(() => importBibles(t("bible.dialogTitle"), t("bible.dialogFilter")), "bible");

  function resetSongs() {
    if (confirmed("songs")) run(resetLibrary, "songs");
  }

  function removePlans() {
    if (confirmed("plans")) run(resetPlans, "plans");
  }

  function removeOfficeCopy() {
    if (confirmed("office")) run(removeOffice, "office");
  }

  function resetAll() {
    if (!confirmed("all")) return;
    run(async () => {
      await resetSettings();
      await resetLibrary();
      await resetPlans();
      await platform.setOutputStyle(DEFAULT_STYLE);
    }, "all");
  }

  const saveWaitMessage = () =>
    run(async () => {
      setSetting("obsWaitMessage", waitDraft.trim());
      // Si OBS ya tiene los diseños de EmetPro, el nuevo texto se ve en la escena «Espera» al momento.
      if (obs.connected && obs.status?.emetproReady) await platform.obsSetup(waitMessage());
    }, "stream");

  onMount(() => {
    if (bibles.all.length === 0) loadBibles().catch((e) => onerror(toMessage(e)));
    platform.storePath().then((p) => (path = p)).catch(() => (path = null));
    loadOffice().catch((e) => onerror(toMessage(e)));
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
    <label class="check">
      <input type="checkbox" checked={settings.confirmSend} onchange={(e) => setSetting("confirmSend", e.currentTarget.checked)} />
      {t("settings.confirmSend")}
    </label>
    <StylePanel {style} onchange={onstylechange} {onerror} />
    {@render resetButton("projection", "settings.resetProjection")}
  </section>

  <section class="card">
    <h2>{t("settings.bibles")}</h2>
    <p class="muted small">{t("settings.biblesIntro")}</p>
    {#if bibles.all.length === 0}
      <p class="muted">{t("bible.empty")}</p>
    {:else}
      <ul class="versions">
        {#each bibles.all as b (b.id)}
          {@const hidden = settings.hiddenBibles.includes(b.id)}
          <li class:hidden title={`${b.name}\n${b.copyright || t("bible.noCopyright")}`}>
            <span class="abbr">{b.abbreviation}</span>
            <span class="vname">
              {b.name}
              <span class="muted small">{t(b.builtin ? "settings.bibleBuiltin" : "settings.bibleImported")}{hidden ? ` · ${t("settings.bibleHidden")}` : ""}</span>
            </span>
            {#if hidden}
              <button class="btn ghost" onclick={() => showBible(b.id)}>{t("settings.bibleShow")}</button>
            {:else}
              <button class="btn ghost" class:danger={confirming === `bible:${b.id}`} onclick={() => removeVersion(b)}>
                {confirming === `bible:${b.id}` ? t("settings.confirm") : t(b.builtin ? "settings.bibleHide" : "settings.bibleRemove")}
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    <div class="foot">
      {#if done === "bible"}<span class="ok small">{t("settings.restored")}</span>{/if}
      <button class="btn" onclick={importMore}>{t("bible.import")}</button>
      <button class="btn" class:danger={confirming === "bibles"} onclick={removeAll} disabled={bibles.all.length === 0}>
        {confirming === "bibles" ? t("settings.confirm") : t("settings.bibleRemoveAll")}
      </button>
      <button class="btn ghost" onclick={() => resetSection("bible")}>{t("settings.resetBible")}</button>
    </div>
  </section>

  <section class="card">
    <h2>{t("settings.songs")}</h2>
    <p>{t("settings.songsCount", { songs: library.songs.length })}</p>
    <div class="foot">
      {#if done === "songs"}<span class="ok small">{t("settings.restored")}</span>{/if}
      <button class="btn" class:danger={confirming === "songs"} onclick={resetSongs} disabled={library.songs.length === 0}>
        {confirming === "songs" ? t("settings.confirm") : t("settings.resetSongs")}
      </button>
    </div>
  </section>

  <section class="card">
    <h2>{t("settings.plans")}</h2>
    <p>{t("settings.plansCount", { count: plans.all.length })}</p>
    {#if plans.filesBytes}
      <p class="muted small">{t("settings.plansFiles", { size: (plans.filesBytes / 1048576).toFixed(1) })}</p>
    {/if}
    <div class="foot">
      {#if done === "plans"}<span class="ok small">{t("settings.restored")}</span>{/if}
      <button class="btn" class:danger={confirming === "plans"} onclick={removePlans} disabled={plans.all.length === 0}>
        {confirming === "plans" ? t("settings.confirm") : t("settings.resetPlans")}
      </button>
    </div>
  </section>

  <section class="card">
    <h2>{t("settings.presentations")}</h2>
    <p class="muted small">{t("settings.presentationsIntro")}</p>
    {#if office.status}
      <div class="row">
        <span class="label">{t("settings.officeSystem")}</span>
        <span class="muted">{t(office.status.system ? "settings.officeFound" : "settings.officeNotFound")}</span>
      </div>
      <div class="row">
        <span class="label">{t("settings.officePrivate")}</span>
        <span class="muted">
          {office.status.privateCopy
            ? t("settings.officeReady", { version: office.status.version })
            : t(office.status.downloadable ? "settings.officeNone" : "settings.officeUnavailable")}
        </span>
      </div>
      {#if office.status.privateCopy}
        <div class="foot">
          {#if done === "office"}<span class="ok small">{t("settings.restored")}</span>{/if}
          <button class="btn" class:danger={confirming === "office"} onclick={removeOfficeCopy}>
            {confirming === "office" ? t("settings.confirm") : t("settings.officeRemove")}
          </button>
        </div>
      {:else if office.status.downloadable}
        <OfficeDownload />
      {/if}
    {/if}
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

  <RemoteSettings />

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
  .versions { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .versions li { display: flex; align-items: center; gap: 12px; padding: 6px 8px; border-radius: 8px; }
  .versions li:hover { background: var(--surface-2); }
  .versions li.hidden { opacity: 0.6; }
  .abbr { min-width: 5.5em; font-weight: 750; letter-spacing: 0.02em; }
  .vname { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .foot { flex-wrap: wrap; }
</style>
