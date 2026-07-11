---
slug: what-the-author-already-knew
title: Lo que el autor ya sabía
authors:
  - jose
tags:
  - straymark
  - baton
  - cost-routing
  - framework
  - schema
  - sentinel
date: 2026-06-28T00:00:00.000Z
description: El dry run probó que la cobertura de señal — no la granularidad — era lo que mantenía el 57% del routing sobre conjeturas de baja confianza. El fix fue casi gratis. La persona que crea una unidad de trabajo ya sabe si es diseño, implementación, una auditoría o labor mecánica; lo sabe por el costo de teclear una palabra. Así que Baton descontinuó el escaneo de títulos por completo y volvió a un verbo declarado la única señal autoritativa — y luego lo graduó al framework.
---
*La Fase 2 terminó en un número incómodo: el 57% del routing descansaba en conjeturas, porque el 45% de las unidades de trabajo no surfaceaba ningún cue que un clasificador barato pudiera leer. El instinto era echar mano de un clasificador más potente. La respuesta correcta era lo opuesto — dejar de inferir la cosa del todo. La persona que crea una unidad de trabajo ya sabe si es diseño, implementación, una auditoría o labor mecánica. Lo sabe gratis, en tiempo de autoría, por el costo de una palabra declarada. Así que Baton tiró el escaneo de títulos como input de clasificación, volvió a un `work_verb` declarado la única señal autoritativa, ratificó el vocabulario contra un corpus real de 762 unidades, y lo graduó al framework como campo de primera clase.*

<!-- truncate -->

> *Undeclared es un estado honesto, no un error. Una unidad sin verbo enruta conservadoramente hacia arriba y recibe un nudge — nunca una conjetura de baja confianza desde su título.*

Este es el tercer post sobre **Baton**, y aquel en el que un experimento hermano hace lo que Loom hizo antes: entrega una primitiva limpia al core. Después de [el coherence bridge](what-the-spec-path-only-proved-existed) y [el dry-run router](what-the-dry-run-would-have-spent), el arco cierra no con un motor de routing sino con un **campo** — dos, en realidad — agregado al framework que todos los demás usan.

## 1. El pivote: dejar de conjeturar lo que el autor puede simplemente decir

El hallazgo del dry run fue específico ([#328](https://github.com/StrangeDaysTech/straymark/issues/328)): la palanca del routing confiable es *señal estructurada*, no unidades más finas. Y el input más débil del clasificador era aquel del que más dependía — el **título**. Los títulos se escaneaban en busca de cues (`scan_cues`, una `CUE_TABLE`), y los títulos mienten en ambas direcciones: un título de task verboso fabricaba "conflicto" fantasma, un título de charter escueto no surfaceaba nada. Peor, el escaneo de títulos es trivialmente **gameable** e inherentemente **calibrado a un stack** ([#321](https://github.com/StrangeDaysTech/straymark/issues/321)) — las palabras-cue que funcionan en un repo Go/TypeScript no transfieren.

Así que el giro arquitectónico ([#332](https://github.com/StrangeDaysTech/straymark/issues/332), prototipado en #333) descontinuó el escaneo de títulos como input de routing por completo y dejó una única señal autoritativa en su lugar: un **verbo declarado**, dicho por el autor cuando la unidad se crea. La economía de eso es imbatible — la clasificación cuesta **≈ 0 tokens**, porque el autor ya sabe la respuesta con certeza y la teclea una vez. El título no desapareció; fue **degradado** de oráculo a cross-check advisory, usado solo en la herramienta de calibración periódica, **nunca** en la ruta de routing.

## 2. Los dos campos

**`work_verb`** — la señal autoritativa, un vocabulario controlado y cerrado:

| valor | significado |
|---|---|
| `design` | Decisión de arquitectura / patrón / escalabilidad abierta; autoría de spec. |
| `implement` | Lógica de servicio; un fix con diagnóstico real; una query compleja; tooling nuevo; **definir un contrato fundacional acotado**. |
| `audit` | Revisión / verificación / contraste independiente contra la realidad. |
| `operate` | Trabajo mecánico: tests, migraciones, scaffolding, docs, ceremonia de cierre, ediciones en bulk. |

**`design_provenance`** — `{new, upstream}`, opcional, default `new`. Captura la carga cognitiva residual: una unidad cuyo trabajo solo *instrumenta un diseño ya hecho aguas arriba* es mecánica, aunque su verbo de superficie parezca `implement`. `new` significa que la decisión difícil ocurre *en esta unidad*; `upstream` significa que el diseño ya existe y esta unidad solo lo transcribe o cablea.

## 3. Las reglas de determinación — donde un enum no basta

Un enum por sí solo no resuelve la ambigüedad que de verdad muerde. La ratificación ([`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md)) fija cuatro reglas, cada una surfaceada por datos reales:

1. **La regla del contrato fundacional.** "Definir un contrato/interface fundacional acotado" es `implement`, **no** `design`. `design` se reserva a arquitectura *abierta* — un patrón de partición, una decisión de escalabilidad — y a la autoría de spec. En la muestra calibrada de Sentinel hubo **cero** unidades `design` reales, consistente con el hallazgo del dry run de que la clase `planner` estaba vacía. La mayoría del trabajo que *se siente* como diseño es en realidad implementación acotada.
2. **La carga cognitiva residual degrada el tier.** `implement` + `design_provenance: upstream` **degrada a operator**. Esto es lo que separó un charter que cableó métricas *declaradas en un charter anterior* (mecánico → operator) de uno que implementó lógica nueva (implementer). El pensamiento difícil ya se gastó aguas arriba; instrumentarlo es trabajo barato.
3. **`design_provenance` solo es significativo para `implement`.** Un `design` cuyo diseño "ya existe aguas arriba" es una contradicción — si el diseño está hecho, el trabajo no es diseñar, es implementar. Así que la degradación se ancla en `implement`; `audit` y `operate` ignoran la procedencia.
4. **No-trabajo mapea a `operate`.** Higiene, ceremonia de cierre, meta-notas — no se introduce un quinto valor. Rutean barato como `operate`; "en rigor no-trabajo" es una observación de telemetría, no un tier.

El mapa verbo → tier resultante es pequeño y legible: `design` → planner, `implement` → implementer (degradando a operator cuando es upstream), `audit` → auditor, `operate` → operator.

## 4. Grano homogéneo, y undeclared honesto

El verbo se declara **una vez, al grano homogéneo más amplio**, en la unidad más fina que ya tenga un slot de declaración — no se inventa frontmatter nuevo para unidades que no lo tienen:

| Unidad | Slot de declaración |
|---|---|
| Charter | Frontmatter: `work_verb:` / `design_provenance:` |
| Follow-up | Líneas de la entrada: `- **Work verb**:` / `- **Design provenance**:` |
| Batch | Una línea `- **Work verb**:` en el ledger del AILOG |
| Task | **Sin slot.** Hereda del charter/spec padre. Cero frontmatter nuevo en `tasks.md`. |

Batch y Task heredan del padre; haces override a un grano más fino **solo** cuando una unidad genuinamente mezcla trabajo (diseño + una parte mecánica). No fragmentas una unidad homogénea para satisfacer el schema. (Estado honesto: el prototipo todavía no implementa la herencia — cosecha el verbo del frontmatter del charter y de las líneas de follow-up y deja batch/task `undeclared`. La herencia es una *regla ratificada*, pendiente de la implementación de la graduación. Documentarla ahora impide que la graduación invente un mecanismo por-task que esta regla explícitamente rechaza.)

Y la decisión de diseño load-bearing: **undeclared es un estado honesto.** Una unidad sin verbo declarado es inclasificable — enruta conservadoramente *hacia arriba*, a frontier, y emite un nudge ("declara el verbo"), y **nunca** fabrica una conjetura de baja confianza desde el título. El hueco se vuelve una *acción* — una tarea de higiene — en vez de un número fingido. La telemetría reporta `undeclared_fraction`, la métrica accionable, no el artefacto `conflict_fraction` que el escaneo de títulos producía. El incómodo 57% del dry run no se escondió detrás de un conjeturador más listo; se convirtió en una lista de pendientes.

## 5. La graduación — fw-4.31.0 / cli-3.30.0

Aquí está la disciplina que hizo que esto tomara un Charter entero adicional en vez de un commit. El adopter Sentinel puso una restricción explícita: *no instrumentar contra un schema sin ratificar.* Así que el vocabulario, las reglas, el placement y la postura de enforcement se fijaron primero en un documento de ratificación — calibrado contra el corpus de gobernanza real de 762 unidades de Sentinel — y solo *entonces* los campos graduaron al framework en [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0) ([#332](https://github.com/StrangeDaysTech/straymark/issues/332) paso 2).

La graduación es deliberadamente callada:

- `work_verb` y `design_provenance` se agregan a `charter.schema.v0.json` como propiedades **string opcionales** — intencionalmente **no** un `enum`, para que un valor fuera de vocabulario sea una advertencia advisory, nunca un error de schema bloqueante. La plantilla de Charter (EN/es/zh-CN) los carga como guía comentada; el schema de entrada de follow-up gana líneas `Work verb` / `Design provenance` correspondientes.
- `straymark validate` obtiene dos reglas **warning-only**, `CHARTER-WORK-VERB` y `CHARTER-DESIGN-PROVENANCE`, que disparan *solo* cuando un Charter declara el campo con un valor fuera del vocabulario. Un campo ausente no emite **nada**. El check nunca toca el exit code.

Esa última propiedad es todo el punto. El corpus legacy de Sentinel es 100% undeclared, y se queda completamente callado — sin romper CI, sin ruido, sin migración. Declarar el schema no castiga a nadie que no lo haya adoptado. `undeclared` es un estado honesto también a nivel del framework.

## 6. Lo que deliberadamente no hicimos

No shippeamos **ejecución.** Dispatch real de modelos, pricing real, archivos de config, un dashboard de vigilancia — todo Fase 3, todo diferido. Este arco graduó un *vocabulario de clasificación*, no un router.

No hicimos el campo **requerido**, ni un gate de CI. La postura de enforcement es un nudge, acorde con la cautela v0-experimental y la línea explícita del adopter de "no rompas el corpus legacy".

No declaramos hecha la **forward-validation.** Si los autores realmente declaran *bien* a través de un corpus variado tras la adopción es el paso 3 de #332 — responsabilidad de StrayMark, fuera del alcance de la ratificación. Un vocabulario fácil de declarar también es fácil de declarar mal, y solo la adopción real mostrará si las cuatro reglas se sostienen fuera de Sentinel.

Y no reintrodujimos un quinto verbo para "no-trabajo", aunque era tentador. Cuatro valores, una superficie anti-gaming pequeña, calibración periódica. Reconsiderado solo si la calibración futura muestra que `operate` está fundiendo señal que importa.

## 7. Si llegaste hasta aquí

El movimiento portátil es el que cerró el hueco por casi nada. Baton pasó una fase entera construyendo maquinaria para *inferir* algo — la clase de trabajo de cada unidad — y la maquinaria topó en 43% de confianza porque estaba reconstruyendo, desde un título, un hecho que el autor había tenido en la cabeza todo el tiempo y nunca se le pidió escribir. La señal más barata y de mayor calidad en un sistema muy a menudo es la que un humano ya sabe en el momento de la creación y simplemente no se le pide registrar. Antes de construir un clasificador, una heurística o un modelo de ML para recuperar alguna propiedad de tus datos, revisa si alguien aguas arriba la sabía con certeza y gratis — y si lo único que faltaba era un campo donde ponerla. La forma más cara de saber algo es inferirlo después del hecho. La más barata es preguntar, una vez, cuando la respuesta todavía es obvia.

El próximo post es una coda desde otro rincón del framework — el audit cycle — donde la pregunta no es *qué* se verificó, sino *quién* creyó el framework que hizo la verificación.

---

*Baton Fase 3 (graduación de schema) — [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0) · ratificación [`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md). Issues [#321](https://github.com/StrangeDaysTech/straymark/issues/321) · [#328](https://github.com/StrangeDaysTech/straymark/issues/328) · [#331](https://github.com/StrangeDaysTech/straymark/issues/331) · [#332](https://github.com/StrangeDaysTech/straymark/issues/332). Predecesores: [Lo que el spec path solo probó que existía](what-the-spec-path-only-proved-existed) · [Lo que el dry run habría gastado](what-the-dry-run-would-have-spent).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*