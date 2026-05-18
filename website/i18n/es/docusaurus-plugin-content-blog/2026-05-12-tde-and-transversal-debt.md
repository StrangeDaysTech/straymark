---
slug: tde-y-la-deuda-transversal
title: TDE y la deuda transversal
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - tde
  - deuda-técnica
  - governance
  - git-workflow
date: 2026-05-12T00:00:00.000Z
description: El tipo existía. El trigger no. Y la lección stacked-PR que llegó de regalo.
---
## 1. Cero TDEs en trece Charters

El Issue [#128](https://github.com/StrangeDaysTech/straymark/issues/128) abre con una asimetría inquietante:

> *"Sentinel adopter (primary, fw-4.12.0) created zero TDEs across 13 closed Charters despite ≥7 instances of transversal debt routed through parallel mechanisms."*

Trece Charters cerrados. Al menos siete piezas reconocibles de deuda transversal — heredada de Charters anteriores, atravesando módulos, persistente entre sesiones. Cero TDEs. Si el operador hubiera estado discutiendo cumplir cuotas, el dato no significaría nada. Pero el operador era yo y el adoptante era el mismo proyecto que valida empíricamente el framework. Si Sentinel — el repo donde más se intentó usar el framework completo — no producía TDEs ni una vez en trece intentos, había algo roto.

Lo roto no era la deuda, que estaba siendo capturada por otros canales (`R<N> (new, not in Charter)` dentro de los AILOGs, entradas en `follow-ups-backlog.md`). Lo roto era que ningún criterio del framework decía *"esta pieza concreta merece TDE, no R<N>"*. El tipo de documento existía. El trigger operativo, no.

Este post cubre el episodio del 11-12 de mayo de 2026: cómo se diagnosticó la falta del trigger, cómo se cerró con cuatro criterios canónicos (`fw-4.13.0`), y cómo el ciclo de PRs que arregló el problema dejó al pasar una lección operativa sobre stacked-PRs que terminó codificada en el `CLAUDE.md` del proyecto. Es el primer episodio del blog donde el framework arregla *dos cosas distintas* en el mismo arco, y donde la segunda es accidental.

---

## 2. El tipo existía. El trigger no.

`TDE` — *Technical Debt Entry* — existe en el framework desde el primer commit de enero (Post 2 lo registra: estaba en la lista de ocho tipos originales del README v1.0.0). El template estaba. El campo en la taxonomía estaba. La carpeta `06-evolution/technical-debt/` estaba.

Lo que no estaba era una frase, en algún sitio del aparato canónico, que dijera *"crea un TDE ahora"*. El Issue #128 lo formuló sin atenuar:

> *"Reading the adopter-facing docs (`AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `QUICK-REFERENCE.md`, `CLAUDE.md` autonomous rules), there is no clear trigger that says 'create a TDE now'. The trigger language exists for AILOG (creation freely), AIDEC (when alternatives considered), ETH (high risk PII), ADR (architectural decisions), but TDE is just listed as 'agent can Identify'."*

*"Agent can identify."* Identifica si quiere. Un agente entrando al repo lee esa línea, encoge los hombros, y sigue con AILOG y AIDEC, que sí le dicen *cuándo*. El defecto es el mismo del Issue #113 (Post 5): el artefacto técnicamente existe, pero la *superficie* canónica no le da al agente el verbo necesario para activarlo. Lo que diferencia este caso del #113 es la consecuencia. Para Charters, no verlos significaba no usar el flujo nuevo. Para TDE, no verlos significaba mucho peor: la deuda transversal *se capturaba* — pero fragmentada, sin vista consolidada, sin priorización entre artefactos, sin campo `assigned_to` ni `priority` para que un revisor humano pudiera ordenarla.

El Issue cita esa consecuencia con un nivel de detalle que da vergüenza, porque enumera todo lo que no estaba ocurriendo:

> *"No queryable 'all open technical debts' view at the document level. No `impact × effort` prioritization matrix per item. No `assigned_to` / `priority` fields surfaced to the human reviewer. Architectural debt that legitimately spans multiple charters (e.g., scope authorization gap heritage across modules) gets fragmented into per-charter R-numbers instead of consolidated into a single TDE."*

Trece Charters de deuda fragmentada en `R<N>` per-Charter. Vista del operador del repo: ninguna lista consolidada, ninguna heredada explícitamente nombrada como tal, ninguna pieza de deuda priorizable a nivel de proyecto. Todo estaba en el repo. Nada estaba en el sitio que el repo construye para los humanos que vienen a leerlo de afuera.

---

## 3. Cuatro criterios para una pieza de deuda

El PR [#129](https://github.com/StrangeDaysTech/straymark/pull/129) (`fw-4.13.0`, fusionado el 11 de mayo a las 19:38 UTC, exactamente 13 horas después de abrirse el Issue) cerró el hueco con una sección nueva en `AGENT-RULES.md §3` titulada *"TDE vs `R<N> (new, not in Charter)"*. Los cuatro criterios literales:

| Criterio | Cuándo aplica |
|---|---|
| **1. Heritage from a prior Charter** | La deuda viene heredada literalmente del Charter anterior (*strict heritage*) o reaparece al seguir el patrón del Charter anterior (*pattern propagation*). |
| **2. Applies to multiple modules or Charter execution boundaries** | La deuda atraviesa módulos del código *o* atraviesa límites de ejecución entre Charters (governance-trail debt). |
| **3. Requires a dedicated Charter outside current scope** | Resolverla requiere abrir un Charter propio, fuera del envelope del Charter actual. |
| **4. Requires human prioritization/assignment** | Decidir prioridad o asignación está fuera del rango del agente. |

La regla operativa: si una pieza de deuda satisface *al menos uno* de los cuatro criterios, es TDE. Si no satisface ninguno, sigue siendo `R<N>` dentro del Charter donde apareció.

Los cuatro tienen su lógica. Heritage (1) captura la deuda que persiste a través del tiempo, lo que un revisor querría ver agrupado por origen, no por Charter. Multi-module o multi-Charter (2) captura la deuda que vive *entre* artefactos, no *dentro* de uno. Dedicated Charter (3) reconoce que algunas piezas de deuda son tan grandes que merecen su propio ciclo y, por tanto, no caben como riesgo declarado *ex-ante*. Human prioritization (4) reconoce el límite de autonomía: hay decisiones que el agente puede señalar pero no resolver, y esas decisiones merecen un documento donde un humano pueda hacer cola.

El mismo PR añadió un campo nuevo al frontmatter del template TDE — `promoted_from_followup: FU-NNN | null` — y una sección en `FOLLOW-UPS-BACKLOG-PATTERN.md` que documenta cómo ascender una entrada de follow-ups a TDE. La promoción tiene dos formas, ambas registradas literalmente: promoción de entrada existente (un follow-up que llevaba semanas en la lista madura a TDE) y *retropromotion at creation* (al crear el TDE se reconoce que originó en un follow-up anterior y se anota). El campo del frontmatter cierra el rastro: cualquier TDE puede ahora apuntar hacia atrás al follow-up que lo originó, si existió.

---

## 4. Tres TDEs en Sentinel — el loop empírico cerrado

El Issue #128 no cerró por argumento; cerró por evidencia. El mismo día, Sentinel — el mismo adoptante que llevaba trece Charters sin un solo TDE — creó tres:

- Un TDE para una **brecha arquitectónica de RequireScope**, herencia que atravesaba módulos. Criterios 1 + 2 + 3.
- Un TDE sobre **cobertura de tests faltante en la capa HTTP**, deuda que persistía a través de Charters de feature. Criterios 1 + 4.
- Un TDE de **revisión de AILOGs legacy** anteriores al format v3 de Charter, una pieza que necesitaba su propio ciclo. Criterios 1 + 3.

Tres documentos en horas. La velocidad importa menos que el hecho de que cada uno aplicó *al menos uno* de los criterios y ninguno cayó en zona gris. Los criterios funcionaron como heurística sin ambigüedad — el operador no tuvo que negociar consigo mismo sobre si una pieza era TDE o `R<N>`. El loop empírico que el Issue había abierto en la mañana cerró con tres documentos esa misma tarde.

El PR [#136](https://github.com/StrangeDaysTech/straymark/pull/136) (`fw-4.13.1`, fusionado al día siguiente) refinó dos criterios a partir de esa misma experiencia. *Heritage* (1) se desdobló explícitamente en *strict heritage* vs *pattern propagation* porque uno de los tres TDEs de Sentinel mostraba el segundo caso de forma clara: la deuda no era heredada literal, era el mismo patrón reaparecido al seguir el mismo procedimiento. *Multi-module* (2) se reformuló como *"multiple modules **or Charter execution boundaries**"* porque otro de los tres TDEs era de governance-trail (cruzaba límites de sesión sin cruzar módulos de código). Los criterios crecieron una sola refinement-pass después de ejercerse en tres casos reales. El framework volvió a validar el principio #12: ningún schema cristaliza sin un segundo dominio.

---

## 5. La lección stacked-PR

Aquí es donde el post se desvía con propósito. El arco de PRs #129 → #131 → #133 que cerró el Issue #128 dejó una lección operativa adyacente — no sobre TDE sino sobre cómo *no* hay que armar branches encadenadas. La lección está hoy documentada literal en `CLAUDE.md` (líneas 181-220) del repo público bajo la sección **"Stacked PRs — avoid, or use merge commits"**.

### Cómo se rompió

PR #131 (`cli-3.12.1`, fix del validator que rechazaba `status: identified` en TDEs) se abrió con `base = #129` porque `#131` necesitaba el código de `#129` para compilar sus tests. Después, `#129` se fusionó a `main` con *squash*. El `CLAUDE.md` actual lo describe con una precisión que vale la pena citar entera:

> *"When you stack PR B on top of PR A's branch (base of B is A's head, not `main`), and A is squash-merged to `main`, B's content gets stranded:*
>
> *— The squash merge of A creates a new commit on `main` with content equivalent to A but a different SHA than A's original commits.*
>
> *— B's branch still points at A's original commits, which now have no descendant on `main`.*
>
> *— When B is merged, GitHub merges it into A's branch (its declared base), not `main`. The 'merge to main' never happens — even though GitHub UI shows B as MERGED."*

Resultado en la práctica del ciclo #129/#131: la UI de GitHub mostraba `#131` como *MERGED*, los tests del validator estaban en main aparentemente, pero el contenido nunca llegó. PR #133 fue el sync procedural que tuvo que abrirse después para llevar el contenido a su destino. Tiempo perdido en diagnóstico: alrededor de tres horas. Tiempo de la operación de recuperación una vez entendida: cinco minutos.

### Cómo se previene

El `CLAUDE.md` documenta dos estrategias, una conservadora y una pragmática:

> *"1. Sequential, not stacked. Wait for PR A to merge to `main`, then rebase B onto `main` and open B as a standalone PR with `base = main`. Slower but bulletproof.*
>
> *2. If you must stack, use merge commits (not squash) for the parent. A merge commit preserves shared history, so subsequent merges of stacked PRs into `main` resolve cleanly. Pay the cost of a noisier `git log` for the stacked-PR safety."*

Sentinel y StrayMark por defecto usan squash. Esa elección es buena para tener un `git log` limpio, una decisión por feature, un commit por PR. Pero cuesta exactamente esto: incompatibilidad con stacked-PRs. La regla que el `CLAUDE.md` ahora codifica es honesta: o no apilas, o pagas en historia de commits para poder apilar. No hay tercera opción que no rompa algo.

### Cómo se recupera (si ya pasó)

La sección de recovery del `CLAUDE.md` es la pieza más operativa del documento. Cuatro pasos:

> *"1. `git checkout -b chore/sync-<B-content>-to-main main`*
>
> *2. `git cherry-pick <B's merge commit SHA>` — should be clean because B touches files outside A's conflict zone (which is why it could be stacked in the first place).*
>
> *3. Push, open PR with `base = main, head = chore/sync-...`. Cherry-pick produces a fresh commit on top of `main`, so no conflicts.*
>
> *4. The branch protection may require `--admin` merge if the content was already reviewed in B (the original PR) — sync PRs are purely procedural."*

El último paso es importante editorial mente: nota explícita de que un merge `--admin` está autorizado *solo* para PRs procedurales como este — donde el contenido ya fue revisado en el PR original. Es la única excepción a la regla *"el contenido va por revisión humana"* del proyecto.

La razón por la que vale la pena registrar la lección como sub-sección del post sobre TDE — y no como post separado — es que comparten un rasgo: ambas son piezas de **deuda transversal de proceso** que el framework ahora nombra explícitamente. Una es deuda de código (TDE como tipo nuevo); la otra es deuda de workflow (stacked-PR como anti-patrón documentado). Las dos comparten origen: la fricción operativa real fue lo que disparó el documento.

---

## 6. Lo que el episodio dejó en el repo

Cuatro PRs en menos de veinticuatro horas, tres lecciones registradas:

- **PR [#129](https://github.com/StrangeDaysTech/straymark/pull/129)** (`fw-4.13.0`, 11 may 19:38 UTC) — TDE activation trigger. Cuatro criterios + sección nueva en `AGENT-RULES.md §3` + path FU → TDE.
- **PR [#131](https://github.com/StrangeDaysTech/straymark/pull/131)** (`cli-3.12.1`, 11 may 19:39 UTC) — validator acepta `status: identified` en TDEs. Sin el fix, el primer TDE creado bajo el nuevo trigger fallaba la validación.
- **PR [#134](https://github.com/StrangeDaysTech/straymark/pull/134)** (12 may 04:54 UTC) — la lección stacked-PR documentada en `CLAUDE.md`. No cambió código del framework; cambió las reglas operativas del repo.
- **PR [#136](https://github.com/StrangeDaysTech/straymark/pull/136)** (`fw-4.13.1`, 12 may 04:54 UTC) — refinamientos del trigger TDE basados en los tres casos reales de Sentinel.

Lo que más me interesa señalar de la cronología: el `CLAUDE.md` (PR #134) se actualizó el mismo arco que cerró el Issue #128. La lección stacked-PR no esperó a olvidarse. Si la disciplina del framework es *"todo cambio significativo deja rastro documentado"*, eso aplica también a la fricción operativa, no solo al diseño. El framework existe, en parte, para que los aprendizajes caros no se evaporen.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Un tipo de documento sin trigger es un tipo invisible.** Igual que en el Issue #113, lo que el agente no ve al entrar al repo no existe para él. TDE existía técnicamente desde enero, pero solo empezó a usarse cuando los cuatro criterios aparecieron en `AGENT-RULES.md §3`. Estructura sin verbo activador es decoración.

2. **La deuda transversal merece tipo propio.** Fragmentar deuda en `R<N>` per-Charter destruye la propiedad más importante de un sistema de gobernanza: la vista consolidada. Cuatro criterios, cualquiera basta, sostienen la línea entre *este es riesgo del Charter* y *esto es deuda del proyecto*.

3. **Los aprendizajes operativos también se documentan.** PR #134 actualizó el `CLAUDE.md` del proyecto el mismo día que la lección apareció. No es paranoia archivística; es la misma regla aplicada hacia los procesos: si la fricción te costó tres horas hoy, escribe la lección hoy.

4. **El framework crece tanto por features como por anti-patrones.** Este episodio dejó un tipo nuevo (TDE como entidad operativa) *y* un anti-patrón documentado (stacked-PRs incompatibles con squash). Ambas cosas son material del framework, aunque solo una aparezca como bump de versión.

Sigue, en el siguiente post, un episodio breve pero estructural: *"El audit-prompt en español era el outlier"* (`H-10`). Un detalle de convención i18n que reveló qué idioma estaba siendo el canónico — y por qué corregirlo cambió, sin proponérselo, cómo el framework piensa sobre traducción.

---

*Anclas: [Issue #128](https://github.com/StrangeDaysTech/straymark/issues/128). PRs [#129](https://github.com/StrangeDaysTech/straymark/pull/129) · [#131](https://github.com/StrangeDaysTech/straymark/pull/131) · [#134](https://github.com/StrangeDaysTech/straymark/pull/134) · [#136](https://github.com/StrangeDaysTech/straymark/pull/136). Releases: `fw-4.13.0` / `cli-3.12.1` / `fw-4.13.1`. Lección stacked-PR codificada en [`CLAUDE.md` líneas 181-220](https://github.com/StrangeDaysTech/straymark/blob/main/CLAUDE.md) del repo.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
