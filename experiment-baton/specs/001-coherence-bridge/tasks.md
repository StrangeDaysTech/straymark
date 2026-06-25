# Tasks 001 — Baton Coherence Bridge

> **Spec:** [spec.md](spec.md) · **Plan:** [plan.md](plan.md) · **Charter:** [CHARTER-01-coherence-bridge.md](../../CHARTER-01-coherence-bridge.md)
> Read-only, no models. Each batch is its own PR; maintain the `## Batch Ledger` in the execution AILOG.

## B1 — Crate scaffold + SpecKit adapter (FR1, FR7)

- [ ] T1.1 — `experiment-baton/Cargo.toml`: `straymark-baton` bin crate; `straymark-core = { path = "../core" }`. Add `experiment-baton` to root `Cargo.toml` `members`; keep out of the default release matrix (loom precedent).
- [ ] T1.2 — `src/speckit/mod.rs`: locate `.specify/` + `specs/`; read `integration.json` → `speckit_version`; advisory version gate (warn, never crash) on a version other than the tested `0.11.x`.
- [ ] T1.3 — `src/speckit/specs.rs`: parse `spec.md` (`FR-NNN`, consumes hints), `post-mvp-backlog.md` (`PM-NNN` + status + referenced AILOG id), `contracts/**`. Tolerant; no panics on shape variation.
- [ ] T1.4 — `src/speckit/memory.rs`: tolerant miner — `Arquitectura - <X>.md` / `Requisitos - <X>.md` → `IntendedComponent`. Low-confidence; memory-only → `info`.
- [ ] T1.5 — Fixtures under `experiment-baton/tests/fixtures/` (sanitized Sentinel shapes: health contract producer/consumer + a designed-but-unimplemented component). Unit tests for the adapter.

## B2 — IntentModel + provenance (FR2, FR3)

- [ ] T2.1 — `src/intent.rs`: typed structs (spec §3.1/§3.2), serde for `--out json`.
- [ ] T2.2 — `src/provenance.rs`: producer discovery (path+symbol+field/enum-tag heuristic), `ContractId` normalization across languages (Q4), decision discovery via governance footprint intersection.
- [ ] T2.3 — Confidence scoring (High/Medium/Low); Low suppressed unless `--min-confidence low`.
- [ ] T2.4 — Calibrate on the health contract fixture: producer (Go-struct shape) ↔ consumer (TS-type shape) ↔ PM-002 decision resolve to one `ContractId` at High confidence. Tests.

## B3 — Coherence engine + CLI (FR4, FR5)

- [ ] T3.1 — `src/coherence.rs`: pure `analyze(intent, gov, disk) -> CoherenceReport`. Reuse `core::drift::glob_match` (NFR2); no second matcher.
- [ ] T3.2 — Finding classes C1–C4 (spec §5) with severities; memory-derived findings capped at `info` (R1).
- [ ] T3.3 — `src/main.rs`: `clap` `coherence [PATH] --out text|json|markdown --speckit DIR --min-confidence LEVEL`; exit `0/1/2`.
- [ ] T3.4 — Read-only integration test: snapshot `git status` before/after a run on a fixture repo → unchanged (NFR1).
- [ ] T3.5 — Consistency test (NFR2): the file→component matching matches `charter drift` byte-for-byte on a shared input.

## B4 — Loom intent overlay (FR6)

- [ ] T4.1 — Typed intent projection: per-component `intent_state` (intended-and-implemented / intended-not-implemented / implemented-not-intended). Pure; I/O-free.
- [ ] T4.2 — Wire into Loom's existing overlay machinery (debt/wiring-gap overlay precedent). Minimal, typed `core` touch only if reuse can't avoid it (R2).
- [ ] T4.3 — Render check on at least one repo.

## B5 — Dogfood Sentinel + acceptance (spec §8)

- [ ] T5.1 — Run `straymark-baton coherence` **read-only** against `/home/montfort/StrangeDaysTech/sentinel`. Confirm it surfaces (a) the US1 health-contract drift (#304: C2/C3/C4) and (b) PolicyEngine as `intended-not-implemented` (C1).
- [ ] T5.2 — Verify `git status` in Sentinel is unchanged after the run (NFR1, acceptance #3).
- [ ] T5.3 — Acceptance pass (spec §8.1–§8.6): three output formats + exit codes; fixture findings; shared-matcher test; clippy + `cargo test --workspace` green.
- [ ] T5.4 — Execution AILOG (`risk_level`, `review_required`); `## Batch Ledger` reconciled; `straymark charter batch-complete` per batch.
- [ ] T5.5 — Update concept §7 roadmap (Phase 1 → done) and the Charter closure; resolve spec §10 open questions as settled.

## Verification (per batch + at close)

- [ ] Local: `cargo build -p straymark-baton`, `cargo test --workspace`, `cargo clippy` green in a clean shell.
- [ ] Dogfood (B5): read-only run on Sentinel reproduces #304 + PolicyEngine; zero mutations.
- [ ] Drift check: `straymark charter drift CHARTER-01-coherence-bridge <range>` clean pre-commit and at close.
