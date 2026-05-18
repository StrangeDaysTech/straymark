---
slug: charters-y-el-audit-cycle-externo
title: 'Charters como entidad de primera clase, y el audit cycle externo'
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - charters
  - audit
  - cli
  - governance
date: 2026-05-06T00:00:00.000Z
description: De ritual manual en Sentinel a comando CLI canonizado
---
*Cuarenta minutos de copy-paste por Charter encima de dos horas de trabajo. La disciplina funcionaba; el ritual se estaba volviendo el problema. Cómo el Charter pasó a ser entidad de primera clase del CLI, y la decisión arquitectónica que dejó la orquestación de auditoría al framework y la evaluación del prompt a cualquier otro.*

<!-- truncate -->

## 1. Cuando la disciplina amenaza con volverse ceremonia

A finales de abril, Sentinel cerró su cuarto Charter del mes con una sensación que, escrita, suena un poco patética: la auditoría externa funcionaba — Copilot 9.25, Gemini 9.5, el drift script con cero falsos positivos — pero cada cierre me hacía abrir tres archivos a mano, copiar tres prompts a tres ventanas distintas, esperar respuestas, copiar tres YAMLs de calibración de vuelta al archivo de telemetría, y verificar a ojo que los hashes coincidían. Cuarenta minutos de copy-paste por Charter, sobre un trabajo que tomaba dos horas. La disciplina estaba funcionando; el ritual estaba volviéndose un problema.

La propuesta `audit-skills-design.md`, escrita el 3 de mayo, lo nombró sin atenuar:

> *"Si en cada cierre el operador tuviera que mover datos manualmente entre archivos, el throughput colapsaría y la disciplina dejaría de ser fricción virtuosa para volverse ceremonia. Lo que se pueda automatizar de forma reversible debe automatizarse; los documentos quedan para auditoría humana ex-post."*

Este post cubre tres días — del 2 al 6 de mayo de 2026 — en los que dos arcos paralelos terminaron a la vez. Uno: el Charter dejó de ser práctica artesanal y se volvió entidad de primera clase del CLI. Dos: el audit cycle externo dejó de ser ritual manual con prompts ad-hoc y se volvió comando canónico (con una decisión arquitectónica contraintuitiva detrás que merece su propio párrafo).

---

## 2. Lo que Sentinel ya hacía, y lo que StrayMark no tenía

La propuesta del 3 de mayo lo dice con una frase que se queda corta de elegante pero precisa exactamente lo que estaba pasando:

> *"Sentinel tiene los skills, StrayMark no."*

Lo que Sentinel tenía: `sentinel/.claude/skills/plan-audit/` y `plan-audit-review/` — skills locales que generaban un prompt, calibraban la respuesta al regreso, y la fusionaban en la telemetría. Funcionaban. Llevaban seis ciclos validándose. Pero estaban acoplados a paths Sentinel-específicos (`docs/plans/`, `internal/modules/`, `go vet`) y al vocabulario *"Plan"*, que ya había sido renombrado a Charter una semana antes.

Lo que StrayMark no tenía: nada equivalente. El framework hasta abril era documentación + skills + un CLI con `init`, `update`, `remove`. La unidad que en Sentinel se llamaba *Plan* — la unidad acotada, declarada *ex-ante*, auditada *ex-post* — no existía como artefacto del CLI. Existía como práctica.

El arco de fw-4.4.0 (2 de mayo) y de fw-4.7 → 4.9 (3-5 de mayo) consiste, casi al pie de la letra, en portar lo que Sentinel hacía a mano hacia comandos genéricos que cualquier adoptante del framework pueda invocar. La tabla de migración, copiada del `cli-roadmap.md`, es honesta sobre el origen:

| Artefacto Sentinel *(abril 2026)* | Equivalente StrayMark *(mayo 2026)* |
|---|---|
| `TEMPLATE.md` (v3) en `docs/plans/` | `dist/.straymark/templates/charter-template.md` |
| `scripts/check-plan-drift.sh` (145 líneas bash) | `straymark charter drift` (subcomando Rust) |
| Skill local `plan-audit` | Skill genérica `straymark-audit-prepare` |
| Reportes duales en `audit/plans/05,06/{copilot,gemini,claude}.md` | Output canónico de `straymark charter audit` |
| YAML de telemetría a mano | `dist/.straymark/schemas/charter-telemetry.schema.v0.json` |

El CHANGELOG de fw-4.4.0 lo enmarca sin metáfora: *"Crystallizes the Charter pattern — bounded, auditable units of work declared ex-ante and validated ex-post — that emerged from the 6-cycle Sentinel /plan-audit experiment."* La cristalización es el punto. Lo que en abril era costumbre con script bash, en mayo es comando con schema validable.

---

## 3. Qué pasa cuando algo se vuelve "entidad de primera clase"

PR #65 (fw-4.4.0 / cli-3.6.0, 2 de mayo) hace tres cosas concretas:

- Crea el comando `straymark charter new`. Auto-incrementa el número (`NN-slug.md`), pre-popula el origen (`originating_ailogs` si nace de un AILOG previo; `originating_spec` si nace de un SpecKit `plan.md`), y soporta el flag `--type X|S|M|L` para fijar la escala desde el primer momento.
- Introduce `charter-template.md` portado del `TEMPLATE.md` v3 de Sentinel. Lleva embebidas las seis convenciones que el experimento de abril fue cristalizando ciclo a ciclo: separación Local/Production checks, esfuerzo en tiempo (no en story points), sub-secciones estructuradas, documentación de riesgos `R<N>`, cierre con reconciliación post-merge vía AILOG, y auto-checklist drift.
- Publica `dist/.straymark/schemas/charter.schema.v0.json` — el schema marcado como `v0` deliberadamente, no `v1`, porque el principio #12 del framework dice que ningún schema cristaliza sin un segundo dominio que lo valide. Sentinel es un dominio. Falta el segundo.

PR #68 (3 de mayo) hace algo que parece pequeño pero importa: el *atomic Charter closure pattern*, format v4. En Sentinel, el paso *"actualizar el Plan-doc post-merge si el AILOG documenta divergencias"* existía como nota en el TEMPLATE pero no tenía gatillo sistemático. En la práctica, eso significaba que cuando había diferencia entre lo declarado y lo entregado, el operador (yo) confiaba en la memoria para reconciliar el documento — y la memoria fallaba. El drift quedaba en el repo días, a veces semanas, hasta que el siguiente ciclo lo descubría. Format v4 vuelve la reconciliación un paso obligatorio del cierre, no una nota al margen.

PR #69 (mismo día) cierra un detalle de fricción: la numeración manual. Antes había que decidir si el siguiente Charter era 11 o 12; ahora `straymark charter new` lee el directorio y propone el siguiente número. UX puro. Pero es la clase de fricción que, sumada sobre veinte Charters, decide si el framework se usa o se abandona.

Los tres PRs salieron en el mismo *commit storm* de un domingo y un lunes. Esa cadencia es deliberada: una vez que el patrón se cristaliza, los detalles UX hay que cerrarlos *junto*, no en bumps separados que dejen al adoptante con una mitad del flujo.

---

## 4. Tres releases en un día

Lo que vino después — fw-4.7.0, fw-4.8.0, fw-4.9.0 — son releases consecutivos del audit cycle externo. La propuesta `audit-skills-rollout.md` registra el ritmo sin disimular:

> *"Fase 1 ejecutada en 1 día calendar (5 PRs secuenciales el 3 de mayo de 2026), substancialmente más rápido que la estimación de 1.5-2 semanas focused. El throughput se debe a que el diseño en `audit-skills-design.md` ya tenía las decisiones cristalizadas (D1/D2/D3) y la heurística arborist con graceful-degradation explícita — no hubo decisiones pendientes durante la implementación. Sirve como evidencia operativa adicional para principio #6 (cuando la propuesta es bien escrita, la implementación es ejecución, no diseño)."*

El audit cycle externo concreto consiste en tres comandos encadenados:

1. `straymark charter audit prepare <CHARTER-NN>` — genera prompts canónicos para auditores externos a partir del Charter cerrado, su AILOG asociado, y el diff de archivos tocados. Output: tres archivos `.prompt.md` en `audit/<CHARTER-NN>/{copilot,gemini,claude}.prompt.md`.
2. *(El humano lleva esos prompts a su auditor de elección, recibe respuestas, las pega de vuelta.)*
3. `straymark charter audit collect <CHARTER-NN>` — toma las respuestas, valida cada una contra el schema de auditoría, fusiona calibraciones en la telemetría del Charter, y produce un summary con la convergencia (o divergencia) entre auditores.

Decisión D1 de la propuesta es la que sostiene todo lo demás:

> *"Las skills delegan vía `Bash(straymark charter audit *)` a la implementación canónica. Plantillas viven solo en `dist/.straymark/audit-prompts/`. Cero drift posible entre skill y CLI."*

Hay solo un lugar donde los prompts viven, una sola implementación del flow, y las skills (Claude, Cursor) lo invocan vía Bash. Es un patrón humilde — el CLI es la fuente única — pero evita lo que en cualquier framework con dos surfaces (skill + CLI) termina siendo el problema crónico: que el skill y el CLI digan cosas distintas según quién los actualizó la última vez.

---

## 5. Por qué el CLI orquesta pero no invoca APIs

Esta es la decisión más rara de mayo, y la que más me costó tomar. Está documentada literal en `cli-roadmap.md` §0 como *"decisión arquitectónica A1"*:

> *"A1 (Phase 3): orchestration-only, no HTTP API clients en v0. El roadmap §5.4 originalmente sugería 'soportar OpenAI/Google/Anthropic en v0' con manejo de API keys. La realidad implementada es que el CLI prepara prompts, valida outputs contra schema, e integra con telemetría — pero NO invoca APIs. El operador pega los prompts en su auditor de elección manualmente."*

El razonamiento, también literal:

> *"Implementar 3 clientes HTTP es 1-2 semanas + mantenimiento perpetuo cuando cambian las APIs (premature para una v0 experimental); el patrón humano-en-el-loop coincide con `/plan-audit` de Sentinel; cumple principio #10 ('no es un LLM gateway'); cierra RFC #82 por diseño. Los HTTP clients se reabren en v1 cuando un adoptante real lo justifique con datos."*

Tres argumentos, cada uno con peso propio.

**El primero es pragmático**. Tres clientes HTTP — OpenAI, Anthropic, Google — son entre una y dos semanas de implementación, más mantenimiento permanente cada vez que alguno de los tres cambia algo. Para un comando que en su forma manual ya estaba funcionando empíricamente en Sentinel, ese es un costo absurdo. Es ingeniería para resolver un problema que ningún adoptante había pedido resolver.

**El segundo es de continuidad**. Lo que Sentinel validó en abril fue precisamente el patrón humano-en-el-loop: el operador pega el prompt en Copilot/Gemini/Claude *a mano*, lee la respuesta, la pega de vuelta. El experimento de los seis Planes que dio origen a este arco completo *nunca* invocó APIs. Si el CLI ahora invocara APIs por su cuenta, estaría sustituyendo el patrón validado con uno no validado, sin razón empírica.

**El tercero es de identidad**. StrayMark no es un *LLM gateway*. El principio #10 del framework lo dice explícitamente. Hay docenas de productos que sí lo son — LangChain, LiteLLM, LiteLLM Proxy, OpenRouter, todos los wrappers — y son útiles para lo que hacen. StrayMark hace otra cosa: estructura la disciplina alrededor del trabajo que se hace *con* esos modelos, sea cual sea el modelo. Que el CLI invoque o no invoque APIs cambia lo que el framework es; mantenerlo *orchestration-only* mantiene la identidad limpia.

La parte que más me importa del párrafo: *"Los HTTP clients se reabren en v1 cuando un adoptante real lo justifique con datos."* La decisión no se cerró por dogma; se difirió por umbral de evidencia. Si en seis meses un adoptante demuestra que el flow humano-en-el-loop le rompe el throughput, los HTTP clients aterrizan. Hasta entonces, la fricción manual de los cuarenta segundos que toma pegar el prompt en otra ventana es trivial comparada con el costo de mantener tres SDKs vivos.

---

## 6. Lo que el framework decidió *no* automatizar

Vale la pena cerrar con la otra cara de la misma decisión. La propuesta `audit-skills-design.md` dice:

> *"Lo que se pueda automatizar de forma reversible debe automatizarse; los documentos quedan para auditoría humana ex-post."*

La primera mitad explica los tres releases de mayo: prompts generados automáticamente, calibraciones fusionadas automáticamente, drift detectado automáticamente. La segunda mitad — *"los documentos quedan para auditoría humana ex-post"* — explica qué se decidió mantener manual a propósito.

Lo que el framework *no* automatiza:

- **La lectura del AILOG por parte del operador antes de cerrar el Charter.** El comando `straymark charter audit prepare` genera los prompts, pero el operador *tiene* que abrir el AILOG y leerlo. Si automatizáramos eso — un agente que lee, resume, decide — perderíamos el momento de juicio humano que justifica que el Charter exista.
- **La decisión de aceptar o rechazar la auditoría externa.** El CLI fusiona las calibraciones en la telemetría, pero no actúa sobre ellas. Si un auditor califica el Charter con un 4, el Charter no se reabre automáticamente: el operador decide si reabrir, si argumentar, si declararlo *acceptable-with-known-debt*. Esa decisión es la disciplina; automatizarla la destruiría.
- **La invocación de los modelos.** Como ya argumenté arriba: la pausa donde el operador copia el prompt es exactamente donde decide qué modelo usar, qué prompt ajustar, si la pregunta tal cual está formulada le parece justa. Esa pausa es feature, no bug.

El criterio que recorre las tres es uno solo: *automatizar lo que sea reversible y mecánico; preservar como manual lo que sea juicio*. Cuarenta segundos de copy-paste no son demasiados si compran que el juicio sigue siendo humano.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Una práctica que funciona empíricamente no necesita ser reinventada para canonizarse.** Sentinel hizo el trabajo de validación en abril; mayo solo lo trasladó al framework. La cristalización es traslado, no diseño desde cero.

2. **Cuando la propuesta está bien escrita, la implementación es ejecución.** Tres releases en un día solo son posibles cuando todas las decisiones quedaron cerradas antes de tocar código. El principio #6 lo dice más prosaicamente, pero esto es lo que significa.

3. **No todo lo automatizable debe automatizarse.** La decisión A1 — el CLI orquesta pero no invoca APIs — es la versión técnica de una decisión filosófica: el humano-en-el-loop no es legacy del que hay que deshacerse, es feature que sostiene la disciplina.

4. **La identidad del framework se define tanto por lo que hace como por lo que no hace.** *"No es un LLM gateway"* es una de las pocas frases del principio #10 que vale la pena recordar literal. Lo que StrayMark hace lo hace bien porque hay muchas cosas que se niega a hacer.

Sigue, en el siguiente post, el episodio metodológico que conecta con el inicio mismo del blog — *Issue #113: Charters invisibles para los agentes* — donde el framework descubre que crear comandos y schemas no basta si la *superficie* del repo no le habla al agente. Es el contrapeso natural a este post: aquí hablamos de lo que se canonizó; allá hablamos de lo que no se vio.

---

*Anclas: PRs [#65](https://github.com/StrangeDaysTech/straymark/pull/65) · [#68](https://github.com/StrangeDaysTech/straymark/pull/68) · [#69](https://github.com/StrangeDaysTech/straymark/pull/69) (Charters first-class). Propuestas [`audit-skills-design.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-design.md) · [`audit-skills-rollout.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-rollout.md) · [`audit-cli-flow.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-04-audit-cli-flow.md) · [`cli-roadmap.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-cli-roadmap.md) (decisión A1 §0). Releases fw-4.4.0 → fw-4.9.0.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
