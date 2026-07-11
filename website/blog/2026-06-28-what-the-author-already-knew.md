---
slug: what-the-author-already-knew
title: What the author already knew
authors:
  - jose
tags:
  - straymark
  - baton
  - cost-routing
  - framework
  - schema
  - sentinel
date: 2026-06-28T00:00:00.000Z
description: The dry run proved that signal coverage — not granularity — was what kept 57% of the routing on low-confidence guesses. The fix was almost free. The person creating a unit of work already knows whether it's design, implementation, an audit, or mechanical labor; they know it for the cost of typing one word. So Baton discontinued title-scanning entirely and made a declared verb the single authoritative signal — then graduated it to the framework.
---
*Phase 2 ended on an uncomfortable number: 57% of the routing rested on guesses, because 45% of the work units surfaced no cue a cheap classifier could read. The instinct was to reach for a more powerful classifier. The right answer was the opposite — stop inferring the thing at all. The person who creates a unit of work already knows whether it's design, implementation, an audit, or mechanical labor. They know it for free, at authoring time, for the cost of one declared word. So Baton threw out title-scanning as a classification input, made a declared `work_verb` the sole authoritative signal, ratified the vocabulary against a real 762-unit corpus, and graduated it into the framework as a first-class field.*

<!-- truncate -->

> *Undeclared is an honest state, not an error. A unit with no verb routes conservatively up and gets a nudge — never a low-confidence guess from its title.*

This is the third post about **Baton**, and the one where a sibling experiment does what Loom did before it: hands a clean primitive back to the core. After [the coherence bridge](what-the-spec-path-only-proved-existed) and [the dry-run router](what-the-dry-run-would-have-spent), the arc closes not with a routing engine but with a **field** — two of them, actually — added to the framework everyone else uses.

## 1. The pivot: stop guessing what the author can just say

The dry run's finding was specific ([#328](https://github.com/StrangeDaysTech/straymark/issues/328)): the lever on trustworthy routing is *structured signal*, not finer units. And the classifier's weakest input was the one it leaned on hardest — the **title**. Titles were being scanned for cues (`scan_cues`, a `CUE_TABLE`), and titles lie in both directions: a verbose task title manufactured phantom "conflict," a terse charter title surfaced nothing. Worse, title-scanning is trivially **gameable** and inherently **calibrated to one stack** ([#321](https://github.com/StrangeDaysTech/straymark/issues/321)) — the cue words that work in a Go/TypeScript repo don't transfer.

So the architectural turn ([#332](https://github.com/StrangeDaysTech/straymark/issues/332), prototyped in #333) discontinued title-scanning as a routing input entirely and left a single authoritative signal in its place: a **declared verb**, stated by the author when the unit is created. The economics of that are unbeatable — the classification costs **≈ 0 tokens**, because the author already knows the answer with certainty and types it once. The title didn't disappear; it was **demoted** from oracle to an advisory cross-check, used only in the periodic calibration tool, **never** in the routing path.

## 2. The two fields

**`work_verb`** — the authoritative signal, a closed, controlled vocabulary:

| value | meaning |
|---|---|
| `design` | Open architecture / pattern / scalability decision; spec authoring. |
| `implement` | Service logic; a fix with real diagnosis; a complex query; new tooling; **defining a bounded foundational contract**. |
| `audit` | Independent review / verification / contrast against reality. |
| `operate` | Mechanical work: tests, migrations, scaffolding, docs, closure ceremony, bulk edits. |

**`design_provenance`** — `{new, upstream}`, optional, default `new`. It captures residual cognitive load: a unit whose work only *instruments a design already done upstream* is mechanical, even if its surface verb looks like `implement`. `new` means the hard decision happens *in this unit*; `upstream` means the design already exists and this unit only transcribes or wires it.

## 3. The determination rules — where a plain enum isn't enough

An enum alone doesn't resolve the ambiguity that actually bites. The ratification ([`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md)) fixes four rules, each surfaced by real data:

1. **The foundational-contract rule.** "Define a bounded foundational contract/interface" is `implement`, **not** `design`. `design` is reserved for *open* architecture — a partitioning pattern, a scalability decision — and spec authoring. In the calibrated Sentinel sample there were **zero** real `design` units, consistent with the dry run's finding that the `planner` class was empty. Most work that *feels* like design is actually bounded implementation.
2. **Residual cognitive load degrades the tier.** `implement` + `design_provenance: upstream` **degrades to operator**. This is what separated a charter that wired metrics *declared in an earlier charter* (mechanical → operator) from one that implemented new logic (implementer). The hard thinking was already spent upstream; instrumenting it is cheap work.
3. **`design_provenance` is only meaningful for `implement`.** A `design` whose design "already exists upstream" is a contradiction — if the design is done, the work isn't designing, it's implementing. So the degradation anchors on `implement`; `audit` and `operate` ignore provenance.
4. **No-work maps to `operate`.** Hygiene, closure ceremony, meta-notes — no fifth value is introduced. They route cheap as `operate`; "strictly not-work" is a telemetry observation, not a tier.

The resulting verb → tier map is small and legible: `design` → planner, `implement` → implementer (degrading to operator when upstream), `audit` → auditor, `operate` → operator.

## 4. Homogeneous grain, and honest undeclared

The verb is declared **once, at the widest homogeneous grain**, at the finest unit that already has a declaration slot — no new frontmatter is invented for units that don't have one:

| Unit | Declaration slot |
|---|---|
| Charter | Frontmatter: `work_verb:` / `design_provenance:` |
| Follow-up | Entry lines: `- **Work verb**:` / `- **Design provenance**:` |
| Batch | A `- **Work verb**:` line in the AILOG ledger |
| Task | **No slot.** Inherits from the parent charter/spec. Zero new frontmatter in `tasks.md`. |

Batch and Task inherit from the parent; you override to a finer grain **only** when a unit genuinely mixes work (design + a mechanical part). You don't fragment a homogeneous unit to satisfy the schema. (Honest status: the prototype doesn't yet implement inheritance — it harvests the verb from charter frontmatter and follow-up lines and leaves batch/task `undeclared`. Inheritance is a *ratified rule*, pending the graduation implementation. Documenting it now stops the graduation from inventing a per-task mechanism this rule explicitly rejects.)

And the load-bearing design choice: **undeclared is an honest state.** A unit with no declared verb is unclassifiable — it routes conservatively *up* to frontier and emits a nudge ("declare the verb"), and it **never** fabricates a low-confidence guess from the title. The gap becomes an *action* — a hygiene task — instead of a faked number. The telemetry reports `undeclared_fraction`, the actionable metric, not the `conflict_fraction` artifact that title-scanning produced. The dry run's uncomfortable 57% didn't get hidden behind a cleverer guesser; it got turned into a to-do list.

## 5. The graduation — fw-4.31.0 / cli-3.30.0

Here's the discipline that made this take a full extra Charter instead of a commit. The Sentinel adopter set an explicit constraint: *don't instrument against an unratified schema.* So the vocabulary, the rules, the placement, and the enforcement posture were fixed in a ratification document first — calibrated against Sentinel's real 762-unit governance corpus — and only *then* did the fields graduate to the framework in [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0) ([#332](https://github.com/StrangeDaysTech/straymark/issues/332) step 2).

The graduation is deliberately quiet:

- `work_verb` and `design_provenance` are added to `charter.schema.v0.json` as **optional string** properties — intentionally **not** an `enum`, so an out-of-vocabulary value is an advisory warning, never a blocking schema error. The Charter template (EN/es/zh-CN) carries them as commented guidance; the follow-up entry schema gains matching `Work verb` / `Design provenance` lines.
- `straymark validate` gets two **warning-only** rules, `CHARTER-WORK-VERB` and `CHARTER-DESIGN-PROVENANCE`, that fire *only* when a Charter declares the field with a value outside the vocabulary. An absent field emits **nothing**. The check never touches the exit code.

That last property is the whole point. Sentinel's legacy corpus is 100% undeclared, and it stays completely quiet — no CI break, no noise, no migration. Declaring the schema doesn't punish anyone who hasn't adopted it. `undeclared` is an honest state at the framework level too.

## 6. What we deliberately didn't do

We didn't ship **execution.** Real model dispatch, real pricing, config files, a watch dashboard — all Phase 3, all deferred. This arc graduated a *classification vocabulary*, not a router.

We didn't make the field **required**, or a CI gate. The enforcement posture is a nudge, matching v0-experimental caution and the adopter's explicit "don't break the legacy corpus" line.

We didn't declare **forward-validation** done. Whether authors actually declare *well* across a varied corpus after adoption is step 3 of #332 — StrayMark's responsibility, out of scope of the ratification. A vocabulary that's easy to state is also easy to state wrongly, and only real adoption will show whether the four rules hold outside Sentinel.

And we didn't reintroduce a fifth verb for "not-work," even though it was tempting. Four values, a small anti-gaming surface, periodic calibration. Reconsidered only if future calibration shows `operate` is fusing signal that matters.

## 7. If you've read this far

The portable move is the one that closed the gap for almost nothing. Baton spent a whole phase building machinery to *infer* something — the kind of work each unit was — and the machinery topped out at 43% confidence because it was reconstructing, from a title, a fact the author had held in their head the whole time and never been asked to write down. The cheapest, highest-quality signal in a system is very often the one a human already knows at the moment of creation and simply isn't prompted to record. Before you build a classifier, a heuristic, or an ML model to recover some property of your data, check whether someone upstream knew it for certain and for free — and whether the only thing missing was a field to put it in. The most expensive way to know something is to infer it after the fact. The cheapest is to ask, once, when the answer is still obvious.

The next post is a coda from a different corner of the framework — the audit cycle — where the question isn't *what* was verified, but *who* the framework thought did the verifying.

---

*Baton Phase 3 (schema graduation) — [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0) · ratification [`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md). Issues [#321](https://github.com/StrangeDaysTech/straymark/issues/321) · [#328](https://github.com/StrangeDaysTech/straymark/issues/328) · [#331](https://github.com/StrangeDaysTech/straymark/issues/331) · [#332](https://github.com/StrangeDaysTech/straymark/issues/332). Predecessors: [What the spec path only proved existed](what-the-spec-path-only-proved-existed) · [What the dry run would have spent](what-the-dry-run-would-have-spent).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*