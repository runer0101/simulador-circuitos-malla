# Plan de mejoras por fases

Este documento mantiene solo el backlog pendiente. Las tareas completadas se resumen al final.

## Backlog activo

### Fase 6 — Despliegue y operación (en curso)

- Contenerización con Docker. **Hecho**: `Dockerfile` sobre `python:3.11-slim`, usuario sin privilegios,
  `HEALTHCHECK` contra `/api/health` y `docker-compose.yml` para el arranque local.
- Configuración de producción (WSGI, variables de entorno, logs estructurados). **Hecho**: `wsgi.py`
  con Gunicorn, `gunicorn.conf.py` configurable por entorno y registro JSON opcional con
  `LOG_FORMAT=json`. Pendiente: despliegue real en un anfitrión y su documentación.

## Actualización de estado (2026-11-10)

- Fase 6 (primera mitad):
  - Imagen de contenedor reproducible, con Gunicorn como servidor y usuario `appuser` (uid 10001).
  - Sonda de salud propia (`healthcheck.py`) usada por Docker y por Compose.
  - Registro estructurado: `LOG_FORMAT=json` emite una línea JSON por evento con nivel, logger,
    mensaje, marca temporal y campos extra; cualquier otro valor produce texto plano.
  - Job `docker` en el pipeline: construye la imagen y comprueba el endpoint de salud dentro del
    contenedor, de modo que una imagen que no arranca rompe la construcción.
  - Total acumulado de pruebas: 57 (pytest), `ruff check` y `black --check` en verde.

## Actualización de estado (2026-11-10 — endurecimiento)

- Fase 6 (segunda mitad, primera parte):
  - Límite de peticiones por cliente con ventana deslizante sobre los POST de la API: 60 por minuto
    por defecto, configurable por `RATE_LIMIT_REQUESTS` y `RATE_LIMIT_WINDOW_SECONDS`, desactivable
    con 0.
  - Respuesta `429` con el contrato de error uniforme y cabecera `Retry-After`; las lecturas no se
    limitan y una petición rechazada no contamina el historial.
  - Total acumulado de pruebas: 62 (pytest), `ruff check` y `black --check` en verde.

## Criterio de priorización

1. Estabilidad y consistencia funcional.
2. Mantenibilidad del código.
3. UX y accesibilidad.
4. Funciones avanzadas.
5. Operación en producción.

---

## Actualización de estado (2026-03-05)

- Fase 2 (parcial, completada en esta iteración):
  - `create_app` y separación por módulos ya operativa.
  - Centralización de constantes en `src/config.py`.
  - Validaciones extraídas a `src/validators/inputs.py`.
- Fase 3 (completada en esta iteración):
  - Pruebas con `pytest` para validadores, lógica de mallas y endpoints API.
  - Configuración de `ruff` y `black`.
  - Workflow de CI en GitHub Actions para lint + formato + tests.

## Actualización de estado (2026-03-13)

- Fase 3 (refuerzo):
  - Cobertura ampliada de errores API (415 no JSON, 400 JSON malformado).
  - Cobertura de handlers web 404 y 500.
- Fase 6 (avance operativo):
  - Observabilidad con logs enriquecidos (method, path, query).
  - Endpoints operativos: `GET /api/health`, `GET /api/version` y `GET /api/metrics`.

## Actualización de estado (2026-03-16)

- Fase 4 (completada en esta iteración):
  - Sistema de tokens de diseño (CSS custom properties).
  - Tamaños migrados a `rem` + `clamp()` para tipografía fluida.
  - Accesibilidad: `:focus-visible` implementado en todos los controles.
  - Indicadores por severidad de corriente (baja/normal/alta/crítica) con color y peso tipográfico.
  - Preview del circuito con debounce, controles `[+]`/`[-]` y sincronización en tiempo real.
  - Responsive completo: breakpoints 760 px, 520 px, 400 px con prevención de overflow.
  - Botón Volver visible solo en la sección de resultados.

## Actualización de estado (2026-06-29)

- Fase 5 (completada en esta iteración):
  - Exportación real de resultados: `POST /api/export/csv` y `POST /api/export/pdf` con `reportlab`.
  - Botones "Exportar CSV" y "Exportar PDF" en la sección de resultados, asociados al `id` de la simulación.
  - Servicio `simulation_store` con buffer circular en memoria (50 entradas) registrado para `POST /api/calculate` y `POST /`.
  - Endpoints de historial: `GET /api/history`, `GET /api/history/<id>`, `DELETE /api/history` con código 404 para `id` desconocido.
  - Comparación de escenarios: `POST /api/compare` con deltas absolutos/porcentuales y clasificación de mejora/empeoramiento.
  - Reporte técnico agregado: `GET /api/report` con totales, distribución por severidad, potencia mín./máx./promedio y marcas temporales.
  - Sección de UI "Herramientas de análisis avanzado" con tarjetas para historial, comparación y reporte.
  - Total acumulado de pruebas: 46 (pytest), `ruff check` y `black --check` en verde.

## Actualización de estado (2026-06-29 — tarde)

- Mejoras de UX/estética (Fase 6 iteración previa al despliegue):
  - **Modo oscuro** con detección de preferencia del sistema y persistencia en `localStorage` (toggle en el header).
  - Tokens de diseño ampliados: cada color y superficie se expone como CSS custom property y se reescribe bajo `[data-theme="dark"]` para invertir toda la paleta sin tocar componentes.
  - **Notificaciones toast** con tipos `info/success/error` en esquina superior derecha (reemplazan `showError` inline).
  - **Diálogos de confirmación** modales para acciones destructivas (limpiar historial).
  - **Reusar simulación del historial**: clic en una card del historial o en el botón "↺ Reusar en calculadora" carga los parámetros al formulario principal y hace scroll a la calculadora.
  - **Sparkline** SVG por simulación en el historial (I₁, I₂, I₃).
  - **Comparación mejorada**: veredicto con flecha ↑/↓/→ y color, mini-barra de delta centrada en cero.
  - **Chart de potencia en el reporte** dibujado en `<canvas>` con soporte HiDPI y tema oscuro/claro automático.
  - **Contador animado** en los valores de I₁, I₂, I₃ al mostrar resultados.
  - **Iconos SVG inline** en encabezados de sección y subsección (sin dependencias externas).
  - **Barra de acciones sticky** en la sección de resultados (Exportar CSV / PDF / Volver).
  - **Atributo `data-severity`** en cada `.result-card` + franja lateral coloreada según severidad.
  - **Botón "Generando…"** mientras se prepara la exportación y toast `success` al completar.
  - Macros Jinja `severity_class` y `severity_token` para evitar duplicación en la plantilla.
  - Total acumulado de pruebas: 50 (pytest), `ruff check` y `black --check` en verde.

---

## Historial de fases completadas

- Fase 1 — Estabilización: completada.
- Fase 2 — Reestructuración de arquitectura: completada.
- Fase 3 — Calidad y pruebas: completada (con refuerzos posteriores).
- Fase 4 — Mejora visual y UX: completada.
- Fase 5 — Funcionalidad avanzada: completada.

## Actualización de estado (2026-11-10 — presentación)

- Defectos encontrados al capturar la aplicación para el README:
  - Las reglas `.severity-baja|normal|alta|critica` pintaban `background` global, así que el valor
    de corriente de severidad normal quedaba ilegible: letra y fondo del mismo azul. Las barras del
    reporte no usaban esas clases. Corregido acotando las reglas a `.sparkline rect.severity-*` con
    `fill`, que además hace que las barras del sparkline se pinten por primera vez.
  - El contador animado reemplazaba el texto completo «I₁ = 2.143 A» por un número suelto y escribía
    `0.000` antes del primer fotograma, de modo que una pestaña sin animación perdía el valor. Ahora
    anima sólo el número y conserva prefijo y sufijo.
  - La animación no respetaba `prefers-reduced-motion`; ahora se omite cuando el sistema pide menos
    movimiento.
- Capturas reales de la aplicación en `docs/capturas/`, enlazadas desde el README.
- Estos cambios son de CSS y JavaScript: no hay ejecutor de pruebas de interfaz en el proyecto, así
  que la verificación fue visual, con capturas tomadas de la aplicación servida por Gunicorn.

## Auditoría de seguridad (2026-11-10)

Revisión previa a la publicación de los commits. Hallazgos y correcciones:

- **polyfill.io retirado del HTML.** El dominio fue comprometido en 2024 y sirvió malware a los
  sitios que lo cargaban. Además, la configuración de MathJax se movió a
  `static/mathjax-config.js` para poder prohibir los scripts en línea.
- **Cabeceras de seguridad en todas las respuestas**: CSP, `X-Content-Type-Options`,
  `X-Frame-Options` y `Referrer-Policy`. Antes no había ninguna.
- **Tamaño de petición acotado a 64 KB** (`MAX_CONTENT_LENGTH`) con respuesta 413 que respeta el
  contrato de error en la API y la página de error en el formulario. Un `except Exception` en
  `api_calculate` convertía ese 413 en un 500: ahora se captura antes.
- **Límite de peticiones ampliado** al formulario `POST /` y a `/circuito.png`, que hacían el mismo
  trabajo caro que los POST de la API y se saltaban el límite.
- **Servidor de desarrollo en `127.0.0.1`** por defecto, con `HOST` para exponerlo a propósito.
- **Dependencias de ejecución separadas** de las de desarrollo: la imagen ya no instala `pytest`,
  `ruff` ni `black`. `werkzeug` fijado a 3.1.9 (PYSEC-2026-4312) y `pillow` a 13.0.0, que corrige
  veinte avisos abiertos en la línea 12.x. `black` subido a 26.10.1 sin cambios de formato.
- **Registros saneados**: la cadena de consulta se trunca y se le quitan saltos de línea antes de
  escribirla, para que la entrada del cliente no pueda falsear una línea de registro.
- Códigos de error unificados en mayúsculas (`RATE_LIMITED`, `PAYLOAD_TOO_LARGE`), como el resto del
  contrato.
- Total acumulado de pruebas: 72 (pytest), `ruff check` y `black --check` en verde.

## Cobertura medida (2026-11-10)

- `pytest-cov` añadido a las dependencias de desarrollo; el CI ejecuta la suite con cobertura y las
  pruebas de `tests/test_errors.py` cubren ahora las ramas de rechazo de la API, los validadores y las
  rutas HTML.
- Cobertura medida: **96 %** (de 92 %), con `src/validators/inputs.py` al 100 %. Lo que queda sin
  cubrir son las guardas `except Exception` que ya se ejercitan mediante inyección de fallos.
- Defectos corregidos al escribir esas pruebas:
  - Un valor fuera de rango en la API se reportaba como `CALCULATION_ERROR` («error de cálculo») en
    lugar de `INVALID_PAYLOAD`; el validador no capturaba su propia `ValueError`.
  - Un fallo no capturado en una ruta `/api/` devolvía HTML en lugar del contrato JSON. Ahora los
    handlers 404 y 500 responden JSON bajo `/api/` y siguen renderizando la página en el resto.
  - El almacén de simulaciones y el contador del límite son globales y se filtraban entre pruebas;
    `tests/conftest.py` los reinicia para que la suite sea determinista.
- Total acumulado de pruebas: 111 (pytest), `ruff check` y `black --check` en verde.

## Accesibilidad (2026-11-10)

- **Contraste corregido**: `--severity-high-color` era `#e6862e`, que sobre blanco da 2,7:1 y no
  alcanzaba el mínimo ni como texto grande. Ahora es `#b35a00` (4,8:1). El resto de la paleta supera
  4,5:1 en ambos temas; comprobado por cálculo, no a ojo.
- **El error del formulario se anuncia**: `role="alert"` en el bloque de error, que antes aparecía en
  pantalla pero no llegaba a un lector de pantalla.
- **Los resultados se anuncian**: `aria-live="polite"` en la rejilla de resultados, igual que ya
  tenían el historial, la comparación y el informe.
- **Mensaje de error en español**: cuando un campo no era numérico se mostraba el mensaje crudo de
  Python («could not convert string to float») sin decir qué campo fallaba. Ahora dice
  «R1: Debe ser un número válido».
- Ya estaban bien y no se tocaron: `:focus-visible` en todos los controles, 27 etiquetas asociadas a
  sus campos, `alt` en todas las imágenes, `aria-label` en los botones sólo-icono y `lang="es"`.
- Total acumulado de pruebas: 114 (pytest), cobertura 96 %, `ruff` y `black` en verde.

## Caso de estudio (2026-11-10)

- `docs/CASO-DE-ESTUDIO.md`: el problema, las decisiones técnicas, la verificación con cifras, los
  cinco defectos de interfaz y los seis hallazgos de seguridad, y qué se haría distinto. Enlazado
  desde la cabecera del README.

## Rediseño de la página (2026-11-10)

Crítica sobre capturas reales, no sobre impresiones. La página medía **12 000 px** de alto con
resultados y la herramienta quedaba sepultada bajo el material didáctico.

- **El material didáctico va plegado** con el elemento nativo details, sin JavaScript: los ocho
  pasos de la resolución matemática y las secciones «cómo se calcula» e «importancia». La página
  baja a unos 2 400 px con resultados.
- **La introducción pasa a «Cómo se usa»**: una línea de contexto visible y el resto detrás de un
  desplegable. En móvil, antes, la primera pantalla era un muro de texto y la calculadora no
  aparecía; ahora la calculadora entra en la primera pantalla.
- **Los desplegables son tarjetas**: heredan el mismo fondo, borde y sombra que las secciones, que
  antes sólo se aplicaban a las secciones hijas directas de main.
- Capturas del README regeneradas, incluida la vista móvil. Se toman con
  --force-prefers-reduced-motion para que el contador animado no congele cifras intermedias.
- 114 pruebas, ruff y black en verde.

## Cabecera con ondas de corriente (2026-11-10)

El señor pidió un salto visual y puso a disposición su biblioteca de diseños en
`entorno/animacion-lib` (7 GB, 1 108 archivos, 14 categorías).

- Se evaluaron 26 hero, 11 fondos, 12 rejillas y 11 animaciones SVG mediante hojas de contactos
  extraídas de sus vídeos con ffmpeg: 60 fotogramas revisados en dos mosaicos.
- Los heroes eran plantillas de agencia con titulares gigantes: no encajan con una herramienta de
  ingeniería. Se eligió el fondo **B5**: ondas de ruido Perlin en canvas 2D, sin WebGL ni
  dependencias, que evoca líneas de corriente.
- Se integró como `static/hero-waves.js` con la paleta del proyecto, opacidad contenida y máscara de
  degradado para no estorbar al texto. Se desactiva con `prefers-reduced-motion`.
- La cabecera estrena antetítulo, titular de tres líneas y subtítulo acotado.
- 114 pruebas, ruff y black en verde.

## Migración a Rust (2026-11-10)

Primera mitad de la migración, con paridad verificada contra la implementación Python.

- `rust/`: crate sin dependencias en el núcleo (dominio, solver 3×3 con pivoteo parcial, análisis) y
  `axum` para la API.
- **15 pruebas**: 6 de paridad generadas desde el oráculo Python (cinco casos numéricos con límites y
  los mensajes de rango) y 9 de contrato HTTP.
- Comparación en vivo de los dos servicios: **9 de 9 peticiones idénticas** (estado, cifras dentro de
  1e-9, textos de interpretación y cuerpos de error).
- Defectos de paridad encontrados y corregidos: el mensaje de parámetros faltantes usaba el formato de
  Rust en vez del de Python (`['R2']`, con comillas simples), y el formateo de los extremos de rango
  imprimía `1000` donde Python imprime `1000.0`.
- Pendiente: PNG del circuito, CSV, PDF, servir la interfaz y contenedor. La versión Python sigue
  siendo la de producción.

## Migración a Rust completada (2026-11-10)

- `rust/` implementa el sistema completo: cálculo, validación, API, CSV (idéntico byte a byte), PNG
  del circuito con plotters, PDF con printpdf y la misma interfaz servida con minijinja.
- **21 pruebas**: 6 de paridad numérica generadas desde el oráculo Python, 13 de contrato HTTP y 2
  del exportador.
- **Desplegado** en https://simulador-circuitos-malla-rust.onrender.com con imagen multi-stage, que
  necesita pkg-config y libfontconfig1-dev porque plotters dibuja texto con fontconfig.
- Diferencias conocidas: el PNG es un esquema equivalente, no idéntico píxel a píxel; las cifras
  coinciden dentro de 1e-9, no bit a bit; el Content-Type del CSV arrastra un charset duplicado que
  añade una capa del proxy, sin efecto funcional.
