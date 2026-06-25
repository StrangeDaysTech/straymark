---
id: AILOG-2026-06-25-003
title: Baton B3 — coherence engine (C1–C4) + coherence CLI
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 600
files_modified: [experiment-baton/src/lib.rs, experiment-baton/src/coherence.rs, experiment-baton/src/main.rs, experiment-baton/src/speckit/specs.rs, experiment-baton/tests/coherence.rs, experiment-baton/tests/fixtures/sample-project/specs/005-frontend/spec.md]
observability_scope: none
tags: [baton, coherence, findings, ci-gate, coherence-bridge]
related: [CHARTER-01-coherence-bridge, AILOG-2026-06-25-002]
---

# AILOG: Baton B3 — coherence engine

## Summary

Executed batch **B3** of `CHARTER-01-coherence-bridge` (tasks T3.1–T3.4): the
coherence engine that turns the `IntentModel` (B2) into actionable findings, and
the CI-gateable `coherence` CLI. Run read-only against the fixture it catches
the #304 drift end-to-end — **C1** (PolicyEngine designed-not-implemented),
**C2** (consumer fields `status/latency_p95_ms/cpu/memory` with no producer),
**C3** (enum `OPERATIONAL/…` vs `GREEN/…`), **C4** (spec 005 consumes the health
contract but never references PM-002) — and exits 1 on the blocking pair. This is
the batch where Baton starts *catching* drift, not just modeling it. Touches no
models.

## Context

B1 read SpecKit intent; B2 joined it with code shapes into contracts +
provenance edges. B3 reconciles all three planes (intent ⨯ governance ⨯ code)
and emits the four high-confidence finding classes of spec §5. The engine is a
pure function over `(IntentModel, Inventory)`; the only I/O is building those
inputs (read-only, NFR1).

## Actions Performed

1. **T3.1** — `coherence.rs`: pure `analyze(&IntentModel, &Inventory) ->
   CoherenceReport`; `CoherenceReport::build(root)` does the read-only I/O.
   `Inventory::scan` lists implementation files, deliberately **excluding
   `.specify/` and `specs/`** so a component isn't deemed "implemented" merely
   because its own memory doc names it.
2. **T3.2** — finding classes C1–C4 with severities (C2/C3 `blocking`, C1/C4
   `warning`) and per-finding confidence (inherited from the contract's best
   provenance edge; C1 `medium`). `Severity` ordered; `--min-confidence` filter.
3. **T3.3** — `coherence` CLI subcommand: `--out text|json|markdown`,
   `--min-confidence low|medium|high`, **exit 1** when any reported finding is
   blocking (CI-gateable); exit 2 on usage (clap).
4. **T3.4** — read-only test (NFR1): a before/after file snapshot of the fixture
   proves `build` mutates nothing.
5. Adapter extension: `ParsedSpec.referenced_decisions` (scans a spec's own
   spec/plan/tasks for `PM-/AILOG-/AIDEC-/ADR-` ids) so C4 can tell whether a
   consumer acknowledges the defining decision; the decision's home spec is
   exempt. Fixture `005-frontend/spec.md` reworded so it no longer cites PM-002
   literally (which would have masked the very gap C4 detects).

## Scope note (T3.5 deferred to B4)

T3.5 (the NFR2 "shares `glob_match` with `charter drift`" consistency test) moves
to **B4**: B3's C1 uses path-substring matching because memory-derived components
carry no globs, so there is no glob matcher in B3 to test for divergence. B4's
Loom intent overlay joins with the architecture model (which *does* carry globs
and uses `core::drift::glob_match`), where the consistency assertion is
meaningful. Flagged here so the task isn't silently dropped.

## Batch Ledger

### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)
Completed 2026-06-25 — AILOG-2026-06-25-001.

### Batch 2 — B2: IntentModel + provenance inference (T2.1–T2.4)
Completed 2026-06-25 — AILOG-2026-06-25-002.

### Batch 3 — B3: coherence engine (C1–C4) + CLI (T3.1–T3.5)
Completed 2026-06-25 — this AILOG's PR. (T3.5 → B4, see scope note.)

### Batch 4 — B4: Loom intent overlay + NFR2 consistency (T4.1–T4.3, T3.5)
Completed 2026-06-25 — see AILOG-2026-06-25-004.

### Batch 5 — B5: Sentinel dogfood + acceptance (T5.1–T5.5)
(pending)

## Modified Files

| File | Description |
|---|---|
| `experiment-baton/src/coherence.rs` | New — `Inventory`, finding classes C1–C4, pure `analyze`, `CoherenceReport` |
| `experiment-baton/src/main.rs` | New `coherence` subcommand (text/json/markdown, exit gate) |
| `experiment-baton/src/lib.rs` | Wire `coherence` module |
| `experiment-baton/src/speckit/specs.rs` | `ParsedSpec.referenced_decisions` (for C4) |
| `experiment-baton/tests/coherence.rs` | New — C1–C4 catch + read-only (NFR1) tests |
| `experiment-baton/tests/fixtures/.../005-frontend/spec.md` | Reword so it doesn't cite PM-002 literally |

## Verification

- `cargo clippy -p straymark-baton` clean ✓; `cargo test --workspace` ✓ — 31
  `straymark-baton` tests (14 lib + 7 coherence + 4 intent + 6 speckit), no
  regressions.
- `straymark-baton coherence <fixture>` emits C1–C4 and **exits 1**;
  `--out markdown` renders a findings table; `--min-confidence high` keeps only
  the blocking C2+C3 pair.

## Impact

Read-only analysis only; no behavior change to CLI/core/Loom. C2/C3 are blocking
(concrete shape mismatch), C1/C4 are warnings (inferential) — conservative gate
to keep CI signal trustworthy (R3).

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference.

## Additional Notes

B5 will run this engine read-only against Sentinel to confirm it catches the real
US1 health-contract drift and the unimplemented PolicyEngine — the Charter's
graduation gate. B4 adds the Loom visual overlay and the deferred NFR2 test.
