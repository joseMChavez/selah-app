// Único archivo de la interfaz que importa APIs de Tauri. Migrar a Electron = reescribir este archivo.
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { OutputState, Platform } from "./types";

export const tauriPlatform: Platform = {
  listMonitors: () => invoke("list_monitors"),
  openOutput: (monitorIndex) => invoke("open_output", { monitorIndex }),
  closeOutput: () => invoke("close_output"),
  setOutputContent: (content) => invoke("set_output_content", { content }),
  getOutputState: () => invoke("get_output_state"),
  setOutputStyle: (style) => invoke("set_output_style", { style }),
  onOutputState: (handler) => listen<OutputState>("output-state", (event) => handler(event.payload)),
  pickMedia: (title, filterName) => invoke("pick_media", { title, filterName }),
  mediaUrl: (path) => convertFileSrc(path),
  reportOutputError: (message) => invoke("report_output_error", { message }),
  logError: (message) => invoke("log_client_error", { message }),
  onOutputError: (handler) => listen<string>("output-error", (event) => handler(event.payload)),
  listBibles: () => invoke("list_bibles"),
  importBibles: (title, filterName) => invoke("import_bibles", { title, filterName }),
  removeBible: (id) => invoke("remove_bible", { id }),
  getChapter: (bibleId, book, chapter) => invoke("get_chapter", { bibleId, book, chapter }),
  searchBible: (bibleId, query) => invoke("search_bible", { bibleId, query }),
};
