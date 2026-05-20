# What is a "Charter" in StrayMark (and where it fits relative to GitHub SpecKit)

**Version:** 0.2 (Plan → Charter rename)
**Date:** 2026-04-30
**Author:** Jose Villaseñor Montfort — StrangeDaysTech
**Purpose:** Pin the conceptual scope of the **Charter** artifact as it emerges from the Sentinel experiment (where it was called *Plan*), and explain how it coexists with design and development flows already using SpecKit (spec → plan → tasks).
**Related documents:** `straymark-cli-roadmap.md` (§3 Phase 1, §6 Sentinel→CLI mapping), `straymark-thesis-validation.md`, `straymark-charter-telemetry.md` — all preserved in `docs/decisions/proposals/` for historical reference.

---

## 1. What is a Charter in StrayMark

**An ex-ante declaration of a unit of work, large enough to deserve a verifiable contract and an ex-post audit.**

The Charter has three simultaneous faces:

1. **Scope contract** (Context + Scope in/out + Files to modify + Risks): what will be touched and, deliberately, what will *not*. The `Out of scope` section exists specifically so external auditors don't classify it as a gap.
2. **Verification contract** (Local checks + Production smoke): literal executable commands, separated by *when they apply*. The Local/Prod split is not stylistic — it emerges from the fact that in 3/3 cycles external auditors classified prod-only failures as `real_debt`.
3. **Audit anchor** (frontmatter + Charter closure + drift check): the Charter is the subject on which `straymark charter drift` and `straymark charter audit` operate. Without a Charter there is nothing to audit.

### 1.1 Not just technical debt

Of the closed Plans from Sentinel (which are the empirical evidence for the Charter pattern), only one (`03-contract-bumps`) smells like debt. The rest are features (`02-admin-endpoint`, `05-per-service-anomaly-thresholds`), governance work (`01-deploy-governance`), or operational work (`06-baseline-recompute-job`). The README of `sentinel/docs/plans/` says it literally: *"Roadmap of work identified during the closing of the post-MVP backlog"* — backlog, not debt-log.

### 1.2 When a Charter is warranted vs. just an AILOG

Inferred from the corpus, not formally written:

- **AILOG only**: a change that fits in a single session, without pre-declaring files, with low structural risk (typo, point fix, local refactor).
- **Charter + AILOG**: work that touches multiple files, has enumerable ex-ante risks, requires verification with executable commands, and benefits from external post-merge audit. Effort XS–L (minutes to days).
- **ADR + one or more Charters**: deep architectural decision, where each Charter executes a portion of the ADR.

### 1.3 Atomic Charter closure (format v4)

From fw-4.4.2 onward, the `charter-template.md` adopts **format v4**: when the pre-commit drift check (Tasks #7) reports drift between `## Files to modify` and the files actually modified, the implementer must update the Charter atomically — in the same commit/PR as the implementation — adding a `## Closing notes` block that documents what changed, why, and references the corresponding AILOG. **It is not deferred to a post-merge housekeeping PR.**

The pattern emerged empirically from Sentinel's PLAN-07 (closed 2026-05-02), where a legitimate omission drift (a live test turned out to be agnostic to the change, so a file declared in §Files to modify was not touched) was documented late via a separate housekeeping PR, leaving the Plan-doc stale for days. Sentinel's AIDEC-2026-05-02-001 formalized the gap and proposed format v4; this document reflects the upstream evolution. PLAN-05 (`docs/plans/05-per-service-anomaly-thresholds.md`) and PLAN-07 itself are the canonical examples of the `## Closing notes` pattern applied retrospectively — both in Sentinel.

---

## 2. Why it's called "Charter" and not "Plan"

StrayMark originally called this artifact **Plan** during the Sentinel experiment (April 2026). The term was renamed to **Charter** before the CLI implementation for a pragmatic reason: GitHub SpecKit already uses the word "plan" (`/speckit.plan`, `plan.md`) for a different artifact, and the nominal collision generated friction in adopter docs that combine both flows.

Sentinel's historical records (Plans 01–06, `sentinel/docs/plans/`, `sentinel/scripts/check-plan-drift.sh`, AILOGs and YAML telemetries `PLAN-NN.telemetry.yaml`) preserve the original name — they are empirical evidence of a specific moment in time, and rewriting history would falsify the record. References to those files in this document deliberately keep "Plan".

The concept SpecKit calls "plan" and the one StrayMark calls "Charter" are distinct artifacts:

| | **SpecKit `plan.md`** | **StrayMark Charter** |
|---|---|---|
| Granularity | Complete feature (weeks, several stories) | Bounded unit (hours to days, one branch) |
| Dominant content | Stack, dependencies, project structure, constitution gates | Concrete files to touch, verification commands, R<N> risks |
| Sub-artifacts | Generates `research.md`, `data-model.md`, `quickstart.md`, `contracts/` | Generates (when executed) an AILOG |
| Validation | Constitution Check (ex-ante gate) | Drift-check + external audit (ex-post gates) |
| Optimization | *Upfront clarity* | *Ex-post traceability* |

What SpecKit calls "plan" looks more like an **ADR + architectural skeleton**. What StrayMark calls "Charter" looks more like a **task-card with contractual verification and audit anchor**.

---

## 3. The SpecKit flow, mapped piece by piece

SpecKit is linear and top-down: `/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → implement.

| SpecKit | What it solves | Closest equivalent in StrayMark |
|---|---|---|
| `spec.md` (US, FRs, SCs) | *What* to build, tech-agnostic | `01-requirements/REQ-*.md` |
| `plan.md` (stack, structure) | *How* to architect it | `02-design/ADR-*.md` |
| `tasks.md` (T001..TNNN) | *In what order* to execute it | `## Tasks` section inside a StrayMark Charter |
| (does not exist) | Execution log | **AILOG** |
| (does not exist) | Declared vs. executed drift | **`straymark charter drift`** |
| (does not exist) | Multi-model ex-post audit | **`straymark charter audit`** |

The last three rows are StrayMark's specific contribution. SpecKit ends when `tasks.md` is produced; what happens after doesn't matter to the framework — if implementation deviates from the plan, there is no mechanism to detect it. StrayMark enters precisely there.

---

## 4. Three modes in which a StrayMark Charter can coexist with SpecKit

### Mode A — Greenfield with SpecKit as driver

```
spec.md  →  plan.md  →  tasks.md  →  ┐
                                     ├→  Charter-NN (covers US1 + verification + risks)
                                     ├→  Charter-NN+1 (covers US2 + verification + risks)
                                     └→  Charter-NN+2 (covers US3 + verification + risks)
                                                ↓
                                         AILOG per Charter
                                                ↓
                                         drift-check + audit
```

Each StrayMark Charter takes a subset of tasks (typically a user story or a phase) and adds the three layers SpecKit doesn't provide: executable verification, file declaration to measure drift against, and AILOG/audit hookup.

> **What happens when a single spec drives many Charters over weeks?** The SpecKit artifacts (`plan.md`, `data-model.md`, `contracts/`, `tasks.md`) drift relative to the code shipped by intermediate Charters. Naively re-running `/speckit-plan` is *worse* — it regenerates assertions about already-shipped user stories that the actual code does not implement. See `dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md` → **"Spec maintenance during multi-Charter execution"** for the canonical discipline: when to refresh, scope-limited prompt, three mechanical gates (validation against code reality, hunk-by-hunk review, two-PR split), and the explicit warning against re-running `/speckit-tasks` mid-execution.

### Mode B — Maintenance / post-MVP (Sentinel's actual case)

No SpecKit running. Sentinel's Plans 01–06 are born from AILOGs and post-MVP backlog, not from specs. The Charter's `Origen:` field points to an AILOG, not to a `specs/###-feature/`. Here the StrayMark Charter is freestanding.

### Mode C — Hybrid (probably the most realistic)

- Major features → full SpecKit flow + StrayMark Charters when executing tasks.
- Bug fixes, governance, debt, contract changes, small features → StrayMark Charter only.

This is consistent with what Sentinel already shows: of the closed Plans, none had upstream SpecKit because the MVP was already spec'd and closed. What remained was evolutionary work, where the Spec→Plan→Tasks cycle is disproportionate.

---

## 5. The underlying conceptual difference

**SpecKit answers the question:** *"How do I decompose a product idea into executable work?"* — optimizes for upfront clarity and US parallelization.

**StrayMark Charter answers the question:** *"How do I make a unit of work verifiable, auditable, and detectable when it deviates from what was declared?"* — optimizes for ex-post evidence.

That's why they don't compete: they live in different moments of the cycle. SpecKit ends at `tasks.md`; StrayMark Charter starts there (when applied) or stands alone (when there is no prior spec). The Sentinel thesis is not "this replaces SpecKit" — it is "this covers the post-tasks phase that SpecKit doesn't, and covers it with a three-layer pattern (declaration + drift + heterogeneous audit) that has empirical evidence of catching gaps the implementer didn't document".

---

## 6. Note on the Studio vision: "Stage" is not a synonym for Charter

`straymark-studio-vision.md` (an internal-only document) introduces an additional term, **Stage**, which should not be confused with Charter. They are not synonyms: they live at different levels of the cycle.

- **Stage** = a stage of work that traverses the six cognitive phases of the Studio (Spec → Implementation → Audit → Remediation → Governance → Closure & Deliver) and closes with a signed and hash-chained Closure Bundle. Commands proposed in the vision: `straymark stage open/close/status` (not implemented; correspond to Milestone 2 of the evolution toward Studio).
- **Charter** = bounded unit with verification + drift + audit anchor — the artifact this document defines. Charters live *inside* a Stage; a Stage typically contains one or more Charters plus their associated tasks.

The implicit taxonomy of the vision is three-level: **Stage > Charter > Task**. The current CLI roadmap (`straymark-cli-roadmap.md` in `docs/decisions/proposals/`, Phases 1–3) operates entirely at the Charter level (what Sentinel called "Plan"), which is the level where empirical evidence from Sentinel exists. Stage would enter the CLI later as an additional layer, not as a rename of Charter.

## 7. Implication for the StrayMark CLI roadmap

It is worth reflecting in how the `straymark charter new` command is documented when Phase 1 lands (`straymark-cli-roadmap.md` §3.2 in `docs/decisions/proposals/`). Today the planned flag `--from-ailog ID` covers Mode B. Probably a `--from-spec specs/###-feature/` flag should also exist for Mode A — pre-populating `Origen:` pointing at SpecKit's `spec.md`, and optionally inheriting relevant US into the Context section. That is an exercise for Phase 1, not to resolve today, but worth noting in this document as an open decision for when the subcommand is designed.

---

*This document captures the conceptual scope of the Charter artifact (originally called "Plan" in the Sentinel experiment) as understood at the end of April 2026. The definition will continue to evolve as adopters outside Sentinel exercise the pattern in other domains — version 0.3 will be written after observing at least one adopter project complete a Charter with `straymark charter` (Phase 1 exit criterion).*
