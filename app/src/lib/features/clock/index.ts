import type { SourcePanel } from "$lib/core/panels";
import ClockPanel from "./ClockPanel.svelte";

export const clockPanel: SourcePanel = {
  id: "clock",
  label: "nav.clock",
  icon: "M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM12 9v4l2.5 2.5M10 2h4",
  component: ClockPanel,
};
