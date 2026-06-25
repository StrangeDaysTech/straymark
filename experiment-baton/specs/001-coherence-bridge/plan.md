# Implementation Plan 001 — Baton Coherence Bridge

> **Spec:** [spec.md](spec.md) · **Charter:** [CHARTER-01-coherence-bridge.md](../../CHARTER-01-coherence-bridge.md)
> **Scope:** read-only, no models. Phase 1 of Baton.

## 1. Architecture at a glance

```
                 ┌─────────────────────────────────────────────┐
   SpecKit  ───► │ SpecKit adapter (read-only, version-gated)   │
 (.specify/,     │  parse specs + .specify/memory + integration │
  specs/**)      └───────────────────┬─────────────────────────┘
                                     ▼
                        ┌────────────────────────┐
                        │ IntentModel (typed)     │  intended components,
                        │ + ProvenanceEdges       │  contracts, decisions
                        └───────────┬─────────────┘
   straymark-core ──────────────────┤  (ArchModel, glob_match, drift, charter, ailog)
   (governance + emergent arch)      ▼
                        ┌────────────────────────┐
                        │ Coherence engine        │  C1–C4 finding classes
                        │ intent ⨯ governance ⨯ code
                        └───────────┬─────────────┘
                                    ▼
                 ┌──────────────┬───────────────────────┐
                 │ CLI report   │ Loom intent overlay     │
                 │ text/json/md │ (typed projection ext)  │
                 └──────────────┴───────────────────────┘
```

The bridge is a **consumer** of `straymark-core`: it reuses the document model, `glob_match`, `drift`, charter/AILOG parsing. It adds only the SpecKit-side parsing, the intent model, provenance inference, and the coherence engine.

## 2. Crate placement (the load-bearing decision)

Concept §10.6 says the Bridge "lives in `core`". To avoid coupling `straymark-core` to experimental code (Charter R2), Phase 1 prototypes it as a **new workspace member `straymark-baton`** under `experiment-baton/`, mirroring how `straymark-loom` lives apart yet depends on `core`.

- `experiment-baton/Cargo.toml` → `straymark-baton`, `straymark-core = { path = "../core", version = "..." }`.
- Add `experiment-baton` to the root `Cargo.toml` `members` (and keep it out of default-release if appropriate, like loom).
- **Graduation path:** the pure types (`IntentModel`, `ProvenanceEdge`, the C1–C4 logic) are written I/O-free so they can `git mv` into `core/src/intent/` later (the Loom A1.0 precedent: governance primitives moved to `core` in their own PR). The only Phase-1 `core` touch permitted is a **minimal typed extension point** in the projection for FR6 (the intent overlay) — and only if reuse can't avoid it.

## 3. SpecKit adapter (FR1, FR7)

- `src/speckit/mod.rs` — entry: locate `.specify/` + `specs/`; read `integration.json` → `speckit_version`; advisory gate (warn on untested version, never crash).
- `src/speckit/specs.rs` — parse `spec.md` (requirements `FR-NNN`, "consumes" hints), `plan.md`, `tasks.md`, `post-mvp-backlog.md` (PM-NNN items + status + the AILOG they reference), `contracts/**`.
- `src/speckit/memory.rs` — **tolerant** miner for `.specify/memory/**`: `Arquitectura - <X>.md` / `Requisitos - <X>.md` → `IntendedComponent { id: <X> }`; the navigation map and vision doc as low-confidence hints. Free-form → `info` severity only (R1).
- Fixtures under `experiment-baton/tests/fixtures/` mirroring the Sentinel shapes (sanitized).

## 4. Intent model + provenance (FR2, FR3)

- `src/intent.rs` — the typed structs of spec §3.1/§3.2 (serde-serializable for `--out json`).
- `src/provenance.rs` — conservative inference:
  - **Producer discovery:** a contract's producer is a source file/schema whose declared shape matches the contract id (heuristic on path + symbol + nearby `json:"..."` tags / enum constants). Cross-language keying (Q4): normalize a `ContractId` from the endpoint/handler name + field set; match consumer TS types and producer Go structs by that key. Start with the health contract as the calibration case.
  - **Decision discovery:** scan governance (AILOGs, post-MVP backlog PM items, AIDEC/ADR) for ones whose footprint (`Modified Files` / `affects`) intersects the producer → `defined_by`.
  - Confidence: High when path+symbol+fields align; Medium on partial; Low → suppressed unless `--min-confidence low`.

## 5. Coherence engine + finding classes (FR4)

- `src/coherence.rs` — pure `fn analyze(intent: &IntentModel, gov: &GovernanceState, disk: &Inventory) -> CoherenceReport`.
- C1 `intended-not-implemented`: `IntendedComponent.globs` (or name→path heuristic) matches **zero** on-disk files via `core::drift::glob_match` **and** no governance doc references it.
- C2 `consumer-field-without-producer`: a consumer-required field has no producer field after provenance resolution.
- C3 `contract-shape-mismatch`: field-name or enum-value set differs between producer and consumer for the same `ContractId`.
- C4 `consumer-vs-changed-decision`: consumer spec's authored date / referenced contract version predates a `DecisionRef` that changed the contract, and the consumer never references that decision.
- Severity: C2/C3 `blocking`, C4 `warning`, C1 `warning` (config later); memory-only inputs cap at `info`.

## 6. CLI (FR5)

- `src/main.rs` — `clap` subcommand `coherence`; mirrors `architecture validate` ergonomics (`--out text|json|markdown`, exit `0/1/2`, `--min-confidence`). Read-only invariant asserted by an integration test that snapshots `git status` before/after.

## 7. Loom intent overlay (FR6)

- Expose the intent projection as a typed structure Loom can render: each architecture component gains an `intent_state` (intended-and-implemented / intended-not-implemented / implemented-not-intended). Wire into Loom's existing overlay machinery (the debt/wiring-gap overlay is the precedent). Keep the `core` touch minimal and typed (R2); no Baton logic inside `core`.

## 8. Phasing (each batch = a reviewable increment)

| Batch | Deliverable | FRs |
|---|---|---|
| **B1** | `straymark-baton` crate scaffold + SpecKit adapter (specs + memory) + fixtures | FR1, FR7 |
| **B2** | IntentModel + provenance inference (health contract as oracle) | FR2, FR3 |
| **B3** | Coherence engine (C1–C4) + CLI (text/json/md, exit codes, read-only test) | FR4, FR5 |
| **B4** | Loom intent overlay | FR6 |
| **B5** | Dogfood read-only on Sentinel + AILOG + acceptance | §8 |

Multi-batch → maintain `## Batch Ledger` in the AILOG; run `straymark charter batch-complete CHARTER-01-coherence-bridge N` after each batch's merge.

## 9. Risks

Inherited from the Charter (R1 free-form memory, R2 core coupling, R3 false positives, R4 SpecKit version, R5 scope creep, R6 provenance ambiguity). Plan-specific:

- **P1 — Cross-language contract keying (Q4)** is the hardest part. Mitigation: calibrate on the health contract; ship C2/C3 only when the key resolves with High confidence; otherwise emit nothing.
- **P2 — Workspace build cost.** Adding a member grows CI build. Mitigation: mirror loom's release isolation; keep `straymark-baton` out of the default release matrix.

## 10. References

- Concept [01-baton-concept.md](../../01-baton-concept.md) · Research [02-speckit-integration-research.md](../../02-speckit-integration-research.md)
- `core/src/architecture/projection.rs`, `core/src/drift.rs` (`glob_match`), `core/src/charter*.rs`, `core/src/document.rs`
- Loom precedent: `experiment-loom/specs/002-architecture-plan/` (A1.0 governance-primitives-to-core move; overlay machinery)
- Issue #304 (oracle), #303 (audit-time complement)
