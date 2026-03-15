# Simulación de Circuitos Eléctricos en Mallas

Aplicación web para analizar corrientes de malla en un circuito residencial usando el método de Kirchhoff y resolución matricial con NumPy.

## Stack

- Python 3.10+
- Flask 3+
- NumPy 2+
- Matplotlib 3.10+
- HTML/CSS/JavaScript

## Funcionalidades

- Cálculo de corrientes de malla I1, I2, I3 con validación de entradas.
- Render dinámico del circuito en PNG mediante /circuito.png.
- Vista técnica adaptativa del circuito según la cantidad de resistencias y voltajes activos.
- Selector de cantidad de resistencias y voltajes activos desde la interfaz con actualización del preview.
- Vista de resultados integrada con botón Volver visible solo dentro de la sección de resultados.
- Diseño responsive para escritorio, tablet y celular.
- Manejo uniforme de errores en API JSON.

## Estructura

- app.py: Entry point de ejecución.
- src/app_factory.py: Creación de aplicación, handlers de error y registro de métricas.
- src/routes/web.py: Rutas HTML y endpoint de imagen de circuito.
- src/routes/api.py: Endpoints JSON de cálculo y operación.
- src/services/mesh_analyzer.py: Cálculo, validación y utilidades de dominio.
- src/services/circuit_renderer.py: Render de circuito PNG.
- src/services/usage_metrics.py: Métricas en memoria por ruta.
- templates/index.html: Vista principal.
- static/main.js: Interacción de formulario y preview del circuito.
- static/styles.css: Estilos y reglas responsive.
- tests/: Pruebas unitarias/funcionales.
- MEJORAS_POR_FASES.md: Roadmap técnico por fases.

## Ejecutar en local (Windows PowerShell)

1. Ir al proyecto:
   cd "c:\Users\ccama\Proyectos\proyectos-web\simulacion_mallas"
2. Crear y activar entorno virtual:
   py -m venv .venv
   .\.venv\Scripts\Activate.ps1
3. Instalar dependencias:
   pip install -r requirements.txt
4. Ejecutar:
   python app.py
5. Abrir:
   http://127.0.0.1:5000

Para apagar el servidor: Ctrl + C en la misma terminal.

Si el puerto 5000 queda ocupado, puedes cerrar el proceso con:

```powershell
$conn = Get-NetTCPConnection -LocalPort 5000 -State Listen -ErrorAction SilentlyContinue
if ($conn) { Stop-Process -Id $conn.OwningProcess -Force }
```

## Flujo diario recomendado

1. Activar entorno virtual:
   .\.venv\Scripts\Activate.ps1
2. Iniciar la app:
   python app.py
3. Trabajar y probar en navegador:
   http://127.0.0.1:5000
4. Validar calidad antes de commit:
   pytest -q
   ruff check .
   black --check .
5. Cerrar servidor:
   Ctrl + C

## Endpoints

### Web

- GET /: Interfaz web principal.
- GET /circuito.png: Diagrama de circuito en PNG con parámetros de query.

Parámetros soportados en /circuito.png:

- R1 a R6
- V1 a V3
- resistance_count: entero entre 1 y 6
- voltage_count: entero entre 1 y 3

### API

- POST /api/calculate: Cálculo por API.
- GET /api/example: Carga de valores de ejemplo.
- GET /api/health: Healthcheck del servicio.
- GET /api/version: Versión activa del servicio.
- GET /api/metrics: Métricas básicas de uso en memoria.

## Contrato de error API

En errores, la API responde en formato uniforme:

- success: false
- error.code
- error.message
- error.details

Casos importantes en /api/calculate:

- 415 Unsupported Media Type cuando no se envía JSON.
- 400 Bad Request cuando el JSON está malformado.

## Rangos de validación

- Resistencias: 0.01 a 1000 ohm.
- Voltajes: 0 a 500 V.

## Calidad

Comandos recomendados antes de publicar cambios:

- pytest -q
- ruff check .
- black --check .

Estado actual validado localmente:

- pytest
- ruff
- black

## Variables de entorno

- FLASK_DEBUG: por defecto seguro en false.
- APP_VERSION: valor reportado por /api/version.

## UX de la interfaz

- El preview del circuito se actualiza en función de los valores ingresados y de la cantidad de componentes activos.
- La navegación desde resultados incluye un botón Volver hacia la calculadora.
- En móviles pequeños se reducen bordes, paddings y tamaños para evitar desbordes horizontales.

## Próximos pasos

Consultar MEJORAS_POR_FASES.md para el plan completo de evolución del proyecto.
