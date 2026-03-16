# simulador-circuitos-malla

> Aplicación web para analizar y optimizar la distribución de energía en instalaciones residenciales mediante el **método de análisis de mallas de Kirchhoff** y resolución matricial.

![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB?logo=python&logoColor=white)
![Flask](https://img.shields.io/badge/Flask-3.x-000000?logo=flask&logoColor=white)
![NumPy](https://img.shields.io/badge/NumPy-2.x-013243?logo=numpy&logoColor=white)
![Matplotlib](https://img.shields.io/badge/Matplotlib-3.10%2B-11557c?logo=python&logoColor=white)
![CI](https://github.com/runer0101/simulador-circuitos-malla/actions/workflows/ci.yml/badge.svg)
![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)
![Black](https://img.shields.io/badge/code%20style-black-000000)
![Tests](https://img.shields.io/badge/tests-23%20passed-brightgreen)

---

## Tabla de contenidos

- [Descripción](#descripción)
- [Funcionalidades](#funcionalidades)
- [Tech Stack](#tech-stack)
- [Arquitectura del proyecto](#arquitectura-del-proyecto)
- [Instalación y ejecución](#instalación-y-ejecución)
- [Uso de la interfaz](#uso-de-la-interfaz)
- [Referencia de API](#referencia-de-api)
- [Validaciones de entrada](#validaciones-de-entrada)
- [Flujo de desarrollo](#flujo-de-desarrollo)
- [Variables de entorno](#variables-de-entorno)
- [Roadmap](#roadmap)

---

## Descripción

Implementa la simulación web de un circuito residencial de tres mallas (sala/comedor, cocina/lavandería, dormitorios). A partir de valores de resistencias y fuentes de tensión configurables, resuelve el sistema de ecuaciones de Kirchhoff mediante álgebra matricial (NumPy) y genera un diagrama adaptativo del circuito (Matplotlib).

Orientado a estudiantes y académicos de ingeniería eléctrica para analizar la eficiencia y seguridad de la distribución de energía en instalaciones residenciales.

---

## Funcionalidades

- **Cálculo de corrientes de malla** I₁, I₂, I₃ con validación de entradas en servidor y cliente.
- **Diagrama de circuito adaptativo** en PNG según la cantidad de componentes activos (1–6 resistencias, 1–3 fuentes de tensión).
- **Preview en tiempo real** del circuito con debounce de 300 ms al modificar valores.
- **Selector dinámico de componentes** con controles `[+]`/`[-]` directamente en la interfaz.
- **Indicadores de severidad** por magnitud de corriente: baja, normal, alta y crítica.
- **Diseño responsive** para escritorio, tablet y celular (tres breakpoints: 760 px, 520 px, 400 px).
- **Navegación integrada** con botón *Volver* accesible solo en la sección de resultados.
- **Accesibilidad**: soporte de teclado con `:focus-visible` en todos los controles.
- **API JSON** con contrato de error uniforme y endpoints de observabilidad.

---

## Tech Stack

| Capa | Tecnología |
|---|---|
| Backend | Python 3.10+, Flask 3.x |
| Cálculo matricial | NumPy 2.x |
| Renderizado de circuitos | Matplotlib 3.10+ (backend Agg) |
| Plantillas | Jinja2 |
| Frontend | HTML5, CSS3, JavaScript (vanilla) |
| Calidad de código | Ruff, Black, pytest |
| CI/CD | GitHub Actions |

---

## Arquitectura del proyecto

```
simulacion_mallas/
├── app.py                        # Entry point
├── pyproject.toml                # Configuración de herramientas (pytest, ruff, black)
├── requirements.txt              # Dependencias declaradas con rangos
├── .github/
│   └── workflows/
│       └── ci.yml                # Pipeline de integración continua
├── src/
│   ├── app_factory.py            # Application factory, handlers de error, métricas
│   ├── config.py                 # Constantes y configuración centralizada
│   ├── routes/
│   │   ├── api.py                # Endpoints JSON (/api/*)
│   │   └── web.py                # Rutas HTML y renderizado de circuito
│   ├── services/
│   │   ├── mesh_analyzer.py      # Cálculo y análisis de mallas (KVL matricial)
│   │   └── circuit_renderer.py   # Generación de diagrama PNG
│   └── validators/
│       └── inputs.py             # Validación de resistencias y voltajes
├── static/
│   ├── main.js                   # Lógica de interfaz y preview del circuito
│   └── styles.css                # Estilos, tokens de diseño y responsive
├── templates/
│   └── index.html                # Plantilla Jinja2 principal
└── tests/
    ├── test_api.py               # Tests de endpoints API
    ├── test_mesh_analyzer.py     # Tests del motor de cálculo
    └── test_validators.py        # Tests de validación de entradas
```

---

## Instalación y ejecución

### Requisitos previos

- Python 3.10 o superior
- PowerShell (Windows) o bash (Linux/macOS)

### Windows (PowerShell)

```powershell
# 1. Clonar el repositorio
git clone https://github.com/runer0101/simulacion_mallas.git
cd simulacion_mallas

# 2. Crear y activar entorno virtual
py -m venv .venv
.\.venv\Scripts\Activate.ps1

# 3. Instalar dependencias
pip install -r requirements.txt

# 4. Ejecutar
python app.py
```

### Linux / macOS

```bash
git clone https://github.com/runer0101/simulacion_mallas.git
cd simulacion_mallas

python3 -m venv .venv
source .venv/bin/activate

pip install -r requirements.txt
python app.py
```

Abrir en el navegador: **http://127.0.0.1:5000**

Para detener el servidor: `Ctrl + C`

### Liberar el puerto 5000 en Windows (si queda ocupado)

```powershell
$conn = Get-NetTCPConnection -LocalPort 5000 -State Listen -ErrorAction SilentlyContinue
if ($conn) { Stop-Process -Id $conn.OwningProcess -Force }
```

---

## Uso de la interfaz

1. Seleccionar la cantidad de resistencias (1–6) y fuentes de tensión (1–3) con los controles `[+]`/`[-]`.
2. Ingresar los valores de cada componente. El preview del circuito se actualiza automáticamente.
3. Hacer clic en **Calcular** para resolver el sistema de mallas.
4. Los resultados muestran las corrientes I₁, I₂ e I₃ con indicadores de severidad por color.
5. Usar el botón **Volver** (visible solo en la sección de resultados) para regresar a la calculadora.

---

## Referencia de API

### `GET /api/health`

Healthcheck del servicio.

```json
{ "status": "ok", "service": "simulador-circuitos-malla" }
```

### `GET /api/version`

Versión activa del servicio.

```json
{ "version": "1.0.0" }
```

### `GET /api/metrics`

Métricas básicas de uso en memoria (conteo de llamadas por ruta).

```json
{ "routes": { "/api/calculate": 12, "/": 4 } }
```

### `GET /api/example`

Devuelve valores de ejemplo listos para ingresar en el formulario.

### `POST /api/calculate`

Calcula las corrientes de malla I₁, I₂ e I₃.

**Headers:** `Content-Type: application/json`

**Body:**

```json
{
  "R1": 10, "R2": 20, "R3": 15,
  "R4": 30, "R5": 10, "R6": 25,
  "V1": 120, "V2": 60, "V3": 90
}
```

**Respuesta exitosa (`200`):**

```json
{
  "success": true,
  "I1": 2.143,
  "I2": 1.857,
  "I3": 0.714,
  "interpretacion": ["...", "..."]
}
```

**Contrato de error uniforme:**

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Valor fuera de rango",
    "details": "R1 debe estar entre 0.01 y 1000 ohm"
  }
}
```

| Código HTTP | Causa |
|---|---|
| `400` | JSON malformado o campos faltantes |
| `400` | Valores fuera de rango |
| `415` | `Content-Type` distinto de `application/json` |

---

## Validaciones de entrada

| Componente | Mínimo | Máximo |
|---|---|---|
| Resistencias (R1–R6) | 0.01 Ω | 1 000 Ω |
| Fuentes de tensión (V1–V3) | 0 V | 500 V |

---

## Flujo de desarrollo

```powershell
# Activar entorno
.\.venv\Scripts\Activate.ps1

# Ejecutar tests
python -m pytest -q

# Verificar estilo (lint)
python -m ruff check .

# Verificar formato
python -m black --check .

# Corregir formato automáticamente
python -m black .
```

El pipeline de CI (GitHub Actions) ejecuta los tres pasos en cada `push` y `pull_request`.

---

## Variables de entorno

| Variable | Descripción | Valor por defecto |
|---|---|---|
| `FLASK_DEBUG` | Modo debug de Flask | `false` (seguro en producción) |
| `APP_VERSION` | Versión reportada por `/api/version` | `1.0.0` |

---

## Roadmap

Ver [MEJORAS_POR_FASES.md](MEJORAS_POR_FASES.md) para el plan completo de evolución.

**Próximas mejoras planificadas:**

- Exportación de resultados (CSV / PDF).
- Historial de simulaciones en sesión.
- Comparación de escenarios antes/después.
- Contenerización con Docker.
- Despliegue en producción con WSGI (Gunicorn).

---

> **simulador-circuitos-malla** — Proyecto académico, Universidad Privada Domingo Savio (UPDS), 2026.  
> Tema: *Optimización de sistemas de distribución de energía en instalaciones residenciales.*


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
