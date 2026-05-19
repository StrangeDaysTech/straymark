---
slug: pattern-1-and-pattern-2-chain-evolution
title: Pattern 1 and Pattern 2 — chain evolution
authors:
  - jose
tags:
  - straymark
  - charters
  - chain-evolution
  - governance
date: 2026-05-16T00:00:00.000Z
description: When discipline stops being habit and becomes name
---
*Twenty-seven hours later, the manual discipline got two names: Pattern 1 — pre-declare SpecKit refresh, and Pattern 2 — post-close audit-driven Batch N.4. The canonical document, the telemetry schema, the CLI helpers — and one hour after that, a meta-pattern that reabsorbs both upward.*

<!-- truncate -->

## 1. Twenty-seven hours to name them

PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152), which closed the Post 9 episode, merged on 14 May at 21:39 UTC. It brought `fw-4.14.3` and the three canonized gates of the manual refresh.

PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) — the one this post comes to tell — merged on 16 May at 00:31 UTC. Twenty-seven hours later. It brought `fw-4.16.0 / cli-3.14.0` and, with that, two named meta-patterns:

- **Pattern 1 — *pre-declare SpecKit refresh***. Refresh the spec before declaring the next Charter in a chain.
- **Pattern 2 — *post-close audit-driven Batch N.4***. Amend an already-closed Charter when external audit surfaces findings post-close, without opening a new Charter.

Post 9 covered the manually applied discipline on CHARTER-18. This post covers what came after: the two named patterns, the canonical document that names them (`CHARTER-CHAIN-EVOLUTION.md`), the telemetry schema that measures them, the two *CLI helpers* that scaffold them — and, as the closing of the entire editorial arc, one hour and seventeen minutes later, a philosophical meta-pattern that reabsorbs both upward.

It's the last post of the arc. The blog Post 1 opened from ahead closes here from behind.

---

## 2. Pattern 1 — Pre-declare refresh

The literal definition, copied from the canonical document:

> *"A dedicated refresh PR lands between Charter-N close and Charter-(N+1) declare. It touches only the non-locked sections of the SpecKit artifacts (plan.md, data-model.md, contracts, quickstart.md, research.md)."*

This is exactly what Sentinel did manually for CHARTER-18 in Post 9. The difference between Post 9 and Post 10 is one of kind: Post 9 documents the procedure; Post 10 documents its *conditional activation* — when the framework recommends to the adopter that Pattern 1 should be applied.

The activation criteria are quantifiable, and the document names them without softening:

- The module has **≥ 3 closed Charters** on the same spec.
- The rolling mean (last 3 Charters) of the `agent_quality.r_n_plus_one_emergent_count` field in telemetry is **greater than 6**.
- No *refresh PR* has landed since the last *branch point*.

The `> 6` threshold isn't arbitrary. It's Sentinel CHARTER-18 turned into a rule: the average of emergent findings (risks discovered during the Charter that weren't declared in the original plan) during CHARTER-15, 16, 17 was exactly what triggered the feeling that the spec was accumulating too much debt. The framework, instead of asking the adopter to trust intuition, gives them a number their telemetry can compare against.

What Pattern 1 produces, when applied:

- A categorized table in `research.md` that enumerates what was discovered in prior Charters into four types: *reusable patterns*, *code gaps*, *discipline patterns*, *empirical corrections*.
- Explicit operational decisions (`D1`, `D2`, etc.) if categorization shows trade-offs.
- Citations in the next Charter's `§Context`, by `id`, pointing to the integrated items.
- An opt-in `pre_declare_refresh:` telemetry block in the `charter-telemetry.yaml` of the Charter receiving the refresh.

The metric that holds the pattern up comes from Sentinel CHARTER-18 itself, cited literally in `CHARTER-CHAIN-EVOLUTION.md`:

> *"Sentinel CHARTER-18 was the first Charter in a 7-Charter chain to close cleanly without a mid-flight remediation Charter. `estimation_drift_factor: 1.0`, `pre_work.items_discovered_during_planning: 0`, `overall_satisfaction: 5/5`."*

One single domain. `N=1`. The document says so without shortcuts: the pattern is empirically validated *in Sentinel*. Wait for the second domain before crystallizing it higher.

---

## 3. Pattern 2 — Amend-on-emergence

The literal definition:

> *"The amendment rides the same execute branch as the original Charter (the branch is still mergeable to main; the amendment commit lands on top)."*

The idea: when a closed Charter receives post-close external audit findings — *critical* or *high* — the framework doesn't force opening a new Charter. The operator can *amend* the closed Charter while the `execute` branch is still mergeable to `main`.

The justification, also literal from the document:

> *"A new Charter would have created multi-week governance overhead for ~6h of focused engineering."*

That asymmetry — weeks of governance vs. six hours of engineering — is what the pattern resolves. Opening a new Charter to fix five post-close findings means: declare new context, repeat external audit, generate new AILOG, register telemetry from scratch. For a correction that in code takes an afternoon. Pattern 2 says: no.

The activation criteria, also quantifiable:

- Critical/High *findings* appear post-close, after external audit.
- The Charter's closure criterion was materially unmet by those findings.
- The fix surface is under 25 files.
- It doesn't require architectural reopening (design decisions the original Charter assumed closed).

If all four criteria hold, Pattern 2 applies. If any fails, you have to open a new Charter.

What it produces, when applied:

- An *amendment* commit on the same `execute` branch as the original Charter.
- A new AILOG (don't edit the original) with `risk_level: high`, `review_required: true`, and an `amends:` field pointing to the original AILOG.
- A `## Historical correction (YYYY-MM-DD)` section in the original AILOG pointing *forward* to the new one. It's archival: the original AILOG isn't rewritten; it's linked.
- A `post_close_amendment:` telemetry block in the `charter-telemetry.yaml`.

The empirical metric backing Pattern 2, cited literally:

> *"Sentinel CHARTER-18 closed 2026-05-15 with audit reports landing 2026-05-15..05-17. Five findings (4 from gpt-5.3-codex, 1 from gemini-2.5-pro) were code-level fixes — DI wiring, retry header parsing, multi-tenant filter, timeout default. The Batch 7.4 amendment closed all five in one cohesive commit (19 files, +2257/-106 lines)."*

One commit, 19 files, two auditor models converging on similar findings, five fixes. If it had been a new Charter: minimum two weeks between declare, external audit, and close. Instead: an *amendment* on the same branch, six hours.

---

## 4. Composition, not exclusion

The most interesting piece of the document — and, I think, the one that best closes the arc — is the section on how the two patterns relate. Cited literally:

> *"A Charter that received Pattern 1 is more likely to avoid Pattern 2, because the refresh absorbs pre-execution risk that would otherwise surface as post-close findings. But CHARTER-18 needed both — the refresh handled spec-level drift; the amendment handled runtime-level drift the refresh did not reach into."*

They aren't rival patterns. They're applications of the same principle at different moments of the cycle. Pattern 1 absorbs *spec-level* drift — what the original plan no longer describes correctly. Pattern 2 absorbs *runtime-level* drift — what only shows up once the code runs, external auditors look at it fresh, and it emerges from the code itself.

Sentinel CHARTER-18 needed both. The refreshed spec eliminated emergent findings a later audit cycle would have had to remediate atomically; the later *amendment* absorbed what the refresh couldn't have predicted — DI wiring, retry header parsing, a multi-tenant filter, a timeout default. Things the plan couldn't know and that only emerged when the code existed.

The framework, by naming the two patterns and explicitly declaring their composition, does something I want to underline: it puts on record that discipline isn't linear. There isn't a single inspection point before each Charter; there's one before (Pattern 1) and one after (Pattern 2), and both may be necessary in the same Charter. The chain evolves; the patterns evolve too.

---

## 5. The schema and the helpers

What `fw-4.16.0` added to the telemetry schema are two optional sub-objects in `charter-telemetry.schema.v0.json` v0.2:

```yaml
pre_declare_refresh:
  enabled: boolean
  refresh_pr: string | null
  refresh_aidec: string | null
  reusable_patterns_integrated: integer  # ≥ 0
  code_gaps_integrated: integer
  discipline_patterns_integrated: integer
  empirical_corrections_integrated: integer
  operator_decisions_ratified: integer

post_close_amendment:
  applied: boolean
  trigger: external_audit | production_incident | deferred_implementation
  ailog_id: string
  findings_closed: integer
  files_modified: integer
  effort_hours: number
```

Integer counters, not narratives. The decision is deliberate: what the telemetry schema captures is what the agent or operator can count mechanically. What can't be counted — *how deep the problem was, how clever the fix was* — lives in the AILOG, not in telemetry. The schema measures the *shape* of work; the documents carry the *content*.

And `cli-3.14.0` added two helpers, neither mutator:

- **`straymark charter refresh-suggest <module> [--threshold N]`** — *read-only*. It reads the telemetry of the last three closed Charters of the module, computes the rolling mean of `r_n_plus_one_emergent_count`, and emits a recommendation: *refresh-now*, *not-needed*, or *insufficient-data*. *Exit 0* always. It's informational, never a CI gate.

- **`straymark charter amend <CHARTER-ID> --trigger <kind> --ailog-title <title> [--findings-closed N] [--merge-into <PATH>]`** — scaffolding. It creates an AILOG stub with the correct fields (`risk_level: high`, `review_required: true`, `amends:` pointing to the original), adds the `## Historical correction` section to the original AILOG, and renders the `post_close_amendment:` YAML block. **Doesn't touch git.**

Keeping the two helpers humble — one only recommends, the other only prepares — is consistent with the A1 decision of Post 4: *"the CLI orchestrates but doesn't invoke APIs"*. Here: the CLI suggests but doesn't decide; scaffolds but doesn't mutate. The human remains the point where discipline happens. The framework only puts the tools within reach.

---

## 6. Seventy-seven minutes later

There's one more piece worth recording before the arc's closing. PR #157 merged at 00:31 UTC on 16 May. PR #160 — the one from Post 1, `EMERGENT-OBSERVATION-DESIGN.md`, fw-4.17.0 — merged at 01:48 UTC the same day. Seventy-seven minutes later.

What happened in those seventy-seven minutes is the inverted closing of the arc. The philosophical meta-pattern Post 1 named — *emergent observation composed of formal links plus cultural permission to flag* — crystallized **one hour and seventeen minutes** after the two operational meta-patterns of this post. The philosophical meta-pattern doesn't precede the discipline; it succeeds it. It's the ascending reading of the same evidence.

`CHARTER-CHAIN-EVOLUTION.md` records it explicitly in its `§Related` section:

> *"EMERGENT-OBSERVATION-DESIGN.md — the meta-pattern of which Pattern 1 and Pattern 2 are applications (formal cross-referencing + cultural permission to surface)."*

Pattern 1 is composition of formal links (the `r_n_plus_one_emergent_count` telemetry, the citations by `id` in `§Context`, the categorized AILOGs) with cultural permission (the rule that an agent should flag drift between spec and code when it finds it). Pattern 2 is similar composition (the AILOGs with `amends:`, the `Historical correction` section linking *forward*) with permission to flag post-close findings instead of absorbing them in silence. The two patterns are applications of the same cultural+structural principle that Post 1 named from above.

It's an ending I keep liking: the blog that opened saying *"the agent saw something nobody asked it to see"* closes saying, in the chronologically preceding episode, *"here are the two things the agent can see well because the framework names them"*.

---

## 7. Closing the arc

Ten posts. Eleven episodes covered (`H-01` through `H-13`, with several bundled). One editorial decision — retroactive publication — that Post 2 introduced and the nine following posts honored without violating it. A bilingual commitment that held at every close.

Some things I learned writing the blog and that are worth recording before closing it:

1. **The pattern is born of the discipline.** The phrase already appeared in Post 9, but the entire arc holds it. The meta-patterns of this post weren't designed; they were extracted. Each documented milestone had its operational episode first and its name after.

2. **The cadence isn't uniform.** Nineteen days between the first commit (Post 2, 27 January) and the first bounded unit with external audit (Post 3, 25 April). Five hours between applied discipline and canonized discipline (Post 9, 14 May). Seventy-seven minutes between two meta-patterns and their philosophical reading (Posts 10 and 1, 16 May). The project went through every speed, and the blog records them all.

3. **The framework isn't neutral with respect to its adopter.** Sentinel made it possible for each pattern to be empirically validated before being crystallized. The blog doesn't hide that provenance; it underlines it as a methodological precondition.

4. **What isn't named, isn't seen.** Post 5 formulated it as *structural visibility*; Post 7 applied it to transversal debt; Post 8 pointed it out about the language outlier. It's the most persistent regularity of the arc. Existing as a technical artifact isn't enough. The name activates the artifact.

The arc the blog came to tell — milestones `H-01` to `H-13`, the four names of the project, the six Plans, the first-class Charters, the external audit cycle, Issue #113, the rebrand to StrayMark, the TDE, the audit-prompt outlier, the manual discipline, and the two meta-patterns — closes here. What's left as explicit debt: `H-14`, the `straymark explore` TUI (which appeared laterally in Post 8 as a visibility tool), and the candidate milestones `H-?-A` to `H-?-E` that `PLAN-INVESTIGACION.md §1.43-55` listed as pending. Possible future second batch. No promise.

One last observation, symmetric to Post 1. That post opened saying: *"the agent saw something nobody asked it to see"*. What the blog closes by recording, after ten episodes, is the other half of the sentence: the agent saw because the framework had put it in conditions to see. The emergent observation of Post 1 is the proof; the named patterns of this post are the preconditions. Every thing the agent surfaced during the four months the arc covers was possible because some prior episode had put an artifact, a trigger, a formal link, a visible surface where there was nothing before.

The blog finishes telling the story. The framework goes on.

---

*Anchors: [Issue #156](https://github.com/StrangeDaysTech/straymark/issues/156). PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) — `fw-4.16.0 / cli-3.14.0` (merged 2026-05-16 00:31 UTC). Canonical document: [`CHARTER-CHAIN-EVOLUTION.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md). Schema: `dist/.straymark/schemas/charter-telemetry.schema.v0.json` v0.2. Successor meta-pattern: [`EMERGENT-OBSERVATION-DESIGN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/EMERGENT-OBSERVATION-DESIGN.md), `fw-4.17.0` (merged 2026-05-16 01:48 UTC, 77 minutes later).*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*

*— End of the H-01 → H-13 arc of the StrayMark blog.*
