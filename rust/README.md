# Núcleo en Rust

Porte del simulador desde Flask/Python. **La versión Python sigue siendo la que está en producción**;
esto es la migración, verificada por paridad pero todavía sin la interfaz web.

## Estado real

| Pieza | Estado |
| --- | --- |
| Cálculo de mallas (KVL 3×3, pivoteo parcial) | Hecho, paridad numérica verificada |
| Validación y mensajes de error | Hecho, texto idéntico al de Python |
| Interpretaciones, severidad y potencia | Hecho |
| API HTTP con `axum` | Hecho: salud, versión, ejemplo, cálculo, historial |
| PNG del circuito, CSV y PDF | **Pendiente** |
| Servir la interfaz (plantillas y estáticos) | **Pendiente** |
| Contenedor y despliegue | **Pendiente** |

## Cómo se comprueba la paridad

Las pruebas de `tests/parity.rs` no están escritas a mano: se generan desde la implementación
Python, que actúa como oráculo.

```bash
cargo test                                   # 15 pruebas
cargo clippy --all-targets -- -D warnings    # sin avisos
cargo fmt --check                            # formato limpio
```

Además se comparó el comportamiento en vivo: ambos servicios levantados a la vez, nueve peticiones
idénticas —cinco con resultado y cuatro con error— y **nueve coincidencias**: mismo código de estado,
mismas corrientes dentro de 1e-9, mismo texto de interpretación y mismo cuerpo de error.

## Arranque

```bash
cargo run --release
# escucha en http://0.0.0.0:8000, o en el puerto que indique PORT
```

## Servicio en marcha

Desplegado en https://simulador-circuitos-malla-rust.onrender.com (Render, plan gratuito, sin
tarjeta), con auto-despliegue desactivado: se publica a mano o por API.

## Estado actual

| Pieza | Estado |
| --- | --- |
| Cálculo de mallas (KVL 3×3) | Hecho, paridad numérica verificada |
| Validación y mensajes de error | Hecho, texto idéntico al de Python |
| Interpretaciones, severidad y potencia | Hecho |
| API HTTP con axum | Hecho, 13 pruebas de contrato |
| CSV | Hecho, byte a byte idéntico (CRLF incluido) |
| PNG del circuito | Hecho con plotters, 900×500 |
| PDF | Hecho con printpdf, una página |
| Interfaz web | Hecho: reutiliza la misma plantilla con minijinja |
| Contenedor multi-stage | Hecho y verificado con fontconfig |

21 pruebas, `clippy -D warnings` y `fmt --check` limpios.

### Diferencias conocidas

- El PNG del circuito es un esquema equivalente, no idéntico píxel a píxel al de Matplotlib.
- Las cifras coinciden dentro de 1e-9, no bit a bit: LAPACK y una eliminación gaussiana propia
  difieren en el último dígito.
- El historial guarda el instante como texto fijo; la marca temporal real está pendiente.

### Cómo se desplegó

Servicio `srv-dbpunnajnfac73ehpgk0` en Render, creado por API. Importante: la ruta del Dockerfile
debe ir **dentro** de `serviceDetails.envSpecificDetails`; si se pasa en la raíz del JSON, Render la
ignora y construye el `./Dockerfile` de la raíz, que es el de Python. El primer intento quedó así y
servía Python con nombre de Rust; se detectó porque `/api/metrics` respondía 200 y `version` no era
`0.1.0`.

Comprobación rápida de que el servicio es el de Rust:

```bash
curl -s https://simulador-circuitos-malla-rust.onrender.com/api/version   # 0.1.0
curl -s -o /dev/null -w '%{http_code}' https://simulador-circuitos-malla-rust.onrender.com/api/metrics  # 404
```

### Peculiaridad conocida

El Content-Type del CSV sale como text/csv; charset=utf-8; charset=utf-8: una capa del proxy de
Render anade su propio charset a esa respuesta concreta. Los estaticos servidos por tower-http salen
correctos, asi que no es el codigo de la aplicacion. Es cosmetico: los clientes usan el primer valor.
