<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import { toMessage, type Message } from "$lib/messages";
  import { platform, type MediaFile, type OutputContent } from "$lib/platform";

  let {
    prepared,
    onprepare,
    onerror,
  }: {
    prepared: OutputContent;
    onprepare: (content: OutputContent) => void;
    onerror: (message: Message) => void;
  } = $props();

  const ICONS = { image: "🖼", video: "🎬", pdf: "📄" } as const;

  let media = $state<MediaFile[]>([]);
  let note = $state("");
  let preparedPath = $derived("path" in prepared ? prepared.path : null);

  function prepare(file: MediaFile) {
    switch (file.kind) {
      case "image": return onprepare({ kind: "image", path: file.path });
      case "video": return onprepare({ kind: "video", path: file.path, paused: false });
      case "pdf": return onprepare({ kind: "pdf", path: file.path, page: 1 });
    }
  }

  /** Añade sin repetir y prepara el primero nuevo, para verlo enseguida en la vista previa. */
  function append(picked: MediaFile[]) {
    const known = new Set(media.map((m) => m.path));
    const fresh = picked.filter((m) => !known.has(m.path));
    media = [...media, ...fresh];
    if (fresh.length > 0) prepare(fresh[0]);
  }

  async function addFiles() {
    note = "";
    try {
      append(await platform.pickMedia(t("dialog.title"), t("dialog.filter")));
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  async function addFolder() {
    note = "";
    try {
      const picked = await platform.pickMediaFolder(t("media.folderTitle"));
      if (picked === null) return;
      if (picked.length === 0) note = t("media.folderEmpty");
      append(picked);
    } catch (e) {
      onerror(toMessage(e));
    }
  }

  // Solo la lista: el archivo sigue en el disco y lo que está en pantalla no cambia.
  function remove(path: string) {
    media = media.filter((m) => m.path !== path);
  }

  function folderOf(path: string) {
    const parts = path.split(/[\\/]/);
    return parts.length > 1 ? parts[parts.length - 2] : "";
  }
</script>

<div class="media-panel">
  <div class="actions">
    <button class="btn" onclick={addFiles}>{t("media.addFiles")}</button>
    <button class="btn" onclick={addFolder}>{t("media.addFolder")}</button>
    {#if media.length > 0}
      <span class="muted small count">{t("media.count", { count: media.length })}</span>
      <button class="btn ghost" onclick={() => (media = [])}>{t("media.clear")}</button>
    {/if}
  </div>
  {#if note}<p class="muted small">{note}</p>{/if}

  {#if media.length === 0}
    <p class="muted">{t("media.empty")}</p>
  {:else}
    <p class="muted small">{t("media.removeHint")}</p>
    <ul>
      {#each media as file (file.path)}
        <li class:chosen={file.path === preparedPath}>
          <button class="item" onclick={() => prepare(file)} title={file.path}>
            <span class="icon" aria-hidden="true">{ICONS[file.kind]}</span>
            <span class="name">
              {file.name}
              <span class="muted small folder">{folderOf(file.path)}</span>
            </span>
            <span class="kind muted small">{t(`kind.${file.kind}`)}</span>
          </button>
          <button class="remove" onclick={() => remove(file.path)} aria-label={t("media.remove", { name: file.name })} title={t("media.remove", { name: file.name })}>✕</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .media-panel { display: flex; flex-direction: column; gap: 10px; }
  .actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .count { margin-left: auto; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  li {
    display: flex;
    align-items: stretch;
    border: 1px solid transparent;
    border-radius: 8px;
  }
  li:hover { background: var(--surface-2); }
  li.chosen { border-color: var(--accent); background: var(--accent-soft); }
  .item {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 6px 10px;
    font: inherit;
    text-align: left;
    color: var(--text);
    background: transparent;
    border: 0;
    cursor: pointer;
  }
  .icon { font-size: 18px; }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder { margin-left: 6px; }
  .remove {
    flex: none;
    width: 40px;
    font: inherit;
    font-size: 15px;
    color: var(--muted);
    background: transparent;
    border: 0;
    border-radius: 0 8px 8px 0;
    cursor: pointer;
  }
  .remove:hover { color: var(--live); background: var(--live-soft); }
</style>
