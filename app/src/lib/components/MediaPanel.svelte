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
  let preparedPath = $derived("path" in prepared ? prepared.path : null);

  function prepare(file: MediaFile) {
    switch (file.kind) {
      case "image": return onprepare({ kind: "image", path: file.path });
      case "video": return onprepare({ kind: "video", path: file.path, paused: false });
      case "pdf": return onprepare({ kind: "pdf", path: file.path, page: 1 });
    }
  }

  async function add() {
    try {
      const picked = await platform.pickMedia(t("dialog.title"), t("dialog.filter"));
      const known = new Set(media.map((m) => m.path));
      media = [...media, ...picked.filter((m) => !known.has(m.path))];
      if (picked.length > 0) prepare(picked[0]);
    } catch (e) {
      onerror(toMessage(e));
    }
  }
</script>

<div class="media-panel">
  <button class="btn" onclick={add}>{t("media.add")}</button>
  {#if media.length === 0}
    <p class="muted">{t("media.empty")}</p>
  {:else}
    <ul>
      {#each media as file (file.path)}
        <li>
          <button class="item" class:chosen={file.path === preparedPath} onclick={() => prepare(file)} title={file.path}>
            <span class="icon" aria-hidden="true">{ICONS[file.kind]}</span>
            <span class="name">{file.name}</span>
            <span class="kind muted small">{t(`kind.${file.kind}`)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .media-panel { display: flex; flex-direction: column; gap: 10px; align-items: flex-start; }
  ul { list-style: none; margin: 0; padding: 0; width: 100%; display: flex; flex-direction: column; gap: 4px; }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 6px 10px;
    font: inherit;
    text-align: left;
    color: var(--text);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
  }
  .item:hover { background: var(--surface-2); }
  .item.chosen { border-color: var(--accent); background: var(--accent-soft); }
  .icon { font-size: 18px; }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
