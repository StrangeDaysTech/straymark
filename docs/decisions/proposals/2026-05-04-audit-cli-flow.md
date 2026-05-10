# StrayMark — Audit v1: flujo CLI-driven con disciplina de tool use y zero copy/paste

**Versión:** 0.2 (cierra design phase tras revisión con material concreto de Sentinel — D11/D12/D13/D14 incorporadas, lift completo del prompt mature pre-StrayMark de Sentinel, wording de zero-friction afinado)
**Fecha:** 4 de mayo de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Articular el rediseño integral del flujo de auditoría externa de Charters tras el primer encuentro empírico con un Charter L multi-commit en Sentinel (CHARTER-07, CommsHub Etapa 2). El rediseño aborda integralmente los 3 hallazgos del [issue #102](https://github.com/StrangeDaysTech/straymark/issues/102) más un cuarto eje que el operador surfaceó después: la eliminación total del copy/paste entre IDE principal y CLIs auditoras vía paths canónicos en el repo del adoptante. Una sola iteración de upstream que devuelve a Sentinel un flujo audit substantivo a tiempo para su Etapa 2 — la evidencia que aporta justifica la inversión.
**Documentos relacionados:** `straymark-audit-skills.md` v0.2 (diseño v0 que esta propuesta extiende), `straymark-audit-skills-implementacion.md` v0.2 (plan que se cierra parcialmente con esta iteración), `straymark-design-principles.md` v0.2 (especialmente A1 orchestration-only que esta propuesta preserva en espíritu), `straymark-telemetria-etapa2-sentinel.md` v0.1 (flujo telemetría que se ajusta a la nueva convención de paths), [Issue #102](https://github.com/StrangeDaysTech/straymark/issues/102) (reporte original de Sentinel), `Propuesta/sentinel-skill-prompt-audit.md` y `Propuesta/sentinel-skill-prompt-audit-review.md` (skills mature pre-StrayMark de Sentinel — base directa de la plantilla unificada y del review consolidado v1).

---

## 1. Contexto y problema

### 1.1 Lo que sucedió en CHARTER-07

El 4 de mayo de 2026, Sentinel cerró el primer audit cycle del flujo `audit-skills` shippeado en `fw-4.8.0` / `cli-3.9.0`. CHARTER-07 (CommsHub Etapa 2 foundation: 8 commits, ~4150 líneas, 7 migraciones, 12 query files SQLC, 4 ports, 28 errores, 21 modelos, 16 events, PII guard, 8 OTel metrics, Wire stub) cerró el audit con `findings_total: 1` (un self-categorized false positive de auditor-primary; auditor-secondary 0 findings).

Esto **se ve limpio. No lo es.** El reporte de Sentinel (issue #102) articula con precisión por qué, y el operador tomó la decisión correcta de pausar `charter close` antes de pollutar la telemetría Phase 2 con un cycle vacuamente exitoso.

### 1.2 Los cuatro ejes a resolver

**Eje 1 — R10: resolver duplica contenido via HTML-comment placeholder expansion.**

El resolver del CLI hace `string.replace({{placeholder}}, content)` global, sin reconocer si el placeholder está dentro de `<!-- ... -->`. La plantilla `auditor-primary.md` documenta sus placeholders en un header HTML con sintaxis literal `{{placeholder}} — descripción`, y cada uno se expande con su contenido completo. Resultado: prompt de 1300 líneas en lugar de ~700, ~30k tokens duplicados, posible degradación de calidad del audit.

Categoría: bug mecánico. Severidad media. Reversibilidad alta.

**Eje 2 — R11(A): `git_range` default es `HEAD~1..HEAD`.**

CHARTER-07 tiene 8 commits en feature branch. Los auditores procesaron solo el último (un commit metadata-only de atomic update). No vieron migrations, SQLC, scaffolding, Wire, ni la PII guard test — los ~4150 líneas que constituyen la implementación real. La convergencia en "0 findings substantivos" carga vacuamente, no es evidencia de corrección.

Categoría: decisión de diseño que fue válida bajo asunción no explícita (Charters single-commit). El primer encuentro empírico con Charter L multi-commit la invalida.

**Eje 3 — R11(B): modo paste-based produce audits estructuralmente limitados.**

Aún con `git_range` correcto, los auditores en modo paste no pueden:
- Abrir archivos fuera del diff para verificar cross-references.
- Consultar `data-model.md` para validar migrations.
- Confirmar coverage de mappings (`eventTypeToPayload`).
- Inspeccionar patterns existentes (e.g., RLS de spec-001 para validar nueva policy).

Sin tool use, los auditores extienden a training data + assumptions. La pre-StrayMark methodology de Sentinel tenía un prompt iteradamente refinado contra slip-ups específicos (Gemini asumiendo corrección de clases por naming es el ejemplo canónico). Esa disciplina **no es representable en prompt paste-based**.

Categoría: extensión arquitectónica. **NO es regresión a A1** (orchestration-only): la solución no es implementar HTTP API clients. Es extender el alcance a un segundo modo (CLI auditor-side con tool use) donde el operador maneja sus propias CLIs auditoras y StrayMark solo provee prompts disciplinados.

**Eje 4 — Eliminación de copy/paste vía paths canónicos + skill auditor-side.**

Surfaceado por el operador después del issue #102 desde su experiencia operativa: aún con (B) resuelto, el flujo "operador genera prompt → operador copia → operador pega en LLM externo → operador copia respuesta → operador pega en archivo → operador regresa" es fricción independiente. El rediseño puede usar el filesystem del repo como canal estructurado: el agente principal escribe el prompt en path canónico; las CLIs auditoras leen ese path, ejecutan, y escriben los reports en otro path canónico; el agente principal lee los N reports al regresar.

Esto convierte el copy/paste en **invocación de skills**: el operador solo invoca `/straymark-audit-prompt`, `/straymark-audit-execute` (en N CLIs), y `/straymark-audit-review`. El intercambio de datos entre agentes es vía filesystem, no manual.

### 1.3 Por qué iteración integrada y no patches secuenciales

Patches secuenciales (R10 hoy, R11(A) la próxima semana, R11(B) en dos semanas, eje 4 en tres) forzarían a Sentinel a re-correr CHARTER-07 múltiples veces contra fixes parciales, gastando cycles audit reales sin generar evidencia marginal. Una iteración integrada permite:

- Sentinel re-audita CHARTER-07 **una sola vez** con el flujo completo nuevo.
- Sentinel continúa Etapa 2 desde CHARTER-08 bajo el flujo unificado, sin migrar mid-stage.
- La evidencia v1 que se acumule durante CHARTER-07..13 es coherente (un solo flujo, una telemetría, una convención).
- StrayMark evita el costo de coordinación de múltiples releases con migraciones overlapping.

El costo de la iteración integrada: ~5-7 PRs ordenados en lugar de 3-4 patches paralelos. Manejable en una sesión de implementación dedicada (similar al day de Phase 1).

---

## 2. Decisiones de diseño

### D1 — Convención de paths canónica nueva: `.straymark/audits/`

**Decisión:** los outputs del flujo audit viven bajo `.straymark/audits/<CHARTER-ID>/`. La carpeta `audit/` actual se considera convención v0 deprecada.

**Justificación:** `audit/` (singular, sin prefijo) en el root del repo es genérica y puede chocar con carpetas legítimas del adoptante (auditorías de seguridad de la app, auditorías de licencias, etc.). `.straymark/audits/` (plural, bajo dotdir StrayMark) es claramente de la herramienta y nunca colisiona porque `.straymark/` ya está reservado.

**Estructura interna:**

```
.straymark/audits/
└── CHARTER-07/
    ├── audit-prompt.md          # prompt unificado, generado por audit-prompt skill
    ├── report-claude-sonnet-4-6.md   # generado por audit-execute en CLI claude
    ├── report-gemini-2.5-pro.md      # generado por audit-execute en CLI gemini
    └── report-gpt-5.3-codex.md       # opcional, tercera auditoría
```

Naming de reports: `report-<sluggified-model-id>.md`. El skill `audit-execute` deriva el slug del model identifier que el agente conoce sobre sí mismo. Si un mismo modelo audita dos veces (raro pero posible), el segundo agrega sufijo `-2`, etc.

### D2 — Plantilla unificada: un solo `audit-prompt.md`

**Decisión:** retirar la distinción `auditor-primary.md` / `auditor-secondary.md`. Una sola plantilla `audit-prompt.md` que cualquier auditor ejecuta.

**Justificación:** en v0 la diferencia entre primary y secondary era cosmética — mismo schema de output, mismas instrucciones, mismo framing modulo tres líneas de wording. La distinción servía para que el calibrator pudiera comparar simétricamente, pero el calibrator funciona igual leyendo N reports independientes. Una sola plantilla:

- Reduce mantenimiento (una superficie de actualización en lugar de dos).
- Permite N auditorías (no fijado a 2 — el operador decide cuántas).
- Elimina la pregunta "¿cuál es el primary?" que era arbitraria.
- Simplifica el modelo mental para el adoptante.

La plantilla unificada absorbe la disciplina de tool use (D5) y el wording mature de Sentinel pre-StrayMark.

### D3 — Soporte de N auditorías, no 2

**Decisión:** `straymark-audit-review` itera todos los `report-*.md` en `.straymark/audits/<CHARTER-ID>/` en lugar de leer exactamente 2 archivos.

**Justificación:** el operador puede tener razones legítimas para querer 3 o 4 auditorías:

- Charters de alto riesgo (security-critical) merecen más cobertura cross-modelo.
- Charters con superficie especializada (ML, criptografía, infra cloud) pueden beneficiarse de un auditor de modelo especializado además de los generales.
- Adoptantes con presupuesto suficiente quieren más señal.

El minimum viable sigue siendo 2 (heterogeneidad inter-familia), pero el flujo no impone el techo. La convención de paths permite N reports naturalmente.

**Borde inferior:** si `.straymark/audits/<CHARTER-ID>/` tiene 0 o 1 reports cuando el operador invoca `audit-review`, el skill emite warning (heterogeneidad insuficiente) pero procede — la decisión final es del operador.

### D4 — Nuevo skill `straymark-audit-execute`

**Decisión:** se crea un tercer skill StrayMark (`straymark-audit-execute CHARTER-XX`) que se invoca en CLIs auditoras (gemini-cli, claude-cli, copilot-cli, codex-cli, etc.) en lugar del flujo paste manual.

**Comportamiento del skill:**

1. Recibe `CHARTER-XX` como argumento.
2. Lee `.straymark/audits/CHARTER-XX/audit-prompt.md`.
3. Sigue la disciplina del prompt: lee archivos relevantes vía tool use (Read), recolecta evidencia, produce findings con citas `path:line`.
4. Escribe `.straymark/audits/CHARTER-XX/report-<self-model-slug>.md` con frontmatter conforme a `audit-output.schema.v0.json`.
5. Imprime al operador la ruta del reporte + instrucción de regresar al agente principal y ejecutar `/straymark-audit-review CHARTER-XX` cuando todas las auditorías estén completas.

**Distribución (igual que los demás skills):**

- `dist/.claude/skills/straymark-audit-execute/SKILL.md` (allowed-tools: Read, Write, Bash, Glob, Grep)
- `dist/.gemini/skills/straymark-audit-execute/SKILL.md`
- `dist/.agent/workflows/straymark-audit-execute.md`

**Crítico:** este skill **no invoca APIs externos**. Corre dentro de la CLI auditora que el operador ya está usando con sus propias credenciales. StrayMark solo provee la skill que dice "lee este archivo, audit con tool use, escribe este otro archivo".

### D5 — Disciplina de tool use enforced por el prompt

**Decisión:** la plantilla unificada `audit-prompt.md` incluye una sección de **Disciplina de evidencia** que enuncia explícitamente:

> *"Solo puedes opinar sobre archivos que has abierto vía tool call (`Read`, `Grep`, etc.). Cualquier finding que produzcas debe citar `path:line` de los archivos específicos que abriste. Findings sin citas se consideran inválidas y se rechazan en review. Si no abriste un archivo, no puedes inferir comportamiento, estructura, ni corrección sobre él."*

**En modo CLI** (`audit-execute` en gemini-cli/claude-cli/etc.): el agente cumple la disciplina literalmente. El skill `audit-review` valida que cada finding incluye al menos una cita `path:line`; findings sin cita se descartan o se marcan como `low-confidence`.

**En modo paste** (operador pega manualmente en LLM de chat): la disciplina opera como guía aspiracional. Si el modelo no tiene tool use, no puede cumplirla literalmente; produce findings sin citas. El skill `audit-review` los procesa pero los marca con menor peso. Esto degrada gracefully sin romper.

El prompt mature de Sentinel pre-StrayMark (que el operador ofreció contribuir) es la base. Se incorpora con atribución.

### D6 — `git_range` default cambia a `origin/main..HEAD` con fallback

**Decisión:**

- Si la rama actual tiene `origin/main` reachable → default `origin/main..HEAD`.
- Si no → fallback a `HEAD~1..HEAD` con warning explícito.
- Flag `--range` sigue intacto para override.

**Justificación:** R11(A) del issue #102. El comportamiento previo era válido bajo asunción de Charters single-commit; el caso real es Charters multi-commit en feature branches.

**Consideración de breaking-ness:** estrictamente no breaking (output sigue siendo válido), pero scripts que dependan del default cambian. CHANGELOG documenta explícitamente la rationale. Bump a `cli-3.10.0` (minor, no patch).

### D7 — R10 fix: resolver respeta bounds de `<!-- ... -->`

**Decisión:** opción 1 de las dos que Sentinel propuso — el resolver detecta bloques `<!-- ... -->` y skipea reemplazo de placeholders dentro de ellos.

**Justificación:** opción 1 es semánticamente correcta (los HTML comments son no-content por design en markdown). Opción 2 (detectar pattern indentado + ` — descripción`) acopla resolver con convención de documentación de plantillas, que es frágil ante cambios de formato.

**Implementación:** una pasada simple sobre el template antes del replace global, identificando ranges `<!-- ... -->` y excluyendo placeholders dentro. Test de regresión: template con `<!-- {{x}} foo --><body>{{x}}</body>` produce un único replace.

### D8 — CLI subcommand simplificado

**Decisión:** el subcommand `straymark charter audit` se simplifica:

**v0:** `straymark charter audit <id> [--calibrate | --finalize] [--merge-into <path>]` (3 pasos: prepare, calibrate, finalize).

**v1:** `straymark charter audit <id> [--prepare | --merge-reports] [--merge-into <path>]`.

- **`--prepare`** (default): genera `audit-prompt.md` resuelto en `.straymark/audits/<id>/`. Sustituye el "PREPARE" de v0.
- **`--merge-reports`**: lee N `report-*.md` desde `.straymark/audits/<id>/`, valida cada uno contra schema, produce el bloque YAML `external_audit:` array. Combinable con `--merge-into <path>` para auto-mergear en telemetry. Sustituye el "FINALIZE" de v0.
- **No hay paso `--calibrate` separado**: la "calibración" la hace el agente principal vía el skill `straymark-audit-review` que lee los reports + razona inline + invoca `--merge-reports --merge-into <path>` al final.

**Justificación:** el paso `--calibrate` v0 era un pre-procesamiento de calibrator-reconciler.prompt.md que en práctica solo funcionaba si el calibrator también corría en CLI con filesystem access (que el agente principal del IDE sí tiene). En v1, el agente principal **es** el calibrator implícito; no necesita una plantilla separada para reconciliar — itera los reports directamente y produce el análisis inline. Eso elimina un archivo redundante (`calibrator-reconciler.prompt.md`) y un paso CLI redundante.

### D9 — Schema `audit-output.schema.v0.json` ajustado

**Decisión:** schema se mantiene en `v0` (experimental), con dos cambios:

- `audit_role` deja de ser enum `auditor-primary | auditor-secondary | calibrator-reconciler`. Pasa a ser `auditor` (singular) — un solo rol porque ya no distinguimos primary/secondary, y el reconciler ahora es el agente principal in-conversation que no escribe report propio.
- Frontmatter de un report incluye campo opcional `evidence_citations: <int>` — cuántos `path:line` cita el body. El skill `audit-review` lo usa para weighting.

**Cambios al schema viejo:** SI breaking dentro del experimental `v0`. Pero los reports v0 viejos (en `audit/charters/CHARTER-XX/`) no son leídos por v1 (paths distintos), así que no hay path de migración. Los outputs v0 quedan como evidencia histórica del cycle paused.

### D10 — Sin modo `--auditor-mode cli|paste` explícito

**Decisión:** no hay flag CLI para distinguir modo CLI vs paste. El sistema es agnóstico de cómo el operador obtuvo cada `report-*.md`.

**Justificación:** el skill `audit-review` no necesita saber si un report fue generado por `audit-execute` en CLI con tool use, o si el operador lo escribió manualmente después de un chat paste. Lee el archivo, valida el schema, procesa. La diferencia se refleja naturalmente en los `evidence_citations` (alto en CLI, bajo o cero en paste). El operador es libre de mezclar modos.

Esto simplifica enormemente: una sola convención de paths, una sola plantilla, un solo flujo de review. El modo es problema del operador.

### D11 — El review v1 produce un análisis consolidado, no solo YAML

**Decisión:** el skill `straymark-audit-review` v1 evoluciona de "validate + merge YAML" a "consolidated analysis generator". Produce `.straymark/audits/<CHARTER-ID>/review.md` con la estructura completa observada en el skill `audit-review` mature de Sentinel pre-StrayMark (`Propuesta/sentinel-skill-prompt-audit-review.md`):

1. **Resumen ejecutivo** (2-3 párrafos: hallazgos clave, scope confusion si aplica, bug más crítico).
2. **Definición del alcance correcto** (tabla con tasks del Charter, checkpoint, qué está IN vs OUT de scope).
3. **Evaluación por auditoría** — tabla por auditor con findings, severidad reportada, veredicto y justificación.
4. **Plan de remediación** — priorizado P0 (Seguridad) / P1 (Integridad) / P2 (Consistencia) / P3 (Robustez) / P4 (Documentación) con archivo:línea, problema, remediación, complejidad, quién lo detectó.
5. **Hallazgos descartados** (misatribuciones + falsos positivos) — tabla con hallazgo, tipo, etapa real si aplica, auditor que lo reportó.
6. **Calificación de auditores** — score 1-10 sobre 4 criterios con weights:
   - Scope precision: 25%
   - Technical depth: 25%
   - Bug detection: 30%
   - False positive rate: 20%
7. **Conclusión** (estado del Charter, hallazgos críticos, próximo paso).

El veredicto por finding sigue el vocabulario del Sentinel skill: **VALID / PARTIALLY VALID (con reclassification) / MISATTRIBUTED / FALSE POSITIVE / DUPLICATE**.

**Adicional al review.md:** el `external_audit:` array YAML se mergea en la telemetry como en v0 (vía `--merge-reports --merge-into`). Las dos cosas coexisten: el review consolidado es para el operador (legible, accionable), el YAML es para la telemetry agregada (machine-readable, alimenta Phase 2 stats).

**Justificación:** R11(B) del issue #102 articulaba "audits de structurally limited substantive reach". Un YAML mergeado no resuelve eso aunque tenga citaciones. Un análisis consolidado sí — es la diferencia entre "se generó la telemetría" y "el operador tiene un documento usable para remediation". El skill `audit-review` mature de Sentinel ya validó empíricamente que el formato consolidado es el delivery útil al humano; lift directo en lugar de re-derivar.

### D12 — La plantilla unificada `audit-prompt.md` se basa directamente en el Sentinel skill, parametrizada

**Decisión:** la plantilla unificada se construye lifting el contenido del skill `audit/SKILL.md` mature de Sentinel pre-StrayMark (`Propuesta/sentinel-skill-prompt-audit.md`). Se preservan integralmente las 7 secciones que son universales y portables:

1. **§REGLA ABSOLUTA — SOLO LECTURA** (Sentinel líneas 55-77): disciplina enunciada operacionalmente — *"Si encuentras un bug, DOCUMENTALO en tu reporte. NO lo corrijas. Si encuentras un archivo faltante, REPORTALO. NO lo crees. Violación de esta regla invalida toda la auditoría."* Es la solución textual a R11(B) y reemplaza directamente el placeholder de "tool-use enforcement" propuesto en D5.
2. **§Tu rol** ("No eres cheerleader") — anti-pattern del modelo complaciente.
3. **§Reglas de alcance** — out-of-scope handling como sección separada.
4. **§Paso 2 — Verificar cada tarea (OBLIGATORIO)** — chain de verificación con tool use: localizar → leer COMPLETO → trazar (handler → service → repo → SQL) → leer ≥2 test cases → comparar contra tarea con cita `archivo:línea`.
5. **§Paso 5 — Calibrar severidad contra config REAL** — anti-inflation y anti-deflation con checklist obligatorio (driver activo, feature flags, build tags, rol DB, deployment scope) antes de declarar Critical/High.
6. **§Lo que NO debes hacer** — closing reinforcement.
7. **§Formato de salida** — estructura del report markdown (Alcance, Verificación de compilación/tests, Trazabilidad tarea por tarea, Hallazgos por severidad, Notas fuera de alcance, Evaluación de checkpoint, Conclusión).

**Lo que se parametriza** (los hardcodes Sentinel se convierten en placeholders del resolver del CLI):

| Sentinel hardcode | Placeholder StrayMark v1 |
|---|---|
| `specs/001-sentinel-mvp/tasks.md` | `{{charter_path}}` + `{{charter_body}}` |
| `## Etapa N:` heading | `{{charter_id}}` + `{{charter_title}}` |
| Lista de tareas extraída de `tasks.md` | `{{tasks_list}}` (extraído de `## Tasks` del Charter) |
| Checkpoint textual | `{{charter_trigger}}` o sección equivalente |
| `audit/etapas-implementacion/{N}/[tu-nombre]-audit.md` | `.straymark/audits/{{charter_id}}/report-<self-model-slug>.md` |
| Project description "Sentinel es un monolito modular en Go..." | `{{project_context}}` (derivado de README/CLAUDE.md o config opcional) |
| `internal/modules/identity/`, `middleware/auth*` | `{{security_paths_hint}}` (derivado de Charter risk_level + tags) |
| `internal/core/eventbus/factory.go`, etc. | `{{factory_hints}}` (placeholder libre, opcional) |
| `go vet ./...`, `go build`, `go test` | `{{build_commands}}` (derivado de project type) |
| `originating_ailogs[]` (no presente en Sentinel) | `{{originating_ailogs}}` — se inyecta el contenido de los AILOGs origen para context |
| `{{git_diff}}` | `{{git_diff}}` (output de git diff sobre el `--range` resuelto) |

**Lo que se preserva como ilustración didáctica:** los ejemplos concretos del skill Sentinel (la anécdota de Etapa 12 Pub/Sub stub vs gochannel) se mantienen en la plantilla **rotulados como "Ejemplo (de un caso real en proyecto adoptante)"**. Son material didáctico para que el auditor entienda el patrón anti-inflation aún sin trabajar en Sentinel.

**Crédito:** la plantilla incluye una nota al pie: *"Esta plantilla se basa en el skill `audit/SKILL.md` mature pre-StrayMark de Sentinel, contribuido vía issue #102. Las 7 secciones universales se preservan integralmente; los hardcodes específicos a Sentinel se parametrizaron. Crédito a José Villaseñor Montfort (operador Sentinel/StrayMark) por el material base."*

### D13 — `.straymark/audits/` namespaced para extensibilidad futura, sin nest bajo `charters/`

**Decisión:** la convención de paths queda `.straymark/audits/<UNIT-TYPE>-<UNIT-ID>/...`. La primera (y única en v1) categoría es `CHARTER-<NN>`, pero la estructura **no se anida bajo `.straymark/charters/`** porque eso cerraría la puerta a auditar otras categorías de unidades en el futuro.

**Estructura conceptual:**

```
.straymark/audits/
├── CHARTER-07/         # auditoría de un Charter (v1, único caso soportado)
├── CHARTER-08/
└── (futuro)
    ├── MODULE-payments/    # auditoría de un módulo completo
    ├── RELEASE-v2.0/       # auditoría pre-release
    └── EPIC-multi-tenancy/ # auditoría de un epic transversal
```

**Justificación (rationale del operador en sesión de revisión):** los archivos audit ya tienen el ID del unit en el nombre del directorio, así que anidar bajo `charters/` es redundante. Más importante, **el adoptante puede extender el framework para auditar unidades más amplias o estrechas que un Charter** sin que la convención de paths requiera reestructuración. Esto preserva los principios #5 (schema-driven antes que feature-driven) y #12 (espera evidencia antes de cristalizar) — la primera y única categoría hoy es Charter; futuras categorías se exploran cuando un caso operativo concreto las demande.

**Lo que NO se cristaliza ahora:** el shape exacto de las futuras categorías (qué metadata necesita un MODULE audit, cómo se invoca, etc.) queda fuera de v1. El namespacing solo deja la puerta abierta.

### D14 — Discovery automático y detección de slug en `straymark-audit-execute`

**Decisión:** dos enhancements de zero-friction al skill `straymark-audit-execute`:

**(a) Argumento opcional con discovery automático.** Si el operador invoca `/straymark-audit-execute` sin argumento (porque abre la CLI auditora y no recuerda qué CHARTER auditar), el skill busca en `.straymark/audits/*/audit-prompt.md` los prompts existentes que **aún no tienen** un `report-<self-model-slug>.md` (ya hecho por este modelo). Tres casos:

- **Exactamente uno encontrado** → procede automáticamente.
- **Múltiples encontrados** → lista al operador con CHARTER IDs + títulos, pide selección numérica.
- **Ninguno encontrado** → mensaje: "No hay audit prompts pendientes para este modelo. Verifica que el operador haya ejecutado `/straymark-audit-prompt` en el agente principal primero."

**(b) Detección automática del model slug.** El skill identifica su propio `model-id` desde el contexto runtime de la CLI auditora (ej. `claude-sonnet-4-6`, `gemini-2.5-pro`, `gpt-5.3-codex`). Construye el slug del filename automáticamente. Si por alguna razón runtime no expone el model-id, el skill pide al operador confirmar — pero el caso default es automático.

**Justificación:** el operador puede abrir 3 CLIs auditoras simultáneamente (gemini-cli en una terminal, claude-cli en otra, copilot-cli en tercera). Recordar exactamente qué CHARTER ID escribir en cada una es fricción innecesaria. El discovery automático elimina esa fricción manteniendo la decisión consciente del operador (sigue invocando el skill explícitamente). La detección de slug elimina la fricción de "¿cómo escribo el filename?".

**Importante:** el discovery automático **no** invoca el skill por sí solo — el operador sigue invocándolo explícitamente. Solo simplifica el "qué argumento pasar".

---

## 3. Arquitectura nueva

### 3.1 Flujo end-to-end

```
[Repo del adoptante]

Operador humano:
    1. Crea Charter via `straymark charter new ...`
    2. Implementa con agente IA principal (multi-task, multi-commit)

Agente IA principal:
    3. Alcanza checkpoint de audit (AGENT-RULES.md §12)
    4. Pregunta SÍ/NO con recomendación

[Si NO]
    5a. Charter cierra normalmente via `straymark charter close`

[Si SÍ]
Operador humano:
    5. Invoca `/straymark-audit-prompt CHARTER-XX` en agente principal

Agente principal (Claude Code, etc.):
    6. Ejecuta `straymark charter audit CHARTER-XX --prepare`
    7. CLI escribe `.straymark/audits/CHARTER-XX/audit-prompt.md`
    8. Skill avisa al operador (sin requerir paths del operador):
       "Prompt generado. Abre las CLIs auditoras que decidas usar
       (recomendación: 2 modelos de familias distintas) y en cada
       una invoca /straymark-audit-execute CHARTER-XX. Cuando TODAS
       las auditorías que encargues hayan terminado (no antes),
       regresa aquí y ejecuta /straymark-audit-review CHARTER-XX."

Operador humano (en N CLIs auditoras, secuencial o paralelo):
    9. Para cada CLI (gemini-cli, claude-cli, copilot-cli, ...):
        - cd al repo del adoptante (los skills ya están instalados
          localmente vía straymark init, no requiere setup extra)
        - Invoca `/straymark-audit-execute CHARTER-XX`
          (o sin argumento, el skill descubre el CHARTER pendiente)

Cada agente auditor (en su propia CLI):
    10. Lee .straymark/audits/CHARTER-XX/audit-prompt.md
        (sin que el operador le pase el path — el skill lo construye
        desde el CHARTER-ID o lo descubre automáticamente)
    11. Audit con tool use: lee archivos relevantes, cita path:line
    12. Escribe .straymark/audits/CHARTER-XX/report-<self-model-slug>.md
        (slug detectado automáticamente desde el runtime)
    13. Skill avisa al operador:
        "Audit completo (CHARTER-XX, este modelo).
         Report en .straymark/audits/CHARTER-XX/report-<slug>.md.

         IMPORTANTE: solo regresa a /straymark-audit-review en tu
         agente principal cuando TODAS las auditorías que decidiste
         encargar hayan terminado. Si tienes otras CLIs auditoras
         corriendo (gemini-cli, copilot-cli, etc.), espera a que
         terminen antes de invocar review. Si invocas review con
         reports incompletos, el análisis consolidado quedará
         parcial y tendrás que descartarlo o re-correrlo."

Operador humano (cuando TODAS las auditorías encargadas terminaron):
    14. Regresa al agente principal
    15. Invoca `/straymark-audit-review CHARTER-XX`

Agente principal:
    16. Itera `.straymark/audits/CHARTER-XX/report-*.md` (N archivos)
    17. Valida cada uno contra schema (audit-output.schema.v0.json)
    18. Por cada finding del master list: lanza Explore agents en
        paralelo (hasta 3 a la vez) para verificar contra el código,
        clasifica veredicto (VALID/PARTIALLY VALID/MISATTRIBUTED/
        FALSE POSITIVE/DUPLICATE), reclasifica severidad si aplica
        (anti-inflation, anti-deflation), busca findings que los
        auditores hayan missed
    19. Produce .straymark/audits/CHARTER-XX/review.md consolidado
        con 6 secciones: Resumen ejecutivo, Definición de alcance,
        Evaluación por auditoría, Plan de remediación P0-P4,
        Hallazgos descartados, Calificación de auditores
    20. Ejecuta `straymark charter audit CHARTER-XX --merge-reports
        --merge-into .straymark/charters/CHARTER-XX.telemetry.yaml`
    21. external_audit array mergeado en telemetry, audit-telemetry
        JSONL log escrito
    22. Skill avisa al operador:
        "Review consolidado en .straymark/audits/CHARTER-XX/review.md
         (incluye remediation plan P0-P4 con N items, calificación
         de auditores). YAML mergeado en telemetry.
         Sugerido: git diff para revisar antes de close."

Operador humano:
    23. Revisa review.md (lectura humana, principal delivery)
    24. Revisa via git diff los cambios al telemetry
    25. (Opcional) Aplica remediation plan items P0/P1 si los hay
    26. Cierra Charter con `straymark charter close CHARTER-XX`
```

**Cero copy/paste manual** entre operador y agentes. **Cero paths que el operador tenga que escribir.** Solo 3 invocaciones de skills con CHARTER-ID (o ninguno, vía discovery), más decisiones de autoría humana (sí/no auditar, qué CLIs usar, cuándo todas terminaron). El intercambio de archivos entre agentes es vía filesystem del repo.

**Cero copy/paste manual entre operador y agentes.** El operador solo invoca skills.

### 3.2 Cambios concretos por componente

**Archivos del framework (`dist/`):**

- `dist/.straymark/audit-prompts/audit-prompt.md` — NEW. Plantilla unificada **basada directamente en `Propuesta/sentinel-skill-prompt-audit.md`** con las 7 secciones universales (REGLA ABSOLUTA, Tu rol, Reglas de alcance, Paso 2 verificación obligatoria, Paso 5 calibración severidad, Lo que NO debes hacer, Formato de salida). Hardcodes Sentinel parametrizados según D12. Ejemplos didácticos (Etapa 12 Pub/Sub stub) preservados con rótulo "Ejemplo de caso real". Crédito explícito a Sentinel al pie.
- `dist/.straymark/audit-prompts/auditor-primary.md` — DELETE.
- `dist/.straymark/audit-prompts/auditor-secondary.md` — DELETE.
- `dist/.straymark/audit-prompts/calibrator-reconciler.md` — DELETE (el rol calibrator ahora lo cumple el agente principal vía el skill `straymark-audit-review`, no requiere plantilla separada).
- `dist/.straymark/schemas/audit-output.schema.v0.json` — UPDATE: `audit_role: auditor` simple (eliminar enum auditor-primary/secondary/calibrator-reconciler), agregar `evidence_citations: <int>` opcional para weighting.
- `dist/.claude/skills/straymark-audit-prompt/SKILL.md` — UPDATE: nueva ruta canónica `.straymark/audits/<id>/audit-prompt.md`, wording de "espera a TODAS las auditorías" en next-steps guidance.
- `dist/.gemini/skills/straymark-audit-prompt/SKILL.md` — UPDATE (mismo).
- `dist/.agent/workflows/straymark-audit-prompt.md` — UPDATE (mismo).
- `dist/.claude/skills/straymark-audit-execute/SKILL.md` — NEW. Body incluye: lectura de `.straymark/audits/<id>/audit-prompt.md`, audit con tool use (Read/Grep/Bash test commands), escritura de `report-<self-model-slug>.md`. Implementa D14 (discovery automático cuando argumento omitido + detección automática de model slug). Aviso final al operador con énfasis "espera a TODAS las auditorías encargadas antes de invocar review".
- `dist/.gemini/skills/straymark-audit-execute/SKILL.md` — NEW (mismo).
- `dist/.agent/workflows/straymark-audit-execute.md` — NEW (mismo, sin allowed-tools).
- `dist/.claude/skills/straymark-audit-review/SKILL.md` — UPDATE: **evoluciona de "validate + merge YAML" a "consolidated analysis generator"** según D11. Body incluye: iteración de N reports, verificación finding por finding contra código vía Explore agents (paralelización hasta 3 a la vez), clasificación de veredictos (VALID/PARTIALLY VALID/MISATTRIBUTED/FALSE POSITIVE/DUPLICATE), severity calibration (anti-inflation/anti-deflation), búsqueda de findings missed, generación de `review.md` con 6 secciones, calificación de auditores 1-10 sobre 4 criterios, merge YAML adicional vía `--merge-reports --merge-into`.
- `dist/.gemini/skills/straymark-audit-review/SKILL.md` — UPDATE (mismo).
- `dist/.agent/workflows/straymark-audit-review.md` — UPDATE (mismo).
- `dist/.straymark/00-governance/AGENT-RULES.md` (3 langs) — UPDATE §12: wording del checkpoint mencionando el flujo nuevo de 3 skills + paths canónicos `.straymark/audits/`. Nota explícita de "espera a TODAS las auditorías encargadas antes de invocar audit-review".

**Archivos del CLI (`cli/`):**

- `cli/src/commands/charter/audit.rs` — UPDATE:
  - Nueva ruta canónica `.straymark/audits/<id>/`.
  - Subcommand simplificado: `--prepare` (default) y `--merge-reports`. Eliminar `--calibrate` y `--finalize` (con shim de deprecation que dirige al nuevo flag — ver §4 migración).
  - `git_range` default cambia con fallback inteligente.
  - Resolver respeta `<!-- ... -->` (R10 fix).
  - `--merge-reports` itera N archivos en lugar de leer 2 fijos.
- `cli/src/main.rs` — UPDATE clap subcommand.
- `cli/tests/charter_audit_test.rs` — UPDATE tests existentes (ahora bajo `.straymark/audits/`) + 4 tests nuevos:
  - `audit_prepare_writes_unified_prompt_to_canonical_location`
  - `audit_merge_reports_iterates_n_reports`
  - `audit_resolver_respects_html_comment_bounds`
  - `audit_git_range_falls_back_when_no_origin_main`

**Documentación adopter (`docs/`):**

- `docs/adopters/CLI-REFERENCE.md` (3 langs) — UPDATE §`straymark charter audit` (subcommand simplificado, paths nuevos), §Skills (skill nuevo `audit-execute`, audit-review evolucionado).
- `docs/adopters/WORKFLOWS.md` (3 langs) — UPDATE Charter audit checkpoint con flujo nuevo.
- `docs/adopters/ADOPTION-GUIDE.md` (3 langs) — UPDATE §External Audit (Optional) con guía de cómo configurar CLIs auditoras read-only.
- `dist/.straymark/00-governance/QUICK-REFERENCE.md` (3 langs) — UPDATE tabla skills (3 audit skills ahora).

---

## 4. Migración desde v0 (estrategia para Sentinel y futuros adoptantes)

### 4.1 Convención de paths

- v0 usaba `audit/charters/<id>/auditor-{primary,secondary,reconciler}.md`.
- v1 usa `.straymark/audits/<id>/audit-prompt.md` + `report-*.md`.

**Para Sentinel (CHARTER-07 paused):** los outputs v0 quedan en `audit/charters/CHARTER-07/` como evidencia histórica. Sentinel re-corre el cycle bajo v1 (`straymark charter audit CHARTER-07 --prepare`), genera nuevos archivos en `.straymark/audits/CHARTER-07/`, ejecuta `audit-execute` en sus CLIs auditoras, y cierra. La carpeta `audit/` queda en el repo (no se borra) — útil para postmortem comparativo entre v0 y v1.

**Para futuros adoptantes:** desde `fw-4.9.0` en adelante, todos los Charters usan la convención nueva. No hay legacy.

### 4.2 Subcommand CLI

- `straymark charter audit <id> --calibrate` → deprecated; emite warning con instrucción de usar el flujo nuevo via skill `straymark-audit-review`.
- `straymark charter audit <id> --finalize` → deprecated; warning + redirect a `--merge-reports`.
- `straymark charter audit <id> --finalize --merge-into <path>` → deprecated; el comportamiento equivalente es `--merge-reports --merge-into <path>` pero leyendo de la nueva ruta.

Los flags deprecated se mantienen en `cli-3.10.0` con warning. Se eliminan en `cli-3.11.0` o cuando todos los adoptantes hayan migrado (lo que llegue antes).

### 4.3 Schema audit-output

- Reports v0 (con `audit_role: auditor-primary | auditor-secondary | calibrator-reconciler`) NO son leídos por el sistema v1.
- Reports v1 usan `audit_role: auditor` simple.
- No hay shim de migración automática de reports — los v0 se quedan como están en `audit/charters/`, los v1 se generan limpios en `.straymark/audits/`.

### 4.4 Charter telemetry

- El campo `external_audit:` array en `charter-telemetry.schema.v0.json` no cambia shape — sigue siendo array de objetos auditor con findings_total, findings_by_category, audit_quality, audit_notes.
- El skill `audit-review` v1 produce el mismo shape; solo cambia la fuente (lee N `report-*.md` en lugar de 2 archivos fijos).
- Telemetrías de Charters cerrados antes de `fw-4.9.0` siguen siendo válidas y leíbles por el sistema.

---

## 5. Plan de implementación

### 5.1 Secuencia de PRs

| # | Título | Critical path | Estimación |
|---|---|---|---|
| 1 | R10 fix: resolver respeta `<!-- ... -->` bounds + tests de regresión | sí | 2-3 h |
| 2 | `git_range` default change con fallback + tests + CHANGELOG note | sí | 2-3 h |
| 3 | Plantilla unificada `audit-prompt.md` (basada en prompt Sentinel pre-StrayMark) + delete primary/secondary/reconciler templates + schema audit-output ajustado | sí | 4-5 h |
| 4 | CLI subcommand simplificado: `--prepare` / `--merge-reports`, paths canónicos `.straymark/audits/`, deprecation shims de `--calibrate` / `--finalize` con warning | sí | 5-6 h |
| 5 | Skill nueva `straymark-audit-execute` × 3 plataformas + tests fixture | sí | 3-4 h |
| 6 | Skill `straymark-audit-prompt` y `straymark-audit-review` actualizadas × 3 plataformas + tests fixture | sí | 3-4 h |
| 7 | Documentación adopter (CLI-REFERENCE, WORKFLOWS, ADOPTION-GUIDE, QUICK-REFERENCE × 3 langs) + AGENT-RULES §12 | parcial | 4-5 h |
| 8 | Bump `fw-4.9.0` / `cli-3.10.0` + CHANGELOG combined section + tag release | sí | 1-2 h |

**Total estimado:** 28-36 horas focused (revisado al alza tras incorporar el lift completo de los Sentinel skills, el review consolidado de D11, y los enhancements de D14). Calendarizable como una sesión densa similar al day de Phase 1, o distribuible en 2-3 días con verificación intermedia.

**Críticos secuenciales:** PR 1 → PR 2 → PR 3 → PR 4 → (PR 5 || PR 6) → PR 7 → PR 8.

PR 5 y PR 6 pueden ir en paralelo desde sus ramas tras PR 4.

### 5.2 Estrategia de tests

Reusar el patrón de Phase 1: tests fixture sobre archivos de skills (existencia, frontmatter shape, paridad cross-language) + integration tests sobre el CLI que ejercitan el flujo end-to-end con fixtures.

Tests de integración nuevos clave:

- `audit_prepare_writes_unified_prompt_to_dot_straymark_audits_path` — verifica que `--prepare` escribe a la ubicación nueva.
- `audit_merge_reports_handles_n_reports` — verifica con 2, 3, y 4 reports en el directorio.
- `audit_merge_reports_warns_on_single_report` — verifica que `audit-review` emite warning si solo encuentra 1 report (heterogeneidad insuficiente).
- `audit_resolver_skips_placeholders_inside_html_comments` — R10 regression test.
- `audit_default_range_uses_origin_main_when_available` — R11(A) test.
- `audit_default_range_falls_back_to_head_minus_one_when_no_origin_main` — R11(A) fallback test.
- `audit_deprecated_calibrate_flag_emits_warning_and_continues` — backwards compat shim.

### 5.3 Verificación end-to-end

Antes de bumpear y release:

1. Sandbox local: `cargo install --path cli/`, `straymark init /tmp/sandbox-v1`, crear Charter ficticio multi-commit, ejercitar el flujo completo con fixtures.
2. Cross-language paridad de skills y docs.
3. Backwards compat: invocar `straymark charter audit <id> --calibrate` y verificar warning + redirect.
4. Schema validation: report con `audit_role: auditor-primary` (v0) es rechazado por validator v1 con error claro.

---

## 6. Versionado y release

**Bump:**

- `cli-3.10.0` (minor) — nuevos flags, paths canónicos cambiados, R10 + R11(A) fixes incluidos. Backwards compat shims con deprecation warnings.
- `fw-4.9.0` (minor) — nuevo skill, plantilla unificada, deletion de plantillas v0, schema audit-output ajustado.

**CHANGELOG combined section:**

`## Framework 4.9.0 / CLI 3.10.0 — Audit v1: CLI-driven flow with tool-use discipline`

Sección detallada cubriendo:
- Added (Framework): nuevo skill `straymark-audit-execute`, plantilla unificada.
- Added (CLI): `--prepare`, `--merge-reports`, soporte para N reports.
- Changed (CLI): `git_range` default `origin/main..HEAD`, paths canónicos a `.straymark/audits/`.
- Fixed (CLI): R10 resolver bug.
- Deprecated (CLI): `--calibrate`, `--finalize` (con warning + redirect).
- Removed (Framework): plantillas `auditor-primary.md`, `auditor-secondary.md`, `calibrator-reconciler.md`.
- BREAKING (Framework): convención de paths cambia (`audit/` → `.straymark/audits/`). Audits v0 paused (Sentinel CHARTER-07) deben re-correrse bajo v1.
- Crédito explícito a José Villaseñor Montfort (operador Sentinel/StrayMark) por los skills `audit/SKILL.md` y `audit-review/SKILL.md` mature pre-StrayMark contribuidos vía issue #102. Las 7 secciones universales de `audit-prompt.md` y la estructura de 6 secciones del review consolidado v1 se basan integralmente en ese material; los hardcodes específicos a Sentinel se parametrizaron contra Charter doc + originating AILOGs + git range. Ejemplos didácticos (Etapa 12 Pub/Sub stub) preservados con rótulo de caso real.

**Tags y release workflows:** `fw-4.9.0` y `cli-3.10.0` push, los workflows de release-cli.yml + release-framework.yml ejecutan, binarios cross-platform y ZIP framework publican.

---

## 7. Lo que NO está en alcance v1

Para preservar foco y no inflar la iteración:

- **No HTTP API clients propios.** Decisión A1 sigue. StrayMark no maneja API keys, no invoca APIs, no mantiene HTTP clients. La CLI auditora del operador hace todo eso.
- **No `--strict-citations` enforcement.** Findings sin citas se marcan `low-confidence` pero no se rechazan automáticamente. Validar la utilidad de la citación es trabajo de Phase 2 telemetría.
- **No detección automática de "qué CLI auditora tiene el operador".** El operador instala lo que quiere; el skill se invoca y funciona si está disponible.
- **No multi-Charter audit batch** (`straymark charter audit CHARTER-07,CHARTER-08`). Si el patrón emerge, propuesta separada.
- **No re-audit support** (rerunear un cycle audit cuando ya existe `external_audit:` en telemetry). El v0 también lo rechaza; v1 mantiene la decisión hasta que un caso operativo concreto la justifique.
- **No automatización del `git diff` review post-merge.** El operador revisa manualmente; ese es el design.
- **No migración automática de Sentinel.** El operador (re)ejecuta los comandos manualmente en CHARTER-07 cuando v1 ship — no hay tooling de migración specific to Sentinel.

---

## 8. Próximos pasos

**Inmediato (esta sesión, si el operador aprueba la propuesta):**

1. Solicitar al operador el prompt mature de Sentinel pre-StrayMark (lo ofreció en issue #102) para incorporarlo como base de la plantilla unificada.
2. Iniciar implementación PR-by-PR según §5.1.

**Sentinel mientras tanto:**

- CHARTER-07 sigue paused (correctamente).
- CHARTERs 08-13 pueden continuar bajo v0 si urgencia, pero conviene esperar v1 para tener un dataset Phase 2 coherente. Decisión del operador.

**Post-release v1:**

- Sentinel re-corre CHARTER-07 audit bajo v1.
- Sentinel continúa Etapa 2 desde CHARTER-08 bajo v1.
- Phase 2 telemetría agregada al cierre de CHARTER-13 incluye CHARTER-07 v1 + CHARTERs 08-13 v1 — dataset coherente, ~7 cycles audit en un solo flujo unificado.
- Esa data alimenta las decisiones §9 promote/defer/discard del rollout audit-skills.

---

## 9. Riesgos

**R1 — La plantilla unificada con disciplina tool use es más larga que la v0.** Más tokens de input por audit cycle.
*Mitigación:* la disciplina justifica el costo (substantive findings vs vacuamente vacíos). El operador puede decidir no auditar Charters de bajo riesgo (heurística NO del checkpoint sigue aplicando).

**R2 — Adoptantes sin CLI auditora disponible.** Algunos adoptantes solo tienen acceso a chat web (claude.ai, chatgpt, gemini.google.com).
*Mitigación:* el flujo agnóstico de modo (D10) permite paste manual: el operador genera el prompt, lo pega en chat, copia el response al path canónico con el naming correcto, y `audit-review` lo procesa igual. Funciona, solo sin la garantía de tool use.

**R3 — Diferentes CLIs auditoras tienen conventions distintas para invocar skills.** `claude-code` lee `.claude/skills/`, `gemini-cli` lee `.gemini/skills/`, etc.
*Mitigación:* StrayMark ya distribuye los skills en las 3 ubicaciones. CLIs no soportadas usan el `.agent/workflows/` genérico. Si un adoptante usa una CLI no cubierta, levanta issue, agregamos.

**R4 — El operador puede mezclar reports v0 y v1 accidentalmente.** Si un adoptante upgrade mid-Charter y queda con `auditor-primary.md` (v0) y `report-claude-sonnet-4-6.md` (v1) en directorios paralelos.
*Mitigación:* el skill `audit-review` v1 solo busca en `.straymark/audits/<id>/`. Los archivos v0 quedan en `audit/charters/<id>/` ignorados. Documentación adopter explica.

**R5 — La eliminación de la plantilla `calibrator-reconciler.md` puede confundir a operadores que esperaban verla.**
*Mitigación:* CHANGELOG explícito + sección en CLI-REFERENCE explicando que el rol calibrator ahora lo cumple el agente principal in-conversation, no una plantilla separada.

**R6 — Disciplina de `path:line` puede generar friction si el modelo audita pero olvida citar.** Findings se marcan low-confidence aunque sean válidos.
*Mitigación:* el prompt unificado lo enuncia tres veces (en Disciplina, en Output Format, en Examples). Si aún así fail, telemetría lo captura (`evidence_citations: 0` en frontmatter).

---

## 10. Criterios de éxito

La iteración v1 cierra con éxito cuando:

- [ ] Sentinel re-corre CHARTER-07 audit bajo v1 y reporta findings substantivos (≥1 finding `implementation_gap` o `real_debt` con citas `path:line` válidas) — confirma que la limitación R11(B) se resolvió.
- [ ] Sentinel completa Etapa 2 (CHARTERs 08-13) bajo v1 y la telemetría agregada del snapshot final tiene ≥6 records audit, todos con `evidence_citations: > 0` en al menos uno de los reports — confirma que la disciplina tool use es operativa en el flujo.
- [ ] Cero issues abiertas en el repo StrayMark atribuibles a regresión introducida por v1 — confirma que el bump no rompió funcionalidad existente.
- [ ] El skill `straymark-audit-execute` se ejecuta correctamente en al menos 2 CLIs auditoras distintas (claude-code, gemini-cli o copilot-cli) — confirma multi-CLI compatibility.
- [ ] El `review.md` consolidado producido por v1 contiene las 6 secciones (Resumen ejecutivo / Definición de alcance / Evaluación por auditoría / Plan de remediación P0-P4 / Hallazgos descartados / Calificación de auditores) y al menos 1 ítem en el plan de remediación con archivo:línea, descripción concreta y complejidad estimada — confirma que el delivery al humano supera al YAML mergeado del v0.
- [ ] El operador completa al menos 1 cycle audit sin escribir manualmente ningún path en cualquier comando ni copiar-pegar contenido entre CLIs — confirma que el flujo zero-friction de D14 funciona.

Si todo lo anterior se cumple, v1 es señal fuerte para considerar transición v0 → v1 estable del schema audit en el siguiente cycle de roadmap (gated por segundo dominio según principio #12, pero la evidencia operativa de Sentinel es suficiente para preparar la propuesta).

---

*v0.2 cierra la design phase. Los 14 puntos de §2 (decisiones de diseño) están aprobados por el operador en sesión de revisión post-issue #102; las consecuencias operativas en §3-§7 fluyen de ellas. La implementación que sigue ejecuta esto, no diseña sobre la marcha. La próxima versión (v0.3) se escribirá tras la implementación completa y el primer re-run de CHARTER-07 en Sentinel, con observaciones operativas que la implementación generó pero que el diseño no anticipó.*
