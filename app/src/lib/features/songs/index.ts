import type { SourcePanel } from "$lib/core/panels";
import TextPanel from "./TextPanel.svelte";

export const textPanel: SourcePanel = {
  id: "text",
  label: "nav.text",
  icon: "M4 6h16M4 10h16M4 14h10M4 18h7",
  component: TextPanel,
};
