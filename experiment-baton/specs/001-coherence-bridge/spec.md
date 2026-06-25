# Feature Spec 001 — Baton Coherence Bridge (intent ↔ governance ↔ architecture)

> **Status:** Draft
> **Experiment:** Baton (sibling of Loom). Phase 1 of [01-baton-concept.md](../../01-baton-concept.md).
> **Charter:** [CHARTER-01-coherence-bridge.md](../../CHARTER-01-coherence-bridge.md)
> **Motivating issue:** StrayMark #304 (cross-spec decision propagation, adopter: Sentinel). Related: #303.
> **Scope rule:** read-only. **Touches no models** (no routing/tier/budget/token/cost — that is Baton Phase 2+).

## 1. Problem & intent

StrayMark governs *implementation* (charters, AILOGs, follow-ups, TDEs); Loom projects the *emergent* architecture from those signals plus the code on disk. The *intended* architecture — the global plan that lives in SpecKit's artifacts (`specs/**` and `.specify/memory/**`) — never enters the graph. Verified: a Charter only validates that `originating_spec` exists, never parses it; Loom's projection (`core/src/architecture/projection.rs`) reads neither `specs/` nor `.specify/memory/`.

The result is silent drift. The intended design and the emergent reality are two planes that nothing reconciles, so a decision in one spec can invalidate a contract assumed by another, and no signal fires until runtime.

**Intent:** ingest SpecKit's intent, reconcile it against governance + the code, and emit a **read-only coherence diagnostic** that makes a contract's source-of-truth singular and surfaces cross-spec decision propagation *structurally* — at authoring/CI time, not at staging.

**The oracle (#304, real Sentinel case):** PM-002 (in `spec 001`'s post-MVP backlog, decided in `AILOG-2026-04-24-006`) extended the health contract to per-component. `spec 005` (frontend) never referenced it and implemented an *assumed* contract (`status`/`score` vs real `state`/`health_score`; enum `GREEN/YELLOW/RED` vs `OPERATIONAL/DEGRADED/MAJOR_OUTAGE/IDLE`; per-component `latency/cpu/memory` that no backend handler models). Mocks encoded the assumption → green tests → staging crash. The information existed in three disconnected places; **no edge** linked them. Phase 1 emits that missing edge as a finding.

## 2. Users & primary stories

- **S1 — Operator / maintainer.** *As the human in command, I want a single command that tells me where the implemented system has drifted from the design SpecKit recorded, so I can correct an agent's plan before it codes against a stale contract.*
- **S2 — Spec author.** *As someone authoring a consumer spec, I want to know which contracts it depends on and whether a later decision changed them, so I don't build against a shape that no longer holds.*
- **S3 — Reviewer / CI.** *As a reviewer, I want a CI-gateable signal (exit ≠ 0) when a high-confidence coherence violation exists, so the triple-mismatch class of bug fails the build instead of staging.*
- **S4 — Loom viewer.** *As an operator looking at the architecture, I want to see the intended plane overlaid on the emergent one, so designed-but-unimplemented components (e.g. PolicyEngine) are visible.*

## 3. The model (contract)

The bridge produces a typed **IntentModel** and reconciles it against the existing governance/architecture state into a **CoherenceReport**. It reuses `straymark-core` types (`ArchModel`, `glob_match`, `drift`) — it does not re-implement matching.

### 3.1 IntentModel (read from SpecKit)

```
IntentModel {
  speckit_version: String,          // from .specify/integration.json (advisory gate)
  specs: [IntentSpec],
  intended_components: [IntendedComponent],   // mined from .specify/memory/ + specs
  contracts: [IntentContract],
}
IntentSpec      { id, title, requirements: [Req], consumes: [ContractRef], decisions: [DecisionRef] }
IntendedComponent { id, label, source: MemoryDoc|Spec, globs?: [String] }   // e.g. PolicyEngine
IntentContract  { id, producer: SourceRef, fields: [Field], enums: [EnumDef], defined_by: [DecisionRef] }
DecisionRef     { kind: PM|AILOG|AIDEC|ADR|Charter, id, location }
```

### 3.2 Provenance edge (the core of #304) — **typed**

A directed edge connecting a contract **consumer** to its **producer/decision**:

```
ProvenanceEdge {
  consumer: { spec, requirement },          // spec005:FR-010
  contract: ContractId,                     // statuscenter:health
  producer: SourceRef,                      // statuscenter/handler.go
  defined_by: [DecisionRef],                // PM-002 / AILOG-2026-04-24-006
  confidence: High|Medium|Low,
}
```

Phase 1 **infers** edges (conservative). Optional explicit declaration is a later extension (see §10).

### 3.3 CoherenceReport (computed)

A list of `Finding`s plus coverage metadata. Each `Finding` is traceable: `{ id, class, severity, confidence, locations: [SourceRef], message, evidence }`.

## 4. CLI surface (contract)

```
straymark-baton coherence [PATH] [--out FMT] [--speckit DIR] [--min-confidence LEVEL]
```

- `PATH` — project root (default `.`); read-only.
- `--out text|json|markdown` (default `text`).
- `--speckit DIR` — override SpecKit root (default `<root>/.specify` + `<root>/specs`).
- Exit codes: `0` clean, `1` findings at/above the gate, `2` usage/parse error.
- **Read-only invariant:** the process never writes inside `PATH` (verifiable: `git status` unchanged after a run).

## 5. Finding classes (the coherence checks)

Start with **high-confidence classes only** (R3: noise erodes trust). Each maps to the #304 oracle.

| ID | Class | Fires when | #304 mapping |
|---|---|---|---|
| C1 | `intended-not-implemented` | An `IntendedComponent` (from `.specify/memory/`) has **zero** files implementing it (no globs match on disk, no governance reference) | PolicyEngine |
| C2 | `consumer-field-without-producer` | A contract field a consumer requires has **no** producing source (handler/schema/model) | phantom `latency/cpu/memory` |
| C3 | `contract-shape-mismatch` | Producer vs consumer disagree on field names or enum values for the same contract | `state`/`status`, enum mismatch |
| C4 | `consumer-vs-changed-decision` | A consumer spec depends on a contract that a **later** decision (PM/AILOG) changed, with no reference to that decision | spec 005 ↛ PM-002 |

Lower-confidence/future classes (documented, not built in Phase 1): "mock encodes the assumption" (#304 proposal 4); free-form `.specify/memory/` findings emit at `info` severity only (R1).

## 6. Functional requirements

- **FR1** — Parse SpecKit artifacts read-only: `specs/**/{spec,plan,tasks}.md`, `specs/**/post-mvp-backlog.md`, `specs/**/contracts/**`, `.specify/{integration.json,extensions.yml,memory/**}`. Tolerant of free-form `memory/` markdown.
- **FR2** — Build the typed `IntentModel` (§3.1).
- **FR3** — Infer `ProvenanceEdge`s (§3.2) linking consumers to producing decisions/sources.
- **FR4** — Compute the `CoherenceReport` with finding classes C1–C4 (§5), reusing `straymark-core` `glob_match`/`drift` for file matching.
- **FR5** — Emit `text|json|markdown`; exit `0/1/2`; CI-gateable.
- **FR6** — Provide an intent overlay consumable by Loom (intended vs emergent vs code), as a typed projection extension.
- **FR7** — Version-gate the adapter to the detected `speckit_version` (advisory warning, never crash, on an untested version).

## 7. Non-functional requirements

- **NFR1 — Read-only.** No mutation of the target repo. Enforced + tested.
- **NFR2 — Consistency.** File→component matching uses the **same** `straymark-core::drift::glob_match` as `charter drift` and the architecture projection (no second matcher).
- **NFR3 — Determinism & purity.** Parsing and report computation are pure given inputs; the only I/O is reading files. Unit-testable without a live repo.
- **NFR4 — Low false-positive bias.** Prefer a silent false-negative over a noisy false-positive in Phase 1 (R3/R6). Confidence is first-class and gateable.
- **NFR5 — Portability.** No network; works offline on any checkout. SpecKit need not be runnable.

## 8. Acceptance criteria (definition of done for Phase 1)

1. `straymark-baton coherence` runs read-only against a checkout and emits a report in all three formats with correct exit codes.
2. A fixture reproducing the #304 triple-mismatch yields findings **C2 + C3 + C4**; a fixture with a designed-but-unimplemented component yields **C1**.
3. Run read-only against Sentinel surfaces (a) the US1 health-contract drift (#304) and (b) PolicyEngine as `intended-not-implemented`; `git status` in Sentinel is unchanged afterward.
4. File matching is byte-for-byte consistent with `charter drift` (shared matcher; a test asserts it).
5. Loom can render the intent overlay (or consume the typed projection) for at least one repo.
6. `cargo test --workspace` and `cargo clippy` are green.

**Graduation-gate tie-in (Charter):** Phase 1 succeeds if the diagnostic, run read-only against Sentinel, catches at least one real drift (#304 and/or PolicyEngine) that human review let through.

## 9. Out of scope (for this spec)

- Any model routing / tiers / budgets / tokens / cost (Baton Phase 2+).
- Writing patches back to SpecKit (diagnostic only; no mutation).
- Shipping a SpecKit **extension/hook** (the *activation* seam, `before_implement`) — the next Charter.
- Graduating the logic into `straymark-core` (prototype as `straymark-baton`; graduate after validation).
- Re-implementing `speckit.analyze` (intra-spec coherence). Baton covers cross-spec + spec↔code↔governance.
- Executing agents.

## 10. Open questions

- **Q1** — Provenance: inferred-only (this spec) vs. an optional explicit `consumes:` declaration in specs. Recommendation: inferred now, explicit later.
- **Q2** — `.specify/memory/` is free-form human markdown; how much structure to assume? Recommendation: tolerant mining, `info`-severity only for memory-derived findings (R1).
- **Q3** — Crate placement: `straymark-baton` member vs. partial `core` graduation for the Loom overlay (FR6). Recommendation: crate now, minimal typed core touch only if FR6 forces it (R2). See `plan.md` §2.
- **Q4** — Contract identity across producer (Go handler), decision (PM/AILOG), and consumer (TS types): how to key a `ContractId` reliably across languages. To settle in `plan.md`.
