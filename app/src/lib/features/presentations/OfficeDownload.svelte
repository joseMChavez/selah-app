<script lang="ts">
  // Descarga de la copia privada de LibreOffice: botón, avance y cancelar. La usan Medios y Ajustes.
  import { t } from "$lib/i18n/index.svelte";
  import { cancelOffice, installOffice, megabytes, office } from "$lib/features/presentations/office.svelte";

  let { onready }: { onready?: () => void } = $props();

  let percent = $derived(
    office.progress && office.progress.total > 0 ? Math.floor((office.progress.downloaded / office.progress.total) * 100) : 0,
  );

  async function start() {
    if (await installOffice()) onready?.();
  }
</script>

{#if office.progress}
  <div class="progress" role="status">
    {#if office.progress.extracting}
      <span class="small">{t("office.extracting")}</span>
      <progress></progress>
    {:else}
      <span class="small">
        {t("office.downloading", {
          percent,
          done: megabytes(office.progress.downloaded),
          total: megabytes(office.progress.total),
        })}
      </span>
      <progress max="100" value={percent}></progress>
      <button class="btn ghost" onclick={cancelOffice}>{t("office.cancel")}</button>
    {/if}
  </div>
{:else if office.status}
  <div class="offer">
    <button class="btn" onclick={start}>
      {t("office.download", { size: megabytes(office.status.size) })}
    </button>
    <span class="muted small">{t("office.downloadHint", { version: office.status.version })}</span>
  </div>
{/if}
{#if office.error}<p class="warn small">{t(office.error.key, office.error.params)}</p>{/if}

<style>
  .progress, .offer { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
  progress { flex: 1; min-width: 120px; accent-color: var(--accent); }
</style>
