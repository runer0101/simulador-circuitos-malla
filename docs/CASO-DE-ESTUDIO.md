# Caso de estudio — simulador de mallas

Cómo una aplicación de escritorio académica se convirtió en un servicio web terminado: qué se
decidió, qué se midió y qué se corrigió al mirarlo de cerca.

Repositorio: <https://github.com/runer0101/simulador-circuitos-malla>

---

## El problema

El análisis de mallas es un método de circuitos: se plantean las ecuaciones de Kirchhoff para cada
lazo y se resuelve un sistema lineal. En papel es manejable con tres mallas; a mano y con prisa, no.

La aplicación resuelve ese sistema para una instalación residencial de tres zonas —sala y comedor,
cocina y lavandería, dormitorios— y dice qué corriente circula por cada lazo, si es alta o crítica, y
cuánta potencia total consume el conjunto. Sirve para estudiantes que quieren comprobar su cálculo y
para quien quiera comparar escenarios antes de cambiar un tramo de cableado.

Lo que había era una aplicación de escritorio. Lo que hay es un servicio web con API, historial,
exportación, imagen del circuito y contenedor de producción.

---

## Las decisiones que importan

**Resolver con álgebra lineal, no con fórmulas cableadas.** `MeshParams::matrix` construye la matriz
de resistencias y `vector` el de fuentes; `solver::resolver` resuelve el 3×3 con eliminación gaussiana
y pivoteo parcial, sin dependencias. Cuando la matriz es singular —un lazo sin resistencia que cierre el
camino— el error se explica en lenguaje del problema («Sistema singular: Las resistencias crean un
circuito indeterminado»), no con el mensaje de una biblioteca.

Ojo con la letra pequeña: la topología no es un dato de entrada, vive en `matrix()`. Añadir una cuarta
malla, o compartir de otra forma las ramas, exige tocar esa función, el dibujo de la plantilla y la
tabla de ramas compartidas. Por eso el dibujo y la matriz se revisan juntos: si discrepan, el
programa resuelve un circuito que el estudiante no está viendo.

**Validar en el borde, una sola vez.** Rangos de resistencia (0,01–1000 Ω) y de tensión (0–500 V), con
NaN e infinito rechazados antes de tocar la matriz. La misma función valida el formulario, la API y
los parámetros de la imagen del circuito. Tres caminos de entrada, una sola definición de lo válido.

**El historial vive en memoria y lo dice.** Un búfer circular de 50 entradas permite comparar
escenarios y generar un informe agregado sin base de datos. Es una decisión de alcance, no un olvido:
la aplicación no guarda nada entre reinicios y el README lo advierte, porque presentarlo como
persistencia sería mentir.

**La imagen del circuito se genera aparte.** `render.rs` dibuja el PNG con `plotters` y el PDF con
`printpdf`, servidos como `/circuito.png` y en la exportación. Nada de JavaScript dibujando el
esquema: el resultado es idéntico en cualquier navegador y se puede enlazar.

---

## La verificación

| Qué | Cómo | Resultado |
| --- | --- | --- |
| Comportamiento | `cargo test` | Suite completa en verde: paridad numérica con la implementación Python que sirvió de oráculo, API, formulario, seguridad y exportación |
| Estilo | `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings` | Limpios, también en el CI |
| Servicio real | Imagen Docker, sonda `/api/health` | Arranca y responde `200` con un solo proceso |
| API | Pruebas de contrato por endpoint | Códigos de error uniformes |
| Dependencias | `Cargo.lock` fijado | Sin dependencias de ejecución fuera del árbol |

El CI hace tres cosas: comprueba formato y lints, ejecuta la suite y **construye la imagen y comprueba
el endpoint de salud dentro del contenedor**. Una imagen que no levanta rompe la construcción, que es
la única forma de que ese fallo no llegue a producción.

Las pruebas cubren también lo que suele quedarse fuera: ramas de rechazo de la API, valores fuera de
rango, identificadores inexistentes, cuerpos demasiado grandes y fallos internos inyectados.

---

## Lo que se corrigió al mirarlo de cerca

Cinco defectos que ninguna prueba unitaria iba a encontrar, porque sólo aparecen al usar la
aplicación:

1. **Los valores de corriente eran ilegibles.** Las reglas `.severity-normal` aplicaban un fondo
   global, y el resultado quedaba con letra y fondo del mismo azul. Se vio en una captura de pantalla,
   no en la suite.
2. **El sparkline del historial nunca se pintaba.** Las clases de severidad usaban `background`, que
   no afecta a un `<rect>` de SVG; hacía falta `fill`.
3. **El contador animado destruía el dato.** Escribía `0.000` antes del primer fotograma y
   reemplazaba el texto completo por un número suelto. En una pestaña sin animación, el resultado
   desaparecía.
4. **La animación ignoraba `prefers-reduced-motion`.**
5. **El naranja de severidad alta no se leía:** 2,7:1 sobre blanco. Ahora 4,8:1.

Y seis hallazgos de seguridad antes de publicar:

| Hallazgo | Corrección |
| --- | --- |
| Se cargaba `polyfill.io`, dominio comprometido en 2024 | Retirado |
| Sin cabeceras de seguridad | CSP estricta, `nosniff`, `DENY`, `Referrer-Policy` |
| Sin límite de tamaño de petición | 64 KB, con 413 correcto |
| El límite de peticiones esquivaba el formulario y el PNG | Ambos incluidos |
| Un 413 se convertía en 500 | Excepción capturada antes |
| `werkzeug` y `pillow` con 21 avisos abiertos | Versiones corregidas |

---

## Lo que haría distinto

- **Separar dependencias de ejecución desde el primer día.** La imagen instalaba `pytest`, `ruff` y
  `black` (eran tres dependencias de desarrollo en la imagen de producción) hasta que la auditoría lo
  señaló.
- **Mirar la aplicación, no sólo las pruebas.** Los cinco defectos de interfaz llevaban tiempo ahí;
  aparecieron en la primera captura de pantalla.
- **Revisar el dibujo contra la matriz.** La primera versión dibujaba tres ventanas en fila y etiquetaba
  R₆ como compartida entre la malla 1 y la 3: una fila no tiene esa rama, así que el esquema mostraba un
  circuito distinto del que se resolvía. Ninguna prueba unitaria lo detectó porque todas comparaban
  números, no la figura.
- **Un límite de peticiones por proceso no es un límite.** Con dos procesos el techo real es el doble
  del configurado. El despliegue fija uno solo mientras el contador no sea compartido.

## Lo que falta

El servicio está desplegado en un anfitrión real —`render.yaml` lo describe— con las limitaciones de
un plan gratuito: la instancia se duerme sin tráfico y el historial y el contador de peticiones viven
en memoria, así que se reinician con el proceso.
