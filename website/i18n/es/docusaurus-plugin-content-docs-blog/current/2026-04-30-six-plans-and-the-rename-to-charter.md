---
slug: six-plans-and-the-rename-to-charter
title: Seis Planes para una tesis (y el rename a Charter)
authors:
  - jose
tags:
  - straymark
  - sentinel
  - charters
  - metodología
  - governance
date: 2026-04-30T00:00:00.000Z
description: Primer experimento sistemático y el día que Plan se llamó Charter
---
*Seis Planes sobre el papel, cinco en la práctica — el primer experimento sistemático de Sentinel entre el 25 y el 28 de abril. El ciclo que convirtió un patrón artesanal en Charters, la unidad acotada de trabajo del framework.*

<!-- truncate -->

## 1. Una nota al pie que dice mucho

En la propuesta `charter-telemetry.md`, escrita el 30 de abril de 2026, hay una nota terminológica al inicio:

> *"Lo que este documento llama 'Charter' se llamó 'Plan' en el experimento Sentinel (PLAN-01..06). Los datos empíricos citados abajo se refieren a esos Plans históricos por su nombre original; el shape del schema y los ejemplos prospectivos usan el nombre going-forward 'Charter'."*

Es una nota al pie, técnica, sin drama. Pero registra el momento exacto en que el artefacto cambió de nombre — y, más interesante, registra una decisión de archivo que el blog también respeta: lo que se llamó *Plan* en su momento se sigue llamando *Plan* en los registros originales. Reescribir el nombre hacia atrás sería falsificar el rastro.

Este post cubre dos cosas que pasaron en cinco días: el primer experimento sistemático del framework — los seis Planes de Sentinel, ejecutados entre el 25 y el 28 de abril — y el rename que vino después, el 30. No son dos eventos separados. Uno trajo al otro.

---

## 2. Seis Planes en el papel, cinco en la realidad

El experimento se diseñó con seis Planes (`PLAN-01` a `PLAN-06`) y se ejecutó con cinco. PLAN-04 nunca corrió: quedó como un catálogo de siete *features* diferidas, una lista de cosas que se quería hacer pero que el ciclo de pruebas no alcanzaba a cubrir. Es honesto decirlo así, porque el propio documento que valida la tesis (`thesis-validation.md`) lo registra abiertamente en su tabla resumen:

| Plan | Escala | Lo que hizo | Formato |
|---|---|---|---|
| PLAN-01 | XS | Deploy de docs de gobernanza | v1 |
| PLAN-02 | S | Endpoint admin (`gcp-resource`) | v1 |
| PLAN-03 | XS | Bumps de contrato | v1 |
| PLAN-05 | M | Per-service anomaly thresholds | **v2** |
| PLAN-06 | XS | Recompute manual de baseline | **v3** |
| *(PLAN-04 catalog)* | — | Roadmap de 7 features diferidas, no ejecutado | — |

Cinco Planes, cuatro escalas distintas: tres XS, una S, una M. Dominios deliberadamente heterogéneos — un deploy operativo, dos endpoints administrativos, un bump de contrato, una feature de backend con lógica de configuración. No era un benchmark sintético. Era el código real de Sentinel del 25 al 28 de abril, partido en unidades acotadas y auditado pieza por pieza con tres modelos (Copilot, Gemini, Claude) actuando como auditores externos.

El sexto Plan no haberse ejecutado importa por dos razones. La primera es la obvia: el experimento se quedó sin tiempo. La segunda es más interesante: PLAN-04 era el único Plan grande del lote — siete features acumuladas, sin acotamiento natural. Que justamente ese Plan fuera el que no entró ya estaba diciendo algo sobre el tamaño que podía tolerar el formato. Los Planes pequeños sí; los Planes-roadmap, no. Esa lección se canonizó después, sin que nadie tuviera que escribirla: los Charters de StrayMark hoy son *"unidades acotadas, de horas a días, una rama"*, no roadmaps trimestrales.

---

## 3. El experimento iteró sobre sí mismo

Cinco Planes ejecutados produjeron tres formatos distintos. No fue por capricho; fue porque cada ciclo dejó al descubierto algo que el formato anterior no capturaba.

- **v1** — los tres primeros Planes (`01`, `02`, `03`) corrieron con un formato originario. Se identificaron cinco patrones recurrentes en los AILOGs de auditoría, pero ninguno estaba formalizado en una plantilla. Existían como costumbre, no como contrato.

- **v2** — para PLAN-05, esos cinco patrones se materializaron en `TEMPLATE.md`, más una sección *"Format conventions"* en el README. Doc-only: nada ejecutable todavía. La hipótesis: si el formato es explícito, los auditores externos convergen mejor. El propio AILOG-020 lo dice sin adorno: *"Los patrones internos que llevan tiempo aplicándose pero no tienen nombre quedan invisibles a auditores externos. Nombrarlos formalmente convierte la práctica en señal pública."*

- **v3** — para PLAN-06, se sumó un patrón nuevo (auto-checklist drift) y, esta vez, *tooling ejecutable*: `scripts/check-plan-drift.sh`, ~145 líneas de bash que validan si la entrega real coincide con lo declarado en el Plan. Por primera vez el formato dejó de ser solo prosa y empezó a tener un verificador.

Lo que me interesa señalar aquí es la forma de la curva. v1 capturó práctica latente. v2 le puso nombre. v3 le puso un programa que la chequea. Es el arco completo de cualquier estandarización útil — y lo recorrimos en cinco Planes, no en cinco años.

Es también la primera evidencia concreta de algo que dos meses después se nombraría explícitamente como Pattern 1 (*pre-declare refresh*) y Pattern 2 (*amend-on-emergence*) en `CHARTER-CHAIN-EVOLUTION.md`. Lo que ahí se canonizó como meta-patrón empezó aquí, en abril, como costumbre sin nombre.

---

## 4. La evidencia más dura: auditores externos convergiendo

El supuesto más cargado de la tesis era: *"notas estructuradas reducen modos de falla, y se nota cuando auditores externos miran"*. La forma de probarlo: enviar el output de cada Plan — los archivos modificados, el AILOG, el Plan-doc bajo TEMPLATE — a tres auditores que no participaron en el desarrollo (Copilot, Gemini, y un análisis crítico de Claude) y comparar las calificaciones agregadas.

El número que `thesis-validation.md` reporta para PLAN-05 (el primer Plan bajo v2 y bajo escala M) es claro:

> *"Mejor calibración combinada de auditores en 5 ciclos (Copilot 9.25, Gemini 9.5). Hipótesis acumulada confirmada: TEMPLATE v2 + AILOGs ricos reducen el espacio de ambigüedad para auditoría externa."*

Dos modelos heterogéneos, instrucciones diferentes, convergiendo en torno a 9.3 sobre 10 — sobre el Plan más grande del lote, además, no el más chico. Eso es señal, no ruido. Y se replicó al ejercitar el `check-plan-drift.sh` de v3:

> *"PLAN-05 (con drifts conocidos): script reporta 3 omitted files: `evaluator_test.go` (F4), `repository.go` (F1/R6), `statuscenter/service.go` (F5) — exactamente los 3 drifts que Copilot+Gemini capturaron en sus auditorías. Cero falsos positivos."*

El script automatizado y los dos auditores externos coincidieron exactamente en qué archivos del Plan habían quedado a deber. No es un argumento sobre que el script sea inteligente — es bash y *grep*, no podría ser más tonto. Es un argumento sobre que el formato del Plan, ya formalizado, hace que *cualquier inspector* (humano, agente, script) vea las mismas cosas. La disciplina estructurada no requiere inteligencia para auditarse: requiere claridad.

---

## 5. Lo que el experimento *no* demostró

Esta es la sección que más me interesa que quede registrada, porque es donde se prueba si el blog está dispuesto a ser honesto.

`thesis-validation.md` se diseñó explícitamente para romper la tesis, no para defenderla. La primera línea del documento dice:

> *"Este documento confronta esa tesis con datos. No la defiende; intenta romperla. El objetivo es que un lector que no compró la tesis pueda, leyendo solo este texto, decidir si la evidencia disponible la sostiene, la matiza o la refuta."*

El veredicto final, con los seis supuestos de la tesis enfrentados a la evidencia, es éste:

| # | Supuesto | Veredicto |
|---|---|---|
| 1 | Vibe coding no escala | Validado parcialmente *(sin brazo de control)* |
| 2 | Notas estructuradas reducen modos de falla | Validado |
| 3 | Subproducto regulatorio sin trabajo extra | Validado *(pendiente prueba con auditor humano real)* |
| 4 | Aprobaciones rara vez binarias | Sin evidencia — pendiente proyecto multi-actor |
| 5 | Stage > commit como unidad de trazabilidad | Validado |
| 6 | Firma in-situ > reconstrucción posterior | Validado parcialmente *(sin firma criptográfica probada)* |

Tres supuestos validados completos, dos parciales, uno *sin evidencia*, cero refutados. El supuesto sobre aprobaciones no binarias se quedó sin probar porque Sentinel es un proyecto de un solo responsable; haría falta un equipo real con varios firmantes para ejercitarlo. Y la firma criptográfica — Sigstore, hash-chaining, las garantías técnicas de un append-only log — no se ejercitó porque ningún Plan tocó esa parte del flujo. Eso queda en el documento como *gap* explícito, no como afirmación al aire.

Lo que el experimento no demostró importa más que lo que sí demostró. Si en lugar de cinco veredictos honestos hubiéramos publicado seis verdes, el blog sería marketing. La forma de evitarlo es decir, sin atenuar: aquí hay supuestos sobre los que no tenemos evidencia, y un proyecto de un solo operador no puede generarla. Otros la generarán; el framework irá madurando con esos datos.

---

## 6. Por qué Plan se volvió Charter

El rename del 30 de abril no fue de marketing. La razón está escrita literal en `WHAT-IS-A-CHARTER.md`:

> *"GitHub SpecKit already uses the word 'plan' (`/speckit.plan`, `plan.md`) for a different artifact, and the nominal collision generated friction in adopter docs that combine both flows."*

SpecKit ya tenía un `plan.md`. StrayMark también. En la documentación de un adopter que usara ambos, *plan* significaba cosas distintas en párrafos contiguos. El rename existió para arreglar esa colisión.

Pero detrás del pragmatismo hay un matiz que sí vale la pena nombrar. Los dos artefactos *son* distintos, y la distinción es estructural:

| | **`plan.md` de SpecKit** | **Charter de StrayMark** |
|---|---|---|
| Granularidad | Feature completa (semanas, varias stories) | Unidad acotada (horas a días, una rama) |
| Contenido dominante | Stack, dependencias, estructura del proyecto, constitution gates | Archivos concretos a tocar, comandos de verificación, riesgos `R<N>` |
| Validación | Constitution Check (gate *ex-ante*) | Drift-check + auditoría externa (gates *ex-post*) |
| Optimización | *Claridad anticipada* | *Trazabilidad ex-post* |

Lo que SpecKit llama *plan* se parece más a un ADR con esqueleto arquitectónico. Lo que StrayMark llama *Charter* se parece más a una task-card con verificación contractual y anclaje de auditoría. Son artefactos complementarios, no competidores; eso lo hace explícito `SPECKIT-CHARTER-BRIDGE.md` después, en mayo. Pero la nomenclatura tenía que dejar de chocar antes de que se pudiera hablar de complementariedad.

La parte que me parece importante destacar es la decisión adyacente: los registros históricos de Sentinel — los AILOGs, las telemetrías, el `check-plan-drift.sh`, las carpetas `sentinel/docs/plans/` — conservan el nombre *Plan*. El propio `WHAT-IS-A-CHARTER.md` lo argumenta así:

> *"Sentinel's historical records (Plans 01–06, `sentinel/docs/plans/`, `sentinel/scripts/check-plan-drift.sh`, AILOGs and YAML telemetries `PLAN-NN.telemetry.yaml`) preserve the original name — they are empirical evidence of a specific moment in time, and rewriting history would falsify the record."*

*Rewriting history would falsify the record.* Esa frase es, para mí, el verdadero corazón del rename. El framework existe para preservar rastro, no para reescribirlo. Cuando el artefacto cambió de nombre, el cambio fue prospectivo: a partir del 30 de abril, los nuevos se llaman Charters. Los viejos, los del experimento, siguen siendo Planes. La inconsistencia es deliberada, y la inconsistencia es la evidencia.

El otro acoplamiento del 30 de abril — la propuesta de telemetría — vino el mismo día por razones que ahora resultan obvias: si el patrón emergente merecía un nombre nuevo, también merecía medición estructurada. El schema `charter-telemetry.schema.v0.json`, con sus campos sobre tiempo invertido, drifts detectados, escala, dominio, es la maquinaria que ejecuta hoy lo que en Sentinel todavía se medía a mano en YAML. Plan se renombró a Charter el mismo día en que se decidió empezar a contar Charters.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **El experimento se diseñó para romperse, no para confirmarse.** *"No la defiende; intenta romperla"* — eso fue, literal, la consigna del documento que validó la tesis. Cualquier blog técnico que se respete tiene que tolerar esa formulación.

2. **Cinco Planes ejecutados, de seis diseñados, en cuatro días.** El que no se ejecutó (el más grande, un roadmap de siete features) fue, sin querer, la primera evidencia de qué tamaño tolera la unidad. Hoy los Charters son acotados por contrato. En abril, lo eran por accidente.

3. **El formato evolucionó dentro del experimento mismo: de costumbre, a plantilla, a script.** Los cinco Planes produjeron tres versiones del formato. La curva de v1 → v2 → v3 es la curva de cualquier estandarización útil — solo que comprimida.

4. **El rename fue de pragmatismo, no de marketing. Y la preservación histórica fue de principio.** *Plan* siguió llamándose *Plan* en Sentinel, donde así se llamó. *Charter* empezó a usarse de abril para adelante. El framework no reescribe su propio archivo: por eso vale la pena rastrearlo.

Sigue, en el siguiente post, dos hitos cercanos que codifican capacidades cualitativamente nuevas: *Charters como entidad de primera clase* (PR #65, fw-4.4.0) y *el audit cycle externo multi-modelo* (`audit-skills-design`, `audit-cli-flow`, releases fw-4.7 → fw-4.9). Es donde el experimento de Sentinel se convirtió en funcionalidad de CLI.

---

*Anclas: propuestas [`thesis-validation.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-04-30-thesis-validation.md) y [`charter-telemetry.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-04-30-charter-telemetry.md) (ambas 2026-04-30). Definición canónica del Charter: [`WHAT-IS-A-CHARTER.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/contributors/WHAT-IS-A-CHARTER.md). Schema operacional: `dist/.straymark/schemas/charter-telemetry.schema.v0.json`.*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
