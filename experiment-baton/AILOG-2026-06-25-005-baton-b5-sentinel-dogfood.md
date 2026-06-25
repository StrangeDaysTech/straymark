---
id: AILOG-2026-06-25-005
title: Baton B5 — Sentinel dogfood + calibration; Phase 1 graduation gate met
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 260
files_modified: [experiment-baton/src/speckit/specs.rs, experiment-baton/src/provenance.rs, experiment-baton/src/codescan.rs, experiment-baton/src/coherence.rs, experiment-baton/tests/coherence.rs, experiment-baton/tests/fixtures/sample-project/specs/001-backend/post-mvp-backlog.md, experiment-baton/03-sentinel-dogfood-report.md, experiment-baton/CHARTER-01-coherence-bridge.md]
observability_scope: none
tags: [baton, dogfood, sentinel, calibration, graduation]
related: [CHARTER-01-coherence-bridge, AILOG-2026-06-25-004]
---

# AILOG: Baton B5 — Sentinel dogfood + calibration

## Summary

Executed batch **B5** (final) of `CHARTER-01-coherence-bridge` (tasks T5.1–T5.5):
ran the full Coherence Bridge **read-only** against the real Sentinel repo, used
the messy real data to calibrate the heuristics (90 → 6 findings), confirmed the
**graduation gate is met**, and closed the Charter. Full evidence in
[`03-sentinel-dogfood-report.md`](03-sentinel-dogfood-report.md). Touches no
models; zero mutation of Sentinel (verified).

## Context

Fixtures prove correctness on clean data; only a real adopter repo exposes
precision bugs. The first Sentinel run (90 findings) surfaced four — each a real
defect, not a fixture artifact — which B5 fixed.

## Actions Performed

1. **T5.1** — Ran `coherence` + `overlay` read-only on Sentinel (HEAD `24d5a66`).
   Headline: one **precise** C4 — `005-frontend-dashboard` consumes
   `services.public-visibility` without referencing its defining decision PM-001
   / `AILOG-2026-04-21-002` — a real, previously-unflagged #304-class drift. The
   overlay correctly shows `Policy Engine → intended & implemented` (the past gap
   is closed) and real gaps `DevPortal` / `UsageGuard → intended, not implemented`.
2. **T5.2** — NFR1 verified: `git status` in Sentinel empty before/after; HEAD
   unchanged.
3. **Calibration (the real value of the dogfood):**
   - `BacklogDecision.endpoints` + provenance now links a decision only to the
     contracts **its own section names** (not every endpoint its spec mentions) —
     C4 84 → 1.
   - `codescan` excludes test files (`*_test.go`/`*.test.*`/`*.spec.*`/`*.d.ts`)
     as producers — killed a bogus C2 whose producer was a test `mockService`.
   - C1 → `info`/low-confidence with first-word matching (`Identity Module` ↔
     `internal/modules/identity/`); memory findings are hints, not blockers (R1).
   - C4 aggregated to one finding per (spec, contract).
4. **T5.3** — Acceptance: read-only ✓, three output formats ✓, exit codes ✓,
   shared `glob_match` (NFR2, B4) ✓, `cargo test --workspace` + clippy green.
5. **T5.4 / T5.5** — This AILOG + dogfood report; Charter flipped to `closed`
   with closing notes.

## Batch Ledger

### Batch 1 — B1: crate scaffold + SpecKit adapter
Completed 2026-06-25 — AILOG-2026-06-25-001.

### Batch 2 — B2: IntentModel + provenance inference
Completed 2026-06-25 — AILOG-2026-06-25-002.

### Batch 3 — B3: coherence engine (C1–C4) + CLI
Completed 2026-06-25 — AILOG-2026-06-25-003.

### Batch 4 — B4: intent overlay + NFR2 consistency
Completed 2026-06-25 — AILOG-2026-06-25-004.

### Batch 5 — B5: Sentinel dogfood + calibration + close
Completed 2026-06-25 — this AILOG's PR. **Phase 1 complete.**

## Modified Files

| File | Description |
|---|---|
| `experiment-baton/src/speckit/specs.rs` | `BacklogDecision.endpoints` (precise decision→contract link) |
| `experiment-baton/src/provenance.rs` | `defined_by` from decision endpoints, not spec-wide |
| `experiment-baton/src/codescan.rs` | exclude test files as producers |
| `experiment-baton/src/coherence.rs` | C1 → info/low + first-word match; C4 aggregation |
| `experiment-baton/tests/coherence.rs` | C1 severity → Info |
| `experiment-baton/tests/fixtures/.../001-backend/post-mvp-backlog.md` | PM-002 names the health endpoint |
| `experiment-baton/03-sentinel-dogfood-report.md` | New — graduation-gate evidence + limitations + follow-ups |
| `experiment-baton/CHARTER-01-coherence-bridge.md` | status → closed + closing notes |

## Verification

- `cargo test --workspace` ✓ (36 baton tests, no regressions); `clippy` clean.
- Sentinel: `coherence` 6 findings / 0 blocking (was 90); `overlay` legible;
  `git status` unchanged.

## Impact

Read-only throughout. The blocking signal is **0 on Sentinel by design**: the
field/enum-level health mismatch (C2/C3) merges into a coarse contract because
`types.gen.ts` keeps all generated types in one file — documented as the top
follow-up. The #304 **decision-propagation** class (C4) is caught precisely.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference.

## Follow-ups

See `03-sentinel-dogfood-report.md` §6: (1) per-type→endpoint keying for generated
type files (unblocks C2/C3 on real frontends), (2) optional explicit
component→path mapping to make C1 trustworthy, (3) graceful EPIPE, (4) the
*activation* seam (SpecKit `before_implement` hook) as the next Charter.
