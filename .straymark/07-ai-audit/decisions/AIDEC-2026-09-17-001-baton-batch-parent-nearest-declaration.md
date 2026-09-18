---
id: AIDEC-2026-09-17-001
title: Baton batch parent — nearest declaration (ledger line → AILOG → Charter)
status: accepted
created: 2026-09-17
agent: claude-opus-5-1m
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: [human_ai_config]
iso_42001_clause: []
tags: [baton, inheritance, adopter-feedback]
related:
  - 07-ai-audit/decisions/AIDEC-2026-09-13-001-baton-task-parent.md
---

# AIDEC: Baton batch parent — nearest declaration

## Context

The work-verb ratification (#332 §3) places a batch's declaration on a `- **Work verb**:` line in
its AILOG ledger entry and says batches inherit from their parent by default. The framework's AILOG
template, however, ships `work_verb` / `design_provenance` in the AILOG **frontmatter**, and the
Track C kit told adopters to use that. Baton read neither, so every batch was `undeclared` (#428,
Estoa).

## Problem

What is a batch's parent, and how do the two existing slots relate, without inventing a new slot
and without guessing ownership?

## Alternatives Considered

1. **Ledger line only, no inheritance.** Minimal, but it leaves the AILOG frontmatter slot inert
   and forces a line on every batch of a homogeneous ledger. That is the artificial
   fragmentation the ratification rules out.
2. **Nearest declaration: ledger line → AILOG frontmatter → Charter.** Each existing slot has a
   role. The AILOG frontmatter is the homogeneous default for its batches, and a ledger line is the
   override for a heterogeneous one. When the AILOG declares nothing, the batches inherit from the
   Charters whose ledger the AILOG is.
3. **Inherit from any Charter that mentions the AILOG** (`originating_ailogs` anywhere, or
   `execution_ailogs`). More coverage, but a Charter that was merely *motivated* by an AILOG
   would become its batches' parent.

## Decision

Option 2, chosen by the maintainer on 2026-09-17.

- **Charter link.** It is the one `straymark charter batch-complete` writes through:
  `originating_ailogs[0]`, else `execution_ailogs[0]`. AILOG ids are compared on their first five
  `-` segments, as the CLI resolves them, so `…-028` and `…-028b` never match.
- **Several such Charters must agree.** This is the rule of AIDEC-2026-09-13-001 for tasks.
- **An override replaces the whole declaration.** A verb line without provenance does not inherit
  the parent's provenance, and a provenance line alone declares nothing. The same unit semantics
  apply to `followups declare`.
- **Unreadable sources.** If an AILOG's frontmatter cannot be read, its batches do not inherit from
  anywhere, because it may declare something unseen. Their own lines still apply. An unreadable
  Charter disables Charter-level inheritance project-wide, as for tasks, but not AILOG-level
  declarations.
- **Scope of the scan.** Only live ledger text counts. HTML comments, fenced blocks and any line
  after another heading are ignored.

## Consequences

- The Track C kit and the ratification now describe one contract instead of two.
- Adopters who already declared on the AILOG frontmatter see their batches classified without
  touching anything. Measured read-only: Sentinel 0 → 22 batches, LNXDrive 0 → 7, Estoa 0 → 8.
- A Charter chain sharing one ledger AILOG with disagreeing declarations leaves its batches
  `undeclared`. That is correct, since ownership is then ambiguous. Such batches can be declared
  on their own lines.
- Not decided here: follow-up parent links and declarations on specs.

## Implementation

`experiment-baton/src/units.rs` (`read_batches`, `batch_parent`, `ledger_ailog`, `agreed`),
covered by `experiment-baton/tests/batch_inheritance.rs`.

## References

- `experiment-baton/06-work-verb-schema-ratification.md` §3
- `experiment-baton/07-track-c-adopter-kit.md` §2
- Issues #332, #428
