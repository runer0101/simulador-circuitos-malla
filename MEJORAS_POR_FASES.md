# Plan de mejoras por fases

Este documento mantiene solo el backlog pendiente. Las tareas completadas se resumen al final.

## Backlog activo

### Fase 4 — Mejora visual y UX

- Crear sistema de diseño básico (tokens de color, tipografía y espaciado).
- Migrar tamaños críticos de `vw` a `rem` + `clamp()` para mejor legibilidad.
- Mejorar accesibilidad:
  - `:focus-visible`
  - contraste de texto
  - mensajes de error accesibles
- Mejorar visualización de resultados:
  - resumen compacto
  - indicadores por severidad de corriente
  - presentación más limpia en móvil

### Fase 5 — Funcionalidad avanzada

- Exportación real de resultados (CSV/PDF) con botón en UI.
- Historial de simulaciones en memoria o almacenamiento liviano.
- Comparación de escenarios (antes/después) para análisis de optimización.
- Endpoint de reporte técnico con métricas agregadas.

### Fase 6 — Despliegue y operación (pendiente)

- Contenerización con Docker.
- Configuración de producción (WSGI, variables de entorno, logs estructurados).

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

## Historial de fases completadas

- Fase 1 — Estabilización: completada.
- Fase 2 — Reestructuración de arquitectura: completada.
- Fase 3 — Calidad y pruebas: completada (con refuerzos posteriores).
