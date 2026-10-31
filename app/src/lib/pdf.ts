// WebKit (macOS/Linux) no muestra PDF dentro de la página, así que se dibujan con pdf.js en un canvas.
import { getDocument, GlobalWorkerOptions, RenderingCancelledException, type PDFDocumentProxy } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

GlobalWorkerOptions.workerSrc = workerUrl;

const documents = new Map<string, Promise<PDFDocumentProxy>>();

// Se descarga en el hilo principal: el worker no siempre puede leer el esquema asset:// del webview.
export function loadPdf(url: string): Promise<PDFDocumentProxy> {
  let doc = documents.get(url);
  if (!doc) {
    doc = fetch(url)
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.arrayBuffer();
      })
      .then((data) => getDocument({ data }).promise);
    doc.catch(() => documents.delete(url));
    documents.set(url, doc);
  }
  return doc;
}

export async function pageCount(url: string): Promise<number> {
  return (await loadPdf(url)).numPages;
}

/** Dibuja la página ajustada (sin recortar) al tamaño del contenedor. Devuelve false si se canceló. */
export async function renderPage(
  url: string,
  pageNumber: number,
  canvas: HTMLCanvasElement,
  box: { width: number; height: number },
  signal: AbortSignal,
): Promise<boolean> {
  const doc = await loadPdf(url);
  const page = await doc.getPage(Math.min(Math.max(pageNumber, 1), doc.numPages));
  if (signal.aborted) return false;

  const base = page.getViewport({ scale: 1 });
  const fit = Math.min(box.width / base.width, box.height / base.height);
  const dpr = window.devicePixelRatio || 1;
  const viewport = page.getViewport({ scale: fit * dpr });
  canvas.width = Math.floor(viewport.width);
  canvas.height = Math.floor(viewport.height);
  canvas.style.width = `${Math.floor(viewport.width / dpr)}px`;
  canvas.style.height = `${Math.floor(viewport.height / dpr)}px`;

  const task = page.render({ canvas, viewport });
  signal.addEventListener("abort", () => task.cancel(), { once: true });
  try {
    await task.promise;
    return true;
  } catch (e) {
    if (e instanceof RenderingCancelledException) return false;
    throw e;
  }
}
