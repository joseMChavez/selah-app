# Revisión de competencia open source (borrador, 2026-10-31)

Fuente: páginas de los repositorios y listados de AlternativeTo. Es una primera pasada; falta leer el código y probar los programas.

| | FreeShow | OpenLP |
|---|---|---|
| Licencia | GPL-3.0 | GPL-3.0 o posterior |
| Repositorio | github.com/ChurchApps/FreeShow | gitlab.com/openlp/openlp |
| Stack | Svelte + Vite, Node/npm (probable Electron; a confirmar) | Python (paquete PyPI) |
| Plataformas | Windows y Linux referenciados; macOS sin confirmar | A confirmar |
| Funciones citadas | stage display, control remoto, medios | canciones, versículos, video, imágenes, presentaciones |
| Madurez | ~600 commits, 1.4k estrellas | ~18.000 commits, 23 releases |

## Por verificar leyendo el código
1. ¿Separan salida de proyección y salida de stream, o es una sola salida con varias vistas?
2. ¿Qué módulo de Biblia usan y qué formatos importan (Zefania, OSIS, etc.)?
3. ¿Cómo gestionan monitores, desconexión y pantalla negra?
4. ¿Tienen integración con OBS/NDI/RTMP?
5. ¿Cómo gestionan i18n y temas?
6. ¿Qué tan fácil es extraer módulos sueltos (importadores, parser de referencias bíblicas)?

## Candidatos a reutilizar (hipótesis)
- Parser de referencias bíblicas y importadores de formatos (OpenLP, FreeShow).
- Lógica de detección de monitores y ventana de salida (FreeShow, si es Electron).
- Importación de letras de canciones.
