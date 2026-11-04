<script lang="ts">
  // Sección «Control remoto» de Ajustes: encenderlo, el QR para el teléfono y desconectar a todos.
  import { t } from "$lib/i18n/index.svelte";
  import { setSetting, settings } from "$lib/core/settings.svelte";
  import { remote, resetRemoteKey } from "./console.svelte";

  const DONE_MS = 4000;
  let resetDone = $state(false);

  async function reset() {
    await resetRemoteKey();
    resetDone = true;
    setTimeout(() => (resetDone = false), DONE_MS);
  }
</script>

<section class="card">
  <h2>{t("settings.remote")}</h2>
  <p class="muted small">{t("remote.intro")}</p>
  <label class="check">
    <input type="checkbox" checked={settings.remoteEnabled} disabled={remote.busy} onchange={(e) => setSetting("remoteEnabled", e.currentTarget.checked)} />
    {t("remote.enable")}
  </label>
  {#if remote.error}
    <p class="warn" role="alert">{t(remote.error.key, remote.error.params)}</p>
  {/if}
  {#if remote.info?.running}
    <div class="pair">
      {#if remote.info.qr}
        <!-- El SVG lo genera el backend a partir de la dirección: no lleva texto del usuario. -->
        <div class="qr" aria-hidden="true">{@html remote.info.qr}</div>
      {/if}
      <div class="stack">
        <p>{t("remote.scan")}</p>
        <code>{remote.info.url}</code>
        {#if !remote.info.onNetwork}<p class="warn">{t("remote.noNetwork")}</p>{/if}
        <p class="label">{t("remote.clients", { n: remote.info.clients })}</p>
        <p class="muted small">{t("remote.firewall")}</p>
      </div>
    </div>
    <div class="foot">
      {#if resetDone}<span class="ok small">{t("remote.resetDone")}</span>{/if}
      <button class="btn" onclick={reset}>{t("remote.resetKey")}</button>
    </div>
  {/if}
</section>

<style>
  .card { padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  h2 { font-size: 16px; }
  .check { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .pair { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
  /* Fondo blanco siempre: los lectores de QR fallan con el tema oscuro. */
  .qr { flex: none; width: 220px; padding: 8px; background: #fff; border-radius: 10px; }
  .qr :global(svg) { display: block; width: 100%; height: auto; }
  .stack { flex: 1; min-width: 220px; display: flex; flex-direction: column; gap: 8px; }
  .label { font-weight: 600; }
  code { font-size: 12.5px; overflow-wrap: anywhere; }
  .warn { padding: 8px 12px; border-radius: 8px; background: var(--warn-bg); color: var(--warn-text); }
  .foot { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 10px; border-top: 1px solid var(--border); padding-top: 10px; }
  .ok { color: var(--ok); font-weight: 600; }
</style>
