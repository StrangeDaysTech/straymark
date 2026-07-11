---
slug: what-a-million-assertions-cant-settle
title: Lo que un millón de aserciones no puede zanjar
authors:
  - jose
tags:
  - straymark
  - audit
  - attribution
  - governance
  - sentinel
  - data
date: 2026-07-10T00:00:00.000Z
description: La reescritura de Bun en Rust corrió revisión adversarial de IA a una escala que vale la pena estudiar — una sola familia de modelo, ventanas de contexto separadas, y 1.386.826 aserciones de test como juez final. Las auditorías de StrayMark se ven distintas porque el objetivo es distinto, y la diferencia no es de estilo. Cuando un suite de tests que pasa puede ser el juez, aíslas a los revisores por contexto. Cuando el juez es un juicio humano, diversificas por familia de modelo — y 25 ciclos de auditoría con datos reales muestran exactamente por qué, incluida una familia que se quedó callada y ciega.
---
*Bun se reescribió de Zig a Rust en once días apoyándose en revisión adversarial de IA — un hito genuino, y uno que vale la pena leer con cuidado por lo que dice sobre cómo desconfías del trabajo de una máquina. Su método y el nuestro se ven distintos, y la diferencia no es de gusto: la dicta qué cosa llega a ser el juez. Bun tenía 1.386.826 aserciones de test y CI en seis plataformas, así que un suite que pasa podía zanjar la corrección, y aislaron a los revisores por ventana de contexto dentro de una sola familia de modelo. StrayMark audita cambios donde la "respuesta correcta" es un juicio sin test que lo zanje — así que diversificamos entre familias de modelo y tratamos su convergencia independiente como la señal. Veinticinco ciclos reales de auditoría en el proyecto Sentinel muestran qué compra eso, qué caza que los tests verdes no ven, y — como curiosidad — cómo se comportaron de verdad siete familias de modelo distintas, incluida una que hubo que degradar por quedarse callada y ciega.*

<!-- truncate -->

> *El ciclo v0 sobre este mismo Charter produjo 0 hallazgos sustantivos. El ciclo v1 sobre el mismo código, con las mismas familias de modelo auditoras, produjo 1 bug runtime-fatal verificado más 3 hallazgos más profundos. El uso de herramientas de sistema de archivos y el git range completo convierten el teatro basado en pegar texto en revisión de ingeniería de verdad.*

Esta es una coda de [Quién creyó la auditoría que era](who-the-audit-thought-it-was) — aquel post iba de volver imposible de fingir la atribución de la auditoría; este va de qué hicieron las auditorías de verdad, con números. Lo detonó el [post de la reescritura de Bun en Rust](https://bun.com/blog/bun-in-rust), que describe una metodología de auditoría con IA a una escala que hay que tomar en serio, y contrastar con honestidad.

## Dos formas de desconfiar de una máquina

El proceso de Bun está bellamente diseñado. Jarred Sumner reescribió Bun con unos 50 workflows dinámicos de Claude a lo largo de 11 días: **un implementador, dos o más revisores adversariales, y un fixer**, con los revisores corriendo en **ventanas de contexto separadas** e instruidos para asumir que el código está mal — porque "el Claude que escribió el código quiere que se acepte; el Claude que revisa quiere encontrarle fallos." En el pico, ~64 Claudes a través de 4 worktrees de git; 6502 commits; ~\$165 000 en tokens. Y la parte que sostiene todo: **todo se apoyaba en un oráculo objetivo** — el suite de tests de Bun está escrito en TypeScript, así que no depende del lenguaje del runtime; **1.386.826 aserciones `expect()` a través de 60 624 tests** pasaron en las seis plataformas antes de que Sumner "presionara el botón de merge", tras verificar a mano que los tests no se estuvieran saltando.

Ese último hecho es lo que vuelve el diseño de una-sola-familia y contexto-partido exactamente correcto para el trabajo. Una reescritura Zig→Rust es un **port que preserva el comportamiento**: "correcto" tiene una definición limpia y comprobable por máquina. Los revisores adversariales no tienen que ser la última palabra, porque el suite de un millón de aserciones sí lo es. La independencia-por-contexto basta, porque lo que sea que los revisores pierdan, el oráculo lo caza.

StrayMark audita el caso en que ese oráculo no existe. La mayoría de la ingeniería no es un port; la pregunta es *¿es este el diseño correcto?, ¿aguanta bajo concurrencia?, ¿este contrato todavía coincide con la decisión que lo definió?* — juicios sin ningún suite que los zanje. Cuando la revisión **es** el oráculo, el punto ciego de una sola mente es invisible: si tu único revisor es sistemáticamente ciego a una clase de bug, nada aguas abajo lo cazará. Así que el ciclo de auditoría de StrayMark diversifica a lo largo del eje que Bun no necesitó — **la familia de modelo** — y trata la *convergencia independiente entre familias* como la señal más fuerte, con un modelo calibrador aparte que reconcilia los reportes y puntúa a los auditores.

| | Auditoría de la reescritura de Bun | Ciclo de auditoría de StrayMark |
|---|---|---|
| **Qué zanja "correcto"** | Un oráculo objetivo — 1.386.826 aserciones de test, CI en 6 plataformas | Un juicio dirigido por humano; ningún test lo zanja |
| **La independencia viene de** | Ventanas de contexto separadas (autor vs. revisor adversarial) | **Familias de modelo** separadas auditando a ciegas, más separación de contexto |
| **Diversidad de modelos** | Una familia (Claude / Fable 5), muchas instancias | Siete familias en el corpus — gpt, gemini, glm, qwen, claude, deepseek, kimi |
| **Reconciliación** | Un fixer aplica las notas del revisor | Un modelo calibrador consolida, dedupea y puntúa a cada auditor |
| **El humano** | Vigila el loop, verifica que los tests corrieron, presiona merge | Dirige la identidad, adjudica la convergencia, decide el cierre |

Ninguno es "mejor". Están calibrados a objetivos distintos. El de Bun es la forma correcta cuando un suite puede ser el juez; el nuestro es la forma que necesitas cuando no puede.

## Por qué diversidad de familia, no solo de contexto

Aquí está el argumento, y luego los datos que lo respaldan. Si el revisor es el juez final y corres una sola familia de modelo, estás apostando a que esa familia no tiene ningún punto ciego sistemático. Esa apuesta es incomprobable desde adentro — un punto ciego es, por definición, aquello que no puedes ver. La única forma de sacarlo a la luz es preguntarle a una mente *distinta* y notar el desacuerdo. La diversidad entre familias no va de redundancia; va de volver los puntos ciegos *representables*.

Sentinel — el proyecto que StrayMark gobierna y sobre el que hace dogfood — ha corrido ya **25 ciclos de auditoría a través de 21 Charters, 68 reportes de auditor, siete familias de modelo**, del 2026-05-05 al 2026-07-10. Y los datos dicen que la independencia hace trabajo real. De los hallazgos consolidados, alrededor del **70% los levantó un solo auditor** — familias distintas genuinamente sacan cosas distintas — mientras que el ~**19% que convergió entre familias** es exactamente lo que el calibrador marca como de mayor confianza. Del review de `CHARTER-40`: *"el tope de presupuesto es un check-then-act no atómico — encontrado independientemente por gpt-5-codex Y claude-fable, la señal convergente más fuerte."*

## Lo que las auditorías cazaron y los tests verdes no

El punto de todo esto es la calidad del software, así que aquí está la calidad, cuantificada. A través de esos 25 ciclos: **211 hallazgos crudos**, consolidados a **169**, desglosados por categoría como **92 implementation-gaps, 89 real-debt, 23 falsos positivos, 6 alucinaciones**. De las **188 entradas de follow-up** de Sentinel, 55 citan una auditoría; **tres de sus cuatro entradas de deuda técnica trazan directamente a reviews de auditoría** (una — un guard de scope en `CommsHub` que se declaraba pero nunca se invocaba de verdad, un hueco de auth — se promovió a TDE y luego se resolvió).

Pero los conteos lo subestiman. El tema recurrente son defectos que un **suite de tests verde y la revisión humana ambos dejaron pasar**, porque no eran la clase de cosa que un test estuviera vigilando:

- **Un bug de SQL runtime-fatal (`CHARTER-07`).** Una cláusula `ON CONFLICT` sobre una expresión `COALESCE(col, '')` que Postgres rechaza en runtime — *"los primeros Charters consumidores lo golpearían en el primer intento de escritura."* El gate de compilación estructuralmente no puede ver un error semántico de SQL. `gpt-5.3-codex` lo cazó; `gemini` lo perdió.
- **Una race de concurrencia bajo un suite verde (`CHARTER-17`).** La implementación shippeó *"build + vet + suite unit + integración verdes, 12/12 tareas declaradas."* El auditor aun así encontró un `SELECT`-luego-`INSERT` bajo READ COMMITTED donde dos réplicas ambas ven `isNew=true` — *"efectos secundarios duplicados: correos de admin, emisiones de texto legal"* — confirmado contra la topología multi-instancia real de `cloud-run.yaml`. Ningún test de un solo nodo lo reproduciría jamás.
- **Un bug que bloquea producción, enmascarado por sus propios tests (`CHARTER-42`).** Los tests de `SafeMode` *"lo enmascararon inyectando un cliente MCP falso permisivo."* Verde, y equivocado.
- **Hallazgos de seguridad de LLM (`CHARTER-40`).** Una acción `AUTO` que dejaba al modelo hacer una supresión global sobre un correo elegido por el modelo; el razonamiento en texto libre del modelo persistido en una fila de auditoría *"marcada `PIIRedacted: true` sin haber pasado nunca por el redactor."*

Y el dato aislado más convincente es un antes/después natural sobre el *mismo Charter*: el ciclo de auditoría legacy v0 — basado en pegar texto, sin uso de herramientas — produjo **0 hallazgos sustantivos**; el ciclo v1, mismo código y mismas familias de modelo pero con uso de herramientas de sistema de archivos y el git range completo, produjo **el bug de SQL runtime-fatal más tres hallazgos más profundos**. La metodología, no los modelos, es lo que convirtió el "se ve bien" en un defecto de producción cazado.

## Una curiosidad: cómo se comportaron de verdad las siete familias

Como cada review puntúa a sus auditores con una rúbrica fija — **precisión de scope (25%), profundidad técnica (25%), detección de bugs (30%), tasa de falsos positivos (20%)** — el corpus funciona también como un registro de comportamiento franco, aunque pequeño y acotado en el tiempo. Agregado a través de los 25 ciclos:

| Familia de modelo | Corridas | Hallazgos levantados | Rango de rating | En una línea |
|---|--:|--:|---|---|
| **gpt** (5.2 / 5.3 / 5-codex / 5.5) | 26 | 99 | 7.6 – 9.8 | El caballo de batalla — corrió `go build`/`go test`, cazó los bugs materiales, "cargó la auditoría." |
| **glm** (5.1 / 5.2) | 6 | 37 | 7.9 – 9.4 | Recién llegado fuerte; alto volumen, inflación ocasional de falsos positivos. |
| **claude-fable-5** | 2 | 24 | 8.4 – 9.2 | El único auditor que *corrió los comandos de verificación declarados*; tiende a sub-severizar. |
| **qwen** (3.7-max / plus) | 6 | 23 | 5.5 – 9.0 | Variable; fuerte en los batches de backend y gestión de llaves. |
| **gemini** (2.5-pro, 3.1-pro, cli) | 24 | 19 | 2.1 – 10 | Ver abajo — la advertencia con moraleja. |
| **deepseek-v4-pro** | 2 | 5 | 6.4 – 6.7 | Profundidad moderada. |
| **kimi-k26** | 1 | 4 | 5.4 | Infló dos severidades, perdió los hallazgos materiales. |

La línea de gemini es la que vindica todo el diseño, y está documentada en el corpus como un patrón rastreado (`FU-030-003`). A través de **24 corridas levantó 19 hallazgos en total** — y, más al punto, seguía *perdiendo los reales*. El calibrador lo escaló a lo largo de cuatro ciclos de una observación a una recomendación formal: *"degradar a advisory-only… poco fiable en hallazgos de clase integration-gap — bypass de RLS, wire-up faltante, deriva de convención."* En una auditoría de seguridad puntuó **2.1/10**, tras *"no detectar ninguno de los nueve hallazgos reales y describir afirmativamente mal el guard como que imponía precedencia — una garantía falsa alucinada que es activamente engañosa para una auditoría de seguridad."* Su tasa de captura medida en esa clase rondó **uno de cada cuatro**.

Detente en lo que eso significa. Si Sentinel hubiera corrido una auditoría de una sola familia sobre esa familia, esos hallazgos simplemente no existirían — ningún test los habría cazado, y el review que se suponía debía hacerlo habría devuelto un PASS confiado. El punto ciego era invisible desde adentro de la familia y obvio en el momento en que una familia *distinta* miró el mismo diff. Ese es el argumento entero a favor de la independencia entre familias, y aquí está como un hecho medido y no como un principio.

## Lo que deliberadamente no afirmamos

Esto no es una tabla de posiciones de modelos. La muestra es un proyecto, a lo largo de unos dos meses, sobre versiones específicas de modelos que estarán rancias en semanas; `gemini` tuvo un punto brillante genuino (una captura limpia de 10/10 en `CHARTER-31`), y un corpus distinto o una versión más nueva podría reordenar la tabla entera. La lectura honesta no es "la familia X es mala" — es la **metodológica**: *cualquier* mente única puede ser sistemáticamente ciega, la ceguera es incomprobable desde adentro, y la diversidad es lo que la vuelve visible. Esa afirmación sobrevive a cualquier modelo que esté arriba el mes que viene.

Tampoco es un golpe a Bun. Su diseño de una-sola-familia, contexto-partido y respaldado-por-oráculo es *correcto para un port*, y a una escala — un millón de aserciones, seis plataformas, cero tests saltados — que la mayoría de los equipos nunca alcanza. La distinción es solo esta: cuando el suite de tests puede ser el juez, aísla a los revisores por contexto; cuando el juez es un humano emitiendo un juicio, aíslalos por familia, y mantén al humano dirigiendo la decisión. Todo en la auditoría de StrayMark — incluido el [fix de identidad provista por el operador](who-the-audit-thought-it-was) que volvió imposible de fingir la convergencia entre familias — existe para proteger ese segundo caso.

## Si llegaste hasta aquí

La pregunta portátil es sobre tu propio proceso de revisión, humano o automatizado. Pregunta qué no cazarían tus revisores *de una forma en que nada aguas abajo lo cazaría tampoco* — la clase de defecto que no tiene test, ni linter, ni tipo que lo detenga. Para esa clase, la redundancia dentro de una sola perspectiva no te compra casi nada; dos del mismo revisor comparten el mismo punto ciego. Lo que sí te compra algo es un revisor de *otra clase*, y una forma de notar cuándo discrepan. Bun podía apoyarse en un millón de aserciones para que fueran el juez. La mayoría de lo que shippeas no puede. Para eso, el único oráculo que tienes es una segunda mente que ve distinto — así que asegúrate de que la segunda mente sea de verdad distinta, y de que una persona siga leyendo el desacuerdo.

---

*Datos del proyecto Sentinel, `.straymark/audits/` — 25 ciclos de auditoría, 21 Charters, 68 reportes de auditor, 7 familias de modelo, 2026-05-05 → 2026-07-10 (read-only). Cifras de Bun de [bun.com/blog/bun-in-rust](https://bun.com/blog/bun-in-rust). Relacionado: [Quién creyó la auditoría que era](who-the-audit-thought-it-was).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*