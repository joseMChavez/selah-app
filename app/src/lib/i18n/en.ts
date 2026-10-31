import type { MessageKey } from "./es";

const en: Record<MessageKey, string> = {
  "app.title": "Selah · Projection",
  "settings.language": "Language",

  "status.openOn": "Output open on",
  "status.onScreen": "On screen",
  "status.closed": "Output closed",

  "monitors.title": "Output monitor",
  "monitors.none": "No monitor detected.",
  "monitors.details": "{width}×{height} · scale {scale} · position {x},{y}",
  "monitors.primary": "primary",
  "monitors.outputHere": "output here",
  "monitors.lost": "Monitor “{monitor}” is no longer connected. The output may have moved to another screen. Choose a monitor and open the output again.",
  "output.openOn": "Open output on {monitor}",
  "output.moveTo": "Move output to {monitor}",
  "output.close": "Close output",

  "prepared.title": "Prepared",
  "prepared.tag": "preview, not visible",
  "prepared.text": "Text",
  "prepared.addFiles": "+ Add files…",
  "prepared.textLabel": "Prepared text",
  "prepared.slidesHint": "Type *** on a line to start another slide.",
  "prepared.send": "Send to screen",
  "prepared.sample": "Welcome\n***\nThis week's announcements\n***\nTime of prayer",

  "live.title": "On screen",
  "live.tag": "what the congregation sees",
  "live.black": "Black screen",
  "live.testPattern": "Test pattern",
  "live.clear": "Clear content",
  "live.pause": "Pause video",
  "live.resume": "Resume video",
  "live.previous": "◀ Previous",
  "live.next": "Next ▶",
  "live.keysHint": "Also with the ← → arrow keys.",
  "live.jumpHint": "Click a slide to show it now.",

  "position.slide": "Slide {n} of {total}",
  "position.page": "Page {n} of {total}",

  "kind.image": "Image",
  "kind.video": "Video",
  "kind.pdf": "PDF",

  "describe.empty": "Nothing (black background, no content)",
  "describe.black": "Black screen",
  "describe.testPattern": "Test pattern",
  "describe.text": "Text, slide {n} of {total}",
  "describe.image": "Image “{name}”",
  "describe.video": "Video “{name}”",
  "describe.videoPaused": "Video “{name}” (paused)",
  "describe.pdf": "PDF “{name}”, page {n}",

  "dialog.title": "Choose images, videos or PDFs",
  "dialog.filter": "Images, videos and PDFs",

  "pattern.title": "Test pattern",
  "pattern.scale": "scale",

  "error.monitors": "Could not read the monitors: {error}",
  "error.monitorDisconnected": "The chosen monitor is no longer connected.",
  "error.fileMissing": "The file no longer exists or cannot be read: {path}",
  "error.invalidContent": "That content cannot be shown.",
  "error.internal": "Unexpected error: {message}",
  "error.pdfOpen": "Could not open the PDF “{name}”: {error}",
  "error.fromOutput": "The output reports: {message}",
  "error.image": "Could not show the image “{name}”.",
  "error.pdf": "Could not show the PDF “{name}”.",
  "error.video": "Could not play the video “{name}”.",
  "error.codec": "Format or codec not supported on this computer.",
  "error.videoStart": "The video could not start: {error}",
};

export default en;
