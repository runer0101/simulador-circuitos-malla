# Simulación de Circuitos Eléctricos en Mallas

Aplicación web para analizar corrientes de malla en un circuito residencial usando el método de Kirchhoff y resolución matricial con NumPy.

## Stack

- Python 3.10+
- Flask 3+
- NumPy 2+
- Matplotlib 3.10+
- HTML/CSS/JavaScript

## Estructura

- app.py: Entry point de ejecución
- src/app_factory.py: Creación de aplicación y registro de errores
- src/routes/web.py: Rutas HTML y generación de circuito
- src/routes/api.py: Endpoints JSON
- src/services/mesh_analyzer.py: Cálculo, validación y utilidades de dominio
- src/services/circuit_renderer.py: Render de circuito PNG
- templates/index.html: Vista principal
- static/main.js: Interacción y validación cliente
- static/styles.css: Estilos
- requirements.txt: Dependencias
- MEJORAS_POR_FASES.md: Roadmap técnico por fases

## Ejecutar en local (Windows PowerShell)

1. Ir al proyecto:
   cd "c:\Users\ccama\Proyectos\proyectos-web\simulacion_mallas"
2. Crear/activar entorno virtual:
   py -m venv .venv
   .\.venv\Scripts\Activate.ps1
3. Instalar dependencias:
   pip install -r requirements.txt
4. Ejecutar:
   python app.py
5. Abrir:
   http://localhost:5000

Para apagar el servidor: Ctrl + C en la misma terminal.

## Endpoints

- GET / : Interfaz web
- POST /api/calculate : Cálculo por API
- GET /api/example : Carga ejemplo
- GET /api/health : Healthcheck del servicio
- GET /api/version : Versión activa del servicio
- GET /api/metrics : Métricas básicas de uso en memoria
- GET /circuito.png : Diagrama de circuito en PNG

## Rangos de validación

- Resistencias: 0.01 a 1000 Ω
- Voltajes: 0 a 500 V

## Estado de la reorganización

Se aplicó una limpieza de código y estructura en esta iteración:

- Eliminación de JavaScript no usado
- Validaciones cliente/servidor alineadas
- Limpieza de estilos inline críticos
- Configuración de Matplotlib para entorno servidor (Agg)
- Dependencias actualizadas para Python moderno

## Próximos pasos

Consultar MEJORAS_POR_FASES.md para el plan completo de evolución del proyecto.
