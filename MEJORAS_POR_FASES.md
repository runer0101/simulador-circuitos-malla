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
