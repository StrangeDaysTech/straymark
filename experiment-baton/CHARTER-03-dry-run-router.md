---
charter_id: CHARTER-03-dry-run-router
status: closed
closed_at: "2026-06-26"
effort_estimate: L
trigger: "Baton concepto §7 Fase 2 — no existe ninguna noción de modelo/tier/presupuesto/costo en StrayMark; todo el trabajo corre con el mismo modelo (commit trivial o rediseño). Presión económica §1.1 (fin de subsidios, pago por Mtoken). Tras la Fase 1 (Coherence Bridge), validar empíricamente el principio económico §4.2 sin tocar modelos."
originating_concept: experiment-baton/01-baton-concept.md
originating_spec: experiment-baton/specs/003-dry-run-router/spec.md
related_charters: [CHARTER-01-coherence-bridge, CHARTER-02-activation-seam]
related_issues: [304]
---

# Charter: Baton Fase 2 — Clasificación + router en seco (dry-run, recomienda no ejecuta)

> **Status (espejo del frontmatter; la fuente de verdad es el frontmatter):** closed (2026-06-26). Effort: L.
> **Origen:** concepto [01-baton-concept.md](01-baton-concept.md) §4.2 + §7 (Fase 2) + §10.4 (unidad enrutable).
> **Encuadre:** segunda fase del experimento Baton. **Recomienda, no ejecuta. Toca modelos solo como telemetría — riesgo cero.** Prototipa en `experiment-baton/`, lógica pura/typed diseñada para graduar a `straymark-core`.

## Encuadre — decisiones de la sesión 2026-06-26

Tres decisiones de framing resueltas con el operador antes de redactar este charter (el concepto las dejó abiertas en §10.4 y §10.8):

1. **Unidad enrutable (§10.4): instrumentar lo existente y medir.** No se inventa vocabulario ("work unit"). Se clasifica a la granularidad que YA existe — Charter / Batch / Follow-up / Task — se mide la heterogeneidad de cada nivel en Sentinel, y la telemetría revela qué granularidad "paga" bajo el techo §4.2. **La Fase 2 responde la sub-decisión #4 con datos, no por decreto.**
2. **Modelo de costo: tiers ilustrativos en config.** Un bloque `baton:` en `config.yml` (patrón config-driven #279) declara costo-por-Mtoken **ilustrativo** por tier. Desbloquea el dry-run sin resolver la identidad de costo real de proveedores (§10.8 sigue diferida a Fase 3). Se mide **ahorro relativo**, no factura real.
3. **Clasificador: solo señales baratas, sin LLM.** Reglas/heurísticas deterministas sobre señales que StrayMark ya computa. Cumple el corolario §4.2 (clasificar debe ser barato). Un clasificador LLM queda como escalamiento futuro, solo cuando el ahorro lo justifique.

## Context

La Fase 1 entregó el Coherence Bridge: detección read-only de deriva intención↔gobernanza↔código. Pero el **driver económico** del concepto (§2.1) sigue sin tocar: hoy no existe en StrayMark ninguna noción de modelo, tier, presupuesto, token o costo (cero ocurrencias en el código). Todo se ejecuta con el mismo modelo, sea un commit trivial o un rediseño arquitectónico.

La Fase 2 construye la **capa de clasificación + recomendación de tier** como un **dry-run retrospectivo**: las unidades enrutables son **el trabajo que StrayMark ya registró** en Sentinel (charters, batches, follow-ups, tasks). Para ese corpus ya existente, Baton clasifica cada unidad con señales baratas, recomienda un tier, y emite **telemetría económica**: qué habría ido a qué tier, y cuánto habría costado bajo una política de routing vs. todo-frontier. No se ejecuta ningún agente ni se invoca ningún modelo.

El objetivo no es ahorrar todavía — es **hacer visible el ahorro potencial y validar el principio económico §4.2 sin riesgo**: que el costo de clasificar/enrutar no iguale ni supere el ahorro de no cargar todo a frontier. Si la telemetría muestra que el overhead borra el ahorro a cierta granularidad, **ese es un resultado válido** (parte de la respuesta a §10.4), no un fallo a esconder.

**Decisión de placement (igual que Fase 1, R5):** prototipo en `straymark-baton`; lógica pura/typed expuesta para graduar a `core` tras validación. El núcleo de StrayMark **no** adquiere conocimiento de modelos/tokens en esta fase.

## Scope

**In scope:**

1. **Inventario de unidades enrutables a granularidad existente.** Lectura read-only de los artefactos de gobernanza como unidades clasificables, **sin inventar vocabulario**: Charters (`charter_id`, scope, risk), entradas del Batch Ledger, Follow-ups (FU-NNN: bucket, severidad), y tasks de `specs/**/tasks.md`. Cada granularidad se instrumenta por separado.
2. **Esquema de clasificación de tareas (cheap-first, determinista).** Mapea una unidad → **clase de tarea** (Planner/Architect, Implementer, Auditor, Operator — tabla §4.2) a partir de señales que StrayMark ya produce: `effort_estimate` (XS/S/M/L), complejidad de `analyze` (cognitiva/ciclomática), `risk_level` del Charter, bucket/severidad del Follow-up, estado de la proyección de Loom (Active/HasDebt/WiringGap/Uncharted), findings de coherencia (Fase 1), y superficie de archivos/globs tocada. Reglas typed, puras, sin I/O.
3. **Modelo de tiers + política de routing declarativa.** Bloque `baton:` en `config.yml`: tiers (`frontier`/`economic`/`local`), costo-por-Mtoken **ilustrativo** por tier, y mapeo clase→tier con fallback conservador. Sin ejecución.
4. **Telemetría económica (dry-run).** Para un corpus de unidades: distribución por tier (% económico/intermedio/frontier), costo estimado bajo **todo-frontier** vs. bajo la **política enrutada**, **ahorro relativo** estimado, y el **overhead de clasificación** estimado — para validar §4.2 explícitamente (el ahorro debe superar el overhead). Salida text/json/markdown, inspeccionable en CI.
5. **Reporte de granularidad enrutable.** Por cada granularidad instrumentada (charter/batch/follow-up/task): cuán homogénea es (mezcla de clases dentro de la unidad) y si "paga" bajo el techo §4.2 — **la respuesta empírica a §10.4.**
6. **Superficie CLI mínima.** `baton classify` (clase por unidad) y `baton route --dry-run` (recomendación + telemetría), espejo de la superficie de `coherence` (text/json/markdown). **Recomienda, nunca ejecuta.**
7. **Dogfood read-only en Sentinel.** Correr sobre los artefactos de gobernanza de Sentinel; emitir la telemetría retrospectiva; documentar en AILOG. Confirmar cero mutación (`git status` limpio).

**Out of scope** (fases posteriores o explícitamente fuera):

- **Ejecución real de modelos / dispatch de agentes** → Fase 3.
- **Pricing real de proveedor / identidad de costo unificada** (§10.8) → Fase 3. Esta fase usa costos ilustrativos declarados.
- **Clasificador LLM** → escalamiento futuro, solo cuando las señales baratas sean ambiguas Y el ahorro lo justifique.
- **Archivos de config de ejecución / interfaces web de vigilancia** → Fase 3 / Track P (Podium).
- **Inventar una sub-unidad enrutable nueva** ("work unit" §4.3b) → diferido; instrumentar lo existente primero y dejar que los datos lo justifiquen.
- **Mutar cualquier artefacto de gobernanza** → read-only, igual que Fase 1.
- **Telemetría histórica/aprendida** (costo esperado por ruta a partir de métricas acumuladas) → Fase 4.

## Files to modify

| File | Change |
|---|---|
| `experiment-baton/specs/003-dry-run-router/spec.md` | New — WHAT: unidades enrutables, esquema de clasificación, modelo de tiers, telemetría, contrato CLI |
| `experiment-baton/specs/003-dry-run-router/plan.md` | New — HOW: fasing por batch, señales baratas, placement crate vs core |
| `experiment-baton/specs/003-dry-run-router/tasks.md` | New — tareas ordenadas B1–B5 |
| `experiment-baton/src/**` | New — inventario de unidades, clasificador, política de tiers, telemetría, subcomandos CLI |
| `experiment-baton/src/main.rs` | Add subcomandos `classify` / `route` (read-only) |
| `experiment-baton/Cargo.toml` | Posible dep para parsear el bloque `baton:` de config (serde_yaml ya presente) |
| `experiment-baton/AILOG-YYYY-MM-DD-NNN-*.md` | New — log(s) de ejecución; `## Batch Ledger` para multi-batch |

> Paths confirmados al ejecutar (reconnaissance-first). `core/**` se toca solo si la graduación parcial lo amerita; por defecto read-only.

## Verification

### Local checks

- `cargo build -p straymark-baton` y `cargo test` (workspace) pasan en shell limpio; `cargo clippy` limpio.
- Fixture con unidades sintéticas de clase conocida → el clasificador asigna la clase esperada (planner/implementer/auditor/operator) a partir de las señales.
- Fixture de corpus → la telemetría computa la distribución por tier, el costo todo-frontier vs. enrutado, el ahorro relativo y el overhead estimados esperados.
- El reporte §4.2 expone **explícitamente** overhead vs. ahorro; cuando el overhead ≥ ahorro a una granularidad, la marca como **no enrutable** en vez de forzarla.
- CLI respeta `--output text|json|markdown`; `route` es dry-run (no ejecuta, no muta).

### Production smoke (after deploy / dogfood)

- Correr `baton route --dry-run` **read-only** contra Sentinel (`/home/montfort/StrangeDaysTech/sentinel`): emitir la telemetría retrospectiva sobre su corpus de gobernanza real.
- Confirmar cero mutaciones (`git status` limpio tras la corrida).
- Identificar qué granularidad (charter/batch/follow-up/task) resulta enrutable bajo el techo §4.2.

## Risks

- **R1 — Heurísticas de clasificación mal calibradas** (tier equivocado → pérdida de calidad o ahorro falso). Severidad: alta. Mitigation: **default conservador — ante ambigüedad, enrutar HACIA ARRIBA (frontier); nunca sacrificar calidad por ahorro**; calibrar en Sentinel; `route` es recomienda-only, sin daño real posible.
- **R2 — El overhead supera el ahorro** (violación §4.2). Severidad: alta. Mitigation: la telemetría **mide** esto explícitamente; cheap-first sin LLM; si una granularidad tiene overhead ≥ ahorro, se reporta como no enrutable, no se fuerza.
- **R3 — Costos ilustrativos engañan.** Severidad: media. Mitigation: etiquetar los costos como ilustrativos; reportar ahorro **relativo** + sensibilidad; diferir costos reales a Fase 3 (§10.8).
- **R4 — Scope creep hacia ejecución/modelos.** Severidad: media. Mitigation: out-of-scope explícito; sin cliente de modelo; `route` jamás invoca un agente.
- **R5 — Las señales existentes no bastan para clasificar limpio.** Severidad: media. Mitigation: es un **hallazgo empírico válido** (parte de la respuesta a §10.4); documentar gaps como follow-ups; no taparlo con un LLM.
- **R6 — Vocabulario de unidad enrutable prematuro.** Severidad: baja. Mitigation: instrumentar solo granularidades existentes; la sub-unidad nueva (§4.3b) se difiere hasta que los datos la justifiquen.

## Tasks

1. Sync `main`, partir de la rama de este experimento.
2. **B1 — Inventario de unidades enrutables**: lectura read-only de charters/batches/follow-ups/tasks a tipos enrutables, por granularidad. Tests con fixtures.
3. **B2 — Clasificador barato**: señales → clase de tarea, reglas deterministas typed. Tests de calibración.
4. **B3 — Modelo de tiers + política + config**: bloque `baton:` (costos ilustrativos), mapeo clase→tier con fallback conservador, routing en seco. Tests.
5. **B4 — Telemetría económica + reporte de granularidad** (superficie CLI `classify`/`route --dry-run`): distribución, ahorro relativo, overhead, validación §4.2. Tests.
6. **B5 — Dogfood Sentinel + AILOG**: telemetría retrospectiva sobre el corpus real; responder §10.4 con datos; documentar.
7. AILOG (`risk_level`, `review_required`); multi-batch → mantener `## Batch Ledger`, `straymark charter batch-complete` post-commit de cada batch.
8. Verificación local; drift check; commit + PR.

## Charter Closure

- Atomic update (format v4): si se detecta drift al cerrar, reconciliar en el **mismo PR**, documentando en `## Closing notes`.
- Post-merge drift check.
- Frontmatter: `declared` → `in-progress` al arrancar; `in-progress` → `closed` al cerrar (+ `closed_at`).
- No borrar el archivo — el historial de planeación importa.
- **Graduation gate de Baton (Fase 2, concepto §7):** la telemetría, corrida read-only sobre Sentinel, demuestra **ahorro relativo neto positivo** (enrutado vs. todo-frontier) **después de restar el overhead de clasificación estimado** (§4.2), a alguna granularidad instrumentada — e **identifica qué granularidad paga**. Si ninguna granularidad paga, el resultado (con su evidencia) sigue siendo una graduación válida del *conocimiento*, no del ahorro: habríamos probado empíricamente que el routing-por-unidad no es rentable en este corpus, lo cual cambia el diseño de la Fase 3.

## Closing notes

Cerrado 2026-06-26 tras B5 (AILOG-2026-06-26-002). Entregado en 5 batches/PRs:
B1 inventario de unidades (#323), B2 señales baratas (#324), B3 clasificador
(#325), B4 tiers + router dry-run + telemetría + CLI (#326), B5 dogfood + cierre.
Evidencia completa en [`04-phase2-dry-run-dogfood.md`](04-phase2-dry-run-dogfood.md).

**Graduation gate: MET — y gradúa *conocimiento*, el resultado más valioso aquí.**
Corrido read-only sobre Sentinel (762 unidades, `git status` intacto):

- **Ahorro neto positivo en TODAS las granularidades** (ALL: bruto ~93% ilustrativo,
  neto $1184.68, robusto a 2× overhead; overhead = 1.2% del ahorro bruto, así que el
  principio §4.2 no se viola por el costo de clasificar).
- **Pero el ahorro es frágil:** solo ~43% de las unidades enrutan con confianza
  high+medium; **57% del ahorro descansa en routing de baja confianza** (dominado por
  el default no-cue, no por conflictos, que son 5–15%).
- **Respuesta empírica a §10.4 (contraria a la hipótesis):** la granularidad **no** es
  la palanca. El conflicto (proxy de heterogeneidad) está confundido por la verbosidad
  del título — Task tiene el conflicto *más alto* (15%), Charter el más bajo (6%) — y
  la confianza es uniforme (37–46%) en todas. **La cobertura de señal, no la
  granularidad, es la restricción.** La decisión "instrumentar lo existente" se mantiene;
  una sub-unidad más fina (§4.3b) no ayudaría (Task ya es la más fina y no es más
  confiable).

**Reencuadre para la Fase 3 (data-justified, no especulativo):** el camino a un ahorro
*confiable* no es otra unidad enrutable, es **cablear las señales diferidas** (complejidad
— requiere graduar `analyze` de `cli` a `core`; arch_state de la proyección Loom;
findings de coherencia de la Fase 1) para subir la confianza antes de ejecutar. Filable
como el siguiente trabajo de Baton cuando se retome.

**Desviaciones vs. el plan declarado (reconciliadas en su PR):**
- **Señales pesadas diferidas en B2** (complejidad/arch_state/coherencia): el plan §4 las
  listaba; se difirieron por el principio cheap-first y porque `analyze` vive en `cli`. El
  dogfood de B5 las *justifica* retroactivamente como el siguiente paso — exactamente el
  enfoque empírico del charter.
- **`homogeneity` → `conflict_fraction`**: la métrica de heterogeneidad del spec §3.4 se
  implementó como tasa de conflicto de cues; B5 documenta que es un proxy débil (confundido
  por largo de título). Una mejor mediría el cuerpo/scope de la unidad, no solo el título.
