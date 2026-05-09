# SpecKit ↔ StrayMark Charter Bridge

> **Status**: Empirical pattern (`v0`). Crystallizes after validation against a second domain (Principle #12). Refine via PRs as new use cases surface.

## Problem this document solves

[SpecKit](https://github.com/StrangeDaysTech/speckit) gives you `spec.md`, `plan.md`, and `tasks.md` for a feature. StrayMark gives you Charters, AILOGs, AIDECs, ADRs. **No canonical doc explained when a SpecKit feature should yield a Charter, how granular it should be, who triggers the creation, or when.** Reported as the central artifact of [issue #113](https://github.com/StrangeDaysTech/straymark/issues/113) — a discoverability gap that left agents (Claude, Gemini, Copilot) building binary mental models (`SpecKit = planning, StrayMark = audit-trail`) and silently dropping the third layer (work-as-auditable-shippable-unit) where Charters live.

This file is the answer.

## Mental model

Three layers, with handoffs:

| Layer | Lives in | Purpose | Owner |
|-------|----------|---------|-------|
| **1. Specification** | `specs/NNN-feature/{spec,plan,tasks,research,quickstart}.md` | What the feature is, why it exists, how it'll be implemented at a technical level. SpecKit's `/speckit-specify` → `/speckit-plan` → `/speckit-tasks` produces these. | Operator (with agent assist). |
| **2. Bounded execution unit** | `.straymark/charters/NN-slug.md` | The contract for a single shippable cut of the feature. Pairs ex-ante scope (files, risks, tasks subset) with ex-post telemetry (drift, audit, lessons). | Operator declares the Charter; agent executes within it. |
| **3. Implementation trace** | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` (and AIDECs, ADRs as warranted) | Day-to-day record of what was actually done, why, with what confidence. Each AILOG references the Charter via `originating_charter:` (or the Charter aggregates them via `originating_ailogs:`). | Agent creates as it works; operator reviews. |

**The bridge is the Charter.** Specs are too high-level to drift-check ("did you ship the spec?" is unanswerable in a useful timeframe). AILOGs are too low-level to ship against ("did you ship this AILOG?" is the wrong unit). Charters sit at the right granularity: a stable scope contract you can audit against in days, not months.

## When does a SpecKit feature yield a Charter?

A SpecKit feature should produce **at least one Charter** when *any* of the following hold:

1. The feature's `tasks.md` has **5 or more tasks** that you cannot complete in a single session.
2. The feature spans **2 or more SpecKit phases** (Setup, Foundation, User Stories, Polish, etc.) that you intend to ship together as one unit.
3. The work warrants **external audit** (cross-model review, cross-team review) at completion.
4. You want **measurable telemetry** at close (effort estimate vs. actual, drift count, lessons).

It should **not** produce a Charter when:

- The feature is small enough to ship in one session (<1 day, <5 tasks). Use AILOGs alone — the Charter overhead exceeds the auditability gain.
- The feature is **purely planning** (no code yet). Wait until `tasks.md` exists; the Charter contract needs concrete tasks to enumerate.
- The feature is **maintenance** without a planned scope (e.g., "fix bugs as they come"). For ad-hoc maintenance, AILOGs are sufficient.

## Granularity heuristics

When a feature warrants Charters, choose granularity by **shippable unit**, not by structural unit. Concretely:

### Heuristic 1 — One Charter per shippable cut

If the feature has Phases (e.g., SpecKit's typical Foundation → US1 → US2 → US3 → Polish), the **first Charter wraps the foundation cut** (everything that ships together as `v0.1`). Subsequent Charters wrap subsequent cuts. Effort estimate **M** is the median bucket for a shippable cut; **L** for a full-feature cut.

```
specs/001-peek-mvp-foundation/
├── spec.md
├── plan.md
└── tasks.md  →  CHARTER-01 (Foundation: T001-T012, effort M)
                  CHARTER-02 (peek MVP: T013-T044, effort L)
```

### Heuristic 2 — NOT per User Story

User Stories are too granular. A US that takes 2-3 tasks belongs *inside* a Charter, not as its own Charter. Telemetry per US is noise; telemetry per shippable cut is signal.

### Heuristic 3 — NOT per feature

A feature that ships in two cuts (e.g., MVP → polish) deserves two Charters, not one. The Charter contract you can drift-check is "what shipped in this cut", not "what we eventually built".

### Heuristic 4 — Edge case: ≥10 tasks across 4+ phases

When a feature is exceptionally large, a third Charter (or splitting the foundation cut into "scaffolding" + "core") may be warranted. Use effort estimate **L** as the cap; if you'd estimate **XL**, that's a sign the feature should be re-specified.

## Creation timing

```
/speckit-specify  → spec.md
/speckit-plan     → plan.md
/speckit-tasks    → tasks.md
                    ↓
                ┌────────────────────────────────────────┐
                │  ★ CHARTER DECLARATION POINT ★         │
                │                                        │
                │  Operator runs `straymark charter new` │
                │   --from-spec specs/NNN-feature/spec.md│
                │   --type <M|L>                         │
                │                                        │
                │  Charter status: declared              │
                │  → Operator fills scope, files, tasks  │
                │  → status: in-progress when execute    │
                └────────────────────────────────────────┘
                    ↓
/speckit-implement  → tasks executed
                    → AILOGs created (`originating_charter:` → Charter)
                    ↓
straymark charter drift CHARTER-NN  → file-vs-commit check
straymark charter audit CHARTER-NN  → external audit (optional)
straymark charter close CHARTER-NN  → telemetry, status: closed
```

**Key invariant**: declare the Charter *before* `/speckit-implement` starts. The Charter is a contract; declaring it after execution defeats the drift check.

## Frontmatter linkage

The Charter's frontmatter explicitly cites the SpecKit feature:

```yaml
charter_id: CHARTER-01-workspace-foundation
status: declared
effort_estimate: M
trigger: tasks.md has 12 ordered tasks across 2 phases; ship as v0.1.
originating_spec: specs/001-peek-mvp-foundation/spec.md
```

The reverse direction (spec → Charter) is by convention — list the active Charter in the spec's "Phase 5: Implementation Tracking" section if your `plan.md` template has one. SpecKit currently has no schema slot for this; emerging convention.

AILOGs created during execution should cite the Charter:

```yaml
id: AILOG-2026-05-08-005
title: T013, T016-T026 — US1 P1 MVP core + TUI + peek bin
agent: claude-code-v4.7
confidence: high
risk_level: medium
review_required: false
originating_charter: CHARTER-02-peek-mvp-foundation
```

## Lifecycle map

| SpecKit phase | Charter event | StrayMark CLI |
|---------------|---------------|---------------|
| `/speckit-tasks` complete | **Declare Charter** | `/straymark-charter-new` skill or `straymark charter new --from-spec …` |
| First task starts | Operator flips `declared` → `in-progress` | (manual frontmatter edit) |
| Each task executed | AILOG produced (when warranted by §6 of STRAYMARK.md) | `/straymark-ailog` |
| Major decision encountered | AIDEC produced | `/straymark-aidec` |
| Architectural shift | ADR produced | `/straymark-adr` |
| Last task done, before close | Drift check | `straymark charter drift CHARTER-NN` |
| Optional external review | Multi-model audit | `straymark charter audit CHARTER-NN` + `/straymark-audit-prompt` + `/straymark-audit-execute` + `/straymark-audit-review` |
| Cut shipped | Close Charter | `straymark charter close CHARTER-NN` (status: `closed`, telemetry yaml emitted) |

## Anti-patterns

**Don't open a Charter "to be safe".** A Charter without a clear shippable cut becomes a wishlist. Operators end up closing it as `closed: aborted` and the telemetry is meaningless.

**Don't open a Charter per User Story.** Telemetry-per-US is too noisy to inform future estimates. Aggregate.

**Don't skip the `originating_spec` field.** Even if the Charter wraps work that doesn't have a SpecKit spec, set `originating_ailogs:` instead. Charters with no origin are an anti-pattern (they signal undocumented motivation).

**Don't run `straymark charter audit` without the auditor CLIs available.** The audit is orchestration-only — `straymark` does not call LLM APIs. If you don't have N auditor CLIs ready, skip the step; close the Charter without external audit.

**Don't flip status to `closed` before drift check + telemetry yaml.** `straymark charter close` does both atomically; manual closure skips invariants.

## When this pattern doesn't fit

This bridge assumes a SpecKit-driven feature flow with multi-task, multi-session implementation. It does not fit:

- **Single-session features** — use AILOGs alone.
- **Architecture-only work with no implementation** (e.g., "design the next-gen schema") — use ADRs.
- **Pure refactors with no new behavior** — use AILOGs + tag with `refactor:`.
- **Incident response and hotfixes** — use INC + AILOG.
- **Compliance-only deliverables** (e.g., quarterly DPIA refresh) — use the relevant doc type directly.

If your work fits one of those, *declare no Charter*. The cost of a Charter exceeds the value when there's no shippable cut to wrap.

## See also

- `STRAYMARK.md` §6 (When to Document) and §15 (Charters as bounded units of work)
- `.straymark/templates/charter/charter-template.md` — declarative template
- `.straymark/templates/charter/charter-telemetry-template.yaml` — telemetry template
- `.straymark/schemas/charter.schema.v0.json` — JSON Schema for declarative frontmatter
- `.straymark/schemas/charter-telemetry.schema.v0.json` — JSON Schema for telemetry
- `.claude/skills/straymark-charter-new/SKILL.md` (and Gemini / agnostic equivalents)

> **Cited the empirical context** (issue #113): Greenfield Rust CLI/TUI suite, Claude Opus 4.7 onboarding via canonical entry points (`STRAYMARK.md`, project constitution, `CLAUDE.md` checklist, available `/straymark-*` skills, `/straymark-status`). Charters were *eventually* adopted (2 Charters: foundation + MVP) only after explicit user prompt — confirming the gap was systemic, not session-specific. This document removes the gap.

---

*Languages*: English | [Español](i18n/es/SPECKIT-CHARTER-BRIDGE.md) | [简体中文](i18n/zh-CN/SPECKIT-CHARTER-BRIDGE.md)
