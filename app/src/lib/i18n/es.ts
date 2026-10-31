// Diccionario de referencia: las claves de este archivo definen las de los demás idiomas.
const es = {
  "app.title": "Selah · Proyección",
  "settings.language": "Idioma",

  "status.openOn": "Salida abierta en",
  "status.onScreen": "En pantalla",
  "status.closed": "Salida cerrada",

  "monitors.title": "Monitor de salida",
  "monitors.none": "No se detectó ningún monitor.",
  "monitors.details": "{width}×{height} · escala {scale} · posición {x},{y}",
  "monitors.primary": "principal",
  "monitors.outputHere": "salida aquí",
  "monitors.lost": "El monitor «{monitor}» ya no está conectado. La salida puede haberse movido a otra pantalla. Elige un monitor y vuelve a abrir la salida.",
  "output.openOn": "Abrir salida en {monitor}",
  "output.moveTo": "Mover salida a {monitor}",
  "output.close": "Cerrar salida",

  "prepared.title": "Preparado",
  "prepared.tag": "vista previa, no visible",
  "prepared.text": "Texto",
  "prepared.addFiles": "+ Añadir archivos…",
  "prepared.textLabel": "Texto preparado",
  "prepared.slidesHint": "Escribe *** en una línea para empezar otra diapositiva.",
  "prepared.send": "Enviar a pantalla",
  "prepared.sample": "Bienvenidos\n***\nAnuncios de la semana\n***\nTiempo de oración",

  "live.title": "En pantalla",
  "live.tag": "lo que ve la congregación",
  "live.black": "Pantalla en negro",
  "live.testPattern": "Patrón de prueba",
  "live.clear": "Quitar contenido",
  "live.pause": "Pausar video",
  "live.resume": "Reanudar video",
  "live.previous": "◀ Anterior",
  "live.next": "Siguiente ▶",
  "live.keysHint": "También con las flechas ← → del teclado.",
  "live.jumpHint": "Clic en una diapositiva para mostrarla ya.",

  "position.slide": "Diapositiva {n} de {total}",
  "position.page": "Página {n} de {total}",

  "kind.image": "Imagen",
  "kind.video": "Video",
  "kind.pdf": "PDF",

  "describe.empty": "Nada (fondo negro sin contenido)",
  "describe.black": "Pantalla en negro",
  "describe.testPattern": "Patrón de prueba",
  "describe.text": "Texto, diapositiva {n} de {total}",
  "describe.image": "Imagen «{name}»",
  "describe.video": "Video «{name}»",
  "describe.videoPaused": "Video «{name}» (en pausa)",
  "describe.pdf": "PDF «{name}», página {n}",

  "dialog.title": "Elegir imágenes, videos o PDF",
  "dialog.filter": "Imágenes, videos y PDF",

  "pattern.title": "Patrón de prueba",
  "pattern.scale": "escala",

  "error.monitors": "No se pudieron leer los monitores: {error}",
  "error.monitorDisconnected": "El monitor elegido ya no está conectado.",
  "error.fileMissing": "El archivo ya no existe o no se puede leer: {path}",
  "error.invalidContent": "No se puede mostrar ese contenido.",
  "error.internal": "Error inesperado: {message}",
  "error.pdfOpen": "No se pudo abrir el PDF «{name}»: {error}",
  "error.fromOutput": "La salida informa: {message}",
  "error.image": "No se pudo mostrar la imagen «{name}».",
  "error.pdf": "No se pudo mostrar el PDF «{name}».",
  "error.video": "No se pudo reproducir el video «{name}».",
  "error.codec": "Formato o códec no soportado en este equipo.",
  "error.videoStart": "El video no pudo empezar: {error}",
};

export type MessageKey = keyof typeof es;
export default es;
