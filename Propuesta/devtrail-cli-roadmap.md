# DevTrail CLI — Roadmap hacia la tesis (post-Sentinel)

**Versión:** 0.3 (post-implementación de las 3 fases — A1 orchestration-only documentada para Fase 3; RFCs #67/#82/#91 cerradas; frictions F1-F8 + observación O3 resueltos; CHARTER-01/02 + AILOG-01 ejemplos shipped en `dist/docs/examples/`)
**Fecha:** 3 de mayo de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Traducir los hallazgos validados de Sentinel en una secuencia accionable de cambios al CLI Rust y al framework, manteniendo el principio #12 (cristalización experimental, no estable).
**Documentos relacionados:** `devtrail-thesis-validation.md` (evidencia que justifica el roadmap), `devtrail-design-principles.md` (anotaciones v0.2 sobre #6, #9, #12), `devtrail-cloud-proposal.md` §4.5 y §8 (Charters en Cloud, Q3), `devtrail-charter-telemetry.md` v0.3 (schema de telemetría refinado), `que-es-un-charter.md` (alcance conceptual del artefacto Charter y coexistencia con SpecKit).

---

## 0. Estado de implementación (al 3 de mayo de 2026)

Las tres fases del roadmap están shippeadas. El gate de cristalización `v0 → v1` (segundo dominio) sigue abierto porque Sentinel es el único adoptante hasta hoy y es Go-backend; el siguiente experimento en un subproyecto de frontend está agendado y abre la puerta a evaluar la promoción a v1 estable.

**Releases shipped:**

| Fase | Release | Issues / RFCs cerradas en este release |
|---|---|---|
| 1 — Charters como entidad de primera clase | `fw-4.4.0` / `cli-3.6.0` (PR #65) | — |
| 1 patches | `fw-4.4.1` (#66), `fw-4.4.2` (#68 format v4 atomic Charter closure), `cli-3.6.1` (#69 charter new numbering F1) | — |
| Reposicionamiento canónico | `fw-4.5.0` (#71), `fw-4.5.1` (#72 i18n + ADOPTION-GUIDE reframe) | — |
| 2 — Telemetría + drift + approval workflow | `fw-4.6.0` / `cli-3.7.0` (PR #80) | RFC #67 (canonical approval workflow) |
| 2 patches part 1 | `fw-4.6.1` / `cli-3.7.1` (PR #83) | F3 / F4 / F6 de issue #81 |
| 2 patches part 2 | `fw-4.6.2` / `cli-3.7.2` (PR #84) | F1 / F8 + drift wildcard glob |
| 3 — Auditoría externa multi-modelo (orchestration-only) + open frictions | `fw-4.7.0` / `cli-3.8.0` (PR #90) | RFC #82 (audit visibility) + frictions F2 / F5 / F7 |
| 3 patches | `fw-4.7.1` / `cli-3.8.1` (PR #92) | Observación O3 (`--no-ailog-suppress` always emits INFO line) |

**Decisiones arquitectónicas que divergen del roadmap original:**

- **A1 (Phase 3): orchestration-only, no HTTP API clients en v0.** El roadmap §5.4 originalmente sugería "soportar OpenAI/Google/Anthropic en v0" con manejo de API keys. La realidad implementada es que el CLI prepara prompts, valida outputs contra schema, e integra con telemetría — pero NO invoca APIs. El operador pega los prompts en su auditor de elección manualmente. Razones documentadas en el commit message de PR #85: implementar 3 clientes HTTP es 1-2 semanas + mantenimiento perpetuo cuando cambian las APIs (premature para una v0 *experimental*); el patrón humano-en-el-loop coincide con `/plan-audit` de Sentinel; cumple principio #10 ("no es un LLM gateway"); cierra RFC #82 por diseño. Los HTTP clients se reabren en v1 cuando un adoptante real lo justifique con datos.

**Diferencias entre §5.5 como escrito y §5.5 como entregado:**

- §5.5 criterio 4 dice "una configuración monocromática es rechazada con error claro; una heterogénea procede sin advertencia". Como shippeado, la heterogeneidad inter-familia es **recomendación documentada** (CLI-REFERENCE.md `devtrail charter audit` section), **no auto-enforcement**. El criterio depende de A1: sin invocación de APIs, el CLI no sabe qué modelo usará el operador, así que no hay punto de inyección donde validar la heterogeneidad. Auto-enforcement queda como gate v1 cuando los HTTP clients se justifiquen.

**Items §6 (mapping Sentinel → CLI) shipped:**

- `dist/docs/examples/charters/CHARTER-01-anomaly-thresholds.md` (anonimizado de PLAN-05 de Sentinel) — ✅
- `dist/docs/examples/charters/CHARTER-02-baseline-recompute.md` (anonimizado de PLAN-06 de Sentinel) — ✅
- `dist/docs/examples/ailogs/AILOG-2026-01-15-001-anomaly-detector-introduction.md` (anonimizado del AILOG originador de CHARTER-01) — ✅

Estos viven fuera del manifest de `devtrail init` por diseño (decisión A6 del plan original): adopters los browse en GitHub o vía clone, no se auto-instalan como artefactos del framework.

**Open issues a la fecha:**

- **#93** — UX polish: inline `[suppressed]` annotation en bloque WARNING de drift. Abierto pendiente de validación empírica (próximo cycle de Sentinel).

**Items diferidos a v1 con criterio de salida explícito** (de §8 + de las decisiones de A1):

- HTTP API clients (Phase 3 v1, gated en demanda real de adopter).
- Auto-enforcement de heterogeneidad inter-familia (Phase 3 v1, dependiente de los HTTP clients).
- `--strict-scope` flag en drift (gated en fricción reportada por adopter).
- Forma estructurada multi-revisor `review:` array (gated en flujo multi-actor real).
- `charter.schema.v1.0` estable (gated en adopter de segundo dominio — el siguiente cycle de Sentinel en frontend toca exactamente este criterio).

---

## 1. Por qué este documento existe

`devtrail-cloud-proposal.md` v0.3 confirma que la tesis se sostiene con la evidencia de Sentinel y nombra qué decisiones la evidencia permite tomar ahora. Este documento traduce esas decisiones en una secuencia concreta de cambios al CLI Rust y al framework `dist/.devtrail/`. No incluye trabajo de Cloud ni features de aprobación condicional — esos quedan bloqueados hasta validación con un segundo proyecto en otro dominio o con flujo multi-actor.

El roadmap está pensado para tres ciclos de release del CLI (estimación: 3-4 patches/minor cada uno) que pueden ejecutarse en serie o con solapamiento mínimo. Cada fase es shippable independientemente y produce señal antes de la siguiente. Si la fase 1 no genera adopción medible, las fases 2-3 se cuestionan antes de invertir.

**Nota terminológica.** Lo que el experimento Sentinel llamó "Plan" (Plans 01-06) se llama **Charter** en el vocabulario DevTrail going-forward, para evitar colisión nominal con `plan.md` de GitHub SpecKit. Los registros históricos de Sentinel preservan "Plan"; este documento usa "Charter" para todo lo prospectivo y "Plan" cuando cita literalmente artefactos de Sentinel. Justificación completa en `que-es-un-charter.md` §2.

## 2. Secuencia recomendada y justificación

| Fase | Capacidad | Pre-requisitos | Estimación CLI |
|------|-----------|----------------|----------------|
| 1 | Charters como entidad de primera clase | Ninguno (artefactos ya validados) | cli-3.6.0 (minor) |
| 2 | Telemetría + drift-check ejecutable | Fase 1 | cli-3.7.0 (minor) |
| 3 | Auditoría externa multi-modelo | Fases 1+2 | cli-3.8.0 (minor) |

**Justificación de la secuencia:**

- Charters son la *unidad* del patrón; Telemetría y Auditoría son *observaciones sobre la unidad*. Sin la primera, las otras dos no tienen sujeto.
- Fase 1 es portar (los artefactos ya existen, validados con cero falsos positivos) — el riesgo técnico es bajo y el ROI inmediato es alto.
- Fase 2 es portar + Rust-ificar con decisión consciente de simplicidad (principio #9): el script bash de 145 líneas puede invocarse desde Rust o reimplementarse según convenga; ambas opciones son aceptables si preservan la propiedad "0 falsos positivos en 2/2 tests" demostrada en Sentinel.
- Fase 3 requiere diseño nuevo de orquestación multi-modelo (mayor riesgo); solo se aborda cuando las dos primeras hayan generado adopción.

## 3. Fase 1 — Charters como entidad de primera clase

**Objetivo:** que un usuario adoptante pueda crear, listar, navegar y validar Charters usando el CLI, sin tener que copiar manualmente el TEMPLATE de Sentinel.

### 3.1 Cambios al framework (`dist/.devtrail/`)

- **Portar `TEMPLATE.md v3`** desde `/E/Proyectos/StrangeDaysTech/sentinel/docs/plans/TEMPLATE.md` a `dist/.devtrail/templates/charter-template.md` con las 6 convenciones embebidas validadas (separación Local/Production checks, esfuerzo en TIEMPO, sub-secciones estructuradas, R<N+1>, Cierre del Charter, auto-checklist drift). El template renombra "Cierre del Plan" → "Cierre del Charter" en su cuerpo; las referencias del template a Sentinel quedan como ejemplos históricos.
- **Schema mínimo `dist/.devtrail/schemas/charter.schema.v0.json`** marcado *experimental*. Frontmatter mínimo: `charter_id`, `origin` (union: `originating_ailogs` array u `originating_spec` path), `trigger`, `effort_estimate` (XS/S/M/L), `status` (declared/in-progress/closed). El campo `origin` admite ambas formas para soportar tanto el modo Sentinel (Charter nacido de un AILOG) como el modo SpecKit-driver (Charter derivado de un `specs/###-feature/`); ver §3.5 para la justificación. El schema NO se cristaliza como `v1.0` hasta tener un segundo proyecto en otro dominio (ver `devtrail-thesis-validation.md` §6).
- **Charters canónicos como ejemplos** en `dist/docs/examples/charters/` (copia anonimizada de PLAN-05 y PLAN-06 de Sentinel, renombrados a `CHARTER-01` y `CHARTER-02` en los ejemplos para que adoptantes los lean como referencia de uso real bajo el vocabulario DevTrail).

### 3.2 Comandos nuevos del CLI

- `devtrail charter new [--type X|S|M|L] [--from-ailog ID | --from-spec PATH] [--title "..."]` — scaffolds un Charter desde el template. Si `--from-ailog` está presente, pre-popula `originating_ailogs` con el ID (caso post-MVP / mantenimiento, p.ej. los Plans 01-06 históricos de Sentinel). Si `--from-spec` está presente (p.ej. `specs/001-sentinel-mvp/`), pre-popula `originating_spec` apuntando al `spec.md` correspondiente y hereda User Stories relevantes a la sección Context (caso greenfield con SpecKit como driver). Los flags son mutuamente excluyentes; si ambos faltan, el Charter se crea sin origen y debe llenarse manualmente. Genera el archivo en `docs/charters/NN-slug.md` con `NN` autoincrementado. Justificación de los dos modos en §3.5.
- `devtrail charter list [--status declared|in-progress|closed|all]` — enumera Charters en `docs/charters/` con estado, esfuerzo declarado, AILOGs origen. Default: `--status all`.
- `devtrail charter status [CHARTER-ID]` — muestra detalle de un Charter: archivos declarados, AILOGs origen, telemetría si existe, drift-check si existe. Si se omite el ID, lista los últimos 5 Charters.

### 3.3 Integración con comandos existentes

- **`devtrail validate`**: añadir validación opt-in del frontmatter de Charters contra `charter.schema.v0.json` cuando el flag `--include-charters` esté presente (default: false, para no romper proyectos existentes sin Charters).
- **`devtrail explore` (TUI)**: añadir vista `Charters` paralela a `Documents`, con navegación por estado y búsqueda por AILOG origen. Usa `pulldown-cmark` ya disponible.

### 3.4 Tests y documentación

- Tests integration en `cli/tests/` que crean un proyecto temporal con `devtrail init`, ejercitan `charter new/list/status` en los tres caminos de origen (`--from-ailog`, `--from-spec` con un `specs/` mock, y sin flag), y verifican el shape del Charter generado contra el schema en cada caso.
- Sección `## Charters` nueva en `docs/adopters/CLI-REFERENCE.md` (EN + ES + zh-CN) con ejemplos de uso. Plantilla para README. La sección debe incluir una nota corta explicando que en docs históricos de Sentinel y en evidencia empírica del experimento `/plan-audit`, este artefacto aparece bajo el nombre "Plan".

### 3.5 Coexistencia con flujos SpecKit

DevTrail llamó originalmente a este artefacto "Plan" durante el experimento Sentinel; se renombró a **Charter** precisamente para evitar la colisión nominal con `plan.md` de SpecKit, que es un artefacto distinto (feature-completo y arquitectónico, más cercano a un ADR + skeleton de proyecto). El Charter DevTrail es una unidad acotada con verificación ejecutable, declaración de archivos y ancla de auditoría ex-post. No compiten — viven en momentos distintos del ciclo. SpecKit termina al producir `tasks.md`; DevTrail Charter empieza ahí (cuando hay spec previa) o se sostiene solo (cuando no la hay). El alcance conceptual completo y la comparación pieza a pieza viven en `que-es-un-charter.md`.

**Tres modos de coexistencia que Fase 1 debe soportar de fábrica:**

1. **Greenfield con SpecKit como driver.** Un Charter DevTrail toma un subconjunto de `tasks.md` (típicamente una user story o una fase) y le añade verificación + drift + audit. Lo cubre el flag `--from-spec PATH` (§3.2): pre-popula `originating_spec` y hereda User Stories al Context.
2. **Mantenimiento / post-MVP.** No hay SpecKit upstream — el Charter nace de un AILOG. Es el caso real de Sentinel (Plans 01-06). Lo cubre el flag `--from-ailog ID` (§3.2).
3. **Híbrido.** Features mayores con flujo SpecKit completo + Charters DevTrail; bug fixes, gobernanza, deuda y features chicas con solo Charter DevTrail. Probablemente el más realista en la práctica; no requiere flags adicionales — emerge de combinar los dos modos anteriores según el tipo de trabajo.

**Implicaciones de diseño:**

- `charter.schema.v0.json` (§3.1) declara `origin` como union `originating_ailogs | originating_spec` precisamente para no privilegiar un modo sobre el otro.
- `devtrail validate --include-charters` (§3.3) acepta ambas formas de `origin`; un Charter sin ninguna falla con error explicativo.
- `devtrail charter list` (§3.2) puede agrupar por origen cuando sea útil (p.ej. `devtrail charter list --origin spec` para ver solo Charters derivados de SpecKit).
- La integración con SpecKit es de *lectura* — el CLI DevTrail no genera ni modifica artefactos de SpecKit, solo los referencia. Esto preserva el principio de no acoplar DevTrail a un flujo de planning particular.

### 3.6 Criterios de salida de la Fase 1

- ✅ `devtrail charter new` genera un Charter compatible con `check-plan-drift.sh` (Sentinel) ejecutado manualmente — la compatibilidad sintáctica se preserva aunque cambien los nombres canónicos en el framework. Se valida en ambos modos (`--from-ailog` y `--from-spec`). *Validado empíricamente por Sentinel CHARTER-02..06.*
- ⏳ Al menos 1 adoptante (idealmente fuera de Go) ha creado un Charter completo con `devtrail charter` y reportado sobre la experiencia. *Sentinel (Go-backend) ha cerrado 6 Charters; el siguiente cycle se realizará en un subproyecto de frontend, abriendo el segundo-dominio.*
- ✅ Schema `charter.schema.v0.json` no ha requerido breaking changes en 2 ciclos de release. *Cumplido — 0 breaking changes a través de fw-4.4.0 → fw-4.7.1.*

## 4. Fase 2 — Telemetría y drift-check ejecutable

**Objetivo:** medir validez del patrón en proyectos adoptantes y detectar drift Charter vs commits sin requerir scripts externos.

### 4.1 Cambios al framework

- **Schema `dist/.devtrail/schemas/charter-telemetry.schema.v0.json`** derivado de `devtrail-charter-telemetry.md` v0.2 con los 4 campos refinados por Sentinel: `external_audit` como array, `outcome.scope_change_notes` con codificación `F1...FN`, `agent_quality.r_n_plus_one_emergent_count`, `qualitative.format_iteration`.

### 4.2 Comandos nuevos del CLI

- `devtrail charter close [CHARTER-ID]` — guía interactiva (estilo `git commit` con prompts) para llenar la telemetría YAML al cierre del Charter. Pregunta campo por campo, valida tipos contra schema, escribe el archivo `.devtrail/charters/CHARTER-ID.telemetry.yaml`. Tiempo objetivo del flujo: 5-10 min (mismo target que `devtrail-charter-telemetry.md` declara).
- `devtrail charter drift [CHARTER-ID] [--range REV..REV]` — reimplementación o invocación del `check-plan-drift.sh` de Sentinel. Decisión técnica abierta (principio #9): invocar el script bash directamente desde Rust si está disponible, o reimplementar nativamente preservando la propiedad "0 falsos positivos en 2/2 tests". El usuario *adoptante* no debe percibir diferencia.

### 4.3 AILOG-awareness para reducir ceremonia

`AILOG-022` §Risk R2 documentó que el script genera ruido cuando alerta sobre R<N> ya documentados en AILOG. La fase 2 debe atacar esa ceremonia (`devtrail-design-principles.md` v0.2 §6 distingue virtud vs ceremonia):

- `devtrail charter drift` debe leer los AILOGs referenciados en el frontmatter del Charter y suprimir alertas sobre paths ya documentados como `R<N>` en algún AILOG. Esta es la mitigación R2 que Sentinel propuso pero no implementó.

### 4.4 Hook opcional pre-PR

- `dist/.devtrail/hooks/pre-pr.sh` que ejecuta `devtrail charter drift` automáticamente antes de abrir un PR. Opt-in via `devtrail init --hooks` o configuración manual. NO es default (principio #6: fricción virtuosa, pero no impuesta sin consentimiento).

### 4.5 Criterios de salida de la Fase 2

- ⏳ Al menos 2 adoptantes han usado `devtrail charter close` y producido telemetría YAML válida. *Solo Sentinel hasta hoy (6 telemetrías cerradas: CHARTER-01..06). Se cumple cuando un adopter de segundo dominio cierre al menos 1 Charter.*
- ✅ `devtrail charter drift` mantiene la propiedad de 0 falsos positivos en proyectos adoptantes. *Validado empíricamente por Sentinel CHARTER-02..06; el único falso positivo histórico (F3 — regex column-1) se cerró en `fw-4.6.1`.*
- ✅ AILOG-awareness reduce el triage manual a cero en al menos 1 caso real reportado. *Validado por Sentinel CHARTER-06 (ver issue #91 para evidencia: drift detectó `subscriber.go` declarado pero no modificado, y la supresión AILOG-aware eliminó el triage manual al 100%).*

## 5. Fase 3 — Auditoría externa multi-modelo

**Objetivo:** cerrar el ciclo cualitativo del experimento Sentinel orquestando auditoría dual (estilo Copilot + Gemini + claude-analisis) sobre Charters cerrados.

### 5.1 Diseño de tres niveles (no solo "ejecuta N modelos")

`devtrail-thesis-validation.md` §7.3 argumenta que la auditoría dual de Sentinel reveló un patrón de tres niveles: dos auditores en paralelo + un calibrador-reconciliador (claude-analisis) que aplica el schema definicional. La fase 3 implementa esa arquitectura, no una orquestación plana de N modelos.

### 5.2 Heterogeneidad inter-familia como restricción de diseño

La auditoría externa **no es sustituible por auto-auditoría del mismo modelo**, y la razón no es operativa — es estadística y estructural. El comando `devtrail charter audit` debe imponer la restricción de heterogeneidad inter-familia desde el inicio, no como configuración opcional.

**Tres bases teóricas que sostienen la restricción:**

1. **Blind spots correlacionados con la familia de modelo.** Cualquier patrón que un modelo tenga sobre-representado en su distribución de entrenamiento (una API ficticia que aparece en muchos tutoriales, una convención arquitectónica que predomina en stacks populares) es por definición invisible a *otra instancia del mismo modelo*. F5 de PLAN-05 es exactamente este caso: Claude generó el Plan asumiendo que `NewAnomalyDetector` se instancia en `service.go` (patrón mainstream); otra sesión de Claude muy probablemente lo dejaría pasar porque comparte el mismo prior. Gemini, con distribución de entrenamiento distinta, lo capturó.

2. **Sycophancy y agreement bias en cadena.** Los LLMs muestran preferencia documentada por estar de acuerdo con texto autoritativo presente en el contexto. Si un modelo lee un Charter + AILOG generados por sí mismo, los trata como premisas válidas más que como hipótesis a interrogar. La auditoría externa rompe esa cadena porque el auditor no tiene continuidad narrativa con el implementador.

3. **Convergencia inter-modelo como señal estructural.** Lo que Sentinel mostró empíricamente — Copilot 9.25 / Gemini 9.5 convergiendo en PLAN-05 sobre los mismos findings, capturando F4 y F5 que el AILOG no documentó — es que la *diversidad* es estructural, no aditiva. Esa convergencia no se reproduce entre dos auditorías del mismo modelo porque comparten el sesgo.

**El matiz importante: no toda auto-auditoría está descalificada.** Para *checks estructurales* (drift de archivos declarados vs modificados, consistencia interna del Charter, syntax/type checks), un mismo modelo se audita bien — `check-plan-drift.sh` no necesita Gemini para funcionar y la Fase 2 es deliberadamente single-model. Para *checks semánticos* (¿esta arquitectura es correcta?, ¿esta categorización es `hallucination` vs `implementation_gap`?), la heterogeneidad es lo que captura la calibración. La distinción es la misma virtud-vs-ceremonia que `devtrail-design-principles.md` v0.2 §6 articula: la heterogeneidad inter-familia es virtud cuando externaliza signal donde el sesgo es probable; es ceremonia cuando solo agrega latencia y costo en checks estructurales.

**Implicación operacional para `devtrail charter audit`:**

- Si el implementador fue de la familia X (ej: Claude), al menos uno de los dos auditores debe ser de una familia distinta a X. El comando rechaza con error explicativo configuraciones donde implementador y ambos auditores son de la misma familia.
- El calibrador-reconciliador puede ser de cualquier familia (incluida la del implementador) porque su trabajo es aplicar el schema definicional sobre veredictos ya producidos, no descubrir gaps.
- La detección de "familia" se hace por mapeo de model-id → family declarado en una tabla del framework (`dist/.devtrail/audit-prompts/model-families.yaml` con entradas como `claude-* → anthropic`, `gpt-*/copilot-* → openai`, `gemini-* → google`). El usuario puede extender la tabla para nuevos modelos.

### 5.3 Cambios al framework

- **Plantillas de prompt en `dist/.devtrail/audit-prompts/`**: una por rol (`auditor-primary.md`, `auditor-secondary.md`, `calibrator-reconciler.md`). Las plantillas se derivan de las que Sentinel usó implícitamente; se documentan formalmente.
- **Schema de output canónico**: cada auditor produce un archivo en `audit/charters/{CHARTER-ID}/{role}.md` con frontmatter parseable que se mapea al campo `external_audit` de la telemetría. El calibrador produce `claude-analisis.md` (o equivalente) con consolidación de findings.

### 5.4 Comando nuevo del CLI

- `devtrail charter audit [CHARTER-ID] [--auditors model1,model2] [--calibrator model3] [--implementer-family X]` — orquesta la ejecución secuencial: auditor primary → auditor secondary (en paralelo si la API lo permite) → calibrator que reconcilia veredictos divergentes según el schema. Genera los 3 archivos de output. Integra automáticamente el campo `external_audit` array en la telemetría. Aplica la restricción de heterogeneidad inter-familia descrita en §5.2: si el flag `--implementer-family` no está presente, el comando lo infiere del último AILOG asociado al Charter; si la inferencia y la configuración resultan en una combinación monocromática (implementador + ambos auditores misma familia), el comando rechaza con error explicativo y sugiere alternativas.

Decisión abierta: qué APIs/modelos soportar. Recomendación inicial: usar el patrón "user provides the API key" (similar a herramientas como `aichat`), no acoplar el CLI a un proveedor específico. Soportar al menos OpenAI (Copilot stand-in), Google (Gemini), Anthropic (Claude) en v0.

> **Decisión arquitectónica A1 al implementar (ver §0):** los HTTP clients se difirieron a v1 — la versión shippeada (`fw-4.7.0`/`cli-3.8.0`) es **orchestration-only**: el CLI prepara prompts, valida outputs contra schema, e integra con telemetría, pero no invoca APIs. El operador pega los prompts en su auditor de elección manualmente. El flag `--implementer-family` y la heterogeneidad inter-familia descritos arriba quedan como recomendación documentada en CLI-REFERENCE.md hasta que los HTTP clients aterricen en v1.

### 5.5 Criterios de salida de la Fase 3

- ⏳ Al menos 1 ciclo de auditoría externa multi-modelo ejecutada en un proyecto adoptante con resultados consistentes con la calibración cross-modelo observada en Sentinel. *Pendiente — Sentinel todavía no ejercita `devtrail charter audit` sobre la nueva versión `fw-4.7.x`. El próximo cycle de telemetría (frontend) lo cubre.*
- ✅ El calibrador-reconciliador produce findings en formato compatible con el array `external_audit` de la telemetría. *Schema `audit-output.schema.v0.json` enforza compatibilidad por construcción; integración con telemetry validated por la integration test del 3-step flow.*
- ✅ Documentación clara de que el CLI orquesta pero no provee modelos; el usuario controla qué APIs usar. *CLI-REFERENCE.md `### devtrail charter audit` lo declara explícito; CHANGELOG fw-4.7.0 documenta la decisión A1.*
- ⏳ La restricción de heterogeneidad inter-familia se ejercita en los tests integration: una configuración monocromática es rechazada con error claro; una heterogénea procede sin advertencia. *Diferido a v1 por la decisión A1 (orchestration-only) — sin invocación de APIs, el CLI no conoce los modelos del operador, así que no hay punto de inyección donde validar. La heterogeneidad queda como recomendación documentada (CLI-REFERENCE.md) hasta que los HTTP clients aterricen.*

## 6. Mapeo artefactos Sentinel → CLI

Tabla de referencia rápida que conecta cada artefacto validado en Sentinel con su destino en el CLI/framework. La columna izquierda preserva el vocabulario histórico de Sentinel ("Plan"); la columna derecha usa el vocabulario DevTrail going-forward ("Charter").

| Artefacto Sentinel (vocabulario "Plan") | Ruta absoluta de origen | Fase | Destino DevTrail (vocabulario "Charter") | Estado |
|---------------------|--------------------------|------|---------|---|
| `TEMPLATE.md` v3 | `sentinel/docs/plans/TEMPLATE.md` | 1 | `dist/.devtrail/templates/charter-template.md` | ✅ shipped (`fw-4.4.0`) |
| Plan-docs canónicos | `sentinel/docs/plans/{05,06}-*.md` | 1 | `dist/docs/examples/charters/CHARTER-{01,02}-*.md` (anonimizados) | ✅ shipped |
| Telemetrías YAML | `sentinel/.devtrail/plans/PLAN-{05,06}.telemetry.yaml` | 2 | Schema validador para `devtrail charter close` output | ✅ shipped (`fw-4.6.0`) |
| `check-plan-drift.sh` | `sentinel/scripts/check-plan-drift.sh` | 2 | `devtrail charter drift` (reimplementación nativa Rust) | ✅ shipped (`cli-3.7.0`) |
| Reportes auditoría dual | `sentinel/audit/plans/{05,06}/{copilot,gemini,claude-analisis}.md` | 3 | Output canónico de `devtrail charter audit` (orchestration-only en v0; ver A1 §0) | ✅ shipped (`cli-3.8.0`) |
| AILOG canónico originador | `sentinel/.devtrail/07-ai-audit/agent-logs/AILOG-2026-04-24-010-pm008-anomaly-detector.md` | Referencia transversal | `dist/docs/examples/ailogs/AILOG-2026-01-15-001-anomaly-detector-introduction.md` (anonimizado, par de `CHARTER-01`) | ✅ shipped |

## 7. Verificación end-to-end de cada fase

Cada fase tiene un test de aceptación operacional que el equipo de DevTrail debe poder ejecutar sin instrucciones adicionales:

**Fase 1:**
1. `cargo install --path cli/` instala `devtrail` con la nueva subcomanda `charter`.
2. En un repo limpio: `devtrail init && devtrail charter new --type M --title "test charter"` produce un Charter en `docs/charters/01-test-charter.md` válido contra `charter.schema.v0.json`.
3. `devtrail charter list` muestra el Charter recién creado en estado `declared`.
4. `devtrail validate --include-charters` pasa sin errores.

**Fase 2:**
1. Tras Fase 1, un commit que toca archivos del Charter + `devtrail charter drift CHARTER-01` reporta 0 drift.
2. Se introduce un drift artificial (declarar archivo extra que no se modifica); el comando reporta 1 omisión y exit code 1.
3. Se documenta el drift en un AILOG asociado; el comando con AILOG-awareness lo suprime y reporta limpio.
4. `devtrail charter close CHARTER-01` produce un YAML válido contra `charter-telemetry.schema.v0.json`.

**Fase 3:**
1. `devtrail charter audit CHARTER-01 --auditors copilot,gemini --calibrator claude` produce 3 archivos de output bien formados.
2. El calibrador reconcilia veredictos divergentes (caso de prueba: F5 PLAN-05-style donde un auditor categoriza como `implementation_gap` y otro como `hallucination`).
3. La telemetría queda enriquecida con el campo `external_audit` array correctamente poblado.

## 8. Lo que NO está en este roadmap (preservación del principio #12)

Para mantener disciplina de cristalización experimental:

- **No `charter.schema.v1.0` estable** — solo `v0` experimental hasta validación con segundo proyecto en otro dominio (`devtrail-thesis-validation.md` §6).
- **No features de aprobación condicional** — supuesto #4 sin evidencia (`devtrail-thesis-validation.md` §4.4).
- **No firma criptográfica de Charters** — la prioridad criptográfica vive en `devtrail stage close` (Cloud roadmap), no en Charters aún.
- **No integración Cloud para Charters** — primero validar el flujo local en CLI con adoptantes; la integración con Inbox y Trust Center sigue al uso real.
- **No comando `devtrail charter extract` (auto-generar Charters desde AILOGs)** — el patrón existe en `cloud-proposal.md` §4.5 pero requiere observación adicional sobre cómo los adoptantes generan Charters en práctica antes de cristalizar la heurística de extracción.

Cada uno de estos puntos tiene un criterio de salida explícito en `devtrail-thesis-validation.md` §8 que, cuando se cumpla, abre la puerta a un nuevo ciclo de roadmap.

---

*Este roadmap es el primer artefacto que traduce evidencia empírica en código accionable para DevTrail. Su evolución sigue el patrón auto-evolutivo observado en Sentinel: cada fase ejecutada genera datos que refinan el roadmap del próximo ciclo. La versión 0.3 (este documento) se escribió tras completar las 3 fases en `fw-4.4.0` → `fw-4.7.1` con Sentinel como adoptante único; la próxima versión (0.4) se escribirá cuando el segundo dominio (subproyecto frontend de Sentinel, agendado) cierre al menos 1 Charter completo y aporte señal sobre qué del schema `v0` debe promoverse a `v1`, qué frictions emergen en un stack distinto a Go, y si los HTTP clients se justifican antes de v1 estable.*
