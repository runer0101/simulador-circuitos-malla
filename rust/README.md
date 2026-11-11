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
