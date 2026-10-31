# Selah

Aplicación de escritorio de código abierto para iglesias: proyección multimedia y control simplificado de transmisión en vivo, con dos hilos independientes (Proyección y Transmisión).

**Estado:** MVP en desarrollo (v0.1). La parte de **Proyección** ya funciona; la de **Transmisión** (OBS) está pendiente.

- Plataformas objetivo: Windows, macOS y Linux (probado por ahora en Linux)
- Interfaz: español, inglés e italiano
- Base técnica: Tauri 2 + SvelteKit; OBS como motor de transmisión (pendiente)
- Licencia: GPL-3.0

## Qué hace ya

- Consola de control y ventana de salida a pantalla completa en el monitor elegido.
- «Preparado» separado de «En pantalla»: nada cambia en la pantalla hasta enviarlo.
- Biblia: navegación libro → capítulo → versículo, búsqueda por referencia («Juan 3:16») o por palabras, doble clic para mostrar al momento y copyright de la versión siempre visible.
- Textos en diapositivas (separadas por `***`), imágenes, videos y PDF, también por carpetas.
- Ajuste automático del tamaño del texto, alineación, posición y fondo; pantalla en negro con vuelta a lo anterior.
- Tema claro, oscuro o del sistema.

## Desarrollo

Requisitos: Node.js, Rust y las dependencias de sistema de Tauri. En Linux, GStreamer con gst-libav para el video.

```sh
cd app
npm install
npm run tauri dev
```

Biblias incluidas en el instalador: archivos XML en formato Beblia dentro de `app/src-tauri/resources/bibles/<idioma>/`. No se suben al repositorio; ver el README de esa carpeta y D-012.

Las decisiones del proyecto están en [docs/DECISIONS.md](docs/DECISIONS.md) y las pruebas técnicas en [docs/PHASE2-TESTS.md](docs/PHASE2-TESTS.md).
