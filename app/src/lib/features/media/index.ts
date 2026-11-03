import type { SourcePanel } from "$lib/core/panels";
import MediaPanel from "./MediaPanel.svelte";

export const mediaPanel: SourcePanel = {
  id: "media",
  label: "nav.media",
  icon: "M4 5h16v14H4zM4 15l4.5-4.5 4 4 2.5-2.5L20 17M15.5 8.5h.01",
  component: MediaPanel,
};
