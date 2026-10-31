# Registro de decisiones

## D-001 — Nombre: Selah (2026-10-31)
- **Decisión:** usar "Selah" como nombre del producto.
- **Motivo:** preferencia del autor; no se encontró otro proyecto de proyección para iglesias con ese nombre.
- **Alternativas:** buscar otro nombre.
- **Consecuencias / pendiente:** verificar marcas (USPTO, EUIPO, Italia), dominio y organización en GitHub. Es un nombre bíblico común y existen otras apps no relacionadas con ese nombre.

## D-002 — Proyecto de código abierto, sin fin de lucro (2026-10-31)
- **Decisión:** el proyecto es open source; el objetivo no es ganar dinero.
- **Motivo:** decisión del autor.
- **Consecuencias:** se elimina la pregunta "gratis/pago/suscripción". Las traducciones bíblicas con copyright siguen necesitando licencia aunque el código sea libre.

## D-003 — Licencia del código: GPL-3.0 (2026-10-31)
- **Decisión:** GPL-3.0.
- **Motivo:** protege contra versiones cerradas derivadas; coincide con OpenLP (GPL-3.0+) y FreeShow (GPL-3.0), lo que permite reutilizar su código.
- **Alternativas:** MIT/Apache-2.0 (permisivas), AGPL.
- **Consecuencias:** todo código copiado debe conservar sus avisos de copyright y su origen se registra en docs/REUSE.md. Revisar compatibilidad de FFmpeg/GStreamer al empaquetar. Los datos (Biblias, fuentes, iconos, traducciones) tienen licencia propia, distinta del código.

## D-004 — Equipo: proyecto personal (2026-10-31)
- **Decisión:** un solo autor por ahora.
- **Consecuencias:** CONTRIBUTING.md y código de conducta quedan para cuando se publique el repositorio. Los calendarios se calculan con una persona.

## D-005 — Revisión de competencia antes de construir (2026-10-31)
- **Decisión:** revisar FreeShow y OpenLP para reutilizar funcionalidades útiles antes de la Fase 0.
- **Resultado inicial:** ver docs/COMPETITIVE-REVIEW.md.

## D-006 — Plataformas: Windows, macOS y Linux (2026-10-31)
- **Decisión:** soportar las tres desde el inicio. Windows por ser la más común en iglesias, macOS porque el autor la tiene y Linux porque es su entorno de desarrollo.
- **Alternativas:** una sola plataforma prioritaria primero.
- **Consecuencias:** las pruebas técnicas de la Fase 2 (monitores, cámara, video, stream) se repiten en las tres. Linux con Wayland y macOS con permisos de cámara/pantalla son riesgos conocidos. La firma de código y la notarización de macOS requieren cuenta de desarrollador de Apple (de pago); Windows requiere certificado o un servicio gratuito para open source. El calendario crece frente a una sola plataforma.

## D-007 — Importar Biblias en el formato de Holyrics (2026-10-31)
- **Decisión:** Selah debe poder importar Biblias en el formato que usa Holyrics, para aprovechar las que cada iglesia ya posee.
- **Proceso:** el autor colocará muestras en samples/holyrics-bibles/ cuando las consiga; con ellas se identifica el formato y se prueba el importador. Esa carpeta está en .gitignore por posible copyright.
- **Estado (actualizado 2026-10-31):** suspendido. Las muestras `.bib` están cifradas (ver D-011). Se sustituye por D-012.
- **Consecuencias:** se implementa como un adaptador más de la capa de proveedor de Biblia. Selah no incluye ni distribuye esas Biblias; solo lee las que el usuario ya tiene. No se usa código de Holyrics. Mostrar siempre licencia y atribución; si faltan metadatos, pedirlos al usuario.

## D-008 — OBS como motor de stream, con detección e instalación asistida (2026-10-31)
- **Decisión:** usar OBS controlado por obs-websocket (OBS 28 o superior). Selah detecta si OBS está instalado; si no, ofrece instalarlo con permiso explícito del usuario.
- **Motivo:** OBS ya resuelve composición, codificación y emisión en las tres plataformas.
- **Consecuencias:** descargar solo desde obsproject.com y comprobar hash. En Linux se muestran instrucciones en lugar de ejecutar sudo. Por verificar en Fase 2: activar WebSocket sin abrir OBS manualmente. Cámara del MVP: USB o capturadora.

## D-009 — Teléfono como cámara: en el plan, fuera del MVP (2026-10-31)
- **Decisión:** la función de usar un teléfono como cámara (servidor local con WebRTC) se mantiene en el plan del producto, pero no entra en el alcance del MVP.
- **Motivo:** requiere HTTPS con certificado local, depende del Wi-Fi y es independiente del resto. Revisar la licencia de VDO.Ninja antes de reutilizarlo.
- **Consecuencias:** el diseño de la capa de fuentes de video debe dejar espacio para una fuente remota sin rehacer lo demás.

## D-010 — Tauri primero, con Electron como plan de respaldo (2026-10-31)
- **Decisión:** construir con Tauri (Rust + webview del sistema). Si se bloquea un punto crítico, migrar a Electron.
- **Motivo:** menor consumo de recursos y proyecto personal que tolera el riesgo. El frontend web se reutilizaría en la migración.
- **Alternativas:** Electron desde el inicio (más predecible, más pesado); Qt (ligero, pero se pierde la reutilización de FreeShow).
- **Consecuencias:** WebView2 en Windows (Chromium) y WebKit en macOS y Linux, así que hay que probar en dos familias de motores. El frontend debe hablar con el backend solo a través de una capa de adaptador propia, sin llamar a APIs de Tauri directamente, para que migrar sea barato. El backend en Rust no se reutiliza en Electron y habría que reescribirlo.
- **Criterios para abandonar Tauri** (revisar en Fase 2): video que no reproduce de forma fiable en WebKit; pantalla completa en segundo monitor inestable en macOS o Linux; permisos o captura de cámara inviables; consumo no mejor que Electron al medirlo.

## D-011 — El formato .bib de Holyrics está cifrado: no se importa (2026-10-31)
- **Hallazgo:** las cinco muestras (LBLA, NTV, NVI, RV, RV1960) no tienen cabecera, tienen entropía de 8,000 bits/byte, no se comprimen y su tamaño es múltiplo de 16 bytes: cifrado por bloques.
- **Decisión:** no descifrarlas. Saltarse una protección técnica es ilegal aparte de la licencia del texto (DMCA §1201; Directiva 2001/29/CE art. 6), y el brief prohíbe depender de formatos protegidos sin autorización.
- **Alternativas:** pedir a Holyrics documentación o una vía de exportación.
- **Consecuencias:** D-007 queda suspendida.

## D-012 — Biblias en el formato XML de Beblia; licencias a cargo del autor (2026-10-31)
- **Decisión:** Selah importa Biblias en el formato XML del repositorio github.com/Beblia/Holy-Bible-XML-Format (`bible > testament > book[number] > chapter[number] > verse[number]`). El autor elige las versiones (RVR1960, NVI, NTV, LBLA, VBL 2022) y se encarga de sus licencias.
- **Motivo:** formato abierto y sencillo; cada archivo declara su procedencia y copyright en los atributos `translation`, `status`/`info` y `link` de `<bible>`.
- **Alternativas:** RV1909 o Reina 1569 (dominio público, descartadas por antiguas); VBL 2022 (CC BY-SA 4.0); Zefania/OSIS.
- **Consecuencias:** siempre que se elige una Biblia se muestra su copyright en la consola y en la pantalla. Actualización 2026-10-31: por decisión del autor, las Biblias van en el instalador como base (`app/src-tauri/resources/bibles/<idioma>/`, se empaquetan como recursos y se cargan sin importarlas); el usuario puede añadir más desde la app. Los `.xml` no se suben al repositorio (.gitignore). El repositorio Beblia no tiene licencia y varias versiones tienen derechos reservados: antes de publicar Selah o distribuir Biblias hay que tener la licencia de cada editorial. Los versículos vacíos (versículos unidos en algunas traducciones) se omiten.

## D-013 — Instaladores en GitHub Releases, compilados en GitHub Actions, sin Biblias (2026-10-31)
- **Decisión:** los instaladores de Linux (.deb, .rpm, AppImage), Windows (.msi y .exe de NSIS) y macOS (.dmg universal para Intel y Apple Silicon) los compila `.github/workflows/instaladores.yml` en máquinas de cada sistema. Cada etiqueta `vX.Y.Z` crea un borrador en Releases con sus instaladores. Los instaladores públicos no llevan Biblias.
- **Motivo:** Windows y macOS no se pueden compilar de forma fiable desde Linux. Guardar binarios en git haría crecer el repositorio con cada versión. Las Biblias tienen derechos reservados y el repositorio es público (D-012).
- **Alternativas:** carpeta de instaladores en el repo con Git LFS (cuota limitada); pasar las Biblias a Actions como secreto (los instaladores públicos las distribuirían igual).
- **Consecuencias:** al instalar Selah desde Releases, la Biblia empieza vacía y se importan los XML desde la app. Un instalador compilado en local sí incluye los XML de `resources/bibles/`, solo para uso propio. Los instaladores no están firmados: Windows (SmartScreen) y macOS (Gatekeeper) muestran un aviso; firmar requiere certificados de pago y queda para más adelante.
