# Tasks 003 — Baton Dry-Run Router

> **Spec:** [spec.md](spec.md) · **Plan:** [plan.md](plan.md) · **Charter:** [CHARTER-03-dry-run-router.md](../../CHARTER-03-dry-run-router.md)
> Read-only, recommend-only, illustrative costs. Each batch is its own PR; maintain the `## Batch Ledger` in the execution AILOG.

## B1 — Unit inventory (FR1)

- [ ] T1.1 — `src/units.rs`: `RoutableUnit` + `Granularity` (Charter/Batch/Followup/Task), serde for `--out json`.
- [ ] T1.2 — Readers (read-only): Charter frontmatter; Batch Ledger entries; Follow-ups registry (reuse existing model); `specs/**/tasks.md` checkboxes.
- [ ] T1.3 — `--granularity charter|batch|followup|task|all` selection.
- [ ] T1.4 — Fixtures under `experiment-baton/tests/fixtures/` — a synthetic corpus spanning all four granularities with clear class exemplars (a planner charter, an operator task, an auditor unit, an implementer batch). Unit tests.

## B2 — Signal aggregation (FR2)

- [ ] T2.1 — `src/signals.rs`: `UnitSignals`; build from existing sources, **reusing** core (no recompute).
- [ ] T2.2 — Wire `effort_estimate` / `risk_level` (frontmatter), `complexity` (core `analyze` over resolvable `surface_globs`, else `None`), follow-up bucket/severity (registry), `arch_state` (Loom projection), `coherence_findings` (Phase-1 report filtered to surface).
- [ ] T2.3 — `cues`: tolerant keyword scan (reuse `scan.rs` philosophy — line-oriented, no regex, char-boundary-safe). Conservative: no cues → ambiguous, not Operator.
- [ ] T2.4 — Per-granularity **signal-coverage** accounting (which signals are present) for the report (P1). Tests with partial-signal units.

## B3 — Cheap classifier (FR3)

- [ ] T3.1 — `src/classify.rs`: pure `classify(&UnitSignals) -> Classification { class, confidence, rationale }`; rules of spec §5, ordered.
- [ ] T3.2 — Conservative route-up on ambiguity/conflict → higher class present + `confidence=Low`.
- [ ] T3.3 — `rationale` records the driving signals (auditable recommendation; legible calibration deltas).
- [ ] T3.4 — Assert **no** model/network/LLM dependency in the module (NFR2/NFR3). Calibration tests on the fixture corpus → expected class per unit.

## B4 — Tier policy + dry-run router + telemetry + CLI (FR4–FR8)

- [ ] T4.1 — `src/tiers.rs`: parse the `baton:` block (serde_yaml) — `tiers`, `work_size`, `routing`, `escalation`, `classification_overhead`; built-in illustrative defaults + visible notice when absent (NFR6).
- [ ] T4.2 — `src/route.rs`: `route(unit, class, &Policy) -> TierDecision`; base tier + escalation predicates (Implementer → frontier). Recommends, never executes.
- [ ] T4.3 — `src/telemetry.rs`: per-granularity `EconomicTelemetry` (spec §3.4) — `illustrative_tokens` (Q1), all-frontier vs routed cost, gross/net savings, classification overhead, `routable`, `homogeneity`, sensitivity line (Q2).
- [ ] T4.4 — `src/main.rs`: `clap` subcommands `classify` and `route --dry-run` (`--out text|json|markdown`, `--granularity`, `--config`, exit `0/1/2`); mirror `coherence` ergonomics.
- [ ] T4.5 — Read-only test (snapshot `git status` before/after, NFR1) + recommend-only structural assertion (no model/network dep in `Cargo.toml`, NFR2). Fixture where overhead ≥ saving → reported **not routable** (acceptance #3).

## B5 — Dogfood Sentinel + acceptance (spec §8)

- [ ] T5.1 — Run `classify` and `route --dry-run` **read-only** against `/home/montfort/StrangeDaysTech/sentinel`; emit the retrospective economic telemetry over its real governance corpus.
- [ ] T5.2 — Verify `git status` in Sentinel unchanged (NFR1); confirm the binary invoked no model/network (NFR2).
- [ ] T5.3 — Record the empirical §10.4 answer: which granularity (charter/batch/follow-up/task) is `routable` under the §4.2 ceiling, with `homogeneity` and net-saving evidence.
- [ ] T5.4 — Acceptance pass (spec §8.1–§8.5): three formats + exit codes; fixture classification + telemetry; overhead-shown-next-to-saving; clippy + `cargo test --workspace` green.
- [ ] T5.5 — Execution AILOG (`risk_level`, `review_required`); `## Batch Ledger` reconciled; `straymark charter batch-complete` per batch.
- [ ] T5.6 — Update concept §7 roadmap (Phase 2 → done) + Charter closure; resolve spec §10 open questions as settled; record the graduation-gate verdict (positive saving, or a documented negative that reframes Phase 3).

## Verification (per batch + at close)

- [ ] Local: `cargo build -p straymark-baton`, `cargo test --workspace`, `cargo clippy` green in a clean shell.
- [ ] Recommend-only: no model client / network dependency linked (structural); `route` never executes.
- [ ] Dogfood (B5): read-only run on Sentinel emits telemetry; zero mutations; names the routable granularity.
- [ ] Drift check: `straymark charter drift CHARTER-03-dry-run-router <range>` clean pre-commit and at close.
