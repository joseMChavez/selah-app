import type { SourcePanel } from "$lib/core/panels";
import TransferPanel from "./TransferPanel.svelte";

export const transferPanel: SourcePanel = {
  id: "transfer",
  label: "nav.transfer",
  icon: "M7 4v13M3.5 13.5 7 17l3.5-3.5M17 20V7M13.5 10.5 17 7l3.5 3.5",
  component: TransferPanel,
};
