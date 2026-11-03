import type { SourcePanel } from "$lib/core/panels";
import BiblePanel from "./BiblePanel.svelte";

export const biblePanel: SourcePanel = {
  id: "bible",
  label: "nav.bible",
  icon: "M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2V5Zm0 16a2 2 0 0 1 2-2h13v2H6M12 7v6M9.5 9.5h5",
  component: BiblePanel,
};
