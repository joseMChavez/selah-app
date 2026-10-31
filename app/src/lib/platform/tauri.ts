// Único archivo de la interfaz que importa APIs de Tauri. Migrar a Electron = reescribir este archivo.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { OutputState, Platform } from "./types";

export const tauriPlatform: Platform = {
  listMonitors: () => invoke("list_monitors"),
  openOutput: (monitorIndex) => invoke("open_output", { monitorIndex }),
  closeOutput: () => invoke("close_output"),
  setOutputContent: (content) => invoke("set_output_content", { content }),
  getOutputState: () => invoke("get_output_state"),
  onOutputState: (handler) => listen<OutputState>("output-state", (event) => handler(event.payload)),
};
