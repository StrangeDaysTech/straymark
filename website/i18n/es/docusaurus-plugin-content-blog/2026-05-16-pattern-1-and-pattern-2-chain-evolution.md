---
slug: pattern-1-y-pattern-2-chain-evolution
title: Pattern 1 y Pattern 2 — chain evolution
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - charters
  - chain-evolution
  - governance
date: 2026-05-16T00:00:00.000Z
description: Cuando la disciplina deja de ser hábito y se vuelve nombre
---
## 1. Veintisiete horas para nombrarlos

El PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152), que cerró el episodio del Post 9, se fusionó el 14 de mayo a las 21:39 UTC. Trajo `fw-4.14.3` y los tres gates canonizados del refresh manual.

El PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) — el que este post viene a contar — se fusionó el 16 de mayo a las 00:31 UTC. Veintisiete horas después. Trajo `fw-4.16.0 / cli-3.14.0` y, con eso, dos meta-patrones nombrados:

- **Pattern 1 — *pre-declare SpecKit refresh***. Refrescar la spec antes de declarar el siguiente Charter en una cadena.
- **Pattern 2 — *post-close audit-driven Batch N.4***. Amendar un Charter ya cerrado cuando la auditoría externa señala findings post-close, sin abrir un Charter nuevo.

El Post 9 cubrió la disciplina aplicada manualmente sobre CHARTER-18. Este post cubre lo que vino después: los dos patrones nombrados, el documento canónico que los nombra (`CHARTER-CHAIN-EVOLUTION.md`), el schema de telemetría que los mide, los dos *CLI helpers* que los scaffold-ean — y, como cierre del arco editorial entero, una hora y diecisiete minutos más tarde, un meta-patrón filosófico que reabsorbe ambos hacia arriba.

Es el último post del arco. El blog que el Post 1 abrió por delante cierra aquí por atrás.

---

## 2. Pattern 1 — Pre-declare refresh

La definición literal, copiada del documento canónico:

> *"A dedicated refresh PR lands between Charter-N close and Charter-(N+1) declare. It touches only the non-locked sections of the SpecKit artifacts (plan.md, data-model.md, contracts, quickstart.md, research.md)."*

Esto es exactamente lo que Sentinel hizo manualmente para CHARTER-18 en el Post 9. La diferencia entre Post 9 y Post 10 es de tipo: el Post 9 documenta el procedimiento; el Post 10 documenta su *activación condicionada* — cuándo el framework recomienda al adoptante que aplique Pattern 1.

Los criterios de activación son cuantificables, y el documento los nombra sin atenuar:

- El módulo tiene **≥ 3 Charters cerrados** sobre la misma spec.
- La media móvil (últimos 3 Charters) del campo `agent_quality.r_n_plus_one_emergent_count` en la telemetría es **mayor a 6**.
- No ha aterrizado un *refresh PR* desde el último *branch point*.

El umbral `> 6` no es arbitrario. Es Sentinel CHARTER-18 convertido en regla: el promedio de hallazgos emergentes (riesgos descubiertos durante el Charter que no estaban declarados en el plan original) durante CHARTER-15, 16, 17 fue exactamente lo que disparó la sensación de que la spec estaba acumulando demasiada deuda. El framework, en lugar de pedirle al adoptante que confíe en su intuición, le da un número que su telemetría puede comparar.

Lo que produce Pattern 1, cuando se aplica:

- Una tabla categorizada en `research.md` que enumera lo descubierto en los Charters anteriores en cuatro tipos: *reusable patterns*, *code gaps*, *discipline patterns*, *empirical corrections*.
- Decisiones operacionales explícitas (`D1`, `D2`, etc.) si la categorización evidencia trade-offs.
- Citas en el `§Context` del siguiente Charter, por `id`, hacia los items integrados.
- Un bloque de telemetría `pre_declare_refresh:` opt-in en el `charter-telemetry.yaml` del Charter que recibe el refresh.

La métrica que sostiene el patrón viene de Sentinel CHARTER-18 mismo, citada literal en `CHARTER-CHAIN-EVOLUTION.md`:

> *"Sentinel CHARTER-18 was the first Charter in a 7-Charter chain to close cleanly without a mid-flight remediation Charter. `estimation_drift_factor: 1.0`, `pre_work.items_discovered_during_planning: 0`, `overall_satisfaction: 5/5`."*

Un solo dominio. `N=1`. El documento lo dice sin atajos: el patrón está validado empíricamente *en Sentinel*. Esperar al segundo dominio antes de cristalizarlo más alto.

---

## 3. Pattern 2 — Amend-on-emergence

La definición literal:

> *"The amendment rides the same execute branch as the original Charter (the branch is still mergeable to main; the amendment commit lands on top)."*

La idea: cuando un Charter ya cerrado recibe findings de auditoría externa post-close — *critical* o *high* — el framework no obliga a abrir un Charter nuevo. El operador puede *amendar* el Charter cerrado mientras la rama `execute` sigue mergeable a `main`.

La justificación, también literal del documento:

> *"A new Charter would have created multi-week governance overhead for ~6h of focused engineering."*

Esa asimetría — semanas de gobernanza vs. seis horas de ingeniería — es lo que el patrón resuelve. Abrir un Charter nuevo para arreglar cinco findings post-close significa: declarar contexto nuevo, repetir auditoría externa, generar AILOG nuevo, registrar telemetría desde cero. Para una corrección que en código toma una tarde. Pattern 2 dice: no.

Los criterios de activación, también cuantificables:

- *Findings* Critical/High aparecen post-close, después de la auditoría externa.
- El criterio de cierre del Charter resultó materialmente incumplido por esos findings.
- La superficie del fix es menor a 25 archivos.
- No requiere reapertura arquitectónica (decisiones de diseño que el Charter original asumió como cerradas).

Si los cuatro criterios se cumplen, Pattern 2 aplica. Si alguno falla, hay que abrir Charter nuevo.

Lo que produce, cuando se aplica:

- Un commit de *amendment* sobre la misma rama `execute` del Charter original.
- Un AILOG nuevo (no editar el original) con `risk_level: high`, `review_required: true`, y un campo `amends:` que apunta al AILOG original.
- Una sección `## Historical correction (YYYY-MM-DD)` en el AILOG original que apunta *forward* al nuevo. Es archivística: el AILOG original no se reescribe; se enlaza.
- Un bloque de telemetría `post_close_amendment:` en el `charter-telemetry.yaml`.

La métrica empírica que sostiene Pattern 2, citada literal:

> *"Sentinel CHARTER-18 closed 2026-05-15 with audit reports landing 2026-05-15..05-17. Five findings (4 from gpt-5.3-codex, 1 from gemini-2.5-pro) were code-level fixes — DI wiring, retry header parsing, multi-tenant filter, timeout default. The Batch 7.4 amendment closed all five in one cohesive commit (19 files, +2257/-106 lines)."*

Un commit, 19 archivos, dos modelos auditores convergiendo en hallazgos similares, cinco fixes. Si hubiera sido Charter nuevo: dos semanas mínimo entre declare, audit externa, y close. En cambio: un *amendment* en la misma rama, seis horas.

---

## 4. Composición, no exclusión

La pieza más interesante del documento — y, creo, la que cierra mejor el arco — es la sección sobre cómo se relacionan los dos patrones. Cita literal:

> *"A Charter that received Pattern 1 is more likely to avoid Pattern 2, because the refresh absorbs pre-execution risk that would otherwise surface as post-close findings. But CHARTER-18 needed both — the refresh handled spec-level drift; the amendment handled runtime-level drift the refresh did not reach into."*

No son patrones rivales. Son aplicaciones del mismo principio en momentos distintos del ciclo. Pattern 1 absorbe deriva *de spec* — lo que el plan original ya no describe correctamente. Pattern 2 absorbe deriva *de runtime* — lo que solo se ve cuando el código corre, los auditores externos lo miran fresco, y emerge desde el código mismo.

Sentinel CHARTER-18 necesitó los dos. La spec refrescada eliminó los hallazgos emergentes que un audit cycle posterior habría tenido que remediar atomicamente; la *amendment* posterior absorbió lo que el refresh no podía haber predicho — wiring de DI, parsing de headers de retry, un filtro multi-tenant, un default de timeout. Cosas que el plan no podía conocer y que solo emergieron cuando el código existió.

El framework, al nombrar los dos patrones y declarar su composición explícita, hace algo que me importa subrayar: deja constancia de que la disciplina no es lineal. No hay un solo punto de inspección antes de cada Charter; hay un punto antes (Pattern 1) y un punto después (Pattern 2), y ambos pueden ser necesarios en el mismo Charter. La cadena evoluciona; los patrones también.

---

## 5. El schema y los helpers

Lo que `fw-4.16.0` añadió al schema de telemetría son dos sub-objetos opcionales en `charter-telemetry.schema.v0.json` v0.2:

```yaml
pre_declare_refresh:
  enabled: boolean
  refresh_pr: string | null
  refresh_aidec: string | null
  reusable_patterns_integrated: integer  # ≥ 0
  code_gaps_integrated: integer
  discipline_patterns_integrated: integer
  empirical_corrections_integrated: integer
  operator_decisions_ratified: integer

post_close_amendment:
  applied: boolean
  trigger: external_audit | production_incident | deferred_implementation
  ailog_id: string
  findings_closed: integer
  files_modified: integer
  effort_hours: number
```

Contadores enteros, no narrativas. La decisión es deliberada: lo que el telemetry schema captura es lo que el agente o el operador pueden contar mecánicamente. Lo que no se puede contar — *qué tan profundo era el problema, qué tan astuto fue el fix* — vive en el AILOG, no en la telemetría. El schema mide la *forma* del trabajo; los documentos llevan el *contenido*.

Y `cli-3.14.0` añadió dos helpers, ninguno mutador:

- **`straymark charter refresh-suggest <module> [--threshold N]`** — *read-only*. Lee la telemetría de los últimos tres Charters cerrados del módulo, computa la media móvil de `r_n_plus_one_emergent_count`, y emite una recomendación: *refresh-now*, *not-needed*, o *insufficient-data*. *Exit 0* siempre. Es informacional, nunca CI gate.

- **`straymark charter amend <CHARTER-ID> --trigger <kind> --ailog-title <title> [--findings-closed N] [--merge-into <PATH>]`** — scaffolding. Crea un AILOG stub con los campos correctos (`risk_level: high`, `review_required: true`, `amends:` apuntando al original), añade la sección `## Historical correction` al AILOG original, y renderiza el bloque YAML de `post_close_amendment:`. **No toca git.**

La elección de mantener los dos helpers humildes — uno solo recomienda, el otro solo prepara — es coherente con la decisión A1 del Post 4: *"el CLI orquesta pero no invoca APIs"*. Aquí: el CLI sugiere pero no decide; scaffold-ea pero no muta. El humano sigue siendo el punto donde la disciplina ocurre. El framework solo le pone las herramientas a la mano.

---

## 6. Setenta y siete minutos después

Hay una pieza más que vale la pena registrar antes del cierre del arco. PR #157 se fusionó a las 00:31 UTC del 16 de mayo. PR #160 — el del Post 1, `EMERGENT-OBSERVATION-DESIGN.md`, fw-4.17.0 — se fusionó a las 01:48 UTC del mismo día. Setenta y siete minutos después.

Lo que pasó en esos setenta y siete minutos es el cierre invertido del arco. El meta-patrón filosófico que el Post 1 nombró — *observación emergente compuesta de vínculos formales más permiso cultural para señalar* — se cristalizó **una hora y diecisiete minutos** después de los dos meta-patrones operativos de este post. El meta-patrón filosófico no precede a la disciplina; la sucede. Es la lectura ascendente de la misma evidencia.

`CHARTER-CHAIN-EVOLUTION.md` lo registra explícitamente en su sección `§Related`:

> *"EMERGENT-OBSERVATION-DESIGN.md — the meta-pattern of which Pattern 1 and Pattern 2 are applications (formal cross-referencing + cultural permission to surface)."*

Pattern 1 es composición de vínculos formales (la telemetría `r_n_plus_one_emergent_count`, las citas por `id` en el `§Context`, los AILOGs categorizados) con permiso cultural (la regla de que un agente debe señalar deriva entre spec y código cuando la encuentra). Pattern 2 es composición similar (los AILOGs con `amends:`, la sección `Historical correction` enlazando *forward*) con permiso para señalar findings post-close en lugar de absorberlos en silencio. Los dos patrones son aplicaciones del mismo principio cultural+estructural que el Post 1 nombró por encima.

Es un final que me sigue gustando: el blog que abrió diciendo *"el agente vio algo que nadie le pidió ver"* cierra diciendo, en el episodio anterior por orden cronológico, *"aquí están las dos cosas que el agente puede ver bien porque el framework las nombra"*.

---

## 7. Cierre del arco

Diez posts. Once episodios cubiertos (`H-01` a `H-13`, con varios agrupados). Una decisión editorial — la publicación retroactiva — que el Post 2 introdujo y que los nueve posts siguientes ejercieron sin violar. Un compromiso bilingüe que se mantuvo en cada cierre.

Algunas cosas que aprendí escribiendo el blog y que vale la pena registrar antes de cerrarlo:

1. **El patrón nace de la disciplina.** La frase ya apareció en el Post 9, pero el arco completo la sostiene. Los meta-patrones de este post no se diseñaron; se extrajeron. Cada hito documentado tuvo primero su episodio operativo y después su nombre.

2. **La cadencia no es uniforme.** Diecinueve días entre el primer commit (Post 2, 27 enero) y la primera unidad acotada con auditoría externa (Post 3, 25 abril). Cinco horas entre disciplina aplicada y disciplina canonizada (Post 9, 14 mayo). Setenta y siete minutos entre dos meta-patrones y su lectura filosófica (Posts 10 y 1, 16 mayo). El proyecto pasó por todas las velocidades, y el blog las registra todas.

3. **El framework no es neutro respecto a su adopter.** Sentinel hizo posible que cada patrón se validara empíricamente antes de cristalizarse. El blog no esconde esa procedencia; la subraya como precondición metodológica.

4. **Lo que no se nombra, no se ve.** El Post 5 lo formuló como *visibilidad estructural*; el Post 7 lo aplicó a la deuda transversal; el Post 8 lo señaló sobre el outlier de idioma. Es la regularidad más persistente del arco. Existir como artefacto técnico no basta. El nombre activa el artefacto.

El arco que el blog vino a contar — los hitos `H-01` a `H-13`, los cuatro nombres del proyecto, los seis Planes, los Charters first-class, el audit cycle externo, el Issue #113, el rebrand a StrayMark, el TDE, el outlier del audit-prompt, la disciplina manual, y los dos meta-patrones — cierra aquí. Lo que queda en deuda explícita: `H-14`, la TUI `straymark explore` (que apareció lateralmente en el Post 8 como herramienta de visibilidad), y los hitos candidatos `H-?-A` a `H-?-E` que `PLAN-INVESTIGACION.md §1.43-55` listó como pendientes de decisión. Posible segunda tanda futura. Sin promesa.

Una última observación, simétrica al Post 1. Aquel post abrió diciendo: *"el agente vio algo que nadie le pidió ver"*. Lo que el blog cierra registrando, después de diez episodios, es la otra mitad de la frase: el agente vio porque el framework lo había puesto en condiciones de ver. La observación emergente del Post 1 es la prueba; los patrones nombrados de este post son las precondiciones. Cada cosa que el agente señaló durante los cuatro meses que cubre el arco fue posible porque algún episodio anterior había puesto un artefacto, un disparador, un vínculo formal, una superficie visible donde antes no había nada.

El blog termina de contar la historia. El framework sigue.

---

*Anclas: [Issue #156](https://github.com/StrangeDaysTech/straymark/issues/156). PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) — `fw-4.16.0 / cli-3.14.0` (fusionado 2026-05-16 00:31 UTC). Documento canónico: [`CHARTER-CHAIN-EVOLUTION.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md). Schema: `dist/.straymark/schemas/charter-telemetry.schema.v0.json` v0.2. Meta-patrón sucesor: [`EMERGENT-OBSERVATION-DESIGN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/EMERGENT-OBSERVATION-DESIGN.md), `fw-4.17.0` (fusionado 2026-05-16 01:48 UTC, 77 minutos después).*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*

*— Fin del arco H-01 → H-13 del blog StrayMark.*
