# StrayMark — Auditoría externa como skill: checkpoint humano-en-el-loop y `/straymark-audit-{prompt,review}`

**Versión:** 0.2 (Fase 1 implementada y shippeada — diseño v0 experimental, segundo dominio aún pendiente para cristalización a v1)
**Fecha:** 3 de mayo de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Cerrar la fricción entre el CLI orchestration-only (`straymark charter audit`) y el agente que está dentro de la conversación del developer. Hoy el CLI escribe prompts a disco y el operador los abre manualmente; falta el paso "el agente principal te dice que llegamos a un checkpoint y te ofrece el prompt inline". Esta propuesta abstrae los skills `plan-audit` y `plan-audit-review` de Sentinel a `straymark-audit-prompt` y `straymark-audit-review` (instalables vía `straymark init` para todos los adoptantes) y agrega guidance de checkpoint a `AGENT-RULES.md`.
**Documentos relacionados:** `straymark-cli-roadmap.md` v0.3 §5 (Phase 3 audit, decisión A1 orchestration-only), `straymark-design-principles.md` v0.2 (#6 fricción virtuosa, #10 CLI no es LLM gateway, #12 cristalización experimental), `straymark-charter-telemetry.md` v0.3 (campo `external_audit` array), `que-es-un-charter.md` (alcance del Charter como ancla de auditoría ex-post), `straymark-audit-skills-implementacion.md` v0.2 (plan de rollout — Fase 1 cerrada, Fase 2 en recolección).

---

## 0. Estado de implementación (al 3 de mayo de 2026)

**Fase 1 cerrada.** La release `fw-4.8.0` / `cli-3.9.0` aterriza las dos skills, el flag CLI `--merge-into`, la guidance del checkpoint en `AGENT-RULES.md`, las docs adopter, y bumps de versión — entregadas en 5 PRs secuenciales mergeados el mismo día.

**PRs shippeados:**

| # | Tipo | PR | Aporte |
|---|---|---|---|
| 1 | feat(framework) | [#96](https://github.com/StrangeDaysTech/straymark/pull/96) | Skill `straymark-audit-prompt` × 3 plataformas (Claude / Gemini / agent) + 4 tests fixture |
| 2 | feat | [#97](https://github.com/StrangeDaysTech/straymark/pull/97) | Skill `straymark-audit-review` × 3 plataformas + flag CLI `straymark charter audit --finalize --merge-into <PATH>` + 4 tests integration sobre el merge + 4 tests fixture sobre el skill + fix bonus de placeholder `<charter-id>` en `audit_notes:` |
| 3 | feat(framework) | [#98](https://github.com/StrangeDaysTech/straymark/pull/98) | `AGENT-RULES.md §12 Audit Checkpoint` × 3 langs (EN / ES / zh-CN) + 4 tests fixture (presencia + paridad de anchors language-agnostic) |
| 4 | docs | [#99](https://github.com/StrangeDaysTech/straymark/pull/99) | Adopter docs: `WORKFLOWS.md`, `CLI-REFERENCE.md` (nueva `## Skills` section listando los 9 skills), `ADOPTION-GUIDE.md` (nueva `## External Audit (Optional)`), `QUICK-REFERENCE.md` — todos × 3 langs |
| 5 | chore | [#100](https://github.com/StrangeDaysTech/straymark/pull/100) | Bump `fw-4.7.1` → `fw-4.8.0` + `cli-3.8.1` → `cli-3.9.0`, version refs en 22 archivos × 3 langs, CHANGELOG combined section, tags `fw-4.8.0` y `cli-3.9.0` push (release workflows ejecutados) |

**Decisiones de §2 materializadas:**

- **D1 (skill = wrapper del CLI)** ✅ — ambas skills delegan vía `Bash(straymark charter audit *)` a la implementación canónica. Plantillas viven solo en `dist/.straymark/audit-prompts/`. Cero drift posible entre skill y CLI.
- **D2 (checkpoint soft, sin escalación a hard)** ✅ — `AGENT-RULES.md §12` declara explícitamente "this is a permanent v0+v1 design decision; see `Propuesta/straymark-audit-skills.md §2.2`". Ningún PR introdujo enforcement. `straymark metrics` no incluye KPI de audit coverage.
- **D3 (`straymark-audit-review` auto-mergea)** ✅ — implementado vía el flag nuevo `--merge-into <PATH>`. String-level append a indent 2 bajo `charter_telemetry:`, preservando el shape hand-written de `charter close`. v0 rechaza re-audit con mensaje claro (manual reconciliation cuando aterriza re-merge en futuro).

**Heurística arborist (§4.3) materializada:**

La heurística "diff con función > 2× threshold cognitivo" está codificada en `AGENT-RULES.md §12` con graceful-degradation explícita ("if the binary lacks the `analyze` feature, silently skip this signal — do not warn, do not mention"). Los binarios oficiales (`release-cli.yml`) compilan con `default = ["tui", "analyze"]`, así que en práctica el 99% de adoptantes la tienen activa.

**Cobertura de tests añadida:** 16 tests nuevos (8 fixture sobre skills × 2 + 4 fixture sobre §12 checkpoint + 4 integration sobre `--merge-into`). Suite completa: 271 unit + todas las integration verdes.

**Items §6 (mapping a documentación) shipped:**

- `dist/.straymark/00-governance/AGENT-RULES.md` § 12 (3 langs) — ✅
- `docs/adopters/WORKFLOWS.md` (3 langs) — ✅
- `docs/adopters/CLI-REFERENCE.md` (3 langs) con nueva sección `## Skills` listando los 9 skills + nueva `### Audit checkpoint` + callout "Skill alternative" en `### straymark charter audit` — ✅
- `docs/adopters/ADOPTION-GUIDE.md` (3 langs) con sección `## External Audit (Optional)` — ✅
- `dist/.straymark/00-governance/QUICK-REFERENCE.md` (3 langs) con tabla de skills completa — ✅
- `README.md` (3 langs) — sin cambios materiales (solo bumps de versión vía sed)
- `CHANGELOG.md` — sección combined `Framework 4.8.0 / CLI 3.9.0` — ✅
- `dist/STRAYMARK.md` — sin cambios (intencionalmente minimal)

**Items diferidos a Fase 2 — observación, no implementación:**

La recolección de telemetría operativa (cuántos checkpoints emit, cuántos audits aceptados/rechazados, correlación complejidad ↔ findings, diff_size ↔ findings_total) está descrita en `straymark-audit-skills-implementacion.md §2`. Requiere que la release ejercite en Sentinel + adoptante de frontend antes de poder entrar a Fase 3 (decisiones §9 sobre integración mayor de arborist).

**Open issues a la fecha:** ninguno atribuible al rollout.

**Items diferidos a v1 con criterio de salida explícito** (de §10 + del §9 del propuesta original):

- Re-audit (append a `external_audit:` ya presente) — gated en demanda real de adopter.
- Auto-detección y enforcement de heterogeneidad inter-familia — diferido a v1, dependiente de que aterricen HTTP clients (decisión A1 del roadmap).
- Integración mayor de arborist (4 gaps de §9) — gated en data acumulada en Fase 2 + ciclo del adoptante de frontend.
- Cristalización del schema `audit-telemetry.schema.v0.json` a v1 estable — gated en segundo dominio (frontend).

---

## 1. Contexto y problema

### 1.1 El estado shippeado (`fw-4.7.1` / `cli-3.8.1`)

El CLI ya orquesta auditoría multi-modelo en 3 pasos:

- **PREPARE** — escribe `audit/charters/CHARTER-XX/prompts/auditor-{primary,secondary}.prompt.md` resolviendo plantillas en `dist/.straymark/audit-prompts/` con el contenido del Charter, git diff y AILOGs origen.
- **CALIBRATE** — valida las respuestas del operador contra `audit-output.schema.v0.json` y escribe `prompts/calibrator-reconciler.prompt.md`.
- **FINALIZE** — valida el calibrator response y **imprime** un bloque YAML formateado para el array `external_audit:` de la telemetría.

El operador, entre paso y paso: (a) abre los archivos de prompt del disco, (b) los pega en su auditor de elección, (c) guarda las respuestas a paths canónicos, (d) re-ejecuta el comando con `--calibrate` o `--finalize`, (e) **copia-pega el YAML del FINALIZE a la telemetría manualmente**. Ver `cli/src/commands/charter/audit.rs` y `docs/adopters/CLI-REFERENCE.md` `### straymark charter audit`.

### 1.2 Tres gaps que la propuesta atiende

**G1 — No hay checkpoint en el workflow del agente.** `docs/adopters/WORKFLOWS.md` describe "passive loop: trabajar → documentar → review → commit" sin mencionar audit. `charter close` (`cli/src/commands/charter/close.rs`) no referencia audit. La decisión "auditar o no" hoy depende enteramente de memoria del developer, sin un punto explícito donde el agente que está co-implementando el Charter le ofrezca la opción.

**G2 — Los prompts viven en disco, no en la conversación.** Los IDEs modernos (Claude Code, Cursor, etc.) son el contexto natural donde el developer interactúa con el agente. El CLI escribe prompts a archivos; el developer tiene que abrirlos en otro pane y copiarlos a otro modelo manualmente. Para un flujo IDE-first, el agente debería poder **imprimir el prompt directamente en la conversación**.

**G3 — Sentinel tiene los skills, StrayMark no.** `sentinel/.claude/skills/plan-audit/` y `plan-audit-review/` (vocabulario "Plan", precediendo el rename a Charter) implementan exactamente este patrón — generar prompt inline, calibrar al regreso, fusionar en telemetría. Pero están acoplados a paths Sentinel-específicos (`docs/plans/`, `internal/modules/`, `go vet`, etc.) y no existen como skills StrayMark genéricos. Sentinel lleva 6 ciclos validándolos empíricamente; portarlos al framework cierra el segundo dominio operativo (skills + CLI dentro de StrayMark) sin esperar al adoptante de frontend.

### 1.3 Por qué esto importa más allá del flow operativo

StrayMark acelera el desarrollo a una cadencia que el operador humano no puede seguir por inspección directa — los Charters cierran, los AILOGs aterrizan, las telemetrías se llenan. Si en cada cierre el operador tuviera que mover datos manualmente entre archivos, el throughput colapsaría y la disciplina dejaría de ser fricción virtuosa para volverse ceremonia (principio #6). **Lo que se pueda automatizar de forma reversible debe automatizarse**; los documentos quedan para auditoría humana ex-post.

## 2. Decisiones tomadas (con justificación)

### 2.1 D1 — Las skills delegan al CLI, no son standalone

`/straymark-audit-prompt CHARTER-XX` ejecuta `straymark charter audit CHARTER-XX` (PREPARE) por debajo, **lee** los archivos `prompts/auditor-{primary,secondary}.prompt.md` que el CLI acaba de escribir, e imprime su contenido inline en la conversación junto con guidance sobre qué modelo usar para cada uno. Análogamente, `/straymark-audit-review CHARTER-XX` ejecuta el `--calibrate` y luego el `--finalize`, leyendo los outputs intermedios.

**Justificación:**

- Single source of truth: las plantillas viven solo en `dist/.straymark/audit-prompts/`. El CLI las resuelve, la skill las consume. Cero drift entre plantilla-CLI y plantilla-skill.
- Compatibilidad con flujo no-IDE: un adoptante en CI o sin agente IDE-side sigue usando solo el CLI; los outputs son idénticos (mismas paths, mismo schema, misma telemetría).
- Cumple principio #10: el CLI sigue siendo el orquestador, la skill es UX inline.

### 2.2 D2 — Checkpoint soft, sin escalación a hard planeada

El checkpoint vive como guidance en `dist/.straymark/00-governance/AGENT-RULES.md` y como recordatorio en `WORKFLOWS.md`. **`charter close` no verifica** si la auditoría se realizó — close puede ejecutarse con o sin `audit/charters/CHARTER-XX/calibrator-reconciler.md` presente, sin warning, sin error, sin flag `--no-audit`.

**Justificación (explícita y permanente):**

- La auditoría externa cuesta dinero. Pedirle al adoptante 2-3 modelos pagos por cada Charter es una barrera real de adopción que no se debe disfrazar de "best practice".
- StrayMark ya provee disciplina suficiente (Charter declarativo + drift check + AILOG con R<N> + telemetría ex-post) para que el resultado sea confiable sin auditoría externa. La auditoría es **señal adicional**, no requisito de cierre.
- La asimetría (close enforcing audit) penalizaría al adoptante por no tener recursos, dejando una marca permanente en su documentación. StrayMark no debe generar fricción que el adoptante percibiría como castigo por no consumir más LLMs.
- Esta decisión se cristaliza ahora y **no se revisa** en ciclos futuros — no hay "v1 hard enforcement" en el roadmap. Si un adoptante quiere enforcement, puede agregarlo en su CI con un check propio sobre los archivos del directorio `audit/charters/`.

### 2.3 D3 — `/straymark-audit-review` auto-mergea el YAML en la telemetría

Después de generar el calibrator response y validar los 3 outputs, la skill **edita** `.straymark/charters/CHARTER-XX.telemetry.yaml` directamente, insertando el array `external_audit:` con el contenido del FINALIZE. El developer ve el diff antes de aceptar (o vía git diff inmediatamente después).

**Justificación:**

- Cierra el último copy-paste manual del flujo audit. La velocidad operativa de StrayMark (descrita en §1.3) hace que dejar este paso al humano sea fricción no-virtuosa.
- Reversibilidad por inspección: el merge produce un cambio en un archivo YAML versionado; cualquier error es trivialmente reversible con `git checkout` o edición manual.
- Coincide con lo que Sentinel `plan-audit-review` hace empíricamente desde Plan-01.

## 3. Diseño de los dos skills

### 3.1 `straymark-audit-prompt`

**Naming:** `straymark-audit-prompt` (kebab-case, prefijo `straymark-`, paralelo a los 7 skills existentes).

**Distribución:** Tres archivos paralelos, instalados vía `dist-manifest.yml` que ya incluye `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/`:

- `dist/.claude/skills/straymark-audit-prompt/SKILL.md` — frontmatter con `name`, `description`, `allowed-tools: [Bash, Read]`.
- `dist/.gemini/skills/straymark-audit-prompt/SKILL.md` — frontmatter sin `allowed-tools`.
- `dist/.agent/workflows/straymark-audit-prompt.md` — frontmatter con solo `description`.

**Argumentos:** `CHARTER-ID` posicional (mismas reglas de resolución que `charter status`).

**Comportamiento:**

1. Verifica que existe el Charter y está en status `in-progress` o `declared` (no `closed`). Si está `closed`, advierte y sale con código 0 — auditar un Charter ya cerrado es válido pero atípico.
2. Ejecuta `straymark charter audit <CHARTER-ID>` (paso PREPARE). El CLI escribe los 2 prompts.
3. Lee `audit/charters/<CHARTER-ID>/prompts/auditor-primary.prompt.md` y `auditor-secondary.prompt.md`.
4. Imprime en la conversación, con bloques de código separados, los 2 prompts completos.
5. Imprime guidance corta: qué familias de modelos usar (recomendación de heterogeneidad inter-familia, igual que CLI-REFERENCE §5.2 del roadmap), dónde guardar las respuestas (paths canónicos), y cómo regresar (`/straymark-audit-review CHARTER-XX`).

**No hace:** invocar APIs, decidir el modelo del operador, validar nada (eso lo hace `audit-review`).

**Ejemplo de salida (en la conversación del agente):**

```
> /straymark-audit-prompt CHARTER-04

✔ Resolved CHARTER-04 (in-progress) → audit/charters/CHARTER-04/prompts/

═══════════════════ AUDITOR PRIMARY PROMPT ═══════════════════
[contenido del prompt resuelto, ~150-200 líneas]
══════════════════════════════════════════════════════════════

═══════════════════ AUDITOR SECONDARY PROMPT ═════════════════
[contenido del prompt resuelto, ~130-150 líneas]
══════════════════════════════════════════════════════════════

Next steps:
  1. Run AUDITOR PRIMARY PROMPT in a model of family A
     (e.g., Anthropic — claude-sonnet-4-6).
  2. Run AUDITOR SECONDARY PROMPT in a model of family B
     (e.g., Google — gemini-2.5-pro). DO NOT use the same family
     for both — see audit-prompts/auditor-primary.md §heterogeneity.
  3. Save responses to:
       audit/charters/CHARTER-04/auditor-primary.md
       audit/charters/CHARTER-04/auditor-secondary.md
  4. Return with: /straymark-audit-review CHARTER-04
```

### 3.2 `straymark-audit-review`

**Naming, distribución, argumentos:** análogos a `straymark-audit-prompt`.

**Comportamiento:**

1. Verifica que existen `audit/charters/<CHARTER-ID>/auditor-primary.md` y `auditor-secondary.md` (las respuestas del operador). Si falta alguno, error con instrucción de invocar `straymark-audit-prompt` primero.
2. Ejecuta `straymark charter audit <CHARTER-ID> --calibrate`. El CLI valida los 2 outputs contra `audit-output.schema.v0.json` y escribe `prompts/calibrator-reconciler.prompt.md`.
3. Lee el calibrator prompt resuelto e **invoca al agente principal de la conversación** (Claude/Gemini/etc.) para producir el calibrator response. Esta es una diferencia importante con `straymark-audit-prompt`: aquí el calibrator SÍ corre dentro del agente principal, porque la heterogeneidad inter-familia solo importa para el par auditor (ver `straymark-cli-roadmap.md` §5.2 — el calibrador puede ser de cualquier familia incluyendo la del implementador).
4. Escribe `audit/charters/<CHARTER-ID>/calibrator-reconciler.md` con el response.
5. Ejecuta `straymark charter audit <CHARTER-ID> --finalize`. El CLI valida los 3 outputs e imprime el YAML para `external_audit`.
6. **Auto-mergea** el YAML en `.straymark/charters/<CHARTER-ID>.telemetry.yaml`:
   - Si la telemetría no existe todavía (Charter aún no cerrado vía `charter close`), la skill **no la crea** — solo escribe `audit/charters/<CHARTER-ID>/external-audit-pending.yaml` con el bloque listo, y deja un mensaje al developer: "Run `straymark charter close CHARTER-XX` first; the audit findings will be auto-merged at close time" (ver §3.3 sobre la integración con close).
   - Si la telemetría existe y ya tiene `external_audit:` poblado (re-audit), la skill **append** al array (no reemplaza), preservando histórico.
   - Si la telemetría existe sin `external_audit:`, inserta el campo con el array nuevo.
7. Imprime resumen al developer: cuántos findings totales, cuántos por categoría, dónde quedó el calibrator analysis, y dónde se mergeó la telemetría (con sugerencia de `git diff` para revisar).

**Ejemplo de salida:**

```
> /straymark-audit-review CHARTER-04

✔ Validated audit/charters/CHARTER-04/auditor-primary.md (5 findings)
✔ Validated audit/charters/CHARTER-04/auditor-secondary.md (4 findings)
✔ Wrote audit/charters/CHARTER-04/prompts/calibrator-reconciler.prompt.md
✔ Generated audit/charters/CHARTER-04/calibrator-reconciler.md
   (calibrator: claude-opus-4-7, 8 findings consolidated:
    agreed=3, disputed=2, unique_primary=1, unique_secondary=1, rejected=1)
✔ Merged external_audit array into
   .straymark/charters/CHARTER-04.telemetry.yaml

  Run `git diff .straymark/charters/CHARTER-04.telemetry.yaml`
  to review the merge before commit.
```

### 3.3 Integración opcional con `charter close`

`charter close` se mantiene **agnóstico** del audit (decisión D2). Pero como el flujo natural es `audit-prompt` → `audit-review` → `charter close`, hay un caso (§3.2 paso 6) donde la skill audit-review se ejecuta antes de close y debe diferir el merge. La propuesta:

- Si `audit-review` corre antes de close: deja `audit/charters/<CHARTER-ID>/external-audit-pending.yaml` listo.
- `charter close` (sin cambios al CLI Rust en este PR) no lo lee. El operador, al cerrar interactivamente, copiará el bloque manualmente al campo `external_audit:` cuando llegue el prompt correspondiente — exactamente igual que hoy con la salida de FINALIZE.
- En un PR futuro **opcional** (no parte de este scope), `charter close` podría leer `external-audit-pending.yaml` automáticamente y pre-poblar el campo en el flujo interactivo. Queda fuera de v0 hasta validar que el patrón se usa.

## 4. Lógica del checkpoint

### 4.1 Cuándo el agente debe proponerlo

El checkpoint **no es una skill que el developer invoque**. Es guidance al agente principal (Claude Code, Gemini Code, Cursor, etc.) sobre cuándo proponerlo proactivamente. La sección se agrega a `dist/.straymark/00-governance/AGENT-RULES.md` con un trigger claro.

**Trigger:** el agente debe proponer el checkpoint cuando, en el contexto de implementación de un Charter, **simultáneamente**:

1. El Charter está en status `in-progress` o `declared`.
2. Todas las tasks del Charter están marcadas como `[x]` (completadas) o el agente acaba de completar la última.
3. `straymark charter drift CHARTER-XX` produce exit 0 (sin drift inacontabilizado).
4. El developer aún **no** ha invocado `straymark charter close` ni mencionado intención de cerrar.

Si los 4 se cumplen, el agente emite un mensaje de checkpoint **una sola vez** por Charter (no repite en cada turno).

### 4.2 Forma de la recomendación Sí/No

El agente debe redactar la recomendación como **opinión informada**, no como pregunta neutra. La forma:

```
Llegamos al checkpoint del Charter CHARTER-XX. Está implementado, drift OK, pendiente solo charter close.

En este punto puedes correr una auditoría externa (típicamente 2 modelos
de familias distintas + un calibrador) que arroje findings cross-modelo
sobre la implementación.

Mi recomendación: [SÍ / NO], porque:
  - [razón concreta basada en el Charter actual]

Si decides hacerla: ejecuta /straymark-audit-prompt CHARTER-XX y te
imprimo aquí mismo los 2 prompts. Cuando regreses con las respuestas
del operador externo, ejecuta /straymark-audit-review CHARTER-XX y yo
calibro + mergeo en la telemetría.

Si decides no auditar: continúa con straymark charter close CHARTER-XX
cuando estés listo. La auditoría es completamente opcional — el
Charter declarado + drift check + AILOG dan suficiente disciplina
para cierre confiable sin auditoría externa.
```

### 4.3 Heurísticas para la recomendación SÍ/NO

`AGENT-RULES.md` debe codificar heurísticas, no reglas rígidas (principio #6: el agente está cerca del contexto, sus heurísticas se afinan con el adoptante). Las iniciales:

**Recomienda SÍ cuando** (cualquiera basta):

- El Charter tocó superficie crítica de seguridad (auth, RLS, secret handling, IAM).
- El Charter introdujo un componente nuevo (no refactor) que el developer no había co-implementado antes.
- Algún AILOG asociado documenta un R<N> con confidence `low` o `medium` y `risk_level` ≥ `medium`.
- El developer marcó el Charter como `effort_estimate: L` y es el primer Charter del adoptante.
- El developer **explícitamente pidió validación cross-modelo** en el trigger del Charter.
- **Señal estructural de complejidad** *(disponible cuando el CLI fue compilado con el feature `analyze` activo — true en los binarios oficiales descargados vía `straymark update-cli`)*: el diff del Charter introduce o modifica al menos una función cuya cognitive complexity supera **2× el threshold configurado** en `.straymark/config.yml` (`complexity.threshold`, default `8` → ≥ `17`). Una función densa nueva es exactamente el caso donde dos auditores cross-modelo capturan implementation gaps que un solo modelo deja pasar. El cómputo se hace invocando la lógica que ya alimenta `straymark analyze` (`cli/src/analysis_engine.rs`) sobre el subset de archivos modificados en `range`. **Graceful-degradation:** si la skill detecta que el binario actual fue compilado sin `analyze` (caso 1% — adoptante en CI ligero, governance-only, target no-oficial, supply-chain audit, lenguaje no soportado), salta esta heurística silenciosamente y evalúa solo las anteriores — no falla, no warning, no menciona la ausencia. El feature flag `analyze` se mantiene como `default = true` precisamente para que esta rama sea improbable en práctica.

**Recomienda NO cuando** (todas):

- El Charter es refactor o documentación (no introduce comportamiento nuevo).
- `effort_estimate` ≤ `S`.
- AILOGs asociados confidence `high` y sin R<N> emergentes.
- `risk_level` del Charter ≤ `low`.

**Caso default (ninguno claro):** recomienda NO con justificación neutra ("no veo señal específica que justifique el costo de 2-3 modelos adicionales; cierra cuando estés listo"). Esto evita inflar el costo del adoptante por inercia.

### 4.4 Lo que el checkpoint **no** hace

- No bloquea ninguna acción del developer.
- No vuelve a aparecer si el developer dijo "no auditar" — el agente recuerda la decisión por el resto del Charter.
- No cuenta hacia métricas de calidad: no hay un KPI "% Charters auditados" en `straymark metrics` (D2 — la auditoría es señal adicional, no requisito).

## 5. Coexistencia CLI ↔ Skills

| Caso de uso | Herramienta indicada |
|---|---|
| Developer trabajando en IDE con agente activo, finaliza Charter | Skills `/straymark-audit-prompt` + `/straymark-audit-review` |
| CI / batch / script sin agente IDE-side | CLI directo: `straymark charter audit` (PREPARE/CALIBRATE/FINALIZE) |
| Adoptante sin presupuesto para audit | Ninguna — `charter close` directo, audit es opcional |
| Re-auditoría de un Charter ya cerrado (post-incident review) | Skills funcionan también en Charters `closed`; auto-merge hace `append` al array |
| Adoptante quiere usar solo CLI (no skills, no agente) | El CLI sigue siendo standalone; las skills son layer adicional |

Las skills son sugaring; el CLI es la base. Quitar las skills no rompe nada. Esta es la propiedad arquitectónica que mantiene principio #10.

## 6. Documentación a actualizar

| Archivo | Cambio |
|---|---|
| `dist/.straymark/00-governance/AGENT-RULES.md` (EN + ES + zh-CN) | Nueva sección "Audit checkpoint" con triggers (§4.1), forma del mensaje (§4.2), heurísticas SÍ/NO (§4.3). |
| `docs/adopters/WORKFLOWS.md` (EN + ES + zh-CN) | Diagrama del loop incluye el checkpoint (entre drift y close), con nota de que es opt-in. |
| `docs/adopters/CLI-REFERENCE.md` (EN + ES + zh-CN) | Nueva sección `## Skills` (paralela a `## Commands`) listando los 9 skills (7 actuales + 2 nuevos). Para cada skill: nombre, propósito, ejemplo de invocación, archivos que produce. La sección `### straymark charter audit` (CLI) gana un párrafo "Skill alternative" referenciando los nuevos skills. |
| `dist/.straymark/00-governance/QUICK-REFERENCE.md` (EN + ES + zh-CN) | Tabla de skills agrega 2 filas. |
| `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` | Sin cambios — el campo `external_audit` ya está en la policy. |
| `docs/adopters/ADOPTION-GUIDE.md` (EN + ES + zh-CN) | Sección "Daily loop" menciona el checkpoint. Sección "External audit (optional)" expande sobre cuándo y cómo. |
| `dist/STRAYMARK.md` | Sin cambios — STRAYMARK.md es minimal y no cubre Phase 3. |
| `README.md` (EN + ES + zh-CN) | Tabla de comandos no cambia (skills no van ahí). Tabla de skills (si existe) gana 2 filas; si no existe, sección breve. |
| `CHANGELOG.md` | `## Framework X.Y.Z` con `### Added`: dos nuevas skills + nuevo checkpoint en AGENT-RULES.md. `## CLI X.Y.Z` solo si la skill `straymark-audit-review` requiere ajustes en cómo el CLI imprime el FINALIZE (parser-friendly), lo cual es probable pero menor. |
| `Propuesta/straymark-cli-roadmap.md` | Nueva entrada en §0 ("Releases shipped") cuando aterrice. Si la implementación cierra el gap §5.5 que diferimos a v1, marcarlo. |

## 7. Plan de implementación (PRs ordenados)

| # | Título | Critical path | Estimación | Tipo |
|---|---|---|---|---|
| 1 | Skill `straymark-audit-prompt` (3 plataformas) + plantillas verificadas | sí | 4-6 h | framework + tests |
| 2 | Skill `straymark-audit-review` (3 plataformas) — incluye lógica de merge YAML | sí | 8-10 h | framework + tests |
| 3 | Checkpoint guidance en `AGENT-RULES.md` (3 langs) + tests de fixture en `cli/tests/` que verifican que la sección existe | sí | 3-4 h | governance |
| 4 | Documentación adopter — `WORKFLOWS.md`, `CLI-REFERENCE.md`, `ADOPTION-GUIDE.md`, `QUICK-REFERENCE.md`, `README.md` (3 langs cada uno) | no (paralelizable) | 5-7 h | docs |
| 5 | Bump `fw-X.Y.Z` (minor: nuevas skills + nueva guidance) y `cli-X.Y.Z` solo si PR 2 requiere ajustes al CLI; CHANGELOG; tag release | sí | 1 h | release |

Total: ~21-28 h focused; 1.5-2 semanas con dependencias de revisor. PR 1-3 son secuenciales; PR 4 puede empezar en paralelo después de que PR 3 estabilice el shape.

## 8. Riesgos

**R1 — Drift entre plantilla CLI y skill.** Si por alguna razón un futuro PR modifica las plantillas en `dist/.straymark/audit-prompts/` y la skill las sobre-procesa por separado, los outputs divergen. *Mitigación (D1):* la skill **no procesa** plantillas; lee los archivos resueltos que el CLI escribe. Drift estructuralmente imposible.

**R2 — El checkpoint inflama costo del adoptante.** Un agente eager-to-help que recomienda SÍ siempre, multiplica el gasto en LLMs. *Mitigación (§4.3):* las heurísticas tienen "default NO" cuando ninguna señal es clara, y el mensaje es explícito sobre el costo. Validar en el primer ciclo de adoptante de frontend que el agente no esté sobre-recomendando.

**R3 — Los IDEs no honran el trigger del checkpoint.** El agente solo emite el checkpoint cuando los 4 triggers se cumplen, pero no todos los runtime de IDE inspeccionan los archivos antes de cada turno. *Mitigación:* el checkpoint también puede invocarse manualmente como `/straymark-audit-checkpoint CHARTER-XX` (skill no incluida en este v0; queda como fallback a explorar si hace falta).

**R4 — Auto-merge falla en YAML mal-formado.** La telemetría existente puede tener el campo `external_audit:` con sintaxis inesperada. *Mitigación:* la skill valida el YAML antes de mergear, falla con error explícito si no parsea, y deja `external-audit-pending.yaml` para merge manual. Nunca destruye datos.

**R5 — El operador olvida regresar con `/straymark-audit-review`.** El flujo se queda colgado: prompts generados, respuestas pegadas, calibrator nunca corre. *Mitigación:* el checkpoint mensaje incluye instrucciones explícitas sobre el regreso; `straymark charter status` (en un PR futuro opcional) podría detectar la presencia de `auditor-*.md` sin `calibrator-reconciler.md` y advertir al developer la próxima vez que consulte el Charter.

**R6 — Sentinel no es un buen proxy para validar las heurísticas.** Sentinel es Go-backend; las heurísticas de "tocó auth/RLS/IAM" pueden no aplicar a otros stacks. *Mitigación:* el ciclo frontend es exactamente el N=2 que valida si las heurísticas del checkpoint son inter-stack. Mantenerse en v0 experimental hasta entonces.

## 9. Lo que NO está en este v0

- **No hay skill `straymark-audit-checkpoint`.** El checkpoint es guidance al agente, no comando del developer. Si los IDEs no honran el trigger, se reconsidera (R3).
- **No hay enforcement en `charter close`.** Decisión D2, cristalizada permanentemente.
- **No hay invocación automática del CLI desde la skill en un loop.** La skill ejecuta el CLI vía Bash y lee outputs; no hay daemon, no hay watcher, no hay polling.
- **No hay traducción i18n del calibrator response.** El calibrator response queda en el idioma que el agente principal use; las plantillas de auditor-prompts están en EN canónico (igual que hoy).
- **No hay métricas en `straymark metrics` sobre audit coverage.** Por D2: no convertir audit en KPI elimina el incentivo a inflar el conteo.
- **No hay charter audit-review para AILOGs sueltos** (sin Charter). El alcance es estrictamente Charter-bound. Auditar AILOGs sin Charter queda como fricción a observar antes de extender el patrón.
- **No se integra arborist más allá del checkpoint.** Hoy `arborist-metrics` (vía feature `analyze`) solo alimenta `straymark analyze` y, con esta propuesta, una heurística opcional del checkpoint en §4.3. **Hay potencial mucho mayor reconocido pero fuera de scope:** (a) campo `agent_quality.complexity_delta` en `charter-telemetry.schema` poblado automáticamente por `charter close`, (b) warning en `validate --include-charters` cuando un Charter cierra con funciones que cruzaron `2× threshold` sin un AILOG explicando, (c) métricas longitudinales en `straymark metrics` (cómo evoluciona la complejidad del proyecto a lo largo de Charters cerrados), (d) sección "Code complexity surface" auto-generada en el Charter al usar `--from-spec` o `--from-ailog`. Cada una abre superficie de schema que requiere su propia validación empírica antes de cristalizar — quedan como propuestas separadas para ciclos posteriores cuando el adoptante de segundo dominio (frontend) aporte señal sobre qué métricas son útiles inter-stack.

## 10. Criterios de salida v0 → v1

Para cristalizar este patrón a v1 estable (siguiendo principio #12, paralelo al §0 del roadmap):

- Al menos 1 ciclo completo de `straymark-audit-prompt` + `straymark-audit-review` ejecutado en el adoptante de segundo dominio (frontend) con findings cross-modelo consistentes con la calibración observada en Sentinel.
- El checkpoint emitido al menos 3 veces, con la decisión SÍ/NO del developer alineada con la heurística en al menos 2/3 (señal de que la heurística es útil, no ruido).
- Cero pérdida de datos por auto-merge fallido (R4).
- 0 issues abiertas en GitHub atribuibles a desincronización CLI ↔ skill.

Si alguno falla, el cycle siguiente del roadmap reabre el diseño antes de promover a v1.

---

*v0.2 corresponde al estado post-Fase-1 (release `fw-4.8.0` / `cli-3.9.0`). La cristalización a v1 sigue gated en el segundo dominio (adoptante de frontend) y en la data de Fase 2. La sección 2 (decisiones D1/D2/D3) se mantiene estable — D2 en particular es decisión definitiva que no se revisa en futuros ciclos. La próxima versión (0.3) se escribirá al cerrar Fase 2 con la data acumulada y la decisión de §9 promote/defer/discard por gap.*
