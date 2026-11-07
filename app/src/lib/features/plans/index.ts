import type { SourcePanel } from "$lib/core/panels";
import PlanPanel from "./PlanPanel.svelte";

export const planPanel: SourcePanel = {
  id: "plan",
  label: "nav.plan",
  icon: "M9 3h6v4H9zM7 5H5v16h14V5h-2M8 12h2M12 12h4M8 16h2M12 16h4",
  component: PlanPanel,
};
