 PROMPT MAESTRO — PLAN DE PROYECTO
Aplicación de escritorio multimedia para iglesias
Versión inicial del brief: 25 de octubre de 2026
Nombre provisional de producto: Selah (no confirmado)
INSTRUCCIONES PARA EL ASISTENTE QUE RECIBA ESTE PROMPT
Actúa como product manager técnico y arquitecto de software senior con experiencia en aplicaciones de escritorio, sistemas audiovisuales en vivo y productos para usuarios no técnicos. Ayúdame a convertir esta idea en un proyecto real, fácil de operar durante un servicio y viable de desarrollar por etapas.
Trabaja de manera colaborativa y concreta. Antes de programar, inspecciona el entorno de trabajo, pregunta por la información crítica que falte y presenta un plan de ejecución priorizado. No inventes respuestas sobre presupuesto, sistemas operativos, equipo, licencias ni disponibilidad del usuario. Mientras esperas respuestas, avanza en las tareas independientes que sean seguras. No conviertas decisiones pequeñas de implementación en bloqueos: declara supuestos razonables y sigue adelante.
No empieces construyendo una aplicación completa. Primero concreta un MVP pequeño, valida el flujo de operación, confirma las restricciones técnicas y de licencias y entrega una primera versión ejecutable. Al planificar, da estimaciones por tarea en horas o días-persona, señala dependencias, riesgos y confianza de la estimación. Distingue tiempo de desarrollo de tiempo de calendario. No prometas fechas sin conocer la dedicación semanal y las personas disponibles. Si solo trabaja una persona unas horas por semana, recalcula el calendario de forma explícita.
Prioriza que el producto sea confiable y comprensible durante un servicio en vivo. No escondas controles esenciales, no uses nombres técnicos cuando una instrucción normal sea más clara y evita cambios accidentales en lo que ya está al aire. Cualquier función que pueda interrumpir una proyección o transmisión debe tener un estado visible y una recuperación sencilla.
CONTEXTO E IDEA DEL PRODUCTO
Quiero crear una aplicación de escritorio para iglesias que reúna proyección multimedia y control simplificado de una transmisión en vivo. Debe servir a voluntarios y operadores con distintos niveles de experiencia técnica. El producto debería sentirse como una consola clara para un servicio, no como un mezclador profesional complejo.
La idea visual inicial usa el nombre provisional “Selah” y contempla una interfaz en español, inglés e italiano, con modo claro y oscuro. El usuario debe poder cambiar el idioma desde Configuración. Hay un mockup conversacional de referencia con cinco secciones: Proyección, Transmisión, Biblioteca, Historial y Configuración. Es una exploración inicial, no una especificación final ni un diseño aprobado. Mantén la arquitectura visual simple y adapta el diseño según pruebas con usuarios.
PRINCIPIO FUNDAMENTAL: DOS HILOS INDEPENDIENTES
La app tiene dos flujos de trabajo y dos destinos diferentes:

    PROYECCIÓN EN EL TEMPLO

        Contenido enviado a uno o más proyectores o pantallas para la congregación.
        Puede incluir pasajes bíblicos, letras de canciones, imágenes, videos, PDFs, anuncios y textos.
        Cuenta con su propia cola, selección, vista previa, contenido al aire, controles y destino de pantalla.

    TRANSMISIÓN ONLINE

        Contenido que verá la audiencia en internet, compuesto por cámara o cámaras, texto, pasajes, contenido multimedia y mensajes.
        Cuenta con su propia cola o selección de fuentes, escenas/diseños, vista previa, estado al aire y controles de emisión.
        Debe poder mostrar al predicador, la proyección, ambos combinados, texto sobre video, una pantalla de espera u otros diseños sencillos.

Regla de independencia: cambiar, avanzar o detener un elemento en Proyección no debe cambiar automáticamente la Transmisión, y viceversa. El operador puede copiar o agregar explícitamente una referencia, pasaje o contenido a ambos hilos. Cuando una fuente de transmisión reutilice contenido de la biblioteca o de la proyección, debe quedar claro si es una copia fija o una referencia que se actualizará. El estado “En pantalla” y “Al aire” se identifica por separado y con claridad.
OBJETIVO DEL PRODUCTO
Permitir a una iglesia preparar, proyectar y transmitir un servicio desde una sola app, operada por una persona voluntaria, reduciendo la complejidad técnica y evitando que una acción en un destino altere accidentalmente el otro.
USUARIOS Y NECESIDADES

    Operador/a de proyección: busca pasajes y medios, prepara el orden del servicio, previsualiza y envía cada elemento a la pantalla.
    Operador/a de transmisión: elige cámara, audio, textos y diseño para la audiencia online; cambia escenas y mensajes sin tocar el contenido de la congregación.
    Voluntario/a con poca experiencia técnica: necesita etiquetas claras, estados visibles, pocas decisiones por pantalla y recuperación ante errores.
    Persona que prepara el servicio: organiza canciones, pasajes, PDFs, videos e imágenes antes del evento y desea reutilizar el historial o duplicar un servicio anterior.
    Administrador/a: configura salidas, biblioteca, traducciones bíblicas, almacenamiento y servicios de transmisión.

REQUISITOS FUNCIONALES DESEADOS
A. Consola y navegación

    Una ventana principal de control con navegación visible.
    Acceso rápido a Proyección y Transmisión con sus estados independientes.
    Vista previa del contenido al aire en cada salida y vista previa del próximo elemento o composición cuando sea viable.
    El contenido preparado no se muestra hasta que el operador use una acción explícita de envío/al aire.
    Botones y estados con lenguaje inequívoco, por ejemplo “Enviar a pantalla”, “Poner al aire”, “Quitar texto del stream”, “Finalizar transmisión”.
    Atajos de teclado opcionales y configurables, sin depender exclusivamente de ellos.
    Modo oscuro y claro; considerar una opción “seguir el tema del sistema”.

B. Proyección

    Detectar/seleccionar un monitor de salida y mostrar una ventana de salida a pantalla completa en el monitor elegido.
    Mantener la consola del operador en otra ventana o pantalla.
    Mostrar Biblia, texto, letras/canciones, imagen, video, PDF y anuncios.
    Cola/lista de elementos para el servicio, con reordenamiento, selección, edición y duplicación.
    Vista previa distinta del contenido actualmente enviado.
    Navegación rápida por diapositivas, versículos y páginas.
    Pantalla en negro o pausa segura para la salida, con estado visible.
    Manejo de desconexión del monitor, resolución distinta, cambio de pantalla, formatos inválidos y archivos que ya no existen.
    Permitir ajustar tipografía, tamaño, alineación, fondos y márgenes con plantillas fáciles de entender.

C. Transmisión

    Una primera experiencia sencilla para preparar una transmisión de video.
    Composiciones/diseños iniciales: cámara completa, contenido completo, cámara con texto bíblico, lado a lado, predicador con banda de texto, espera/inicio y cierre.
    Fuentes de cámara y audio seleccionables; mostrar disponibilidad y errores de dispositivo.
    Añadir y retirar textos, versículos y mensajes inferiores sin modificar Proyección.
    Vista previa clara de lo que se enviaría a la audiencia online.
    Mostrar estado de conexión, plataforma/destino, duración y errores importantes.
    Acciones explícitas para iniciar, finalizar o cambiar el diseño.
    Evaluar como decisión de arquitectura una integración con OBS controlada desde nuestra interfaz versus un motor de transmisión propio basado en FFmpeg/GStreamer. No asumir que el control WebSocket de OBS resuelve por sí solo la composición, el audio, la codificación o la compatibilidad.
    Para un primer MVP, se puede considerar OBS como motor opcional/dependencia si permite validar el flujo más rápido; ocultar la complejidad técnica para el usuario normal. Hacer una prueba técnica antes de comprometerse. Documentar claramente si OBS debe estar instalado.
    No introducir la obligación de utilizar servicios cloud para operar la proyección local. Si el streaming necesita internet, indicarlo sin afectar el flujo de proyección offline.

D. Biblia y traducciones

    Interfaz de búsqueda por referencia, como “Mateo 18:20”, y, si está disponible, búsqueda por palabras.
    Elegir traducción; mostrar nombre, abreviatura, idioma, referencia y atribución requerida.
    Mostrar pasaje y referencias relacionadas, con navegación anterior/siguiente.
    Enviar un pasaje a Proyección, a Transmisión o a ambos mediante acciones explícitas.
    Hacer que la búsqueda y proyección de las versiones locales funcionen sin internet.
    Diseñar una interfaz de proveedor desacoplada para admitir: (1) texto local de dominio público o licencia abierta; (2) archivos importados por el usuario con autorización de uso; y (3) proveedores online con licencia aprobada.
    No copiar módulos, bases de datos ni paquetes de Holyrics ni de otra aplicación sin permiso. Holyrics sirve como referencia de experiencia, pero su catálogo procede de acuerdos propios y traducciones de dominio público.
    Explorar importación de formatos abiertos como Zefania XML, validando la especificación, los metadatos y las licencias de cada archivo. Aceptar un formato no significa que se obtenga permiso para distribuir su contenido.
    Cada versión debe tener metadatos explícitos: título, abreviatura, idioma, editor/propietario, licencia, atribución, restricciones de almacenamiento/offline, fuente, versión del archivo y fecha de importación.
    No incluir traducciones con derechos reservados ni permitir su descarga desde un servidor propio hasta verificar permiso escrito/licencia aplicable y condiciones de distribución. Para RVR1960 u otra traducción popular, confirmar con titular o proveedor que la licencia cubre el producto, uso en iglesia, visualización en transmisión, distribución offline, almacenamiento y modelo comercial.
    Si se integra una API, verificar cuota, acceso por versión, disponibilidad, términos actuales, atribuciones, almacenamiento/cache permitido y protección de claves. No exponer claves privadas en el renderer del escritorio.
    Incluir un panel para ver la información de derechos/atribución antes de añadir una versión y mostrar el crédito requerido en el contexto de visualización según su licencia.

E. Biblioteca multimedia

    Importar archivos locales: imágenes, videos y PDF en primer alcance.
    Organizar por nombre, tipo, carpetas o colecciones; búsqueda y metadatos útiles.
    Mostrar miniaturas/previsualización y duración o páginas donde aplique.
    Decidir con el usuario si la app copia el archivo a su biblioteca local o conserva la ruta original. Explicar consecuencias y ofrecer una forma segura de localizar archivos desconectados.
    No almacenar archivos multimedia grandes dentro de SQLite; guardar contenido en archivos y metadatos/rutas en la base de datos.
    Las letras de canciones pueden añadirse manualmente o importarse de formatos compatibles solo si el usuario tiene derecho de uso.
    Cargar archivos de forma no destructiva; no modificar el original.

F. Servicios, historial y biblioteca de programas

    Crear/guardar un servicio con nombre, fecha, elementos y orden.
    Duplicar un servicio pasado para preparar uno nuevo.
    Historial de qué elementos se enviaron a cada salida, cuándo y desde qué servicio.
    Distinguir elemento preparado, elemento enviado a pantalla y elemento al aire online.
    Exportar/importar una copia de seguridad local con archivos referenciados o incluidos según elección del usuario.

G. Configuración

    Idiomas de interfaz incluidos desde la primera versión: español, inglés e italiano. Permitir cambiar el idioma desde Configuración y recordar la preferencia localmente. Mantener separados el idioma de la interfaz y el idioma de cada traducción bíblica o medio.
    Tema claro/oscuro/sistema.
    Configurar monitores/salidas para proyección.
    Configurar cámara, audio y destinos de transmisión según la solución elegida.
    Configurar carpeta de biblioteca, espacio disponible y copias de seguridad.
    Atajos de teclado y preferencias de operación.
    Diagnóstico accesible: estado de monitor, dispositivos de video/audio, conexión, almacenamiento y versión de la app.

PRINCIPIOS DE UX Y CONFIABILIDAD

    Diseñar para pantallas de escritorio y para operación durante presión/tiempo real.
    Evitar que acciones de preparación alteren el contenido en vivo.
    Separar visualmente previsualización, cola/selección, salida en vivo y controles de cada hilo.
    Identificar siempre el destino seleccionado antes de enviar.
    Confirmación solo para acciones con riesgo real, como finalizar una transmisión o vaciar una cola, sin llenar la experiencia de diálogos innecesarios.
    Ofrecer deshacer para cambios reversibles cuando sea posible.
    Mantener las acciones principales visibles sin pasar el cursor por encima.
    Usar tipografía legible, contraste adecuado, objetivos de interacción amplios y estados acompañados de texto (no solo color).
    Diseñar para que un operador pueda recuperarse si se equivoca: volver al elemento anterior, poner negro, volver a una escena conocida o regresar a la última salida válida.
    No inventar paneles de métricas o notificaciones que no ayuden durante el servicio.
    Hacer pruebas de usabilidad con personas que operen proyección/stream en iglesias, incluyendo voluntarios no técnicos.

PLATAFORMA Y ARQUITECTURA: HIPÓTESIS INICIAL, NO DECISIÓN CERRADA

    Aplicación desktop multiplataforma: Electron + React + TypeScript parece una base práctica para iterar interfaz y ventanas de salida; confirmar objetivo Windows/macOS/Linux con el usuario antes de cerrar elección.
    Separar el proceso principal de Electron, renderer/UI y procesos/servicios audiovisuales. Restringir el acceso del renderer a archivos y capacidades nativas; usar IPC con lista permitida y validación.
    SQLite local para servicios, listas, preferencias, historial y metadatos; sistema de archivos local para medios y paquetes bíblicos autorizados.
    Mantener una capa de dominio para la Biblia y el almacenamiento para no acoplar la app a un proveedor o formato único.
    Ventana principal de control, ventana de proyección dedicada y compositor/vista previa de stream como componentes independientes, compartiendo datos de manera explícita y no estado accidental.
    Tratar captura, reproducción, composición, audio, codificación y envío de stream como subsistemas diferenciados.
    Evaluar en una prueba vertical el acceso a cámara/audio, dos monitores, composición con texto/video y una transmisión de prueba. Medir rendimiento y latencia en equipos modestos.
    No guardar secretos de proveedor en código distribuido. Si se necesita un backend para ocultar claves, gestionar cuotas/licencias o servir paquetes, justificar costo operativo y necesidades de privacidad.
    Implementar guardado seguro, migraciones de base de datos, logs diagnósticos sin contenido sensible y recuperación tras cierre inesperado.
    Mantener dependencias, codecs y licencias de software bajo control; revisar implicaciones de redistribución de FFmpeg/GStreamer u otros componentes antes de empaquetar.

DECISIONES ABIERTAS QUE HAY QUE RESOLVER TEMPRANO

    Sistemas operativos del primer lanzamiento: Windows, macOS, Linux o prioridad concreta.
    La app es personal/de una iglesia, gratuita para iglesias, de pago o producto comercial por suscripción. Esto afecta acuerdos de Biblia y proveedor.
    ¿El MVP debe emitir por sí mismo, o puede requerir que OBS esté instalado como motor oculto/opcional?
    Plataformas de destino iniciales: YouTube, Facebook, RTMP genérico u otra. Verificar protocolos y condiciones actuales.
    Número de cámaras, fuentes y monitores que deben funcionar en el MVP.
    ¿El equipo puede usar la app sin internet durante todo el servicio? La proyección y medios locales deberían poder hacerlo; el streaming dependerá de conexión.
    ¿Qué traducciones bíblicas son imprescindibles? Confirmar idioma, edición exacta, territorio y licencia.
    Importar contenido de Holyrics: ¿se necesita migrar playlists/medios, o solo importar archivos genéricos? Nunca diseñar una dependencia de formatos cerrados o protegidos sin autorización.
    Equipo de referencia: procesador, RAM, GPU, cámaras, capturadoras y sistema de audio disponibles.
    Dedicación semanal, número de desarrolladores y presupuesto para licencias, pruebas de equipos y posible backend.
    ¿Se requiere operación por una sola persona o se planea control remoto desde otro dispositivo?
    ¿Se necesita sincronización entre varios ordenadores en esta primera versión, o puede quedar fuera?

PLAN DE TRABAJO SUGERIDO Y ESTIMACIONES
Las cifras siguientes son rangos iniciales de esfuerzo para una persona desarrolladora senior, expresados en días-persona de 6–7 horas productivas. Son hipótesis para planificación, no compromiso. Deben recalibrarse al conocer plataformas, equipo, experiencia, disponibilidad y alcance de streaming. Las gestiones de licencia y disponibilidad de terceros pueden añadir tiempo de calendario no controlable.
FASE 0 — Alineación de alcance y riesgos (3–5 días-persona; 1–2 semanas de calendario con decisiones del usuario)

    Responder las decisiones abiertas prioritarias.
    Definir usuario principal, plataforma y caso real de servicio.
    Delimitar MVP y fuera de alcance.
    Crear mapa de riesgos: licencias de traducciones, compatibilidad audiovisual, dos salidas, dispositivos y presupuesto.
    Entregables: brief de producto confirmado, matriz de decisiones, alcance MVP y criterios de éxito.
    Puerta de salida: no iniciar integración de textos protegidos sin plan de licencias; no comprometer streaming sin prueba técnica.

FASE 1 — Flujos, prototipo y prueba de usabilidad (5–9 días-persona)

    Refinar mockup: consola, Proyección, Transmisión, Biblioteca, Biblia, Servicios/Historial y Configuración.
    Diseñar estados vacíos, errores, desconexión, archivo no encontrado, monitor no disponible y estado en vivo.
    Probar con 3–5 operadores/voluntarios en tareas concretas.
    Ajustar lenguaje, jerarquía y flujo de “preparar → previsualizar → enviar”.
    Entregables: mapa de navegación, prototipo clicable, lista priorizada de problemas UX y especificación del MVP.
    Criterios: usuarios distinguen sin ayuda qué está en proyección y qué está al aire; no confunden vista previa con salida.

FASE 2 — Pruebas técnicas verticales y decisión audiovisual (6–12 días-persona)

    Crear una app mínima que abre la consola y una ventana de salida en monitor secundario.
    Validar detección/cambio de monitores y pantalla completa en sistemas objetivo.
    Validar reproducción de imagen, video y PDF en formatos representativos.
    Probar cámara y audio y generar una composición de stream con cámara y texto.
    Probar OBS WebSocket como motor externo y evaluar alternativa propia; medir estabilidad, salida RTMP/plataforma y controles imprescindibles.
    Verificar cómo se obtiene un pasaje local y cómo se importa un archivo bíblico permitido.
    Entregables: informe de pruebas, limitaciones confirmadas y ADR (registro de decisión de arquitectura).
    Criterios: decisión informada sobre streaming, no una elección basada solo en intuición.

FASE 3 — Base de aplicación y almacenamiento local (8–14 días-persona)

    Estructurar Electron y React/TypeScript, navegación y estilos claros/oscuros.
    Añadir comunicación segura entre renderer y proceso principal.
    Crear SQLite, migraciones y modelos de servicio, elemento multimedia, cola, salida e historial.
    Añadir importación de medios, carpetas y localización de archivos perdidos.
    Guardar preferencias y crear/duplicar un servicio local.
    Entregables: app ejecutable con navegación, biblioteca inicial y persistencia.
    Criterios: reiniciar la app no pierde configuración, servicios ni metadatos.

FASE 4 — Proyección usable de principio a fin (10–18 días-persona)

    Ventana de salida independiente y selección de monitor.
    Biblioteca con imagen, video, PDF y textos.
    Biblia local autorizada: búsqueda por referencia, navegación y salida.
    Cola de servicio y estado visible de elemento preparado/actual.
    Envío explícito, pantalla negra/pausa y retorno a contenido anterior.
    Ajustes visuales iniciales y pruebas con contenidos reales.
    Criterios: proyección funciona sin internet con contenidos locales; preparar siguiente elemento no cambia pantalla hasta enviarlo; recuperación básica funciona.

FASE 5 — Hilo de transmisión y composiciones sencillas (15–28 días-persona, más incertidumbre)

    Implementar la ruta elegida tras la prueba técnica.
    Selección de fuentes, diseños y vista previa de composición.
    Añadir textos/pasajes de manera independiente a proyección.
    Iniciar, estado, errores de conexión y finalizar transmisión.
    Pruebas con plataforma destino, escenas, audio y reconexión.
    Criterios: cambio de proyección no cambia stream; cambio de stream no cambia proyección; la vista previa muestra fielmente la composición; el operador conoce claramente qué está al aire.
    Riesgo: este rango puede crecer si se necesita emisión propia, múltiples cámaras, codificación por hardware, compatibilidad con varios sistemas o soporte de audio avanzado.

FASE 6 — Biblia empaquetada e importación mantenible (5–10 días-persona para infraestructura, sin incluir licencias ni adquisición del contenido)

    Implementar metadatos/versiones y adaptadores de proveedor.
    Importador de un formato abierto priorizado (por ejemplo, Zefania XML) y validación de estructura.
    Empaquetado/instalación/desinstalación y actualización de una versión local autorizada.
    Atribución visible y registro de procedencia/licencia.
    Integración online solo para versiones con derechos y plan de acceso verificados.
    Criterios: app rechaza archivo mal formado, muestra la versión y licencia, permite búsqueda offline de una versión local autorizada y nunca presenta una traducción sin su atribución requerida.

FASE 7 — Robustez, empaquetado y beta de iglesia (10–18 días-persona)

    Crear instaladores de sistemas elegidos, firma/notarización si corresponde y actualización segura.
    Validar migraciones, backups/restauración, archivos grandes y cierres inesperados.
    Pruebas manuales con escenarios de servicio y diferentes equipos.
    Diagnóstico/exportación de logs sin secretos y guía de recuperación.
    Beta observada en al menos 2–3 servicios reales antes de declarar el producto listo.
    Criterios: ningún fallo crítico conocido en el flujo ensayado; documentación de requisitos de hardware y limitaciones; prueba de restauración completada.

ESTIMACIÓN GLOBAL ORIENTATIVA

    Prototipo y prueba técnica para decidir el camino: aproximadamente 14–26 días-persona.
    MVP local con proyección sólida y transmisión simplificada apoyada en OBS si la prueba confirma que encaja: aproximadamente 50–85 días-persona desde el inicio, incluyendo UX y primera beta, excluyendo trámites de licencias y tiempos de espera de terceros.
    MVP con motor propio de streaming, mayor compatibilidad o varias cámaras: puede superar 75–120 días-persona; recalcular después de la fase 2.
    Para una sola persona a 10 horas por semana, 50–85 días-persona podría equivaler aproximadamente a 7–12 meses de trabajo calendario, sin contar interrupciones ni licencias. A 20 horas por semana, podría ser cerca de 4–6 meses. Estos equivalentes son aproximaciones; recalcular con horas efectivas y vacaciones.
    No fijar fecha de lanzamiento hasta terminar Fase 2 y conocer disponibilidad de traducciones, hardware y plataformas.

MVP RECOMENDADO
El primer producto que una iglesia pueda probar debería incluir:

    App desktop en el sistema operativo prioritario.
    Consola clara con modos claro y oscuro.
    Salida de proyección en monitor separado.
    Biblioteca local de imágenes, videos, PDFs y texto.
    Servicio con cola y duplicación sencilla.
    Una Biblia local autorizada para funcionar offline, con metadatos y atribución correctos.
    Dos hilos realmente independientes: Proyección y Transmisión.
    Flujo de transmisión limitado a diseños predeterminados, cámara, texto bíblico y audio básico, usando la solución audiovisual validada en la fase técnica.
    Historial local, respaldo y recuperación básica.
    Manual corto y guía de ensayo antes del servicio.

FUERA DEL MVP, SALVO QUE EL USUARIO LO PRIORICE

    Sincronización multiusuario/multisede en la nube.
    Aplicaciones móviles completas y control remoto por internet.
    Editor profesional de video, escenas ilimitadas y mezclador de audio avanzado.
    Automatización de culto con IA.
    Reproducción/descarga de cualquier traducción bíblica sin licencias verificadas.
    Marketplace de medios o catálogo global de contenidos.
    Edición simultánea colaborativa.
    Funciones administrativas de iglesia, donaciones, miembros o contabilidad.
    Soporte garantizado para todos los equipos, codecs y plataformas desde la primera versión.

CRITERIOS DE ACEPTACIÓN DEL MVP

    Un operador puede preparar un servicio, encontrar un pasaje autorizado y mandar un medio a pantalla sin ayuda técnica.
    Cambiar el elemento de proyección no muta el stream.
    Cambiar escena, texto o cámara del stream no muta la proyección.
    Antes de enviar/al aire, el usuario puede identificar claramente la salida que cambiará.
    La proyección de medios locales y de una Biblia autorizada funciona sin internet.
    Si un archivo desaparece, la app informa cuál falta y permite repararlo sin perder la lista del servicio.
    El estado de cada salida se recupera o explica después de reiniciar la app; no se afirma que una transmisión siga activa si no hay conexión comprobada.
    Atribuciones y restricciones de traducciones están visibles y asociadas a la versión correcta.
    Hay una copia de seguridad y se prueba su restauración.
    El flujo se ensaya con el equipo que se usará en el servicio.

FORMATO DE RESPUESTA Y GESTIÓN DEL PROYECTO
Al iniciar el trabajo, responde con:

    Resumen de lo que entiendes del producto y del principio de dos hilos independientes.
    Lista breve de preguntas críticas y decisiones pendientes, ordenadas por urgencia.
    Supuestos provisionales que permiten avanzar sin bloquearse.
    Backlog de MVP por épicas e historias de usuario, cada una con criterios de aceptación verificables.
    Estimación en horas/días-persona, dependencias, nivel de confianza y riesgos.
    Plan de validación técnica para monitor secundario, medios, cámaras, audio, streaming y Biblia local.
    Próxima acción concreta de menor riesgo que produzca un resultado revisable.

Usa estimaciones por rangos y explica qué causa la incertidumbre. Reserva tiempo para aprendizaje, integración y pruebas con equipos reales; no estimes solo el tiempo de escribir código. No declares completada una fase solo porque el código compila: revisa sus criterios de aceptación. No agregues pruebas automatizadas o trabajo extra sin explicar su objetivo y relación con los riesgos del producto; las validaciones manuales de hardware y experiencia de usuario sí son necesarias para esta clase de app.
Cuando se tomen decisiones, registra fecha, decisión, motivo, alternativas consideradas y consecuencias. Mantén un backlog priorizado y un registro de riesgos actualizado. Si un riesgo bloquea el objetivo principal, propón una prueba técnica pequeña para despejarlo antes de ampliar el alcance.
FIN DEL PROMPT MAESTRO