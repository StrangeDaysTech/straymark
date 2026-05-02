# DevTrail — Validación empírica de la tesis con datos de Sentinel

**Versión:** 0.2 (rename Plan → Charter en referencias going-forward; las citas históricas a Sentinel preservan "Plan")
**Fecha:** 30 de abril de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Confrontar la tesis del producto, articulada en `devtrail-cloud-proposal.md` §2, contra la evidencia empírica generada por el primer experimento que la pone a prueba: `/plan-audit` ejecutado en el repo Sentinel durante 6 ciclos (PLAN-01..06) entre el 25 y el 28 de abril de 2026.
**Documentos relacionados:** `devtrail-cloud-proposal.md`, `devtrail-design-principles.md`, `devtrail-charter-telemetry.md`, `devtrail-cli-roadmap.md`, `que-es-un-charter.md` (rename Plan → Charter).

**Nota de vocabulario.** Este documento cita extensamente evidencia de Sentinel, donde el artefacto se llamó "Plan" (PLAN-01..06). Esas referencias históricas se preservan en su forma original. Las afirmaciones prospectivas sobre el producto DevTrail going-forward (§6 schemas, §8 decisiones) usan el nombre actual "Charter". Justificación del rename en `que-es-un-charter.md` §2.

---

## 1. Por qué este documento existe

Los otros tres documentos de `Propuesta/` se escribieron antes de tener datos. Son hipótesis: cómo querríamos que fuera la herramienta, qué principios queremos defender, qué patrón de mercado creemos servir. La tesis central es la pieza más cargada y más arriesgada de ese conjunto:

> "DevTrail es la disciplina cognitiva externalizada que un ingeniero senior necesita cuando orquesta agentes de IA en proyectos medianos a grandes. Estructura la memoria del proyecto, restringe el espacio de decisión del agente con reglas vivas, y produce como subproducto una evidencia auditable que sirve también para cumplimiento regulatorio cuando hace falta. El producto principal es la herramienta que mejora el oficio; el cumplimiento es valor adicional aprovechable, no el motor."
> — `devtrail-cloud-proposal.md` §2

Este documento confronta esa tesis con datos. No la defiende; intenta romperla. El objetivo es que un lector que no compró la tesis pueda, leyendo solo este texto, decidir si la evidencia disponible la sostiene, la matiza o la refuta. Si la evidencia no alcanza para algún supuesto, lo nombramos explícitamente y proponemos qué proyecto cerraría el gap.

El sesgo de auto-confirmación es el riesgo principal: el experimento lo diseñamos nosotros, lo ejecutamos nosotros, y lo evaluamos nosotros. La mitigación es metodológica: cada veredicto se ancla en una cita literal con archivo y línea aproximada, y los supuestos que la evidencia no cubre se marcan sin evidencia, no se infieren a fuerza.

## 2. La tesis y sus seis supuestos verificables

`devtrail-cloud-proposal.md` §2 descompone la tesis en seis supuestos que se derivan del uso real del CLI en proyectos como Sentinel. Los reproducimos aquí para anclar la confrontación:

1. **Vibe coding no escala.** El "vibe coding" no escala a proyectos medianos a grandes; el agente, sin estructura externa, acumula deuda técnica oculta, alucinaciones recurrentes y deriva arquitectónica que el ingeniero descubre tarde y caro.
2. **Notas estructuradas reducen modos de falla.** Una capa de notas estructuradas, vivas y reglamentadas sobre el flujo de trabajo del agente reduce significativamente esos modos de falla. AILOGs, ADRs y AIDECs no son burocracia: son memoria extendida y guardarraíles de razonamiento.
3. **Subproducto regulatorio sin trabajo extra.** El ciclo natural de trabajo asistido por IA produce, como subproducto, los artefactos que un auditor ISO 42001 necesita. El cumplimiento no requiere trabajo extra; requiere disciplina en el flujo.
4. **Aprobaciones rara vez binarias.** Las decisiones de aprobación humana raramente son binarias; típicamente son aprobaciones condicionales con condiciones que mezclan obligaciones inmediatas y obligaciones diferidas con triggers observables.
5. **Stage > commit como unidad de trazabilidad.** La unidad mínima de trazabilidad útil no es el commit, sino el Stage: una agrupación coherente de spec + implementación + auditoría + remediación + cierre con verificación reproducible.
6. **Evidencia in-situ + firma > reconstrucción posterior.** La evidencia generada en el momento del cambio y firmada criptográficamente es estructuralmente más confiable y útil que cualquier reconstrucción posterior.

## 3. El experimento: Sentinel `/plan-audit`

Sentinel es el repo de un sistema Go (~15.9 kLOC al cierre del MVP, 96 archivos de producción) que adopta DevTrail como framework de governanza desde sus primeras Etapas. El experimento `/plan-audit` corre en paralelo al desarrollo post-MVP: cada Plan ejecutado se documenta con telemetría YAML, se audita con dos modelos externos (Copilot v1.0.37 y Gemini CLI v1) en paralelo, y se consolida con un análisis crítico de Claude.

**Seis ciclos completos (25–28 abril 2026):**

| Plan | Escala | Lo que hizo | Format |
|------|--------|-------------|--------|
| PLAN-01 | XS | Deploy governance docs (PR #9) | v1 |
| PLAN-02 | S | Admin endpoint gcp-resource (PR #10) | v1 |
| PLAN-03 | XS | Contract bumps (PR #11) | v1 |
| PLAN-05 | M | Per-service anomaly threshold overrides (PR #13) | **v2** |
| PLAN-06 | XS | Manual baseline recompute admin endpoint (PR #18) | **v3** |
| (PLAN-04 catalog) | — | Roadmap de 7 features diferidas, no ejecutado | — |

**Tres iteraciones del formato del Plan:**

- **v1** (originario): 5 patrones identificados pero no formalizados.
- **v2** (AILOG-020): 5 patrones formalizados en `TEMPLATE.md` + sección "Format conventions" del README. Doc-only.
- **v3** (AILOG-022): 1 patrón nuevo (auto-checklist drift) + tooling ejecutable (`scripts/check-plan-drift.sh`, ~145 líneas bash). Bridge entre formato y enforcement.

**Fuentes de evidencia (rutas absolutas en el filesystem del autor):**

- Telemetría YAML: `/E/Proyectos/StrangeDaysTech/sentinel/.devtrail/plans/PLAN-{01,02,03,05,06}.telemetry.yaml`
- Auditorías externas duales: `/E/Proyectos/StrangeDaysTech/sentinel/audit/plans/{05,06}/{copilot-v1.0.37-audit,gemini-cli-v1-audit,claude-analisis}.md`
- Narrativa decision-by-decision: `/E/Proyectos/StrangeDaysTech/sentinel/.devtrail/07-ai-audit/agent-logs/AILOG-2026-04-28-{020,021,022,023,024}-*.md`
- Tooling ejecutable validado: `/E/Proyectos/StrangeDaysTech/sentinel/scripts/check-plan-drift.sh`
- Plan-docs canónicos bajo TEMPLATE: `/E/Proyectos/StrangeDaysTech/sentinel/docs/plans/{05,06}-*.md`

## 4. Confrontación supuesto por supuesto

### 4.1 Supuesto 1 — Vibe coding no escala

**Verdict: Validado parcialmente.**

El experimento no comparó head-to-head "agente con DevTrail" vs "agente sin DevTrail" — Sentinel adoptó DevTrail desde el inicio. Pero sí mostró que, *incluso con* disciplina y AILOGs estructurados, el agente acumula gaps no documentados que solo afloran con auditoría externa. Esto es un proxy: si con disciplina el agente todavía olvida cosas, sin disciplina las olvidaría todas.

> "F4 (IG): `evaluator_test.go` declarado con 3 tests para `ResolveAnomalyThresholds` pero **sin cambios en el rango**. Cobertura unitaria del resolver ausente. **Hallazgo no documentado en AILOG-021** — gap real de cobertura."
> — `audit/plans/05/claude-analisis.md` §3 (línea ~106), captura combinada de Copilot + Gemini.

> "F4 evaluator_test.go gap olvidado: el Plan declaró 3 tests al evaluator_test.go que NO implementé. Ningún checkpoint pre-commit lo atrapó (compile pasa, otros tests pasan). AILOG-021 NO documenta este gap a pesar de documentar otros 3 R<N> nuevos. Patrón: drifts pueden ser de OMISIÓN, no solo de cambio — el agente nombra los drifts que ve, pero no los que olvida."
> — `PLAN-05.telemetry.yaml` §qualitative.friction_points (línea ~165).

> "F5 (hallucination): Plan declaró injection en `statuscenter/service.go` pero Wire instancia `NewAnomalyDetector` en `cmd/sentinel/wire_gen.go`, NOT en `service.go`. The Plan was architecturally wrong from origin about where to inject."
> — `PLAN-05.telemetry.yaml` §scope_change_notes (línea ~144).

**Lo validado:** el agente, incluso con disciplina, comete dos clases de fallo que solo afloran post-hoc: (a) implementación olvidada del Plan (F4: 3 tests prometidos no escritos), (b) hallucination arquitectónica del Plan-doc mismo (F5: el Plan asumió un punto de inyección que no existe). Ninguna de las dos fue capturada por checkpoints internos del agente — solo por auditoría externa multi-modelo.

**Lo no demostrado:** que sin DevTrail el problema sería *peor*. La hipótesis es plausible (los gaps capturados son justamente del tipo que se esfuma cuando no hay un Plan-doc declarado contra el que comparar), pero el experimento no construyó un brazo de control. Para una validación más fuerte, un proyecto futuro con dos brazos (con disciplina vs sin) o un proyecto que adopte DevTrail tarde y mida el delta de gaps capturados antes/después cerraría el supuesto.

### 4.2 Supuesto 2 — Notas estructuradas reducen modos de falla

**Verdict: Validado.**

La evidencia más fuerte: cuando una convención se nombra en el AILOG, los auditores externos la capturan; cuando no se nombra, no la ven. Esta es la propiedad estructural que el supuesto afirma.

> "Los patrones internos que llevan tiempo aplicándose pero no tienen nombre quedan invisibles a auditores externos. Nombrarlos formalmente convierte la práctica en señal pública."
> — `AILOG-020` §Additional Notes (línea 262–264). Observación destilada de que AILOG-019 nombró R4 (riesgo del slice no-auto-extiende) y Gemini lo capturó externamente *exactamente* porque el AILOG lo nombraba.

> "Mejor calibración combinada de auditores en 5 ciclos (Copilot 9.25, Gemini 9.5). Hipótesis acumulada confirmada: TEMPLATE v2 + AILOGs ricos reducen el espacio de ambigüedad para auditoría externa."
> — `PLAN-05.telemetry.yaml` §qualitative.wins (línea ~169).

> "Tendencia inter-Plan (5 datos):
>
> | Auditor | Plan 01 | Plan 02 | Plan 03 | Plan 05 |
> |---------|--------:|--------:|--------:|--------:|
> | Copilot | 8.0 | 8.5 | 7.8 | **9.25** |
> | Gemini | 7.6 | 7.2 | 9.3 | **9.5** |
>
> Ambos auditores en su mejor calibración del experimento. Hipótesis: el TEMPLATE v2 + el AILOG rico de PLAN-05 redujeron el espacio de ambigüedad, permitiendo categorización más precisa. **El experimento está convergiendo**."
> — `audit/plans/05/claude-analisis.md` §6 (línea 162–169).

> "3 R<N> nuevos descubiertos durante ejecución (R6, R7, R8). Patrón validado en 4/4 ciclos del experimento. Confirma que la convención `R<N+1> (nuevo, no en Plan)` del TEMPLATE v2 captura señal real."
> — `AILOG-021` §Additional Notes (línea 351–353).

**Lo validado:** las convenciones nombradas en docs estructurados (R<N+1>, separación Local/Production checks, esfuerzo en TIEMPO) son capturadas por auditores externos heterogéneos (Copilot y Gemini, modelos distintos), lo que se traduce en convergencia de scores hacia 9.25/9.5. La estructura *exporta* señal pública.

**Matiz importante:** los AILOGs ricos no eliminan los modos de falla — los reducen. F4 y F5 muestran que el agente puede no documentar lo que olvida. La conclusión refinada es la del propio claude-analisis: *"los R<N> en el AILOG son señal valiosa pero NO sustituyen auditoría externa rigurosa. La auditoría es complemento, no redundancia"* (`PLAN-05.telemetry.yaml` línea ~213). Esto refuerza la tesis sobre disciplina cognitiva en lugar de debilitarla: la disciplina no promete eliminar errores, promete hacerlos *visibles*.

### 4.3 Supuesto 3 — Subproducto regulatorio sin trabajo extra

**Verdict: Validado.**

El experimento generó, sin pasos adicionales más allá del cierre normal de cada Plan, un cuerpo de evidencia auditable: cinco telemetrías YAML estructuradas con campos compatibles con NIST AI RMF (information_integrity tracking) e ISO/IEC 42001 cláusulas 8 y 9, un AILOG por ciclo con frontmatter normalizado (`risk_level`, `eu_ai_act_risk`, `nist_genai_risks`, `iso_42001_clause`), y reportes de auditoría dual con consolidación crítica.

> Frontmatter de `AILOG-2026-04-28-021` (líneas 9–13):
>
> ```yaml
> risk_level: medium
> eu_ai_act_risk: not_applicable
> nist_genai_risks: [information_integrity, value_chain]
> iso_42001_clause: [6, 8]
> ```

> "Cero `real_debt` — toda la Verification pasó limpia (build, gosec 0 issues / 96 files / 15.9 kLOC, govulncheck clean, integration test 10s). Primer ciclo del experimento bajo M sin deuda técnica detectable por auditores externos."
> — `PLAN-05.telemetry.yaml` §qualitative.wins (línea ~168).

**Lo validado:** los artefactos producidos (AILOGs con frontmatter regulatorio, telemetría YAML auditable, drift-check ejecutable, reportes de auditoría externa) son consumibles por un auditor ISO 42001 sin trabajo extra del ingeniero. El esfuerzo total reportado para PLAN-06 fue ~40 min (vs ~30 min estimado de código puro): el +33% incluye AILOG + drift validation + tests, no horas dedicadas a "compliance" como tarea separada.

**Lo no probado todavía:** que un auditor real (humano externo, no LLM) acepte estos artefactos como suficientes para una certificación. Eso requiere un próximo ciclo donde un compliance officer real revise los outputs. La afirmación a este punto es: la evidencia *existe* en formato estructurado y firmable; queda probar que un auditor la acepte.

### 4.4 Supuesto 4 — Aprobaciones rara vez binarias

**Verdict: Sin evidencia. Pendiente otro proyecto.**

Sentinel lo trabajó un solo ingeniero (Jose Villaseñor Montfort) en sesiones individuales con Claude Code. No hubo flujo de aprobación multi-actor durante el experimento `/plan-audit`. El supuesto se sostiene sobre observaciones empíricas de otros contextos (mencionadas en `cloud-proposal.md` §4.2), pero el experimento *no lo ejercitó*.

Para validar este supuesto se requiere un proyecto con al menos dos personas decisoras (ingeniero implementador + revisor con autoridad para condicionar el merge), idealmente con flujo asíncrono donde la aprobación incluya condiciones diferidas (texto humano nivel 1 → estructurado nivel 2 → policy-as-code nivel 3 según `cloud-proposal.md` §4.2).

**Recomendación accionable:** documentar el supuesto #4 como "hipótesis prometedora pero no validada" en `cloud-proposal.md` v0.3, y nombrar como criterio de salida de la fase 1 (12-18 meses) tener al menos un proyecto adoptante con flujo multi-actor real. No diseñar features de aprobación condicional en Cloud hasta tener este dato.

### 4.5 Supuesto 5 — Stage > commit como unidad de trazabilidad

**Verdict: Validado.**

PLAN-05 (M, 1305 líneas, 19 archivos modificados, 1 commit atómico) es ejemplo paradigmático: el commit `473f6e0` por sí solo no captura las tres decisiones arquitectónicas tomadas durante implementación (R6 simplificación de repository, R7 simplificación de integration test, R8 fix de wire.go). Esas decisiones existen *como Stage*, no *como diff*.

> "3 R<N> nuevos descubiertos durante ejecución (R6, R7, R8). Plan-doc drift positivo y negativo: drift positivo (R6: simplification) + drift negativo (R7: integration test simplification, R8: wire.go fix out-of-scope). El TEMPLATE v2 sección `## Cierre del Plan` recordará actualizar el Plan doc post-merge."
> — `AILOG-021` §Additional Notes (línea 351–357).

> "El Plan declaraba `SetAnomalyThresholds(ctx, caller, serviceID, thresholds)` con SUPER_ADMIN guard EN el service (estilo identity en PLAN-02). La convención del módulo policyengine es: handler hace auth, service confía y recibe `changedBy string`. Adopté la convención local."
> — `AILOG-021` §Decision (línea 213–219).

> "PLAN-05 es la primera ejecución bajo M y bajo TEMPLATE v2. Los datos confirman que el formato v2 es robusto bajo carga mayor y que la disciplina de auditoría externa sigue siendo necesaria incluso con AILOGs detallados — los R<N> en el AILOG son una guía, no un reemplazo."
> — `audit/plans/05/claude-analisis.md` §7 (línea ~196).

**Lo validado:** el conjunto Plan-doc + AILOG + telemetría + auditoría dual + claude-analisis es la unidad mínima útil para reconstruir lo que pasó en PLAN-05. El commit aislado pierde la mitad de la información (qué se descartó, qué convenciones locales aplicaron, qué drift aceptado, qué riesgo emergente). Esto es exactamente el patrón Stage que el supuesto afirma.

### 4.6 Supuesto 6 — Evidencia in-situ + firma > reconstrucción posterior

**Verdict: Validado parcialmente.**

La parte *evidencia in-situ* está fuertemente validada: `check-plan-drift.sh` ejecutado al cierre del Plan detecta drifts archivos-declarados-vs-modificados con cero falsos positivos en 2/2 tests empíricos.

> "Validación empírica: ejecuté el script sobre PLAN-05 (`fd65e87..473f6e0`) y sobre PLAN-01 (`cb2e1e8..3167475`):
>
> - PLAN-05 (con drifts conocidos): script reporta **3 omitted files**: `evaluator_test.go` (F4), `repository.go` (F1/R6), `statuscenter/service.go` (F5) — exactamente los 3 drifts que Copilot+Gemini capturaron en sus auditorías. Más 4 scope expansions, 3 de los cuales son test mocks + wire.go (R8 documentado). **Cero falsos positivos.**
> - PLAN-01 (ciclo más limpio del experimento): script reporta **0 drifts** — exactamente lo que las auditorías de Plan 01 indicaron. **Cero falsos positivos.**"
> — `AILOG-022` §Summary (línea 59–69).

> "Validación de hipótesis v3: `check-plan-drift.sh` reportó 0 drift pre-commit; los 3 findings reales son del tipo que el script no puede detectar por diseño (mitigaciones no implementadas, scope text vs payload, tests promised vs realized). Confirma: script + auditorías son complementarios."
> — `AILOG-024` §Post-audit fixes (línea 154–158).

**Lo no probado todavía:** la parte de *firma criptográfica*. Sentinel no firmó bundles con Sigstore/cosign ni implementó hash-chaining. La afirmación de que firmar in-situ es estructuralmente más confiable que reconstrucción posterior es plausible (es la propiedad técnica de cualquier append-only log) pero el experimento no la ejercitó. Cerrar este gap requiere implementar `devtrail stage close` con firma cosign en un próximo ciclo y un escenario de tampering simulado para verificar detección.

## 5. Fricciones detectadas y marco virtud-vs-ceremonia

Sentinel reveló cuatro puntos de fricción cuantificables. El principio #6 de `devtrail-design-principles.md` ya establece que "fricción justificada es aceptable y a veces deseable". El refinamiento empírico del experimento es distinguir *fricción virtuosa* (la que externaliza signal a auditores externos y captura gaps que el agente solo no captura) de *ceremonia atacable* (la que solo genera triage manual o prescribe sin reflexividad).

### 5.1 Overhead de documentación — virtud parcial

> "Esfuerzo estimado: XS (~30 min). Esfuerzo real: XS (~40 min con tests + AILOG + drift validation). Drift en líneas: ~50 (Plan) → ~280 (real) = 5.6x — consistente con el patrón del experimento (líneas no predictivas; tiempo cumplió 1.0x)."
> — `AILOG-024` §Notes (línea 125–128).

El +33% sobre el estimado XS no es overhead doc puro — incluye tests + AILOG + drift validation. El bloque de "AILOG + drift validation" agregado al cierre genera, a cambio, el material que dos auditorías externas consumieron y que en PLAN-06 capturó pre-merge un fix operacional (timeout R1 no implementado en handler). El ROI del +33% es: un fix sin hotfix post-deploy. Virtud justificada bajo principio #6.

### 5.2 Prescripciones que chocan con convenciones locales — ceremonia atacable

> "El Plan declaraba `SetAnomalyThresholds(ctx, caller, serviceID, thresholds)` con SUPER_ADMIN guard EN el service (estilo identity en PLAN-02). La convención del módulo policyengine es: handler hace auth, service confía y recibe `changedBy string`. Adopté la convención local — handler.go usa `requireSuperAdmin` y pasa `caller.UserID` como changedBy. Esto evita duplicación de auth y mantiene el módulo coherente."
> — `AILOG-021` §Decision (línea 213–219).

El TEMPLATE prescribió una firma arquitectónica que conflictaba con la convención del módulo destino. El ingeniero tuvo que detectar y resolver manualmente, generando un drift documental (F2 en la auditoría) que no aporta señal nueva. Esto es ceremonia: el formato no distinguió *prescripciones arquitectónicas irrenunciables* de *convenciones reflexivas dependientes del módulo*. Mitigación propuesta para format v4: campo separado "Wiring/instancing" que fuerce la pregunta "¿esto se instancia en service.go o en wire_gen.go?" durante planning, en lugar de prescribir la firma directamente. Idea ya capturada en `PLAN-05.telemetry.yaml` §proposed_format_changes #7.

### 5.3 Drift script genera triage manual sobre R<N> ya documentados — ceremonia parcial

> "R2 — Drifts de OMISIÓN ya documentados en AILOG generan ruido: si el agente ya documentó 'evaluator_test no implementado por X razón' como `R<N> nuevo` en AILOG, el script seguirá reportándolo como omitted. Mitigación: el output del script dice claramente 'Action: either complete the work, or document in AILOG'. El agente lee el AILOG y, si ya está documentado, ignora la alerta — costo bajo. Mejora futura: el script podría grep el AILOG por el path omitido y suprimir la alerta si lo encuentra."
> — `AILOG-022` §Risk (línea 241–248).

El script en su forma actual genera alertas que el ingeniero filtra mentalmente contra el AILOG. Es ceremonia parcial — vale el costo (signal vs ruido), pero la integración futura `drift-check` AILOG-aware (R2 en el AILOG-022 mismo es el plan a futuro) puede eliminar el triage. Cuando se porte al CLI Rust (fase 2 del roadmap), implementar AILOG-awareness desde el inicio.

### 5.4 Plantillas prescriptivas no anticipan flakes de testcontainers — virtud

> "El Plan declaraba 'verifica que un override cambia el comportamiento del detector end-to-end'. Implementación inicial encontró timing flake bajo testcontainers (audit pipeline async + RLS context entre heartbeat y override). Decision: simplificar a round-trip policyengine y dejar la behavioral verification al unit test que no tiene flake risk. Defense in depth se mantiene; cobertura se mueve a la capa correcta. R7 (nuevo, no en Plan): integration tests con timing crossing multiple goroutines + RLS contexts pueden ser frágiles. Convención candidata para format v3."
> — `AILOG-021` §Decision (línea 228–242).

El Plan declaró una verificación que la realidad de testcontainers + audit pipeline async no permitió. El ingeniero divergió, documentó la divergencia como R7 nuevo, y propuso una convención candidata para format v3. Esto es exactamente el ciclo auto-evolutivo que el principio #12 valida: la fricción produjo conocimiento accionable que mejoró el formato siguiente. Virtud.

### 5.5 Síntesis del marco

La fricción de DevTrail es virtud cuando externaliza signal pública (R<N+1>, drift detection, auditoría dual) y captura gaps que el agente solo no captura. La fricción es ceremonia atacable cuando solo genera triage o prescribe sin reflexividad. La primera se mantiene; la segunda es bug del formato, no virtud del principio. El refinamiento de `devtrail-design-principles.md` v0.2 incorpora explícitamente esta distinción.

## 6. El argumento N≈2-3: por qué Sentinel cuenta más que un proyecto

El principio #12 de `devtrail-design-principles.md` v0.1 establece: "no debe avanzar más rápido de lo que aprendemos sobre cómo se usa realmente. Cristalizar features prematuramente, antes de tener datos de uso real en al menos tres proyectos distintos, genera costos altos de mantenimiento sobre features que pueden ser equivocadas".

Sentinel es *un solo proyecto* en el sentido literal (un repo, un dominio Go backend, un autor). Pero la evidencia que produjo tiene tres ejes de diversidad estructural que el principio busca capturar:

1. **Escala variable.** PLAN-01/03 (XS, ~30 min, doc-only o cambios pequeños), PLAN-02 (S, 1h, admin endpoint), PLAN-05 (M, 1.5h, 1305 líneas, 19 archivos, 5 capas tocadas), PLAN-06 (XS, 40 min, endpoint admin con timeout). El TEMPLATE v2 fue diseñado con datos XS-S y se estresó por primera vez bajo M en PLAN-05; el TEMPLATE v3 se diseñó retrospectivamente con datos M y se ejercitó prospectivamente bajo XS en PLAN-06. La diversidad de escalas validó que el formato escala hacia arriba *y* hacia abajo.

2. **Iteración del formato bajo presión empírica.** v1 → v2 → v3 con cada iteración derivada del ciclo anterior. v2 emergió de PLAN-01/02/03 (3 datos). v3 emergió de PLAN-05 (1 dato bajo nueva escala). Cada iteración tiene costo decreciente y valor creciente. El experimento es *auto-evolutivo*: cada Plan ejecutado mejora el formato del próximo.

   > "Patrón meta-meta detectado: cada iteración del formato resulta de un ciclo del experimento. v2 emergió de PLAN-01/02/03 telemetry. v3 emerge de PLAN-05 telemetry. Predicción: v4 emergerá del primer Plan que ejercite v3 (PLAN-06 si se materializa). Esto valida que el experimento es **auto-evolutivo**: cada Plan ejecutado mejora el formato del próximo."
   > — `AILOG-022` §Additional Notes (línea 288–292).

3. **Calibración cross-modelo de auditores.** Dos modelos heterogéneos (Copilot v1.0.37 GPT-derived, Gemini CLI v1) auditando los mismos planes con criterios paralelos. La convergencia inter-modelo (9.25/9.5 en PLAN-05) es señal estructural de que la disciplina captura signal generalizable, no idiosincracia de un modelo.

**Conclusión del argumento:** los seis ciclos × tres formatos × tres escalas constituyen el *espíritu* del N≥3 que el principio #12 busca proteger — observación antes de cristalización, evolución incremental, evidencia empírica antes que intuición. Pero queda un eje no diversificado: **un solo dominio (Go backend)** y **un solo autor**. Cualquier patrón que se cristalice del experimento debe marcarse como *experimental/opt-in* y un segundo proyecto en otro dominio (frontend, ML pipeline, infra-as-code) puede invalidar parte del schema.

**Recomendación operacional:** los schemas que se cristalicen en el CLI a partir de Sentinel (`charter.schema.v0.json`, `charter-telemetry.schema.v0.json`) llevan el sufijo `v0` y la marca `experimental`. La transición a v1 estable requiere validación con un segundo proyecto en dominio distinto. Esta es la cláusula de salida explícita del principio #12.

## 7. Hallazgos no anticipados por ningún doc previo

Tres patrones emergieron del experimento que ninguno de los tres documentos de propuesta (escritos antes) había articulado:

**7.1 El formato es auto-evolutivo.** Ninguno de los tres docs anticipó que la aplicación del formato a sí mismo (telemetría → análisis crítico → propuesta de mejora → nuevo formato) sería un ciclo natural. Esto es un meta-patrón estructural que merece nombre propio en futuras versiones de los principios. Candidato: principio #13 ("la herramienta evoluciona consigo misma; cada uso es input para la próxima versión") o ampliación del principio #12.

**7.2 Drift script y auditoría externa son complementarios, no sustitutos.** AILOG-022 hipotetizó complementariedad; PLAN-06 la confirmó empíricamente. Esto cambia el marco mental de cómo diseñar tooling DevTrail: cada layer debe atacar un tipo de drift distinto (declaración vs implementación vs comportamiento), no replicar capacidades existentes.

   > "Hipótesis v3 confirmada: script + auditorías + AILOG son herramientas complementarias. Ninguna sustituye a las otras."
   > — `audit/plans/06/claude-analisis.md` §7 (línea 145).

**7.3 Auditoría dual como calibrador de modelos.** PLAN-05 fue el caso límite: Gemini calibró F5 correctamente como `hallucination` mientras Copilot lo categorizó como `implementation_gap`. La auditoría dual no es solo "dos pares de ojos" — es un mecanismo de calibración cross-modelo. El claude-analisis funciona como tercer modelo que reconcilia veredictos divergentes según el schema definicional. Esto sugiere una arquitectura de tres niveles para `devtrail charter audit` (fase 3 del roadmap): dos auditores en paralelo + un calibrador-reconciliador, no solo "ejecuta un audit con N modelos".

## 8. Verdict global y siguientes pasos

| # | Supuesto | Verdict |
|---|----------|---------|
| 1 | Vibe coding no escala | Validado parcialmente (sin brazo de control) |
| 2 | Notas estructuradas reducen modos de falla | Validado |
| 3 | Subproducto regulatorio sin trabajo extra | Validado (pendiente prueba con auditor humano real) |
| 4 | Aprobaciones rara vez binarias | Sin evidencia — pendiente proyecto multi-actor |
| 5 | Stage > commit como unidad de trazabilidad | Validado |
| 6 | Evidencia in-situ + firma > reconstrucción posterior | Validado parcialmente (sin firma criptográfica probada) |

**Resultado neto:** 3 supuestos validados completamente, 2 validados parcialmente, 1 sin evidencia, 0 refutados. **La tesis se sostiene** con la evidencia disponible. Los gaps son áreas que requieren validación adicional, no contradicciones.

**Próximos proyectos sugeridos para cerrar gaps:**

1. **Adoptante en otro dominio** (frontend, ML pipeline, infra) para diversificar el eje "dominio" del principio #12. Cierra parte del supuesto #1 si se mide delta de gaps capturados antes/después.
2. **Proyecto con flujo multi-actor** (al menos 2 personas decisoras, aprobación asíncrona, condiciones diferidas reales). Cierra el supuesto #4.
3. **Implementación de `devtrail stage close` con firma cosign + escenario de tampering** en uno de los repos existentes. Cierra la parte criptográfica del supuesto #6.
4. **Revisión por un auditor ISO 42001 humano real** de los outputs de Sentinel. Cierra el supuesto #3 frente a un consumidor real, no solo frente a la estructura compatible.

**Decisiones que la tesis valida y se pueden tomar ahora:**

- Cristalizar Charters como entidad de primera clase del CLI con schema `v0.json` marcado experimental.
- Portar `TEMPLATE.md v3` y `check-plan-drift.sh` como artefactos canónicos del framework (renombrados en destino a `charter-template.md` y `devtrail charter drift` — ver `devtrail-cli-roadmap.md` §6).
- Mantener la jerarquía de personas (ingeniero senior primario) sin sesgo hacia compliance officer.
- Posicionar el producto sobre "disciplina cognitiva" en marketing y docs, no sobre "compliance tool".

**Decisiones que la tesis NO permite tomar todavía:**

- Diseñar features de aprobación condicional (Cloud) sin un proyecto multi-actor real que valide el supuesto #4.
- Comprometer un schema `v1.0` estable de Charters sin evidencia de un segundo dominio.
- Marketing de "AI BOM agregado" sin haber probado la firma criptográfica en producción.

Las acciones del primer grupo se reflejan en `devtrail-cli-roadmap.md` v0.1 como Fase 1. Las del segundo grupo quedan explícitamente bloqueadas hasta evidencia adicional, manteniendo el principio #12 (velocidad = velocidad del aprendizaje).

---

*Este documento es la entrada principal para profundización empírica sobre la tesis de DevTrail. Las decisiones de producto que se tomen en `devtrail-cloud-proposal.md` v0.3 y `devtrail-cli-roadmap.md` v0.1 referencian sus secciones específicas como ancla de evidencia.*
