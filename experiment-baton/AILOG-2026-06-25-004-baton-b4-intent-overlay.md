---
id: AILOG-2026-06-25-004
title: Baton B4 — Loom-consumable intent overlay + NFR2 matcher consistency
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 420
files_modified: [experiment-baton/src/lib.rs, experiment-baton/src/overlay.rs, experiment-baton/src/main.rs, experiment-baton/tests/overlay.rs, experiment-baton/tests/fixtures/sample-project/architecture/model.yml]
observability_scope: none
tags: [baton, overlay, loom, architecture, nfr2]
related: [CHARTER-01-coherence-bridge, AILOG-2026-06-25-003]
---

# AILOG: Baton B4 — intent overlay

## Summary

Executed batch **B4** of `CHARTER-01-coherence-bridge` (tasks T4.1, T4.3 +
the deferred T3.5): the **intent overlay** — the third plane Loom can consume,
laying *intention* (`.specify/memory`) over the *emergent* architecture model
(`model.yml`) and code. Per component it computes one of three states:
`intended-and-implemented`, `intended-not-implemented` (the PolicyEngine gap),
`implemented-not-intended` (code the design never named). File→component
matching reuses **`straymark_core::drift::glob_match`** and the source inventory
comes from **`straymark_core::architecture::collect_source_files`** — one
matcher/scanner project-wide (NFR2). Read-only; touches no models.

## Context

B3's findings (C1–C4) are list-shaped. B4 produces the *spatial* view the
operator reads in Loom: which architecture components are designed, implemented,
or drifting. FR6 asked for an overlay "consumable by Loom" as a typed projection
extension; this delivers the typed, serde-serializable `OverlayReport` (text +
json) that Loom's frontend can render. Baton consumes `core`'s public
architecture API directly — **no `core` modification** (Charter R2).

## Actions Performed

1. **T4.1** — `overlay.rs`: pure `compute(model, intended, inventory) ->
   Vec<ComponentIntent>` with `IntentState`. Matches intended components
   (`.specify/memory`) to `model.yml` components by id/containment/glob, and
   surfaces intended components absent from the model (PolicyEngine) as
   `intended-not-implemented`. `OverlayReport::build(root)` does the read-only
   I/O (locate `model.yml`, build the IntentModel, inventory via `core`).
2. **T3.5 (deferred from B3)** — `owned_files(component, inventory)` uses
   `core::drift::glob_match`; an inline test asserts it equals a direct
   `glob_match` fold (NFR2: no second matcher, byte-for-byte with `charter
   drift`/the Loom projection).
3. **T4.3** — `overlay` CLI subcommand (text/json) + a fixture
   `architecture/model.yml` (components `statuscenter`, `web-api`). Render check:
   `statuscenter` ⇒ intended-and-implemented, `policyengine` ⇒
   intended-not-implemented (not modeled), `web-api` ⇒ implemented-not-intended.

## Scope note — Loom web rendering is a follow-on

B4 delivers the overlay **consumable by Loom** (the typed `OverlayReport` +
JSON), satisfying FR6. Rendering it inside Loom's web UI (a server endpoint +
the Vite/TS frontend overlay) is a **Loom-side change**, intentionally not in
this Baton batch — it would couple a Baton PR to the experiment-loom frontend
build. Flagged so the visual wiring is tracked as its own follow-up, not assumed
done.

## Batch Ledger

### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)
Completed 2026-06-25 — AILOG-2026-06-25-001.

### Batch 2 — B2: IntentModel + provenance inference (T2.1–T2.4)
Completed 2026-06-25 — AILOG-2026-06-25-002.

### Batch 3 — B3: coherence engine (C1–C4) + CLI (T3.1–T3.4)
Completed 2026-06-25 — AILOG-2026-06-25-003.

### Batch 4 — B4: intent overlay + NFR2 consistency (T4.1, T4.3, T3.5)
Completed 2026-06-25 — this AILOG's PR. (Loom web rendering → follow-on.)

### Batch 5 — B5: Sentinel dogfood + acceptance (T5.1–T5.5)
Completed 2026-06-25 — see AILOG-2026-06-25-005.

## Modified Files

| File | Description |
|---|---|
| `experiment-baton/src/overlay.rs` | New — typed intent overlay; `owned_files` via core `glob_match` (NFR2) |
| `experiment-baton/src/main.rs` | New `overlay` subcommand (text/json) |
| `experiment-baton/src/lib.rs` | Wire `overlay` module |
| `experiment-baton/tests/overlay.rs` | New — all three intent states |
| `experiment-baton/tests/fixtures/.../architecture/model.yml` | New — fixture model for the overlay |

## Verification

- `cargo clippy -p straymark-baton` clean ✓; `cargo test --workspace` ✓ — 36
  `straymark-baton` tests (15 lib + 7 coherence + 4 intent + 4 overlay + 6
  speckit), no regressions.
- `straymark-baton overlay <fixture>` renders the three states correctly.
- NFR2: `owned_files` proven equal to a direct `core::drift::glob_match` fold.

## Impact

Read-only; consumes `core`'s public architecture API (no `core` change). The
overlay is the Loom-facing product of the Coherence Bridge; the only remaining
Phase-1 work is B5 (dogfood on Sentinel = the graduation gate).

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference.

## Additional Notes

B5 runs the full bridge (`coherence` + `overlay`) read-only against Sentinel to
confirm it catches the real US1 health-contract drift and the unimplemented
PolicyEngine, satisfying the Charter's graduation gate.
