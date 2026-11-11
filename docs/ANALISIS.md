# Análisis del proyecto: para quién es y cómo debe funcionar

Documento de decisión. Se escribe antes de tocar el código, para no seguir corrigiendo a ciegas.

---

## 1. Para quién es esto

**Público principal**: estudiantes de ingeniería eléctrica o electrónica en el segundo o tercer
semestre, en la unidad de «métodos de análisis de circuitos». Están viendo el método de mallas por
primera vez y tienen una tarea que dice algo así:

> *Para el circuito de la figura, obtenga: a) las corrientes I₁, I₂ e I₃; b) la potencia en las
> resistencias de 3 Ω y 5 Ω; c) qué valor de la fuente produciría I₂ = 0 A.*

**Público secundario**: el profesor que quiere un ejercicio resuelto para mostrar en clase.

**Lo que NO es**: no es una herramienta de ingeniería profesional. Nadie diseña una instalación real
con esto. Si el proyecto se presenta como «simulador profesional» pierde credibilidad; si se presenta
como «resuelve y comprueba tu tarea», la gana.

## 2. Qué necesita ese estudiante, en orden

1. **Entender qué le están pidiendo.** Qué es una malla y por qué se plantea una ecuación por malla.
2. **Ver el circuito como lo dibuja su libro**, no como lo dibuja un programador.
3. **Meter sus datos** sin dudar cuál va dónde.
4. **Obtener el resultado** con las cifras que le piden: corrientes, potencias por resistencia.
5. **Comprobar si su cálculo a mano coincide.** Es lo que hace al terminar.
6. **Entender el resultado**: por qué esa corriente, si es peligrosa, qué significa el signo.

## 3. Las convenciones de la asignatura (investigadas)

Los materiales de referencia (Spinning Numbers, apuntes universitarios de la UNAM, la UC3M y la
Universidad de Málaga) coinciden en esto:

- Una **malla** es un lazo que no contiene otro lazo: una «ventana» del circuito. Se numeran I, II, III.
- La **corriente de malla** se dibuja como una **flecha circular dentro de la ventana**, con sentido
  horario o antihorario, pero **el mismo para todas**.
- Las **corrientes de elemento** (las que pasan por cada resistencia) se marcan aparte, sobre la rama.
  En una rama compartida es la diferencia de las dos corrientes de malla.
- Las tensiones se marcan con **+ y −** según la convención pasiva.
- El procedimiento es siempre el mismo: identificar mallas → dibujar corrientes → escribir la ley de
  tensiones en cada malla → resolver el sistema → obtener corrientes y tensiones de cada elemento.

**Consecuencia directa para el dibujo**: las flechas sueltas que había no representan una corriente de
malla. Hace falta un arco cerrado dentro de cada ventana, y las corrientes de elemento sólo en las
ramas compartidas.

## 4. Cómo debe actuar la aplicación

Un recorrido de cuatro pantallas o bloques, en este orden y con este propósito cada uno:

| Bloque | Qué debe conseguir | Qué NO debe hacer |
| --- | --- | --- |
| 1. Qué es y para qué sirve | Que en treinta segundos sepa si esto le sirve para su tarea | Enseñar teoría extensa |
| 2. El circuito | Que reconozca la figura de su libro: ventanas, flechas circulares, valores | Ser un dibujo decorativo |
| 3. Los datos | Que sepa qué es cada número y a qué malla pertenece | Pedir nueve campos sin contexto |
| 4. El resultado | Las cifras que le piden, la comprobación y si puede estar tranquilo | Sólo tres números |

Y una acción transversal: **comprobar su resultado a mano**, que es lo que cierra el estudio.

## 5. Decisiones que se derivan

- El **diagrama es la pieza central**, no un adorno: tiene que usar las convenciones del punto 3.
- Las **potencias por resistencia** son obligatorias, porque el enunciado las pide.
- La **comprobación de la ley de tensiones** también: es el paso que el profesor corrige.
- La **nomenclatura** debe ser la de la asignatura: mallas I, II, III o 1, 2, 3; corrientes I₁, I₂, I₃;
  resistencias R₁…R₆; tensiones V₁…V₃.
- El **lenguaje**, el de un estudiante, no el de un ingeniero veterano: «cable que se calienta» antes
  que «sobrecarga térmica del conductor».
- **Todo en Rust**, con una sola dirección pública.

## 6. Lo que falta y lo que sobra

**Falta**

- El dibujo con la convención correcta: ventana por malla y flecha circular.
- Marcar las corrientes de elemento en las ramas compartidas.
- La pregunta de los enunciados: **qué tensión haría I₂ = 0**.
- Marcar + y − en las resistencias, si se quiere ser fiel al libro.

**Sobra o estorba**

- Los bloques largos de matemáticas con todos los pasos: se conservan plegados, pero no son la
  entrada.
- Comparación de escenarios y reporte agregado: son útiles, pero **no son lo que pide la tarea**.
  Deben quedar al final y discretos, no compitiendo con el resultado.

## 7. Cómo se sabrá si funciona

Una prueba concreta: darle a un estudiante el enunciado de un circuito de tres mallas y pedirle que
llegue a las potencias por resistencia **sin ayuda**. Si lo consigue en menos de cinco minutos, el
diseño sirve. Si pregunta «¿y esto qué es?», no.
