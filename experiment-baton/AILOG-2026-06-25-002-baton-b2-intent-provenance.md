---
id: AILOG-2026-06-25-002
title: Baton B2 — IntentModel + cross-spec contract provenance inference
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 900
files_modified: [experiment-baton/src/lib.rs, experiment-baton/src/scan.rs, experiment-baton/src/intent.rs, experiment-baton/src/codescan.rs, experiment-baton/src/provenance.rs, experiment-baton/src/main.rs, experiment-baton/src/speckit/mod.rs, experiment-baton/src/speckit/specs.rs, experiment-baton/tests/intent_model.rs, experiment-baton/tests/fixtures/sample-project/internal/statuscenter/handler.go, experiment-baton/tests/fixtures/sample-project/web/src/api/types.ts]
observability_scope: none
tags: [baton, intent-model, provenance, contract, coherence-bridge]
related: [CHARTER-01-coherence-bridge, AILOG-2026-06-25-001]
---

# AILOG: Baton B2 — IntentModel + provenance

## Summary

Executed batch **B2** of `CHARTER-01-coherence-bridge` (tasks T2.1–T2.4):
the typed `IntentModel` and **cross-spec contract provenance inference** — the
structural answer to issue #304. The model joins SpecKit intent (B1) with
contract *shapes* mined from code and the *decisions* that defined them, and
emits `ProvenanceEdge`s linking each consumer to its producer and defining
decision. Calibrated on the #304 health contract: a Go producer struct, a TS
consumer interface, and PM-002 resolve to **one** `ContractId`
(`services.health`) at **High** confidence. Read-only; touches no models.

## Context

#304's load-bearing failure was that a contract's truth lived in three
disconnected places (producer code, the consumer spec/code, and a decision in
another spec's post-MVP backlog) with no edge between them. B2 builds those
edges. The hard part (spec Q4) is keying a contract across languages; the
solution is the **normalized endpoint** — the one anchor a Go handler and a TS
type provably share (`GET /api/v1/services/{id}/health` → `services.health`).

## Actions Performed

1. **Refactor** — lifted the low-level scanners into a shared `scan` module
   (`scan_ids`, `scan_endpoints`, new `normalize_endpoint`); `speckit::specs`
   now consumes them (no duplicate matchers; NFR2 spirit).
2. **T2.1** — `intent.rs`: the typed model — `IntentModel`, `IntentContract`,
   `ContractShape` (`Field`/`EnumDef`), `DecisionRef`, `ProvenanceEdge`,
   `Confidence`, `SourceRef`. All serde-serializable and I/O-free (graduation
   path to `core`, Charter R2). `IntentModel::build(root)` orchestrates.
3. **T2.2** — `codescan.rs`: tolerant, line-oriented contract-shape extraction —
   Go structs (json tags) + string-enum const blocks, TS interfaces + string
   unions — each keyed to the nearest `/api/…` endpoint anchor. A file with no
   anchor yields nothing (conservative, R3/R6). Go ⇒ producer, TS ⇒ consumer.
4. **T2.3** — `provenance.rs`: groups shapes by `ContractId`, attaches spec
   consume-hints and backlog decisions (a decision recorded in a spec defines
   every contract that spec references — the co-location anchor), and scores
   confidence (`High` = code producer + code consumer + defining decision).
5. **T2.4** — Calibration fixtures: Go producer (`componentResponse {name,
   state, detail}` + `HealthState` enum) and TS consumer (`ComponentHealth
   {name, status, latency_p95_ms, cpu, memory}` + `GREEN/YELLOW/RED` union),
   encoding the #304 triple-mismatch shapes. 4 integration tests + codescan unit
   tests assert the one-ContractId join, PM-002 linkage, and the High-confidence
   TS→Go edge.
6. CLI `intent` (read-only): dumps contracts + provenance edges (text/json).

## Batch Ledger

### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)
Completed 2026-06-25 — AILOG-2026-06-25-001.

### Batch 2 — B2: IntentModel + provenance inference (T2.1–T2.4)
Completed 2026-06-25 — this AILOG's PR.

### Batch 3 — B3: coherence engine (C1–C4) + CLI (T3.1–T3.5)
Completed 2026-06-25 — see AILOG-2026-06-25-003.

### Batch 4 — B4: Loom intent overlay (T4.1–T4.3)
(pending)

### Batch 5 — B5: Sentinel dogfood + acceptance (T5.1–T5.5)
(pending)

## Modified Files

| File | Description |
|---|---|
| `experiment-baton/src/scan.rs` | New — shared scanners (`scan_ids`, `scan_endpoints`, `normalize_endpoint`) |
| `experiment-baton/src/intent.rs` | New — typed IntentModel + contracts/edges/decisions; `build()` |
| `experiment-baton/src/codescan.rs` | New — heuristic Go/TS contract-shape extraction |
| `experiment-baton/src/provenance.rs` | New — contract grouping + provenance edges + confidence |
| `experiment-baton/src/lib.rs` | Wire new modules |
| `experiment-baton/src/main.rs` | New `intent` subcommand (read-only dump) |
| `experiment-baton/src/speckit/mod.rs` | Use shared `scan` module |
| `experiment-baton/src/speckit/specs.rs` | Use shared `scan` module |
| `experiment-baton/tests/intent_model.rs` | New — B2 calibration tests |
| `experiment-baton/tests/fixtures/.../handler.go` | New — Go producer shape (#304) |
| `experiment-baton/tests/fixtures/.../types.ts` | New — TS consumer shape (#304 assumed contract) |

## Verification

- `cargo build -p straymark-baton` ✓; `cargo clippy -p straymark-baton` clean ✓.
- `cargo test --workspace` ✓ — 24 `straymark-baton` tests now (14 lib + 4 + 6
  integration), no regressions.
- `straymark-baton intent <fixture>` renders the `services.health` contract
  (producer 6 fields + OPERATIONAL enum; consumer 5 fields), `defined by
  PM-002, AILOG-2026-04-24-006`, and the High-confidence TS→Go edge.

## Impact

Adapter/inference only; no behavior change to CLI/core/Loom. The shape
extraction is heuristic by design — it emits nothing rather than guess when no
endpoint anchor is present, keeping B3's findings low-noise.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference.

## Additional Notes

B3 consumes this model to emit the coherence findings: C2 (consumer fields
`latency_p95_ms`/`cpu`/`memory` with no producer field), C3 (`status` vs `state`;
`GREEN/YELLOW/RED` vs `OPERATIONAL/…`), C4 (the frontend spec depends on
`services.health` but never references PM-002). The model already surfaces every
input those checks need.
