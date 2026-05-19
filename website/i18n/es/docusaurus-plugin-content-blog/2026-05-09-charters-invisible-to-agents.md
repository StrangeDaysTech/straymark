---
slug: charters-invisibles-para-los-agentes
title: Charters invisibles para los agentes
authors:
  - jose
tags:
  - straymark
  - charters
  - agentes
  - observabilidad
  - governance
date: 2026-05-09T00:00:00.000Z
description: 'Issue #113 — la brecha entre tener un artefacto y que el agente lo vea'
---
*Seis horas de sesión con un agente capaz y todo el onboarding del framework cargado — y ni una vez se sugirieron los Charters. Cinco minutos para aplicarlos al pedirlos directamente. La asimetría no es de capacidad; es de visibilidad.*

<!-- truncate -->

## 1. Seis horas para no verlo, cinco minutos para verlo

El Issue [#113](https://github.com/StrangeDaysTech/straymark/issues/113) abre con una de esas frases que, cuando uno la lee de regreso meses después, suena obvia y al mismo tiempo costó mucho llegar:

> *"Time to detect: ~6 hours of session work; never autonomously surfaced. Time to resolve once user prompted: ~5 minutes."*

Seis horas trabajando con un agente capaz, atento, con todo el aparato canónico de onboarding del framework cargado — `STRAYMARK.md`, la constitución del proyecto, el checklist de `CLAUDE.md`, las skills `/straymark-*` disponibles, `/straymark-status` ejecutado al inicio — y nunca, en seis horas, el agente sugirió usar un Charter. Cuando se lo pedí directamente, el flujo se incorporó en cinco minutos. La asimetría no es de capacidad. Es de visibilidad.

Este post cubre un episodio corto y específico: el Issue #113, abierto el 7 de mayo de 2026, tres días después de que Charters se cristalizaran como entidad de primera clase del framework (fw-4.4.0, 2 de mayo). Es el contrapeso natural del primer post del blog. Aquel post nombró una propiedad que apareció cuando todo estaba en su sitio. Este post documenta lo que pasa cuando una pieza estructural existe pero *no está en la superficie que el agente lee primero*.

---

## 2. El experimento accidental

El setup no estaba pensado como experimento. Era un proyecto real: un *greenfield* en Rust, suite CLI+TUI, en su primera versión `v0.1`. El agente era Claude Opus 4.7 con ventana de 1M tokens, suficiente para tener todo el repo en contexto sin tener que paginar. El flujo era el canónico de SpecKit: `/speckit-specify`, `/speckit-plan`, `/speckit-tasks`, y entre tasks e implement debía aparecer — *en teoría* — la idea de cortar en Charters.

El Issue lo formula con precisión casi fría:

> *"A capable, attentive agent following the canonical project onboarding (`DEVTRAIL.md`, the project constitution, `CLAUDE.md` checklist, available `/devtrail-*` skills, `/devtrail-status`) did not autonomously identify Charters as a workflow concept during plan/tasks generation, even though the project clearly fit their use case (multi-session implementation, multi-phase tasks, audit value)."*

*(El Issue todavía usa `DEVTRAIL.md` porque, recordando la cronología, el rebrand a StrayMark estaba siendo merged casi al mismo tiempo. La nomenclatura mixta es residuo del calendario, no descuido.)*

El proyecto cumplía cada una de las condiciones donde un Charter es la unidad correcta: implementación multi-sesión, varias fases, valor de auditoría. El agente hizo todo bien — leyó la constitución, ejecutó status, identificó skills disponibles, generó un plan de SpecKit razonable — y nunca conectó con Charters. Como si el concepto no existiera. Para él, en su modelo del repo, no existía.

Lo más incómodo del Issue, mirado desde mayo, es que el agente no falló por incapacidad. Falló porque el framework, sin proponérselo, le había construido un mapa en el que los Charters no aparecían marcados.

---

## 3. Nueve gaps que convergen en uno

Cuando el operador (yo) abrió el Issue y se sentó a auditar por qué había pasado, el problema no era una cosa. Eran nueve. Y todas eran versiones del mismo defecto:

| # | Gap | Dónde |
|---|---|---|
| 1 | El documento canónico del framework no listaba Charter como concepto | `STRAYMARK.md` (entonces `DEVTRAIL.md`) §6/9/10/11/13/15 |
| 2 | El checklist de `CLAUDE.md` no tenía trigger para sugerir Charter | `dist/dist-templates/directives/CLAUDE.md` (y `GEMINI.md`, `copilot-instructions.md`) |
| 3 | Las constituciones de proyecto heredan la brecha del framework | Cualquier `*-constitution.md` instalado vía `straymark init` |
| 4 | No existía skill `/straymark-charter-new` | Skills directory |
| 5 | `/straymark-status` no listaba Charters en su output | Skill + CLI subcommand `straymark status` |
| 6 | Las skills de auditoría hablaban del Charter como artefacto *externo*, no como superficie a generar | `/straymark-audit-*` skills |
| 7 | Los templates de Charter convivían indistinguibles con otros templates | `dist/.straymark/templates/` (raíz) |
| 8 | No había puente conceptual entre SpecKit (`plan.md`) y Charter | Ningún documento explícito |
| 9 | El modelo mental que el agente terminaba construyendo era binario: o SpecKit, o nada | Efecto compuesto |

Nueve fallas y un solo defecto: el Charter existía como artefacto técnico (schema válido, comando funcional, template portado de Sentinel), pero no estaba nombrado en ninguno de los lugares donde el agente construye su modelo mental del proyecto al entrar.

El agente lee el `STRAYMARK.md` y la constitución para entender de qué trata el proyecto. Si Charter no aparece, no está. El agente revisa qué skills tiene disponibles para usar. Si no hay `/straymark-charter-new`, no está. El agente corre `/straymark-status` para entender qué tipos de archivo viven en el repo. Si la salida no menciona Charters, no están. Cada superficie por la que el agente pasa al onboarding repite el mismo silencio. Y la suma de silencios convence al agente de que el concepto no es parte de este proyecto.

---

## 4. Por qué este episodio importa más allá del Issue

Vale la pena hacer una pausa aquí, porque este es exactamente el episodio que el meta-patrón de fw-4.17.0 — el patrón de *observación emergente* — necesitaba haber existido *antes* para poder formularse.

Una semana después de #113, en `EMERGENT-OBSERVATION-DESIGN.md`, el framework nombró dos propiedades cuya composición produce que el agente note cosas que nadie le pidió notar: vínculos formales entre artefactos (`originating_charter`, `originating_ailog`, `originating_spec` en frontmatter), y permiso cultural para señalar disonancias entre fuentes. Esas dos propiedades, *compuestas*, producen que el agente lea el AILOG, cuente las entradas `R<N>(new, not in Charter)`, y señale el delta sin que nadie se lo pida.

Pero hay un prerrequisito que esa formulación da por sentado: que el agente *vea* los Charters al entrar al repo. Si los Charters no están en la superficie visible, ningún vínculo formal entre artefactos puede componerse en observación emergente — porque uno de los términos de la composición no existe para el agente.

El Issue #113 es el episodio que documentó esa precondición desde la inversa. Fw-4.17.0 nombró el patrón cuando estaba presente; fw-4.12.0 (cinco días antes) cerró la brecha que impedía que estuviera presente en absoluto. Sin el Issue #113 y su corrección, el meta-patrón no tendría dónde apoyarse. El framework no señala lo que no ve, y no ve lo que la superficie no nombra.

---

## 5. La respuesta: multiplicar superficies

El PR [#122](https://github.com/StrangeDaysTech/straymark/pull/122) (`fw-4.12.0`, 9 de mayo) cubrió los nueve gaps con una sola estrategia operativa: *nombrar Charter en cada lugar donde el agente construye su modelo del proyecto*. Las cifras del PR son honestas: +1211 líneas, −92 líneas, 36 archivos modificados, casi todo documentación y plantillas. No hubo refactor de código. Hubo redistribución de visibilidad.

Cambios principales, agrupados:

- **En el documento canónico** (`STRAYMARK.md`): una sección nueva dedicada al Charter como concepto (§15), más filas en las tablas de §6/9/10/11/13 que listaban tipos de documento sin Charter.
- **En las directivas de agentes** (`CLAUDE.md`, `GEMINI.md`, `copilot-instructions.md`): un trigger explícito que enseña al agente cuándo *sugerir* un Charter, no solo cuándo crearlo si se le pide.
- **En las skills**: nueva `/straymark-charter-new` (versiones Claude, Gemini, agnóstica). `/straymark-status` actualizada para listar Charters activos.
- **En el CLI**: bloque `Charters` añadido a la salida de `straymark status`, para que el agente que ejecuta el comando lo vea entre los signals iniciales.
- **En los templates**: movidos a un subdirectorio dedicado `dist/.straymark/templates/charter/`, ya no mezclados con templates de otros tipos.
- **Documento nuevo**: `SPECKIT-CHARTER-BRIDGE.md` (171 líneas, EN/ES/zh-CN). Hace explícito el puente entre el `plan.md` de SpecKit y el Charter de StrayMark — los cuatro criterios para que una feature de SpecKit produzca uno o varios Charters, los tres casos donde no aplica, las cuatro heurísticas de granularidad, el enlace de frontmatter `originating_spec ↔ originating_charter ↔ originating_ailog`.

El epílogo de `SPECKIT-CHARTER-BRIDGE.md` registra el caso que originó todo el cambio sin metáfora:

> *"Cited the empirical context (issue #113): Greenfield Rust CLI/TUI suite, Claude Opus 4.7 onboarding via canonical entry points. Charters were eventually adopted (2 Charters: foundation + MVP) only after explicit user prompt — confirming the gap was systemic, not session-specific. This document removes the gap."*

*The gap was systemic, not session-specific.* Esa frase es la lección operativa del Issue. No fue un agente distraído un día; era el framework el que no le hablaba al agente sobre Charters en ningún lugar donde el agente miraba.

---

## 6. Lo que aprendí sobre visibilidad estructural

La lección, escrita en prosa y no en tabla:

Crear un artefacto en un framework no es lo mismo que hacerlo visible para los agentes que trabajan en repos del framework. Son dos pasos. El primero — esquema, comando, template, ejemplos — es el paso visible: queda en el CHANGELOG, sale en el release, se ve en el `git log`. El segundo — anclar el artefacto en cada superficie por donde el agente entra al repo — es invisible: nada del segundo paso aparece como *feature* en sí; son referencias cruzadas, líneas en checklists, items en outputs, secciones en docs. Pero el segundo paso es el que decide si el primero existe para el agente.

El framework gobierna agentes que leen el repo desde fuera de la sesión donde se creó el artefacto. Cada agente que entra al repo construye su modelo mental desde cero, leyendo los documentos canónicos, las skills disponibles, los outputs de status. Si el artefacto no está nombrado ahí, no existe en su modelo. Y un modelo sin el artefacto no propone usarlo, no audita su ausencia, no señala disonancias relacionadas. La capacidad técnica del agente es irrelevante si la superficie del repo no le da el sustantivo correcto al entrar.

Eso es lo que ahora llamo, internamente, *visibilidad estructural*. No es una propiedad del agente. Es una propiedad del repo, y por extensión, una responsabilidad del framework: cada vez que cristaliza un concepto nuevo, tiene que multiplicar las superficies donde ese concepto se nombra. Una sola superficie — el schema, el comando, el template — no alcanza. Son nueve, mínimo, y son las nueve del Issue #113.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Existir como artefacto no es lo mismo que ser visible para el agente.** Charters tenían schema, comando, template, ejemplos — y aun así eran invisibles. La diferencia entre los dos estados son las nueve superficies del Issue #113.

2. **Seis horas vs. cinco minutos.** Un agente capaz puede pasar horas sin detectar lo que, una vez nombrado, le toma minutos integrar. La asimetría no diagnostica al agente; diagnostica al repo.

3. **La visibilidad estructural es responsabilidad del framework.** Cuando cristaliza un concepto, no termina en el comando que lo crea. Termina cuando el concepto está nombrado en cada superficie donde el agente construye su modelo del repo: documento canónico, directiva, skill, status, template, puente con otros frameworks.

4. **Sin visibilidad estructural no hay observación emergente.** Las propiedades que después hicieron posible que un agente señalara disonancias entre fuentes — vínculos formales + permiso cultural — necesitan, como precondición, que los artefactos sean visibles desde el primer momento del onboarding. El Issue #113 documenta qué pasa cuando esa precondición falta.

Sigue, en el siguiente post, un episodio adyacente al de este: *el rebranding a StrayMark* (`H-08`). El Issue #113 se resolvió en el mismo arco temporal que el rename de DevTrail a StrayMark, y los dos comparten algo que vale la pena nombrar — la responsabilidad de que un framework declare con claridad su identidad, hacia sus agentes y hacia sus adopters.

---

*Anclas: [Issue #113](https://github.com/StrangeDaysTech/straymark/issues/113) (abierto 2026-05-07). [PR #122](https://github.com/StrangeDaysTech/straymark/pull/122) — `fw-4.12.0` (fusionado 2026-05-09). Documento generado por el PR: [`SPECKIT-CHARTER-BRIDGE.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md).*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
