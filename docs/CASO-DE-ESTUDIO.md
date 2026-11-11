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

**Resolver con álgebra lineal, no con fórmulas cableadas.** `MeshAnalyzer` construye la matriz de
resistencias y el vector de fuentes, y resuelve con NumPy. Una topología nueva no exige código nuevo.
Cuando la matriz es singular —un lazo sin resistencia que cierre el camino— el error se explica en
lenguaje del problema, no con el mensaje de NumPy.

**Validar en el borde, una sola vez.** Rangos de resistencia (0,01–1000 Ω) y de tensión (0–500 V), con
NaN e infinito rechazados antes de tocar la matriz. La misma función valida el formulario, la API y
los parámetros de la imagen del circuito. Tres caminos de entrada, una sola definición de lo válido.

**El historial vive en memoria y lo dice.** Un búfer circular de 50 entradas permite comparar
escenarios y generar un informe agregado sin base de datos. Es una decisión de alcance, no un olvido:
la aplicación no guarda nada entre reinicios y el README lo advierte, porque presentarlo como
persistencia sería mentir.

**La imagen del circuito se genera aparte.** Matplotlib con backend `Agg`, servida como PNG. Nada de
JavaScript dibujando el esquema: el resultado es idéntico en cualquier navegador y se puede enlazar.

---

## La verificación

| Qué | Cómo | Resultado |
| --- | --- | --- |
| Comportamiento | `pytest` con cobertura | **114 pruebas**, **96 %** de cobertura |
| Estilo | `ruff` y `black` | Limpios, también en el CI |
| Servicio real | Imagen Docker, sonda de salud | Arranca con Gunicorn y dos workers, `healthy` |
| API | Pruebas de contrato por endpoint | 12 endpoints, códigos de error uniformes |
| Dependencias | `pip-audit` | Sin avisos en dependencias de ejecución |

El CI hace tres cosas: comprueba estilo, ejecuta la suite con cobertura y **construye la imagen y
comprueba el endpoint de salud dentro del contenedor**. Una imagen que no levanta rompe la
construcción, que es la única forma de que ese fallo no llegue a producción.

Las pruebas cubren también lo que suele quedarse fuera: ramas de rechazo de la API, valores fuera de
rango, identificadores inexistentes, cargas demasiado grandes y fallos internos inyectados. Una prueba
calcula el contraste de color de la severidad alta y falla si baja del mínimo legible.

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
  `black` hasta que la auditoría lo señaló.
- **Mirar la aplicación, no sólo las pruebas.** Los cinco defectos de interfaz llevaban tiempo ahí;
  aparecieron en la primera captura de pantalla.
- **Un límite de peticiones por proceso no es un límite.** Con dos workers de Gunicorn el techo real
  es el doble del configurado. Está documentado, pero un contador compartido sería lo correcto si el
  servicio creciera.

## Lo que falta

El despliegue en un anfitrión real está pendiente: la imagen se construye y se verifica, pero no hay
URL pública. Un proyecto así se termina cuando alguien más puede abrirlo.
