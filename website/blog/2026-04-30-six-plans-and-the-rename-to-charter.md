---
slug: six-plans-and-the-rename-to-charter
title: Six Plans for one thesis (and the rename to Charter)
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - sentinel
  - charters
  - methodology
  - governance
date: 2026-04-30T00:00:00.000Z
description: 'The first systematic experiment, and the day Plan became Charter'
---
*Six Plans on paper, five in practice — Sentinel's first systematic experiment between April 25 and 28. The cycle that turned an artisanal pattern into Charters, the framework's bounded unit of work.*

<!-- truncate -->

## 1. A footnote that says a lot

In the `charter-telemetry.md` proposal, written on 30 April 2026, there's a terminological note at the top:

> *"What this document calls 'Charter' was called 'Plan' in the Sentinel experiment (PLAN-01..06). The empirical data cited below refers to those historical Plans by their original name; the schema shape and prospective examples use the going-forward name 'Charter'."*

It's a footnote, technical, drama-free. But it records the exact moment the artifact changed names — and, more interestingly, it records an archival decision the blog also respects: what was called *Plan* at the time keeps being called *Plan* in the original records. Rewriting the name backwards would falsify the trace.

This post covers two things that happened in five days: the framework's first systematic experiment — Sentinel's six Plans, run between 25 and 28 April — and the rename that came right after, on the 30th. They aren't two separate events. One led to the other.

---

## 2. Six Plans on paper, five in practice

The experiment was designed with six Plans (`PLAN-01` through `PLAN-06`) and executed with five. PLAN-04 never ran: it stayed as a catalog of seven *deferred features*, a list of things we wanted to do but the test cycle couldn't cover. It's only honest to say so, because the very document that validates the thesis (`thesis-validation.md`) records it openly in its summary table:

| Plan | Size | What it did | Format |
|---|---|---|---|
| PLAN-01 | XS | Governance docs deploy | v1 |
| PLAN-02 | S | Admin endpoint (`gcp-resource`) | v1 |
| PLAN-03 | XS | Contract bumps | v1 |
| PLAN-05 | M | Per-service anomaly thresholds | **v2** |
| PLAN-06 | XS | Manual baseline recompute | **v3** |
| *(PLAN-04 catalog)* | — | Roadmap of 7 deferred features, not executed | — |

Five Plans, four distinct sizes: three XS, one S, one M. Deliberately heterogeneous domains — an operations deploy, two admin endpoints, a contract bump, a backend feature with configuration logic. It wasn't a synthetic benchmark. It was Sentinel's real code between 25 and 28 April, partitioned into bounded units and audited piece by piece with three models (Copilot, Gemini, Claude) acting as external auditors.

That the sixth Plan didn't run matters for two reasons. The obvious one: the experiment ran out of time. The more interesting one: PLAN-04 was the only large Plan in the batch — seven accumulated features, no natural boundary. That precisely *that* Plan was the one that didn't make it through was already saying something about the size the format could tolerate. Small Plans, yes; roadmap-Plans, no. That lesson was canonized later without anyone having to write it: StrayMark's Charters today are *"bounded units, hours to days, one branch"*, not quarterly roadmaps.

---

## 3. The experiment iterated on itself

Five executed Plans produced three different formats. It wasn't on a whim; it was because each cycle exposed something the previous format didn't capture.

- **v1** — the first three Plans (`01`, `02`, `03`) ran on an original format. Five recurring patterns were identified in the audit AILOGs, but none was formalized into a template. They existed as habit, not as contract.

- **v2** — for PLAN-05, those five patterns were materialized into `TEMPLATE.md`, plus a *"Format conventions"* section in the README. Doc-only: nothing executable yet. The hypothesis: if the format is explicit, external auditors converge better. AILOG-020 says it plainly: *"Internal patterns that have been applied for a while but have no name remain invisible to external auditors. Naming them formally turns practice into a public signal."*

- **v3** — for PLAN-06, a new pattern was added (auto-checklist drift) and, this time, *executable tooling*: `scripts/check-plan-drift.sh`, ~145 lines of bash that validate whether the actual delivery matches what the Plan declared. For the first time, the format stopped being prose only and started having a verifier.

What I want to point out here is the shape of the curve. v1 captured latent practice. v2 named it. v3 wrote a program that checks it. That's the full arc of any useful standardization — only compressed into five Plans, not five years.

It's also the first concrete evidence of something that, two months later, would be named explicitly as Pattern 1 (*pre-declare refresh*) and Pattern 2 (*amend-on-emergence*) in `CHARTER-CHAIN-EVOLUTION.md`. What got canonized there as a meta-pattern started here, in April, as habit-without-a-name.

---

## 4. The hardest evidence: external auditors converging

The most loaded assumption of the thesis was: *"structured notes reduce failure modes, and the reduction is visible when external auditors look"*. The way to prove it: send the output of each Plan — the modified files, the AILOG, the Plan-doc under TEMPLATE — to three auditors who hadn't participated in development (Copilot, Gemini, and a critical analysis from Claude) and compare the aggregate scores.

The number `thesis-validation.md` reports for PLAN-05 (the first Plan under v2, and the only M-size one) is clear:

> *"Best combined auditor calibration across 5 cycles (Copilot 9.25, Gemini 9.5). Accumulated hypothesis confirmed: TEMPLATE v2 + rich AILOGs reduce the ambiguity surface for external audit."*

Two heterogeneous models, different instructions, converging around 9.3 out of 10 — on the largest Plan in the batch, no less, not the smallest. That's signal, not noise. And it replicated when v3's `check-plan-drift.sh` was exercised:

> *"PLAN-05 (with known drifts): script reports 3 omitted files: `evaluator_test.go` (F4), `repository.go` (F1/R6), `statuscenter/service.go` (F5) — exactly the 3 drifts that Copilot+Gemini captured in their audits. Zero false positives."*

The automated script and the two external auditors agreed exactly on which Plan files had been left undone. This isn't an argument that the script is smart — it's bash and *grep*; it couldn't be dumber. It's an argument that the Plan format, once formalized, makes *any inspector* (human, agent, script) see the same things. Structured discipline doesn't require intelligence to be audited: it requires clarity.

---

## 5. What the experiment *didn't* prove

This is the section I most want on the record, because it's where it gets tested whether the blog is willing to be honest.

`thesis-validation.md` was explicitly designed to break the thesis, not to defend it. The first line of the document reads:

> *"This document confronts that thesis with data. It does not defend it; it tries to break it. The goal is that a reader who didn't buy the thesis can, reading only this text, decide whether the available evidence supports it, qualifies it, or refutes it."*

The final verdict, with the thesis's six assumptions faced against the evidence, is this:

| # | Assumption | Verdict |
|---|---|---|
| 1 | Vibe coding doesn't scale | Partially validated *(no control arm)* |
| 2 | Structured notes reduce failure modes | Validated |
| 3 | Regulatory byproduct with no extra work | Validated *(pending real human auditor test)* |
| 4 | Approvals are rarely binary | No evidence — pending multi-actor project |
| 5 | Stage > commit as traceability unit | Validated |
| 6 | In-situ signing > later reconstruction | Partially validated *(no cryptographic signing tested)* |

Three assumptions fully validated, two partial, one *without evidence*, zero refuted. The assumption about non-binary approvals went untested because Sentinel is a single-owner project; ruling on it would require a real team with multiple signers. And cryptographic signing — Sigstore, hash-chaining, the technical guarantees of an append-only log — wasn't exercised because no Plan touched that part of the flow. That stays in the document as an explicit *gap*, not as a hand-waved claim.

What the experiment didn't prove matters more than what it did. If we'd published six greens instead of five honest verdicts, the blog would be marketing. The way to avoid it is to say, without softening: here are assumptions we don't have evidence for, and a one-operator project can't generate it. Others will; the framework will mature with that data.

---

## 6. Why Plan became Charter

The 30 April rename wasn't marketing. The reason is written literally in `WHAT-IS-A-CHARTER.md`:

> *"GitHub SpecKit already uses the word 'plan' (`/speckit.plan`, `plan.md`) for a different artifact, and the nominal collision generated friction in adopter docs that combine both flows."*

SpecKit already had a `plan.md`. StrayMark had one too. In the documentation of an adopter using both, *plan* meant different things in adjacent paragraphs. The rename existed to fix that collision.

But behind the pragmatism there's a nuance worth naming. The two artifacts *are* distinct, and the distinction is structural:

| | **SpecKit `plan.md`** | **StrayMark Charter** |
|---|---|---|
| Granularity | Complete feature (weeks, several stories) | Bounded unit (hours to days, one branch) |
| Dominant content | Stack, dependencies, project structure, constitution gates | Concrete files to touch, verification commands, `R<N>` risks |
| Validation | Constitution Check (*ex-ante* gate) | Drift-check + external audit (*ex-post* gates) |
| Optimization | *Upfront clarity* | *Ex-post traceability* |

What SpecKit calls *plan* looks more like an ADR with an architectural skeleton. What StrayMark calls *Charter* looks more like a task-card with contractual verification and an audit anchor. They're complementary artifacts, not competitors; `SPECKIT-CHARTER-BRIDGE.md` makes that explicit later, in May. But the nomenclature had to stop colliding before complementarity could even be discussed.

The part I find important to highlight is the adjacent decision: Sentinel's historical records — the AILOGs, the telemetry, the `check-plan-drift.sh`, the `sentinel/docs/plans/` folders — preserve the name *Plan*. `WHAT-IS-A-CHARTER.md` itself argues it this way:

> *"Sentinel's historical records (Plans 01–06, `sentinel/docs/plans/`, `sentinel/scripts/check-plan-drift.sh`, AILOGs and YAML telemetries `PLAN-NN.telemetry.yaml`) preserve the original name — they are empirical evidence of a specific moment in time, and rewriting history would falsify the record."*

*Rewriting history would falsify the record.* For me, that sentence is the true heart of the rename. The framework exists to preserve a trace, not to rewrite one. When the artifact changed names, the change was prospective: from 30 April onward, new ones are Charters. The old ones, the experiment ones, are still Plans. The inconsistency is deliberate, and the inconsistency *is* the evidence.

The 30 April rename's other companion — the telemetry proposal — came the same day for reasons that look obvious in hindsight: if the emergent pattern deserved a new name, it also deserved structured measurement. The `charter-telemetry.schema.v0.json` schema, with its fields for invested time, detected drifts, size, domain, is the machinery that runs today what Sentinel was still measuring by hand in YAML. Plan got renamed to Charter on the same day we decided to start counting Charters.

---

## 7. Closing

What I took from the process, in four claims:

1. **The experiment was designed to break, not to confirm.** *"It does not defend it; it tries to break it"* — that was, literally, the brief of the document that validated the thesis. Any technical blog worth respecting has to tolerate that framing.

2. **Five Plans executed, of six designed, in four days.** The one that didn't run (the largest, a seven-feature roadmap) was, accidentally, the first evidence of what size the unit can tolerate. Today Charters are bounded by contract. In April, they were bounded by accident.

3. **The format evolved inside the experiment itself: from habit, to template, to script.** Five Plans produced three versions of the format. The v1 → v2 → v3 curve is the curve of any useful standardization — just compressed.

4. **The rename came from pragmatism, not marketing. And historical preservation came from principle.** *Plan* keeps being called *Plan* in Sentinel, where that's what it was called. *Charter* started being used from April onward. The framework doesn't rewrite its own archive: that's why it's worth tracing.

Next, in the following post: two close-by milestones that codify qualitatively new capabilities — *Charters as a first-class entity* (PR #65, fw-4.4.0) and *the external multi-model audit cycle* (`audit-skills-design`, `audit-cli-flow`, releases fw-4.7 → fw-4.9). That's where the Sentinel experiment became CLI functionality.

---

*Anchors: proposals [`thesis-validation.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-04-30-thesis-validation.md) and [`charter-telemetry.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-04-30-charter-telemetry.md) (both 2026-04-30). Canonical Charter definition: [`WHAT-IS-A-CHARTER.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/contributors/WHAT-IS-A-CHARTER.md). Operational schema: `dist/.straymark/schemas/charter-telemetry.schema.v0.json`.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
