---
id: AILOG-2026-06-26-002
title: Baton Phase 2 B1 — routable-unit inventory
status: accepted
created: 2026-06-26
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 380
files_modified: [experiment-baton/src/units.rs, experiment-baton/src/lib.rs, experiment-baton/CHARTER-03-dry-run-router.md, experiment-baton/tests/fixtures/governance-corpus]
observability_scope: none
tags: [baton, phase2, dry-run-router, units, inventory, read-only]
related: [CHARTER-03-dry-run-router, AILOG-2026-06-26-001]
---

# AILOG: Baton Phase 2 B1 — routable-unit inventory

## Summary

First batch of `CHARTER-03-dry-run-router` (Phase 2). Implements the **routable-unit
inventory**: read-only enumeration of the work StrayMark already recorded, at the
four granularities that already exist — **inventing no new vocabulary** (concept
§4.3 / framing decision #1): Charter, Batch, Follow-up, Task.

`src/units.rs` exposes `RoutableUnit` + `Granularity` and `inventory(root, only)`.
Each reader harvests only the signals it can read directly; computed signals
(complexity, arch state, coherence findings) are deferred to B2. A signal a reader
cannot see stays `None` — never fabricated.

## What changed

- **Charter reader** reuses `straymark_core::charter::{discover_and_parse,
  display_title}` and `charter_files::parse_files_to_modify` — no second parser.
  Harvests `effort_estimate` (typed XS..L) and the declared `Files to modify`
  scope globs.
- **Batch / Follow-up / Task readers** are tolerant, line-oriented scans (the
  `scan.rs` philosophy: no regex, char-boundary-safe over accented prose).
  - Batch: `### Batch N — <title>` under `## Batch Ledger` in `AILOG-*.md`; id
    keyed `<ailog>#batch-N`.
  - Follow-up: `### FU-NNN — <desc>` under `## Bucket: <name>` in
    `.straymark/follow-ups-backlog.md`; harvests bucket + optional `**Severity**`.
  - Task: `- [ ] T<b>.<n> — <text>` in `specs/**/tasks.md`; id keyed
    `<spec-dir>:T<b>.<n>`.
- `--granularity` selection via `Granularity::parse` (`all` → `None`).

## Verification

- `cargo test -p straymark-baton` ✓ — 5 new unit tests over a new fixture corpus
  (`tests/fixtures/governance-corpus/`) covering all four granularities; full
  suite green, `cargo clippy` clean.
- **Sentinel dogfood** (read-only, scratch test, `git status` unchanged): the
  inventory runs without panic on the real, messier corpus —
  `charter: 45 · batch: 82 · followup: 135 · task: 500`. It also handled
  Sentinel's `T001`-style task ids alongside the fixture's `T1.1` form.

## Impact

Read-only and non-breaking. No model/network/agent dependency (NFR2 holds by
construction — the module has none). Sets up B2 (signal aggregation) and B3
(classifier).

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference. Read-only over the target tree (NFR1).

## Batch Ledger

### Batch 1 — B1: routable-unit inventory (T1.1–T1.4)
Completed 2026-06-26 — this AILOG's PR. `src/units.rs` + fixture corpus + 5 tests;
Sentinel dogfood read-only. Charter → `in-progress`.

### Batch 2 — B2: signal aggregation (T2.1–T2.4)
Completed 2026-06-26. `src/signals.rs`: pure `signals_for(&RoutableUnit) ->
UnitSignals` — cheap-first universal signals (textual `cues` via word-start-prefix
matching, bilingual EN/ES; effort/follow-up carry-forward; `surface_size`; risk
derived from severity). 6 unit tests. Sentinel dogfood (read-only): over 762 real
units, cue spread Audit 155 / Test 175 / Implement 140 / Operate 38 / Fix 15 /
Architecture 14, with **45% no-cue → route up** — an empirical signal that the
cheap title scan alone is insufficient for ~half, i.e. where the heavier deferred
signals (complexity, arch_state, coherence findings) would earn their cost.
**Deferred (calibration-gated):** per-function complexity (`analyze` lives in
`cli`, not reachable from `core`), architecture state (Loom projection), and
Phase-1 coherence findings — wired only if B5 calibration shows the cheap signals
misclassify.

### Batch 3 — B3: cheap classifier (T3.1–T3.4)
Completed 2026-06-26. `src/classify.rs`: pure `classify(&UnitSignals) ->
Classification { class, confidence, rationale }`. Deterministic cue→class map
(Architecture→Planner, Audit→Auditor, Implement/Fix→Implementer,
Operate/Test→Operator); conflict → route up to the highest-rank class; no signal →
Implementer (the escalatable middle, not Operator — never claim cheap); high
risk + unbounded surface → Planner. Reuses `intent::Confidence`. 5 unit tests.
Sentinel dogfood (read-only, 762 units): implementer 491 (64%) / operator 131
(17%) / auditor 126 (16%) / planner 14 (1%); confidence Low 57% / Medium 41% /
High <1%. The **64%-economic vs 57%-low-confidence tension** is the honest crux
B5 must adjudicate under §4.2 — High needs `effort_estimate`, which only charters
carry, so most units classify on cue alone.

### Batch 4 — B4: tier policy + telemetry + CLI (T4.1–T4.5)
Completed 2026-06-26. `src/tiers.rs` (config-driven `baton:` policy + illustrative
defaults), `src/route.rs` (dry-run tier decision + high-risk escalation),
`src/telemetry.rs` (per-granularity `EconomicTelemetry`), CLI `classify` +
`route --dry-run` (`--dry-run` mandatory; no execution path). 17 new unit tests.
Sentinel dogfood (read-only, `git status` clean): **ALL 762 units — all-frontier
$1293.60 → routed $93.68, gross saving $1199.92 (93%), net $1184.68, routable.**
But the honesty guards tell the real story: **57% low-confidence, 57% of the
saving rests on low-confidence routing**, while `conflict_fraction` is only
5–15%. Reading: the granularities are *not* very heterogeneous (low conflict) —
the blocker is **signal coverage**, not mixing. That already tilts the §10.4
answer and is exactly what B5 must adjudicate before claiming the saving is real.

### Batch 5 — B5: Sentinel dogfood + acceptance (T5.1–T5.6)
Completed 2026-06-26. `tests/dry_run_router.rs` (4 acceptance tests: coverage,
telemetry consistency, overhead≥saving → not-routable, read-only). Full dogfood
writeup in [`04-phase2-dry-run-dogfood.md`](04-phase2-dry-run-dogfood.md); concept
§7 marked Phase 2 done; `CHARTER-03` closed with the graduation-gate verdict.
**Gate MET — graduates knowledge:** routing's illustrative ceiling is ~93% and
survives 2× overhead, but only ~43% of units route at high+medium confidence, so
~57% of the saving is a guess. **§10.4 answer (vs the hypothesis): granularity is
not the lever — signal coverage is** (conflict is confounded by title length; Task
has the *highest* conflict, not Charter). Phase 3's data-justified next step: wire
the deferred signals (complexity/arch_state/coherence) to lift confidence before
executing.
