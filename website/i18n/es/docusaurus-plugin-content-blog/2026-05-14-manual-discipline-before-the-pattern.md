---
slug: la-disciplina-manual-antes-del-patron
title: La disciplina manual antes del patrón
authors:
  - jose
tags:
  - straymark
  - charters
  - speckit
  - disciplina
  - governance
date: 2026-05-14T00:00:00.000Z
description: >-
  Doce aprendizajes en un Issue, tres gates en un PR, y un Charter que cerró
  limpio antes de que existiera el nombre
---
*Siete Charters consecutivos sobre un plan de un mes en Sentinel, ejecutando CHARTER-18 a mano con la sensación de que cualquier paso en falso enterraría seis Charters de trabajo. Cinco horas para canonizar la disciplina manual como gobernanza upstream — sin nombrar todavía el patrón.*

<!-- truncate -->

## 1. La pregunta correcta

El [Issue #150](https://github.com/StrangeDaysTech/straymark/issues/150), abierto el 14 de mayo a las 16:59 UTC, no buscaba un *yes/no*. Lo que el operador estaba pidiendo era reformular la pregunta:

> *"The real question the bridge doc should address is how, not whether: what discipline ensures spec stays in sync with code during multi-Charter execution without the regeneration step lying about the parts that already shipped."*

La disciplina ya existía. Se estaba aplicando en Sentinel en ese momento, sobre `CHARTER-18`, manualmente, con la sensación de que cualquier paso en falso enterraba el trabajo de seis Charters anteriores. Lo que faltaba — y lo que el Issue documentaba con precisión incómoda — era el nombre. El cuándo. Las reglas que un adoptante futuro pudiera seguir sin haber pasado por el dolor de inventarlas.

Este post cubre las cinco horas que separan ese Issue del PR que canonizó la disciplina como gobernanza upstream, los 12 aprendizajes empíricos que justificaron la canonización, y el episodio operativo concreto — `CHARTER-18` de Sentinel — que ejerció el patrón antes de que el patrón existiera. El siguiente post cubrirá lo que vino 27 horas después: el nombre.

---

## 2. Siete Charters sobre un mismo plan

El contexto, en una sola línea: Sentinel ejecutó `specs/002-commshub/plan.md` — escrito en el commit `be29421` el 21 de abril de 2026 — a través de **siete Charters consecutivos** (`CHARTER-07` a `CHARTER-17`), un mes calendario, sin refrescar el plan ni una vez.

El plan original era bueno. Estaba escrito con cuidado, con las cuatro user stories (US1-US4) detalladas, con el data-model declarado, con los contracts esbozados. Pero el plan era una fotografía de cómo el operador imaginaba en abril que el código se iba a comportar. La ejecución descubrió cosas que el plan no podía haber anticipado:

- Patrones de infraestructura que aparecieron *durante* la ejecución y resultaron reutilizables (un namespace en `core.processed_events`, un `withRLS` helper, triggers PL/pgSQL para invariantes).
- Gaps de código que el plan asumió resueltos (`AnomalyDetector` *half-built*, idempotency dedup faltante, `Recipient.TenantID` schema gap).
- Patrones de Charter/audit que se cristalizaron a lo largo del mes (cross-family audit cycle, HTTP layer test coverage gap, cursor pagination con strict-greater-than).

Cuando llegó el momento de planear `CHARTER-18` para US5, el plan tenía un mes de antigüedad y el código tenía un mes de aprendizaje empírico encima. Las dos cosas no estaban sincronizadas. Y `SPECKIT-CHARTER-BRIDGE.md` — el documento del framework que normalmente respondería *"¿qué hago aquí?"* — guardaba silencio sobre el caso. La regla *"cómo se mantiene la spec sincronizada con el código durante ejecución multi-Charter"* no existía todavía.

---

## 3. Los doce aprendizajes empíricos

El Issue #150 enumera los doce aprendizajes que se habían acumulado sin reflejar en el plan. La tabla literal, agrupada por tipo, vale la pena dejarla a la vista:

| # | Tipo | Aprendizaje |
|---|---|---|
| 1 | Patrón de infra | `core.processed_events` con namespace `module='commshub'` reutilizable — el plan describe su propia tabla. |
| 2 | Patrón de infra | `EnvSecretLoader` + Secret Manager projection (PRs #70/71) — el plan describe acceso directo. |
| 3 | Patrón de infra | `withRLS` helper como single-source-of-truth para aislamiento por tenant — el plan no lo ancla. |
| 4 | Patrón de infra | FR-005 triggers PL/pgSQL como enforcement de invariantes — el plan describe checks a nivel app. |
| 5 | Gap de código | `AnomalyDetector` *half-built* — el plan implica funcional. |
| 6 | Gap de código | Idempotency dedup en `delivery_log` faltante — el plan implica coordinado. |
| 7 | Gap de código | `Recipient.TenantID` schema gap — el plan referencia `tenant_id` como si existiera. |
| 8 | Gap de código | Per-tier rate limit overrides ausentes — el plan dice *"3-tier hierarchy"* sin mencionar que US1 implementó un tier uniforme. |
| 9 | Patrón de Charter | Cross-family external audit cycle obligatorio (6 consecutivos) — el plan silencioso. |
| 10 | Patrón de Charter | HTTP layer test coverage gap — el plan silencioso en test layering. |
| 11 | Patrón de Charter | Cursor pagination tuple con strict-greater-than — el plan silencioso. |
| 12 | Patrón de Charter | Dispatcher interface-stable swap pattern — el plan silencioso. |

Cuatro patrones de infraestructura descubiertos o refinados durante la ejecución. Cuatro gaps de código que el plan no anticipó. Cuatro patrones de Charter/audit cristalizados sobre la marcha. Doce piezas, en un solo mes, todas reales. El plan no las tenía. Y el siguiente Charter — `CHARTER-18` — iba a leer el plan como si no existieran.

La estimación que el operador documentó en el Issue es honesta:

> *"Estimated probability of ≥1 critical/high finding from stale-premise inheritance: ~50%."*

Cara o cruz, en una palabra. La mitad de los Charters arrancados sobre plan rancio terminan con un finding crítico que un audit cycle posterior debe remediar atomicamente pre-close. Y la otra mitad solo se libra por suerte.

---

## 4. Alternative 4: la disciplina aplicada a mano

Lo que pasó dentro de Sentinel, ese mismo día, antes de que la disciplina tuviera nombre: el operador evaluó cinco alternativas concretas y eligió la cuarta. El AIDEC privado de Sentinel — `AIDEC-2026-05-14-001-speckit-plan-scope-limited-us5-refresh.md` — registra el análisis sin atenuar. Lo parafraseo aquí porque el contenido detallado vive en el repo privado, pero la estructura del razonamiento es lo que importa:

- **Alternative 1 — *"Léelo como antes"*.** Mantener el plan desactualizado y leerlo como las seis veces anteriores. Costo: pagar la deuda en audit cycle posterior, ~50% probabilidad de finding crítico.
- **Alternative 2 — Regenerar todo con `/speckit-plan`.** Riesgo: sobreescribir aserciones sobre US1-US4 que el código real *no* cumple; el regenerador no sabe del estado actual.
- **Alternative 3 — Refresh en otro PR sin restricciones.** Mismo riesgo que Alt 2 más fricción de revisión.
- **Alternative 4 — Scope-limited refresh + 3 gates.** La elegida.
- **Alternative 5 — Diferir CHARTER-18, hacer un Charter de refactor first.** Costo de calendario alto, beneficio marginal sobre Alt 4.

La cuarta consistió en tres mecanismos concretos:

**Primer mecanismo — *scope-limited prompt*.** Ejecutar `/speckit-plan` con un *prompt* explícitamente restrictivo: regenera solo la Phase 7+8 correspondiente a US5; deja US1-US4 byte por byte como están, marcadas con comentarios `<!-- LOCKED: prior US shipped -->` que el agente debe respetar. Sin esa restricción explícita, la regeneración de SpecKit es destructiva: rescribe todo el plan, sobreescribiendo descripciones que el código real ya contradice.

**Segundo mecanismo — *Gate (a): validación contra realidad de código*.** Un script ad-hoc, ~30 líneas de bash + grep, que difea cada entidad non-US5 de `data-model.md` contra el schema real en `db/migrations/*.sql`, y cada endpoint non-US5 de `contracts/*.md` contra los handler signatures reales en `internal/modules/commshub/handler_*.go`. Cualquier divergencia bloquea merge. El script existe en el repo de Sentinel; no es elegante, es operativo. Lo que el framework canonizará después como gate (a) ya estaba ahí, como bash artesanal.

**Tercer mecanismo — *Gate (b): revisión granular hunk-por-hunk*.** Revisar `git diff` archivo por archivo, hunk por hunk, sin aceptar cambios a secciones *locked* sin justificación. El operador anota explícitamente la fricción esperada en R1 del AIDEC: si el agente regenera más hunks que los necesarios, el revisor humano se cansa y la disciplina colapsa. Mitigación: hunks de más de 50 líneas disparan revisión P1.

**Cuarto mecanismo — *Gate (c): two-PR split*.** El refresh es un PR independiente, revisado contra **código actual**. El Charter-fill que sigue, otro PR, revisado contra el plan ya refrescado. No mezclar las dos revisiones — porque si se mezclan, la deuda del refresh se esconde detrás del trabajo nuevo.

Cuatro mecanismos. Aplicados manualmente, sin documentación upstream que los avalara. Un solo adoptante (yo), una sola sesión, y la sensación todo el tiempo de estar resolviendo un caso que nadie había resuelto antes.

---

## 5. El resultado

`CHARTER-18` cerró el mismo día. Las métricas del Charter Telemetry, registradas literal en el YAML del cierre:

- `estimation_drift_factor: 1.0` — el Charter terminó exactamente en el rango de horas estimado.
- `pre_work.items_discovered_during_planning: 0` — ningún ítem inesperado apareció durante la fase de planning.
- `overall_satisfaction: 5/5` — la mejor calibración subjetiva del operador en toda la cadena.

Más importante que las métricas: `CHARTER-18` fue el **primer Charter de la cadena de siete que cerró sin un Charter de remediación mid-flight**. Los seis anteriores habían necesitado, cada uno, al menos una remediación atómica pre-close para cubrir divergencias entre lo declarado y lo entregado. CHARTER-18 no. El statement del operador en el cierre, citado literal del `CHARTER-CHAIN-EVOLUTION.md` posterior:

> *"The SpecKit refresh from PR #76 eliminated most ambiguity that drove drift in prior Charters. No mid-flight remediation Charter required — the EC1..EC15 empirical-corrections inventory in research.md absorbed what would have been pre-execution risk into in-execution awareness."*

La disciplina manual funcionó. No solo cerró el Charter limpio; cambió la naturaleza del riesgo. Lo que antes era riesgo pre-ejecución (descubrir divergencias durante la ejecución del Charter, remediar atomicamente) se volvió consciencia pre-planning (el inventario `EC1..EC15` enumera las correcciones empíricas antes de declarar el Charter). El riesgo no se elimina; se mueve a un momento donde es manejable.

---

## 6. Cuatro horas y cuarenta minutos

La cronología vale la pena dejarla precisa, porque es lo que el blog viene a registrar:

- **14 may, 16:59 UTC** — Issue #150 abierto. Los doce aprendizajes enumerados, los tres gates propuestos, el análisis de probabilidad publicado.
- **14 may, 21:39 UTC** — PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152) fusionado. `fw-4.14.3` shipped. *"Spec maintenance during multi-Charter execution"* añadido como sección nueva a `SPECKIT-CHARTER-BRIDGE.md` con los tres gates canonizados literalmente como gobernanza del framework. **Cuatro horas y cuarenta minutos** entre la pregunta y la respuesta upstream.

Lo que el PR #152 documentó como gobernanza es exactamente lo que el AIDEC privado había aplicado a mano horas antes. Las tres gates con los mismos nombres: validation against code reality (gate a), granular hunk-by-hunk review (gate b), two-PR split (gate c). Más cuatro heurísticas de *cuándo refrescar*: ≥3 Charters cerrados sobre el mismo plan, ≥4 semanas + ≥2 Charters, conteo de `R<N>(new)` > 6, target US toca infra refinada. Más una nota explícita sobre por qué NO volver a correr `/speckit-tasks` (porque destruye los marcadores `[X]` y las anotaciones `*CHARTER-NN:* <sha>` que forman el rastro histórico). Más una nota de roadmap mencionando un futuro `straymark spec-drift` CLI que mecanizaría gate (a).

Cuatro horas. No es exagerado decir que el framework escribió la guía mientras el operador todavía tenía el `git diff` abierto en otra ventana. La disciplina y su canonización fueron prácticamente simultáneas. Y eso me importa registrarlo porque es el orden inverso al que el aparato académico de governance suele asumir: primero la teoría, después la práctica. Aquí fue al revés. Primero la práctica, mientras dolía. Después la teoría, mientras el dolor todavía estaba fresco.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **El patrón nace de la disciplina, no al revés.** Los tres gates de `fw-4.14.3` no se diseñaron en abstracto; se *extrajeron* de un episodio operativo concreto, horas después de aplicarse manualmente. La gobernanza es post-empírica cuando el framework respeta el orden.

2. **La probabilidad explícita es disciplina.** *"~50% probabilidad de finding crítico por herencia de premisa rancia"* no es exactitud bayesiana; es una afirmación pública que obliga a justificar la decisión. Sin esa cifra, la pregunta *"¿refrescamos?"* se decide por intuición. Con esa cifra, se decide por aritmética de riesgo.

3. **Doce aprendizajes en treinta días es la cadencia real.** No es ruido capturable con *"déjame leerlo después"*. Es densidad empírica que el plan original no podía haber anticipado, y que destruye cualquier asunción de planning de largo plazo. La cadencia del desarrollo con agentes es esto.

4. **Cuatro horas no es récord; es propiedad del proceso.** Cuando la disciplina ya está aplicada en un dominio, canonizarla upstream es ejecución, no diseño. Lo mismo que vimos en el Post 4 sobre el audit cycle externo (*"cinco PRs en un día"*), lo mismo en el Post 6 sobre el rebrand (*"cuarenta y tres minutos"*). El framework no inventó la disciplina; la extrajo y le puso nombre.

Sigue, en el siguiente post, el último episodio del arco que el blog vino a contar: *"Pattern 1 + Pattern 2 — chain evolution"* (`H-12`). Veintisiete horas después del PR #152, los tres gates de gobernanza se cristalizaron como dos meta-patrones nombrados — *pre-declare refresh* y *amend-on-emergence* — con telemetry schema y CLI helpers. Es el cierre del arco que el Post 1 abrió por delante.

---

*Anclas: [Issue #150](https://github.com/StrangeDaysTech/straymark/issues/150) (14 may 16:59 UTC). PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152) — `fw-4.14.3` (14 may 21:39 UTC). Sección canonizada: `SPECKIT-CHARTER-BRIDGE.md §"Spec maintenance during multi-Charter execution"`. Métricas del CHARTER-18 reportadas en `CHARTER-CHAIN-EVOLUTION.md` (fw-4.16.0). Fuente privada parafraseada conforme `GUIA-EDITORIAL.md §7`: `AIDEC-2026-05-14-001-speckit-plan-scope-limited-us5-refresh.md` en el repo de Sentinel.*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
