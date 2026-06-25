---
charter_id: CHARTER-01-coherence-bridge
status: closed
closed_at: "2026-06-25"
effort_estimate: L
trigger: "Issue #304 — deriva real de contrato cross-spec en Sentinel (adopter); más los casos telemetría-mockup y PolicyEngine. Ninguna herramienta compara la arquitectura intencionada (SpecKit) con la emergente (StrayMark/Loom)."
originating_spec: experiment-baton/specs/001-coherence-bridge/spec.md
originating_concept: experiment-baton/01-baton-concept.md
originating_research: experiment-baton/02-speckit-integration-research.md
related_issues: [304, 303]
---

# Charter: Baton Fase 1 — Coherence Bridge (intención SpecKit ↔ gobernanza ↔ arquitectura), read-only

> **Status (espejo del frontmatter; la fuente de verdad es el frontmatter):** closed (2026-06-25). Effort: L.
> **Origen:** concepto [01-baton-concept.md](01-baton-concept.md) §4.1 + investigación [02-speckit-integration-research.md](02-speckit-integration-research.md). Caza el patrón del issue #304.
> **Encuadre:** primera fase del experimento Baton. **No toca modelos.** Prototipa en `experiment-baton/`, con la intención explícita de graduar la lógica pura/typed a `straymark-core` una vez validada.

## Context

StrayMark gobierna la implementación (charters, AILOGs, follow-ups, TDEs) y Loom proyecta la arquitectura **emergente** desde esas señales + el código en disco. Pero la arquitectura **intencionada** —el plan global que vive en los artefactos de SpecKit— nunca entra al grafo. Verificado: el Charter solo valida que `originating_spec` exista, nunca parsea su contenido; la proyección de Loom (`core/src/architecture/projection.rs`) nunca lee `specs/` ni `.specify/memory/`.

La consecuencia es deriva silenciosa. El issue #304 la documenta con un caso real de Sentinel: una decisión post-MVP (PM-002, en el backlog del spec 001) extendió un contrato de salud per-componente; el spec 005 (frontend) nunca la referenció, implementó un contrato **asumido** (campos y enums equivocados, métricas inexistentes), los mocks codificaron la suposición, los tests pasaron en verde, y el bug solo apareció en staging (`TypeError: t.find is not a function`). El mismo mecanismo produjo la telemetría mockup y la dispersión del módulo PolicyEngine (cuya arquitectura está documentada en `.specify/memory/Arquitectura - PolicyEngine.md` pero nunca se implementó como componente).

Esta fase construye la **costura de lectura** (de las tres del documento de investigación): ingerir la intención de SpecKit, reconciliarla contra la gobernanza y el código, y emitir un diagnóstico de coherencia. Read-only: diagnostica, no muta SpecKit ni ejecuta agentes.

**Decisión de placement a confirmar en ejecución:** el concepto (§10.6) sitúa el Coherence Bridge en `core`. Para no acoplar `straymark-core` a código experimental, esta fase lo prototipa como crate/módulo del experimento (`straymark-baton`, espejo de cómo `straymark-loom` vive aparte), exponiendo la lógica pura para graduarla a `core` después. Ver R2.

## Scope

**In scope:**

1. **Adaptador SpecKit read-only, versionado.** Parsea, anclado a `speckit_version` / `integration_state_schema`: `specs/**/{spec,plan,tasks}.md`, `specs/**/post-mvp-backlog.md`, `specs/**/contracts/**`, y `.specify/{extensions.yml,integration.json,memory/**}`. Parsing tolerante para el markdown libre de `.specify/memory/` (tratado como pista, no contrato duro).
2. **Modelo de intención (tercer plano).** Representación typed de componentes/módulos/contratos **declarados** por SpecKit (incluye decisiones de backlog post-MVP tipo PM-002). Reusa los tipos del modelo de arquitectura de Loom donde aplique; no duplica `glob_match`/`drift`.
3. **Edges de procedencia de contrato cross-spec** (el corazón de #304). Vincula un consumidor (spec/FR) con el productor/decisión (handler/PM/AILOG) que define el contrato. Inferidos en esta fase; declaración explícita opcional como extensión futura.
4. **Diagnóstico de coherencia (read-only).** Clases de finding de alta confianza primero: (a) componente intencionado en `.specify/memory/` sin archivos que lo implementen (PolicyEngine); (b) campo de contrato exigido por un consumidor sin fuente productora (las métricas fantasma); (c) consumidor construido contra una forma que una decisión posterior cambió (PM-002 vs spec 005). Severidad conservadora; anti-ruido.
5. **Overlay de intención en Loom.** Extiende la proyección a *intención vs. gobernanza vs. código*; el operador ve dónde el plano intencionado y el emergente divergen.
6. **Superficie CLI mínima** para emitir el diagnóstico (text/json/markdown; exit ≠ 0 si hay findings → CI-gateable), espejo de `architecture validate`.
7. **Dogfood read-only en Sentinel.** El caso #304 debe reproducirse como finding emitido; documentar en AILOG.

**Out of scope** (fases posteriores o explícitamente fuera):

- Cualquier routing/tier/presupuesto/token/costo de modelos → Fase 2+ del concepto.
- Escribir patches de vuelta a SpecKit (esta fase es read-only; diagnóstico, sin mutación).
- Enviar una **extensión/hook** de SpecKit (la costura de *activación*, `before_implement` en autoría) → Charter siguiente.
- Graduar la lógica a `straymark-core` → tras validación (R2).
- Ejecutar agentes / clasificar tareas.
- Reimplementar `speckit.analyze` (coherencia intra-spec; Baton cubre cross-spec + spec↔código↔gobernanza).

## Files to modify

| File | Change |
|---|---|
| `experiment-baton/specs/001-coherence-bridge/spec.md` | New — spec SpecKit (WHAT): modelo de intención, edges de procedencia, clases de finding, contrato CLI |
| `experiment-baton/specs/001-coherence-bridge/plan.md` | New — HOW: fasing por batch, placement crate vs core, parsing tolerante de memory |
| `experiment-baton/specs/001-coherence-bridge/tasks.md` | New — tareas ordenadas B1–B5 |
| `experiment-baton/Cargo.toml` | New — crate `straymark-baton` (binario del diagnóstico), miembro del workspace |
| `experiment-baton/src/**` | New — adaptador SpecKit, modelo de intención, motor de coherencia, CLI |
| `core/src/architecture/projection.rs` | Read-only consumido; posible punto de extensión typed si la graduación lo amerita (decidir en ejecución, no clobber) |
| `Cargo.toml` (workspace root) | Add `experiment-baton` a `members` |
| `experiment-baton/AILOG-YYYY-MM-DD-NNN-*.md` | New — log(s) de ejecución; `## Batch Ledger` para multi-batch |

> Paths confirmados al ejecutar (reconnaissance-first). Los `core/**` se tocan solo si la graduación parcial es necesaria; por defecto, read-only.

## Verification

### Local checks

- `cargo build -p straymark-baton` y `cargo test` (workspace) pasan en shell limpio.
- Fixture que reproduce el triple mismatch de #304 (contrato productor + consumidor divergente) → el diagnóstico emite los 3 findings esperados (campo/enum mismatch, campo sin productor, consumidor contra forma cambiada).
- Fixture con un componente declarado en `.specify/memory/` sin archivos implementadores → finding "intencionado-no-implementado".
- `cargo clippy` limpio; el binario respeta `--output text|json|markdown` y exit code.

### Production smoke (after deploy / dogfood)

- Correr el diagnóstico **read-only** contra Sentinel (`/home/montfort/StrangeDaysTech/sentinel`): surface (a) la deriva del contrato de salud US1 (#304) y (b) PolicyEngine como intencionado-no-implementado.
- Confirmar cero mutaciones en el repo Sentinel (read-only verificable: `git status` limpio tras la corrida).

## Risks

- **R1 — `.specify/memory/` es markdown humano libre** (no estructurado como `spec.md`). Severidad: media. Mitigation: parsing tolerante + convención ligera; tratar como pista de baja confianza, nunca como contrato duro; los findings de `memory/` arrancan como `info`, no `blocking`.
- **R2 — Acoplamiento prematuro de `core` a código experimental.** Severidad: media. Mitigation: prototipar como `straymark-baton` aparte (espejo de `straymark-loom`); exponer lógica pura/typed para graduar a `core` solo tras validación. Fallback: si la extensión de la proyección de Loom obliga a tocar `core`, hacerlo mínimo y typed, sin lógica de Baton dentro.
- **R3 — Falsos positivos / ruido** erosionan la confianza del adopter. Severidad: alta. Mitigation: empezar SOLO con clases de finding de alta confianza (componente con cero archivos; campo de consumidor sin productor); severidad conservadora; el caso #304 es el oráculo de calibración.
- **R4 — Deriva de versión de SpecKit** (adaptador anclado a 0.11.3). Severidad: baja. Mitigation: adaptador version-gated; advisory (no crash) ante versión no probada.
- **R5 — Scope creep hacia routing/modelos.** Severidad: media. Mitigation: Out-of-scope explícito; esta fase no importa ninguna noción de modelo/token/costo.
- **R6 — Inferir procedencia cross-spec sin metadata explícita es ambiguo.** Severidad: media. Mitigation: inferencia conservadora + permitir declaración explícita opcional; preferir falso-negativo silencioso a falso-positivo ruidoso en esta fase.

## Tasks

1. Sync `main`, partir de la rama de este experimento.
2. **B1 — Adaptador SpecKit** (read-only, versionado): parsea specs + `.specify/`. Tests con fixtures.
3. **B2 — Modelo de intención + edges de procedencia**: tipos, inferencia cross-spec, reuse de `glob_match`.
4. **B3 — Motor de coherencia + CLI**: clases de finding de alta confianza; salida text/json/markdown + exit code.
5. **B4 — Overlay de intención en Loom**: tercer plano en la proyección.
6. **B5 — Dogfood Sentinel + AILOG**: reproducir #304 como finding; documentar.
7. AILOG (`risk_level`, `review_required`); multi-batch → mantener `## Batch Ledger`, correr `straymark charter batch-complete` post-commit de cada batch.
8. Verificación local; drift check; commit + PR.

## Charter Closure

- Atomic update (format v4): si se detecta drift al cerrar, reconciliar en el **mismo PR**, documentando en `## Closing notes`.
- Post-merge drift check.
- Frontmatter: `declared` → `in-progress` al arrancar; `in-progress` → `closed` al cerrar (+ `closed_at`).
- No borrar el archivo — el historial de planeación importa.
- **Graduation gate de Baton (parcial, ver concepto §7):** esta fase se considera exitosa si el diagnóstico, corrido read-only contra Sentinel, **caza al menos una deriva arquitectónica real (#304 y/o PolicyEngine) que la revisión humana había dejado pasar.**

## Closing notes

Cerrado 2026-06-25 tras B5 (AILOG-2026-06-25-005). Entregado en 5 batches/PRs:
B1 adaptador SpecKit (#308), B2 IntentModel + procedencia (#309), B3 motor de
coherencia C1–C4 (#310), B4 overlay de intención para Loom + NFR2 (#311), B5
dogfood + calibración + cierre.

**Graduation gate: MET.** Corrido read-only contra Sentinel (HEAD `24d5a66`, cero
mutación verificada), el bridge caza una deriva real #304-class previamente no
señalada — `005-frontend-dashboard` consume `services.public-visibility` sin
referenciar su decisión definitoria PM-001 / `AILOG-2026-04-21-002` — además de
gaps reales (DevPortal, UsageGuard) vía el overlay. Evidencia completa en
[`03-sentinel-dogfood-report.md`](03-sentinel-dogfood-report.md).

**Desviaciones vs. el plan declarado (reconciliadas en su PR):**
- **T3.5** (consistencia NFR2 con `glob_match`) se movió de B3 a **B4**, donde el
  overlay une con el `model.yml` que sí lleva globs (en B3 C1 usa substring, no
  hay matcher de globs que comparar). Documentado en AILOG-003/004.
- **Render web en Loom** (parte de T4.2): B4 entrega el overlay *consumible* por
  Loom (tipado + JSON, FR6); el render en el frontend Vite/TS de experiment-loom
  queda como follow-on para no acoplar un PR de Baton al build del frontend.
- **Calibración en B5**: los datos reales de Sentinel obligaron a 4 fixes de
  precisión (liga decisión→contrato por endpoint, excluir tests como productores,
  C1→info, agregación C4) — 90→6 findings. Es el valor esperado de un dogfood.

**Limitación conocida (top follow-up):** el keying de contrato sobre archivos de
tipos generados (`types.gen.ts`) funde contratos, así que el mismatch de
campos/enums de salud (C2/C3) no se aísla en Sentinel (0 blocking by design). El
seguimiento y la *costura de activación* (hook `before_implement` de SpecKit) se
documentan en `03-sentinel-dogfood-report.md` §6.
