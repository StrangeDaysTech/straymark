# Implementation Plan 003 — Baton Dry-Run Router

> **Spec:** [spec.md](spec.md) · **Charter:** [CHARTER-03-dry-run-router.md](../../CHARTER-03-dry-run-router.md)
> **Scope:** read-only, recommend-only, illustrative costs. Phase 2 of Baton.

## 1. Architecture at a glance

```
   Governance      ┌─────────────────────────────────────────────┐
   artifacts  ───► │ Unit inventory (read-only)                  │
 (charters,        │  charters · batch ledger · follow-ups · tasks│
  follow-ups,      └───────────────────┬─────────────────────────┘
  specs/tasks)                         ▼
                        ┌────────────────────────┐
   straymark-core ─────►│ UnitSignals (reused)    │  effort, risk, complexity,
   (analyze, glob_match,│                         │  followup, arch_state,
    projection, docs)   └───────────┬─────────────┘  coherence findings, surface
   Baton Phase 1 ───────────────────┤
   (coherence findings)             ▼
                        ┌────────────────────────┐
                        │ Cheap classifier        │  signals → TaskClass (no LLM)
                        │ (deterministic rules)    │  conservative route-up
                        └───────────┬─────────────┘
                                    ▼
   config.yml `baton:` ───►┌────────────────────────┐
   (tiers, routing,        │ Dry-run router +        │  tier per unit +
    illustrative costs)    │ economic telemetry      │  all-frontier vs routed
                           └───────────┬─────────────┘  + overhead (§4.2)
                                       ▼
                 ┌──────────────┬───────────────────────┐
                 │ CLI classify │ CLI route --dry-run     │
                 │ text/json/md │ + telemetry/granularity │
                 └──────────────┴───────────────────────┘
```

The router is a **consumer** of `straymark-core` and of Baton's Phase-1 `IntentModel`/coherence findings. It adds only the unit inventory, the signal aggregation, the classifier, the tier policy, and the telemetry.

## 2. Crate placement (mirror Phase 1)

Same decision as CHARTER-01 (R2/Q3): prototype in `straymark-baton`; keep the pure logic (`RoutableUnit`, `UnitSignals`, the classifier, the telemetry) **I/O-free** so it can `git mv` into `core/src/routing/` at graduation. No `core` mutation in Phase 2 — the router only *reads* core's outputs (`analyze`, projection, `glob_match`). The concept's "core acquires no knowledge of models/tokens" (§1.3) holds: model/tier/cost lives entirely in `straymark-baton` + the `baton:` config block.

## 3. Unit inventory (FR1)

- `src/units.rs` — `RoutableUnit` + `Granularity`. Readers, all read-only:
  - **Charter** — parse `CHARTER-*.md` frontmatter (`charter_id`, `effort_estimate`, `risk_level`/`trigger`, scope globs).
  - **Batch** — entries from the `## Batch Ledger` in execution AILOGs.
  - **Follow-up** — `FU-NNN` from the follow-ups registry (bucket, severity) via the existing follow-up model.
  - **Task** — checkbox tasks in `specs/**/tasks.md` (`T<batch>.<n>` + text).
- Each granularity is independent; `--granularity` selects one or `all`.
- Fixtures under `experiment-baton/tests/fixtures/` (a small synthetic corpus spanning all four granularities + clear class exemplars).

## 4. Signal aggregation (FR2) — reuse, don't recompute

- `src/signals.rs` — build `UnitSignals` from sources StrayMark already owns:
  - `effort_estimate` ← charter/spec frontmatter.
  - `risk_level` ← charter frontmatter / AILOG.
  - `complexity` ← `straymark-core` `analyze` over the unit's `surface_globs`, **only when files are resolvable** (else `None`); reuse the existing engine, no reimplementation.
  - `followup_bucket` / `followup_severity` ← the follow-ups registry model.
  - `arch_state` ← Loom's projection (`core/src/architecture/projection.rs`) for the components the unit touches.
  - `coherence_findings` ← Baton Phase-1 `CoherenceReport` filtered to the unit's surface.
  - `cues` ← tolerant keyword scan of the unit's title/scope text (reuse `scan.rs` philosophy: line-oriented, no regex, char-boundary-safe over accented prose).
- Missing signals are `None`, never fabricated (honest inputs → honest classification).

## 5. Classifier (FR3) — deterministic, conservative

- `src/classify.rs` — pure `fn classify(&UnitSignals) -> Classification { class, confidence, rationale }`.
- Rules of spec §5, ordered; first strong match wins; on ambiguity/conflict **route up** to the higher class present and set `confidence=Low`.
- `rationale` records which signals drove the class (so the operator can audit a recommendation — and so calibration deltas are legible).
- **No LLM, no network, no model client** (NFR2/NFR3). The module has no provider dependency at all.

## 6. Tier policy + dry-run router (FR4, FR5)

- `src/tiers.rs` — parse the `baton:` block (serde_yaml, already a dep): `tiers`, `work_size`, `routing`, `escalation`, `classification_overhead`. Built-in illustrative defaults when the block is absent (emit a visible "using illustrative defaults" notice — NFR6).
- `src/route.rs` — `fn route(unit, class, &Policy) -> TierDecision`: base tier from `routing[class]`; apply `escalation` predicates (Implementer → frontier on `risk_level=High` / `coherence_findings>0` / complexity over threshold). **Returns a recommendation; executes nothing.**

## 7. Economic telemetry (FR6, FR7)

- `src/telemetry.rs` — pure aggregation per granularity (spec §3.4):
  - `illustrative_tokens(unit)` from `work_size[effort_estimate]` (Q1), refined by complexity when present.
  - `cost(unit, tier) = illustrative_tokens · tiers[tier].cost_per_mtok / 1e6`.
  - `cost_all_frontier`, `cost_routed`, `gross_savings`, `classification_overhead = units·per_unit`, `net_savings`, `routable = net_savings > 0`.
  - `homogeneity = mean classes per unit` at this granularity (1.0 = clean; >1 = mixed). For Charter, a unit spans planner+implementer+auditor+operator stages → high mixing → expect *not* cleanly routable; for Task, ≈1 → expect routable. This is the empirical §10.4 instrument.
  - A **sensitivity line** (Q2): the saving's sign across a ±range of `classification_overhead`, so a knife-edge result is visible.

## 8. CLI (FR8)

- `src/main.rs` — `clap` subcommands `classify` and `route` (mirror the `coherence` ergonomics: `--out text|json|markdown`, `--granularity`, `--config`, exit `0/1/2`). `route` requires `--dry-run` in Phase 2.
- Read-only + recommend-only integration tests: snapshot `git status` before/after (NFR1); assert the binary links no model/network client (NFR2 — a compile-time/structural guarantee: no such dependency in `Cargo.toml`).

## 9. Phasing (each batch = a reviewable increment)

| Batch | Deliverable | FRs |
|---|---|---|
| **B1** | Unit inventory across the four granularities + fixtures | FR1 |
| **B2** | Signal aggregation (reuse analyze/projection/follow-ups/coherence) | FR2 |
| **B3** | Cheap classifier (deterministic rules, route-up) + calibration tests | FR3 |
| **B4** | Tier policy + `baton:` config + dry-run router + economic telemetry + granularity report + CLI | FR4–FR8 |
| **B5** | Dogfood read-only on Sentinel + AILOG + acceptance; answer §10.4 with data | §8 |

Multi-batch → maintain `## Batch Ledger` in the AILOG; run `straymark charter batch-complete CHARTER-03-dry-run-router N` after each batch's merge.

## 10. Risks

Inherited from the Charter (R1 miscalibration → route-up bias; R2 overhead > saving → measured explicitly; R3 illustrative costs → labelled + relative; R4 scope creep → no execution path; R5 signals insufficient → empirical finding; R6 premature unit vocabulary → instrument existing only). Plan-specific:

- **P1 — Signal coverage is sparse on real corpora** (not every unit has effort/risk/complexity). Mitigation: classify on whatever signals exist; `None` → lower confidence → route up; report per-granularity signal coverage so gaps are visible (and become follow-ups).
- **P2 — Homogeneity metric is only as good as the cue scan.** Mitigation: keep cues conservative; treat a unit with no cues as ambiguous (route up), not as a confident Operator.

## 11. References

- Concept [01-baton-concept.md](../../01-baton-concept.md) §4.2 (cost-aware router + economic principle), §4.3 (routable unit), §7 (Phase 2), §10.4/§10.8 (open/deferred decisions).
- `core/src/architecture/projection.rs` (arch_state), `core` `analyze` (complexity), `core/src/drift.rs` (`glob_match`), follow-ups registry model.
- Baton Phase 1: `src/intent.rs`, `src/coherence.rs` (findings as a signal), `src/scan.rs` (cue-scan philosophy).
- Config-driven precedent: `architecture:` block (#279). Language-agnostic boundary: #321.
