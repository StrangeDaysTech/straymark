# Qué es un "Charter" en DevTrail (y dónde encaja respecto a GitHub SpecKit)

**Versión:** 0.2 (rename Plan → Charter)
**Fecha:** 30 de abril de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Fijar el alcance conceptual del artefacto **Charter** tal como emerge del experimento Sentinel (donde se llamó *Plan*), y explicar cómo coexiste con flujos de diseño y desarrollo que ya usan SpecKit (spec → plan → tasks).
**Documentos relacionados:** `devtrail-cli-roadmap.md` (§3 Fase 1, §6 mapeo Sentinel→CLI), `devtrail-thesis-validation.md`, `devtrail-charter-telemetry.md`, `sentinel/docs/plans/TEMPLATE.md` v3, `sentinel/docs/plans/README.md`.

---

## 1. Qué es un Charter en DevTrail

**Una declaración ex-ante de una unidad de trabajo, lo bastante grande como para merecer un contrato verificable y una auditoría ex-post.**

El Charter tiene tres caras simultáneas:

1. **Contrato de scope** (Context + Scope in/out + Archivos a modificar + Riesgos): qué se va a tocar y, deliberadamente, qué *no*. La sección `Out of scope` existe específicamente para que auditores externos no lo clasifiquen como gap.
2. **Contrato de verificación** (Local checks + Production smoke): comandos ejecutables literal, separados por *cuándo aplican*. El split Local/Prod no es estilo — emerge de que en 3/3 ciclos los auditores externos clasificaban fallos prod-only como `real_debt`.
3. **Ancla de auditoría** (frontmatter + Cierre del Charter + drift check): el Charter es el sujeto sobre el que `devtrail charter drift` y `devtrail charter audit` operan. Sin Charter no hay nada que auditar.

### 1.1 No es solo deuda técnica

De los Plans cerrados de Sentinel (que son la evidencia empírica del patrón Charter), solo uno (`03-contract-bumps`) huele a deuda. El resto son features (`02-admin-endpoint`, `05-per-service-anomaly-thresholds`), trabajo de gobernanza (`01-deploy-governance`) o trabajo operacional (`06-baseline-recompute-job`). El README de `sentinel/docs/plans/` lo dice literal: *"Roadmap de trabajo identificado durante el cierre del backlog post-MVP"* — backlog, no debt-log.

### 1.2 Cuándo amerita un Charter vs. solo un AILOG

Inferido del corpus, no escrito formalmente:

- **Solo AILOG**: cambio que cabe en una sola sesión, sin pre-declaración de archivos, riesgo estructural bajo (typo, fix puntual, refactor local).
- **Charter + AILOG**: el trabajo toca múltiples archivos, tiene riesgos enumerables ex-ante, requiere verificación con comandos ejecutables, y se beneficia de auditoría externa post-merge. Esfuerzo XS–L (minutos a días).
- **ADR + uno o más Charters**: decisión arquitectónica de fondo, donde cada Charter ejecuta una porción del ADR.

### 1.3 Atomic Charter closure (format v4)

A partir de fw-4.4.2, el `charter-template.md` adopta **format v4**: cuando el drift check pre-commit (Tasks #7) reporta drift entre `## Files to modify` y los archivos efectivamente modificados, el implementor debe actualizar el Charter atómicamente — en el mismo commit/PR que la implementación — añadiendo un bloque `## Closing notes` que documente qué cambió, por qué, y referencia al AILOG correspondiente. **No se difiere a un housekeeping PR post-merge.**

El patrón emergió empíricamente de PLAN-07 de Sentinel (cerrado 2026-05-02), donde un drift de omisión legítimo (un live test resultó agnóstico al cambio, así que un archivo declarado en §Archivos a modificar no se tocó) se documentó tarde via housekeeping PR separado, dejando el Plan-doc stale por días. AIDEC-2026-05-02-001 de Sentinel formalizó el gap y propuso format v4; este Charter (`que-es-un-charter.md`) refleja la evolución upstream. PLAN-05 (`docs/plans/05-per-service-anomaly-thresholds.md`) y PLAN-07 mismo son los ejemplos canónicos del patrón `## Closing notes` aplicado retrospectivamente — ambos en Sentinel.

---

## 2. Por qué se llama "Charter" y no "Plan"

DevTrail llamó originalmente a este artefacto **Plan** durante el experimento Sentinel (abril 2026). El término se renombró a **Charter** antes de la implementación en el CLI por una razón pragmática: GitHub SpecKit ya usa la palabra "plan" (`/speckit.plan`, `plan.md`) para un artefacto distinto, y la colisión nominal generaba fricción en docs adopters que combinan ambos flujos.

Los registros históricos de Sentinel (Plans 01-06, `sentinel/docs/plans/`, `sentinel/scripts/check-plan-drift.sh`, AILOGs y telemetrías YAML `PLAN-NN.telemetry.yaml`) preservan el nombre original — son evidencia empírica de un momento puntual en el tiempo, y reescribir la historia falsificaría el registro. Las referencias a esos archivos en este documento mantienen "Plan" deliberadamente.

El concepto que SpecKit llama "plan" y el que DevTrail llama "Charter" son artefactos distintos:

| | **SpecKit `plan.md`** | **DevTrail Charter** |
|---|---|---|
| Granularidad | Feature completa (semanas, varias historias) | Unidad acotada (horas a días, una rama) |
| Contenido dominante | Stack, dependencias, estructura de proyecto, gates de constitución | Archivos concretos a tocar, comandos de verificación, riesgos R<N> |
| Sub-artefactos | Genera `research.md`, `data-model.md`, `quickstart.md`, `contracts/` | Genera (al ejecutarse) un AILOG |
| Validación | Constitution Check (gate ex-ante) | Drift-check + auditoría externa (gates ex-post) |
| Optimización | *Claridad upfront* | *Trazabilidad ex-post* |

Lo que SpecKit llama "plan" se parece más a un **ADR + skeleton arquitectónico**. Lo que DevTrail llama "Charter" se parece más a un **task-card con verificación contractual y ancla de auditoría**.

---

## 3. El flujo SpecKit, mapeado pieza a pieza

SpecKit es lineal y top-down: `/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → implementar.

| SpecKit | Qué resuelve | Equivalente más cercano en DevTrail |
|---|---|---|
| `spec.md` (US, FRs, SCs) | *Qué* construir, agnóstico a tech | `01-requirements/REQ-*.md` |
| `plan.md` (stack, estructura) | *Cómo* arquitectarlo | `02-design/ADR-*.md` |
| `tasks.md` (T001..TNNN) | *En qué orden* ejecutarlo | sección `## Tasks` dentro de un Charter DevTrail |
| (no existe) | Bitácora de ejecución | **AILOG** |
| (no existe) | Drift declarado vs. ejecutado | **`devtrail charter drift`** |
| (no existe) | Auditoría multi-modelo ex-post | **`devtrail charter audit`** |

Las tres últimas filas son la contribución específica de DevTrail. SpecKit termina al producir `tasks.md`; lo que pasa después no le importa al framework — si la implementación se desvía del plan, no hay mecanismo que lo detecte. DevTrail entra exactamente ahí.

---

## 4. Tres modos en que un Charter DevTrail puede convivir con SpecKit

### Modo A — Greenfield con SpecKit como driver

```
spec.md  →  plan.md  →  tasks.md  →  ┐
                                     ├→  Charter-NN (cubre US1 + verificación + riesgos)
                                     ├→  Charter-NN+1 (cubre US2 + verificación + riesgos)
                                     └→  Charter-NN+2 (cubre US3 + verificación + riesgos)
                                                ↓
                                         AILOG por Charter
                                                ↓
                                         drift-check + audit
```

Cada Charter DevTrail toma un subconjunto de tasks (típicamente una user story o una fase) y le añade las tres capas que SpecKit no provee: verificación ejecutable, declaración de archivos contra la que medir drift, y enganche con AILOG/auditoría.

### Modo B — Mantenimiento / post-MVP (caso real de Sentinel)

No hay SpecKit corriendo. Los Plans 01–06 de Sentinel nacen de AILOGs y backlog post-MVP, no de specs. El campo `Origen:` del Charter apunta a un AILOG, no a un `specs/###-feature/`. Aquí el Charter DevTrail es freestanding.

### Modo C — Híbrido (probablemente el más realista)

- Features mayores → flujo SpecKit completo + Charters DevTrail al ejecutar tasks.
- Bug fixes, gobernanza, deuda, cambios de contrato, features chicas → solo Charter DevTrail.

Esto es coherente con lo que ya muestra Sentinel: de los Plans cerrados, ninguno tenía SpecKit upstream porque el MVP ya estaba spec'd y cerrado. Lo que quedaba era trabajo evolutivo, donde el ciclo Spec→Plan→Tasks es desproporcionado.

---

## 5. La diferencia conceptual de fondo

**SpecKit responde a la pregunta:** *"¿Cómo descompongo una idea de producto en trabajo ejecutable?"* — optimiza para claridad upfront y paralelización de US.

**DevTrail Charter responde a la pregunta:** *"¿Cómo hago que una unidad de trabajo sea verificable, auditable y detectable cuando se desvía de lo declarado?"* — optimiza para evidencia ex-post.

Por eso no compiten: viven en momentos distintos del ciclo. SpecKit acaba en `tasks.md`; DevTrail Charter empieza ahí (cuando se aplica) o se sostiene solo (cuando no hay spec previa). La tesis Sentinel no es "esto reemplaza SpecKit" — es "esto cubre la fase post-tasks que SpecKit no cubre, y la cubre con un patrón de tres niveles (declaración + drift + auditoría heterogénea) que tiene evidencia empírica de capturar gaps que el implementador no documentó".

---

## 6. Nota sobre la visión Studio: "Stage" no es sinónimo de Charter

`devtrail-studio-vision.md` introduce un término adicional, **Stage**, que conviene no confundir con Charter. No son sinónimos: viven en niveles distintos del ciclo.

- **Stage** = etapa de trabajo que recorre las seis fases cognitivas del Studio (Spec → Implementation → Audit → Remediation → Governance → Closure & Deliver) y se cierra con un Closure Bundle firmado y hash-chained. Comandos propuestos en la visión: `devtrail stage open/close/status` (no implementados; corresponden al Hito 2 de la evolución hacia Studio).
- **Charter** = unidad acotada con verificación + drift + ancla de auditoría — el artefacto que define este documento. Los Charters viven *dentro de* un Stage; un Stage típicamente contiene uno o varios Charters más sus tasks asociados.

La taxonomía implícita de la visión es de tres niveles: **Stage > Charter > Task**. El roadmap actual del CLI (`devtrail-cli-roadmap.md`, Fases 1-3) opera enteramente al nivel de Charter (lo que Sentinel llamó "Plan"), que es el nivel donde existe evidencia empírica de Sentinel. Stage entraría al CLI más adelante como capa adicional, no como rename de Charter.

## 7. Implicación para el roadmap del CLI DevTrail

Vale la pena reflejarlo en cómo se documenta el comando `devtrail charter new` cuando llegue Fase 1 (`devtrail-cli-roadmap.md` §3.2). Hoy el flag previsto `--from-ailog ID` cubre el Modo B. Probablemente convenga tener también `--from-spec specs/###-feature/` para el Modo A — pre-popula `Origen:` apuntando al `spec.md` de SpecKit, y opcionalmente hereda US relevantes a la sección Context. Eso es ejercicio para Fase 1, no para resolver hoy, pero conviene anotarlo en este documento como decisión abierta para cuando se diseñe la subcomanda.

---

*Este documento captura el alcance conceptual del artefacto Charter (originalmente llamado "Plan" en el experimento Sentinel) tal como se entiende a finales de abril de 2026. La definición seguirá evolucionando conforme adoptantes fuera de Sentinel ejerciten el patrón en otros dominios — la versión 0.3 se escribirá tras observar al menos un proyecto adoptante completar un Charter con `devtrail charter` (criterio de salida de Fase 1).*
