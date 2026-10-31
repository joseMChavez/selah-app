import type { MessageKey } from "./es";

const it: Record<MessageKey, string> = {
  "app.title": "Selah · Proiezione",
  "settings.language": "Lingua",

  "status.openOn": "Uscita aperta su",
  "status.onScreen": "Sullo schermo",
  "status.closed": "Uscita chiusa",

  "monitors.title": "Monitor di uscita",
  "monitors.none": "Nessun monitor rilevato.",
  "monitors.details": "{width}×{height} · scala {scale} · posizione {x},{y}",
  "monitors.primary": "principale",
  "monitors.outputHere": "uscita qui",
  "monitors.lost": "Il monitor «{monitor}» non è più collegato. L'uscita potrebbe essersi spostata su un altro schermo. Scegli un monitor e riapri l'uscita.",
  "output.openOn": "Apri uscita su {monitor}",
  "output.moveTo": "Sposta uscita su {monitor}",
  "output.close": "Chiudi uscita",

  "prepared.title": "Preparato",
  "prepared.tag": "anteprima, non visibile",
  "prepared.text": "Testo",
  "prepared.addFiles": "+ Aggiungi file…",
  "prepared.textLabel": "Testo preparato",
  "prepared.slidesHint": "Scrivi *** su una riga per iniziare un'altra diapositiva.",
  "prepared.send": "Invia allo schermo",
  "prepared.sample": "Benvenuti\n***\nAnnunci della settimana\n***\nMomento di preghiera",

  "live.title": "Sullo schermo",
  "live.tag": "ciò che vede la congregazione",
  "live.black": "Schermo nero",
  "live.testPattern": "Schermata di prova",
  "live.clear": "Rimuovi contenuto",
  "live.pause": "Metti in pausa il video",
  "live.resume": "Riprendi il video",
  "live.previous": "◀ Precedente",
  "live.next": "Successiva ▶",
  "live.keysHint": "Anche con le frecce ← → della tastiera.",
  "live.jumpHint": "Clic su una diapositiva per mostrarla subito.",

  "position.slide": "Diapositiva {n} di {total}",
  "position.page": "Pagina {n} di {total}",

  "kind.image": "Immagine",
  "kind.video": "Video",
  "kind.pdf": "PDF",

  "describe.empty": "Niente (sfondo nero senza contenuto)",
  "describe.black": "Schermo nero",
  "describe.testPattern": "Schermata di prova",
  "describe.text": "Testo, diapositiva {n} di {total}",
  "describe.image": "Immagine «{name}»",
  "describe.video": "Video «{name}»",
  "describe.videoPaused": "Video «{name}» (in pausa)",
  "describe.pdf": "PDF «{name}», pagina {n}",

  "dialog.title": "Scegli immagini, video o PDF",
  "dialog.filter": "Immagini, video e PDF",

  "pattern.title": "Schermata di prova",
  "pattern.scale": "scala",

  "error.monitors": "Impossibile leggere i monitor: {error}",
  "error.monitorDisconnected": "Il monitor scelto non è più collegato.",
  "error.fileMissing": "Il file non esiste più o non si può leggere: {path}",
  "error.invalidContent": "Questo contenuto non può essere mostrato.",
  "error.internal": "Errore imprevisto: {message}",
  "error.pdfOpen": "Impossibile aprire il PDF «{name}»: {error}",
  "error.fromOutput": "L'uscita segnala: {message}",
  "error.image": "Impossibile mostrare l'immagine «{name}».",
  "error.pdf": "Impossibile mostrare il PDF «{name}».",
  "error.video": "Impossibile riprodurre il video «{name}».",
  "error.codec": "Formato o codec non supportato su questo computer.",
  "error.videoStart": "Il video non è potuto partire: {error}",
};

export default it;
