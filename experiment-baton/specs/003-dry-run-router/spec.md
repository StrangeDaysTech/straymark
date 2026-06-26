# Feature Spec 003 — Baton Dry-Run Router (cost-aware classification + economic telemetry)

> **Status:** Draft
> **Experiment:** Baton (sibling of Loom). Phase 2 of [01-baton-concept.md](../../01-baton-concept.md) (§4.2, §4.3, §7).
> **Charter:** [CHARTER-03-dry-run-router.md](../../CHARTER-03-dry-run-router.md)
> **Scope rule:** read-only, **recommends — never executes**. Touches models only as telemetry.
> **Framing decisions (2026-06-26):** routable unit = *instrument existing granularities & measure*; cost = *illustrative tiers in config*; classifier = *cheap signals only, no LLM*.

## 1. Problem & intent

StrayMark has **no notion of model, tier, budget, token or cost** — verified, zero occurrences in the code. Every piece of work runs on the same model, whether it is a trivial commit or an architectural redesign. The concept's economic driver (§1.1, §2.1): as per-Mtoken pricing replaces subsidised subscriptions, paying frontier rates for mechanical work becomes unaffordable for an independent developer.

Not all work needs a frontier model (§1.2). Commits, PRs, atomic task lists, classification are done well by economic tiers; frontier should be reserved for design, architecture, adjudication and open-ended diagnosis. The thesis: **focus token spend on the model appropriate to each task class**, leaning on the governance discipline StrayMark already produces.

**Intent (Phase 2):** build the classification + tier-recommendation layer as a **retrospective dry-run**. The routable units are the work StrayMark *already recorded* in an adopter repo (charters, batches, follow-ups, tasks). For that existing corpus, classify each unit with cheap signals, recommend a tier, and emit **economic telemetry**: what would have routed where, and what it would have cost under a routing policy vs. all-frontier. **No agent is dispatched; no model is invoked.**

The goal is not to save yet — it is to make the *potential* saving visible and **validate the hard economic principle §4.2 at zero risk**: the cost of classifying/routing must not equal or exceed the saving of not loading everything onto frontier. And to answer the open routable-unit question (§10.4) **empirically** — by measuring which granularity is homogeneous enough to route.

## 2. Users & primary stories

- **S1 — Operator / maintainer.** *As the human in command, I want to see what fraction of my recorded work would route to an economic tier and how much that would save vs. all-frontier, so I can judge whether cost-aware routing is worth pursuing.*
- **S2 — Skeptic of the thesis.** *As someone wary of overhead, I want the telemetry to show the classification overhead next to the saving, so I can see whether the §4.2 ceiling is respected before any execution is built.*
- **S3 — Designer of Phase 3.** *As the person scoping execution, I want to know which granularity (charter/batch/follow-up/task) is actually routable, so Phase 3 instruments the right unit instead of guessing.*
- **S4 — Adopter contributor (N=2).** *As a second adopter on a different stack, I want to run the same dry-run on my repo to contribute calibration data, so the classifier generalises beyond one project.*

## 3. The model (contract)

The router reads existing governance artifacts into typed **RoutableUnit**s, attaches **UnitSignals** computed from signals StrayMark already produces, classifies each into a **TaskClass**, recommends a **Tier** via a config policy, and computes **EconomicTelemetry**. All pure given inputs; the only I/O is reading files. Reuses `straymark-core` (document model, `glob_match`, `analyze`, projection) — it does not recompute them.

### 3.1 RoutableUnit + signals

```
RoutableUnit {
  id: String,                 // CHARTER-03 · B3 · FU-005-006 · spec005:T3.5
  granularity: Charter | Batch | Followup | Task,
  source: SourceRef,
  signals: UnitSignals,
}
UnitSignals {
  effort_estimate:   Option<XS|S|M|L>,             // human time (concept §3)
  risk_level:        Option<Low|Medium|High>,       // charter / AILOG
  complexity:        Option<{ cognitive, cyclomatic }>,  // analyze, when files known
  followup_bucket:   Option<String>,                // FU-NNN bucket
  followup_severity: Option<Low|Medium|High|Critical>,
  arch_state:        Option<Active|Implemented|HasDebt|WiringGap|Uncharted>,  // Loom projection
  coherence_findings: usize,                        // Phase-1 findings touching this unit
  surface_globs:     [String],                      // files/globs the unit touches
  cues:              [Cue],                          // textual: design|architecture vs commit|docs|cleanup|test
}
```

### 3.2 TaskClass (the routing target — concept §4.2 roles)

```
TaskClass = Planner | Implementer | Auditor | Operator
```

| Class | Typical work | Default tier |
|---|---|---|
| Planner / Architect | complex decomposition, architecture, trade-offs, criteria | Frontier |
| Implementer | bounded implementation on a routable unit | Economic → Frontier on escalation |
| Auditor | independent contrast (already `charter audit`) | Economic (distinct families) |
| Operator | commits, PRs, docs, cleanup, context prep | Local / Economic |

### 3.3 Tier model + routing policy (config `baton:` block)

```yaml
baton:
  tiers:
    frontier: { illustrative_cost_per_mtok: 15.0 }   # labelled illustrative
    economic: { illustrative_cost_per_mtok: 1.0 }
    local:    { illustrative_cost_per_mtok: 0.0 }
  work_size:                                          # illustrative token volume by effort
    XS: 20000  S: 60000  M: 200000  L: 600000
  routing: { planner: frontier, implementer: economic, auditor: economic, operator: local }
  escalation: [ "risk_level=High", "coherence_findings>0", "complexity.cognitive>=<t>" ]  # implementer → frontier
  classification_overhead: { illustrative_cost_per_unit: 0.02 }   # the §4.2 ceiling term
```

### 3.4 EconomicTelemetry (computed, per granularity)

```
EconomicTelemetry {
  granularity, units_total,
  tier_distribution: { frontier: %, economic: %, local: % },
  cost_all_frontier, cost_routed,              // illustrative
  gross_savings: cost_all_frontier - cost_routed,
  classification_overhead: units_total * per_unit,
  net_savings: gross_savings - classification_overhead,
  routable: net_savings > 0,                   // §4.2 verdict for this granularity
  homogeneity: mean_classes_per_unit,          // 1.0 = clean; >1 = mixed = not cleanly routable
}
```

## 4. CLI surface (contract)

```
straymark-baton classify [PATH] [--out FMT] [--granularity G] [--config FILE]
straymark-baton route    [PATH] --dry-run [--out FMT] [--granularity G] [--config FILE]
```

- `PATH` — project root (default `.`); read-only.
- `classify` — per-unit `TaskClass` + confidence + the signals that drove it.
- `route --dry-run` — per-unit tier recommendation **+** the `EconomicTelemetry` summary. `--dry-run` is mandatory in Phase 2 (there is no execution path to omit it for).
- `--granularity charter|batch|followup|task|all` (default `all` → one telemetry block per granularity).
- `--config FILE` — the `baton:` block (default: `config.yml` in the repo; falls back to built-in illustrative defaults with a visible notice).
- `--out text|json|markdown` (default `text`).
- **Read-only + recommend-only invariant:** never writes inside `PATH`, never opens a network connection, never invokes a model (verifiable: `git status` unchanged; no client dependency linked).

## 5. Classification rules (cheap-first, deterministic)

Rules over precomputed signals; **no LLM** (NFR3). Calibrated on the adopter corpus (Sentinel now, N=2 later). Conservative: when signals are ambiguous or conflicting, **route up** (toward the more capable tier) — never down (NFR4 / R1).

| Signal pattern | Class |
|---|---|
| cues ∈ {architecture, design, trade-off}; or `arch_state=Uncharted`; or `risk_level=High` with no bounded surface | **Planner** |
| bounded `surface_globs`; `effort_estimate ∈ {XS,S,M}`; cues ∈ {implement, fix, feature} | **Implementer** (→ Planner-tier on an escalation signal) |
| cues ∈ {audit, review, verify}; or unit derived from `charter audit` | **Auditor** |
| cues ∈ {commit, PR, docs, cleanup, rename, bump}; `effort_estimate=XS`; low complexity | **Operator** |
| ambiguous / conflicting | **route up** (default to the higher class present) + `confidence=Low` |

## 6. Functional requirements

- **FR1** — Inventory routable units read-only at each existing granularity (Charter / Batch / Follow-up / Task) from governance artifacts. No new vocabulary.
- **FR2** — Compute `UnitSignals` by **reusing** existing StrayMark signals (`effort_estimate`, `analyze` complexity, charter `risk_level`, follow-up bucket/severity, Loom projection `arch_state`, Phase-1 coherence findings, surface globs). No recomputation of a signal StrayMark already owns.
- **FR3** — Deterministic classifier (§5): `UnitSignals → (TaskClass, confidence)`. Conservative route-up on ambiguity.
- **FR4** — Config-driven tier model + routing policy (§3.3): parse the `baton:` block; fall back to built-in illustrative defaults with a notice.
- **FR5** — Dry-run routing: per-unit tier recommendation with escalation applied. **Never executes.**
- **FR6** — `EconomicTelemetry` (§3.4): tier distribution, all-frontier vs routed cost, gross/net savings, classification overhead, `routable` verdict — **per granularity**, with costs labelled illustrative.
- **FR7** — Granularity report: `homogeneity` per granularity → which granularity is routable (the empirical answer to §10.4).
- **FR8** — Emit `text|json|markdown`; `classify` and `route --dry-run`; read-only + recommend-only invariants asserted by tests.

## 7. Non-functional requirements

- **NFR1 — Read-only.** No mutation of the target repo. Enforced + tested.
- **NFR2 — Recommend-only.** No model client, no agent dispatch, no network. Phase 2 cannot execute by construction (no execution code path, no provider dependency).
- **NFR3 — Cheap-first.** Classification is deterministic rules over precomputed signals; zero LLM calls. The classifier's own compute is negligible by design (§4.2 corollary).
- **NFR4 — Conservative quality bias.** Ambiguity routes **up**. Never trade quality for saving (R1). A false "this is cheap" is worse than a missed saving.
- **NFR5 — Determinism & purity.** Classification + telemetry are pure given inputs; unit-testable without a live repo.
- **NFR6 — Honest economics.** Costs are labelled illustrative; the report states **relative** savings and shows classification overhead next to gross saving (§4.2 made visible, not hidden). Real provider pricing is deferred to Phase 3 (§10.8).
- **NFR7 — Consistency.** Reuse `straymark-core` (`glob_match`, `analyze`, projection, charter/follow-up parsing). No second matcher, no duplicated signal.

## 8. Acceptance criteria (definition of done for Phase 2)

1. `classify` and `route --dry-run` run read-only against a checkout and emit reports in all three formats with correct exit codes; neither links a model client nor opens a network connection.
2. A fixture corpus with units of known class yields the expected `TaskClass` per unit and the expected tier distribution + gross/net savings.
3. The telemetry shows classification overhead next to gross saving and marks each granularity `routable` / not (§4.2 made explicit). A fixture where overhead ≥ saving is correctly reported as **not routable** rather than forced.
4. Run read-only against Sentinel produces the retrospective economic telemetry over its real governance corpus; `git status` in Sentinel is unchanged; the report **identifies which granularity pays** (the empirical §10.4 answer).
5. `cargo test --workspace` and `cargo clippy` are green.

**Graduation-gate tie-in (Charter):** the telemetry on Sentinel shows **net-positive relative saving after subtracting classification overhead** at some granularity, and names which. A net-negative result, with evidence, is still a valid graduation of *knowledge* (per-unit routing does not pay on this corpus → it reframes Phase 3).

## 9. Out of scope (for this spec)

- Real model execution / agent dispatch → Phase 3.
- Real provider pricing / unified cost identity (§10.8) → Phase 3. This spec uses illustrative declared costs.
- An LLM classifier → future escalation, only when cheap signals are ambiguous **and** the expected saving justifies it.
- Config / web interfaces for execution & monitoring → Phase 3 / Track P (Podium).
- A new routable-unit vocabulary ("work unit", §4.3b) → deferred until the data justifies it.
- Historical/learned routing (expected cost per route from accumulated metrics) → Phase 4.
- Mutating any governance artifact.

## 10. Open questions

- **Q1 — Work-size proxy.** Illustrative token volume per unit from `effort_estimate` (primary) refined by `analyze` complexity when files are known, vs. surface size. Recommendation: `effort_estimate`-driven via the config `work_size` map; complexity as a refiner. Settle in `plan.md`.
- **Q2 — Classification-overhead model.** The §4.2 ceiling term. Since the classifier is rule-based (near-zero compute), overhead models the *setup* cost of routing per unit, declared illustratively. Recommendation: a declared `cost_per_unit` with a sensitivity line in the report (saving holds across a range).
- **Q3 — Escalation signals.** Which signals lift Implementer → frontier (`risk_level=High`, `coherence_findings>0`, complexity over a threshold). Recommendation: declared in config with conservative defaults; calibrate on Sentinel.
- **Q4 — Crate / core placement.** The classifier + telemetry are pure and graduable to `core` later (mirror R5/Phase-1 Q3). Recommendation: prototype in `straymark-baton`; graduate the pure logic after validation.
- **Q5 — Adopter calibration set.** How to accumulate N=2 calibration data without coupling the classifier to Sentinel's idioms. Recommendation: keep rules signal-driven (not project-specific); record per-adopter calibration deltas as fixtures. Ties to the language-agnostic boundary (#321) on the producer/consumer side.
