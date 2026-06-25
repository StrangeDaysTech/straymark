---
id: AILOG-2026-06-25-001
title: Baton B1 — straymark-baton crate scaffold + read-only SpecKit adapter
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 720
files_modified: [Cargo.toml, experiment-baton/Cargo.toml, experiment-baton/src/lib.rs, experiment-baton/src/main.rs, experiment-baton/src/speckit/mod.rs, experiment-baton/src/speckit/specs.rs, experiment-baton/src/speckit/memory.rs, experiment-baton/tests/speckit_adapter.rs, experiment-baton/tests/fixtures/sample-project]
observability_scope: none
tags: [baton, speckit, coherence-bridge, adapter]
related: [CHARTER-01-coherence-bridge]
---

# AILOG: Baton B1 — SpecKit adapter

## Summary

Executed batch **B1** of `CHARTER-01-coherence-bridge` (tasks T1.1–T1.5 of
`specs/001-coherence-bridge/tasks.md`): scaffolded the experimental
`straymark-baton` workspace member and built the **read-only, version-gated
SpecKit adapter** — the read side of the Coherence Bridge. It mines the intent
inputs (parsed specs + intended components from `.specify/memory/`) that the
coherence engine (B2/B3) will reconcile against governance and code. **Touches
no models** (Phase 1 scope rule). Full workspace suite green (16 new tests).

## Context

Verified earlier (research `02-speckit-integration-research.md`, issue #304)
that StrayMark/Loom never read SpecKit's content: a Charter only validates that
`originating_spec` exists, and the architecture projection reads neither
`specs/` nor `.specify/memory/`. B1 closes the first of the three integration
seams — *reading* the intended plane — without any reconciliation logic yet.

## Actions Performed

1. **T1.1** — `experiment-baton/Cargo.toml` (`straymark-baton`, `publish = false`,
   `straymark-core = { version = "0.9.0", path = "../core" }`), added as the
   fourth workspace member in the root `Cargo.toml`. Lib + bin split so the
   adapter is unit-testable.
2. **T1.2** — `speckit/mod.rs`: `SpecKitSource::discover` (tolerant layout
   detection), `load`/`load_from` orchestration, `read_integration_version`
   from `.specify/integration.json`, and an advisory version gate
   (`TESTED_VERSION_PREFIX = "0.11"`; untested versions warn, never crash —
   FR7). Shared char-boundary-safe `scan_ids` helper (matches both `FR-010`
   and `FR1` shapes, plus `PM-`, `AILOG-`).
3. **T1.3** — `speckit/specs.rs`: per-`specs/<id>/` parsing of `spec.md`
   (title, `FR` requirements, `/api/` consume hints), `post-mvp-backlog.md`
   (`PM-NNN` decisions with status + referenced `AILOG-` ids), and `contracts/`.
   Deterministic (sorted listings), tolerant (absent files → empty).
4. **T1.4** — `speckit/memory.rs`: tolerant miner of `.specify/memory/` —
   `Arquitectura - <X>.md` / `Requisitos - <X>.md` (EN equivalents too) →
   `IntendedComponent`; Arquitectura+Requisitos for the same `<X>` collapse to
   `Both`; `INDEX.md`/constitution/etc. ignored.
5. **T1.5** — `tests/fixtures/sample-project/`: sanitized, Sentinel-shaped
   SpecKit project encoding the #304 intent side (PM-002 → AILOG ref; FR-010
   consuming the health endpoint; a designed-but-unimplemented PolicyEngine).
   6 integration tests + 10 inline unit tests; `cargo clippy` clean.
6. CLI `inspect` (read-only dump, text/json) to exercise the adapter; the
   reconciling `coherence` command (C1–C4) is deferred to B3 as planned.

## Batch Ledger

### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)
Completed 2026-06-25 — this AILOG's PR.

### Batch 2 — B2: IntentModel + provenance inference (T2.1–T2.4)
Completed 2026-06-25 — see AILOG-2026-06-25-002.

### Batch 3 — B3: coherence engine (C1–C4) + CLI (T3.1–T3.5)
(pending)

### Batch 4 — B4: Loom intent overlay (T4.1–T4.3)
(pending)

### Batch 5 — B5: Sentinel dogfood + acceptance (T5.1–T5.5)
(pending)

## Modified Files

| File | Description |
|---|---|
| `Cargo.toml` | New `experiment-baton` workspace member |
| `experiment-baton/Cargo.toml` | New — `straymark-baton` crate manifest (lib + bin) |
| `experiment-baton/src/lib.rs` | New — crate root, `speckit` module |
| `experiment-baton/src/main.rs` | New — `inspect` CLI (read-only dump) |
| `experiment-baton/src/speckit/mod.rs` | New — discovery, version gate, `scan_ids`, `SpecKitArtifacts` |
| `experiment-baton/src/speckit/specs.rs` | New — `specs/**` parser (FR / PM / consume hints / contracts) |
| `experiment-baton/src/speckit/memory.rs` | New — `.specify/memory/` intended-component miner |
| `experiment-baton/tests/speckit_adapter.rs` | New — 6 integration tests over the fixture |
| `experiment-baton/tests/fixtures/sample-project/**` | New — sanitized Sentinel-shaped SpecKit project (#304 intent side) |

## Verification

- `cargo build -p straymark-baton` ✓; `cargo clippy -p straymark-baton` clean ✓.
- `cargo test --workspace` ✓ — 16 new `straymark-baton` tests, no regressions
  (cli 348, core suites unchanged).
- Bin smoke: `straymark-baton inspect <fixture>` renders the #304 intent inputs
  (PM-002 → AILOG-2026-04-24-006; FR-010; PolicyEngine as Architecture-only;
  StatusCenter as Both).

## Impact

Read-only adapter only; no behavior change to the CLI, core, or Loom. The
`straymark-baton` crate is `publish = false` and outside the release matrices,
so no release surface is affected.

## EU AI Act Considerations

Not applicable — developer tooling that parses local documents; no automated
decision-making, no personal data, no model inference.

## Additional Notes

The adapter output types (`SpecKitArtifacts`, `ParsedSpec`, `IntendedComponent`)
are intentionally I/O-free and serde-serializable so B2 can map them into the
typed `IntentModel` + provenance edges, and so the pure logic can later graduate
into `straymark-core` (Charter R2, the Loom A1.0 governance-to-core precedent).
