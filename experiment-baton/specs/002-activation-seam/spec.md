# Feature Spec 002 — Baton activation seam (SpecKit `before_implement` hook)

> **Status:** Implemented · **Charter:** [CHARTER-02-activation-seam.md](../../CHARTER-02-activation-seam.md) · **Closes:** #316 · **Refs:** #304, #303
> **Scope rule:** read-only, no models. Reuses the Phase-1 coherence engine.

## 1. Problem & intent

Phase 1 detects cross-spec contract drift **on demand**. The #304 failure happens
when an agent **implements before** anyone runs the check. Intent: surface the
same findings **at authoring time** — on SpecKit's `before_implement` event —
scoped to the feature about to be coded, so the agent sees the relevant drift
before writing code.

## 2. Users & stories

- **S1 — Implementing agent.** *Before I implement a feature, I want the coherence
  findings for the contracts it consumes, so I don't code against a stale/assumed shape.*
- **S2 — Operator.** *I want this as an opt-in, non-breaking hook with an
  advisory default and an optional hard gate.*

## 3. Surface (contract)

- A SpecKit extension `straymark` (`extension.yml`) providing the command
  `speckit.straymark.coherence-check`, auto-registered on the `before_implement`
  hook.
- A bash script `coherence-check.sh <event>` that discovers the binary, resolves
  the active feature, runs the scoped check, and applies the gate.
- A new CLI flag: `straymark-baton coherence … --spec <feature-id>` — keeps only
  findings about contracts that feature consumes (drops repo-wide C1).

## 4. Functional requirements

- **FR1** — `--spec <id>` scopes the report to the contracts a feature consumes.
- **FR2** — The extension manifest wires `before_implement` → the coherence command.
- **FR3** — The script discovers `straymark-baton` (config `binary:` → `PATH` →
  `cargo`/`$BATON_REPO`) and resolves the feature from `.specify/feature.json`.
- **FR4** — Gate is configurable: `advisory` (default, always continue) or `block`
  (fail `before_implement` on blocking findings).
- **FR5** — Graceful degradation: missing binary / no feature / not a repo never
  breaks the SpecKit flow.

## 5. Non-functional

- **NFR1 — Read-only** (the hook never mutates the target repo; verified on Sentinel).
- **NFR2 — Non-breaking** (advisory by default; binary-absent → skip with note).

## 6. Acceptance criteria

1. `coherence --spec <id>` returns only the feature's contract findings.
2. The extension manifest validates and wires the `before_implement` hook.
3. Dogfood: the hook, run from Sentinel for `005-frontend-dashboard`, surfaces the
   real #304-class C4 finding; `git status` unchanged.

## 7. Out of scope

Models/routing (Phase 2); mutating SpecKit; the generated-type keying limitation
(#313); graduating the engine to `core`; a formal `straymark-baton` release.
