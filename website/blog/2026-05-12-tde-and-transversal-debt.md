---
slug: tde-and-transversal-debt
title: TDE and transversal debt
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - tde
  - technical-debt
  - governance
  - git-workflow
date: 2026-05-12T00:00:00.000Z
description: >-
  The type existed. The trigger didn't. And the stacked-PR lesson that came
  along for the ride.
---
## 1. Zero TDEs across thirteen Charters

Issue [#128](https://github.com/StrangeDaysTech/straymark/issues/128) opens with a disquieting asymmetry:

> *"Sentinel adopter (primary, fw-4.12.0) created zero TDEs across 13 closed Charters despite ≥7 instances of transversal debt routed through parallel mechanisms."*

Thirteen closed Charters. At least seven recognizable pieces of transversal debt — inherited from prior Charters, crossing modules, persisting between sessions. Zero TDEs. If the operator had been discussing quotas, the data wouldn't mean anything. But the operator was me and the adopter was the same project that empirically validates the framework. If Sentinel — the repo where the full framework was being used most aggressively — wasn't producing TDEs even once in thirteen tries, something was broken.

What was broken wasn't the debt, which was being captured by other channels (`R<N> (new, not in Charter)` inside the AILOGs, entries in `follow-ups-backlog.md`). What was broken was that no criterion in the framework said *"this specific piece deserves TDE, not R<N>"*. The document type existed. The operational trigger didn't.

This post covers the 11-12 May 2026 episode: how the missing trigger was diagnosed, how it was closed with four canonical criteria (`fw-4.13.0`), and how the PR chain that fixed the problem left behind, in passing, an operational lesson about stacked-PRs that ended up codified in the project's `CLAUDE.md`. It's the first blog episode where the framework fixes *two different things* in the same arc, and where the second is accidental.

---

## 2. The type existed. The trigger didn't.

`TDE` — *Technical Debt Entry* — has existed in the framework since the January initial commit (Post 2 recorded it: it was on the list of eight original types in the v1.0.0 README). The template was there. The taxonomy field was there. The `06-evolution/technical-debt/` folder was there.

What wasn't there was a sentence, somewhere in the canonical apparatus, that said *"create a TDE now"*. Issue #128 put it without softening:

> *"Reading the adopter-facing docs (`AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `QUICK-REFERENCE.md`, `CLAUDE.md` autonomous rules), there is no clear trigger that says 'create a TDE now'. The trigger language exists for AILOG (creation freely), AIDEC (when alternatives considered), ETH (high risk PII), ADR (architectural decisions), but TDE is just listed as 'agent can Identify'."*

*"Agent can identify."* Identify if you feel like it. An agent entering the repo reads that line, shrugs, and moves on to AILOG and AIDEC, which do tell it *when*. The defect is the same one from Issue #113 (Post 5): the artifact technically exists, but the canonical *surface* doesn't give the agent the verb needed to activate it. What makes this case different from #113 is the consequence. For Charters, not seeing them meant not using the new flow. For TDE, not seeing them meant something worse: transversal debt *was* being captured — but fragmented, with no consolidated view, no prioritization across artifacts, no `assigned_to` or `priority` field for a human reviewer to order it.

The Issue cites that consequence with embarrassing detail, because it enumerates everything that wasn't happening:

> *"No queryable 'all open technical debts' view at the document level. No `impact × effort` prioritization matrix per item. No `assigned_to` / `priority` fields surfaced to the human reviewer. Architectural debt that legitimately spans multiple charters (e.g., scope authorization gap heritage across modules) gets fragmented into per-charter R-numbers instead of consolidated into a single TDE."*

Thirteen Charters' worth of debt fragmented into per-Charter `R<N>`. From the repo operator's point of view: no consolidated list, none explicitly named as inherited, no piece of debt prioritizable at project level. Everything was in the repo. None of it was in the place the repo builds for the humans coming in from outside to read it.

---

## 3. Four criteria for a piece of debt

PR [#129](https://github.com/StrangeDaysTech/straymark/pull/129) (`fw-4.13.0`, merged 11 May at 19:38 UTC, exactly thirteen hours after the Issue was opened) closed the gap with a new section in `AGENT-RULES.md §3` titled *"TDE vs `R<N> (new, not in Charter)"*. The four criteria, literal:

| Criterion | When it applies |
|---|---|
| **1. Heritage from a prior Charter** | The debt is literally inherited from the previous Charter (*strict heritage*) or reappears when following the previous Charter's pattern (*pattern propagation*). |
| **2. Applies to multiple modules or Charter execution boundaries** | The debt crosses code modules *or* crosses execution boundaries between Charters (governance-trail debt). |
| **3. Requires a dedicated Charter outside current scope** | Resolving it requires opening its own Charter, outside the current Charter's envelope. |
| **4. Requires human prioritization/assignment** | Deciding priority or assignment is beyond the agent's range. |

The operational rule: if a piece of debt satisfies *at least one* of the four criteria, it's a TDE. If it satisfies none, it stays as `R<N>` inside the Charter where it appeared.

The four each have their logic. Heritage (1) captures debt that persists across time, which a reviewer would want grouped by origin, not by Charter. Multi-module or multi-Charter (2) captures debt that lives *between* artifacts, not *inside* one. Dedicated Charter (3) acknowledges that some pieces of debt are large enough to deserve their own cycle and therefore don't fit as risk declared *ex-ante*. Human prioritization (4) acknowledges the autonomy limit: there are decisions the agent can flag but can't resolve, and those decisions deserve a document where a human can queue up.

The same PR added a new field to the TDE template's frontmatter — `promoted_from_followup: FU-NNN | null` — and a section in `FOLLOW-UPS-BACKLOG-PATTERN.md` documenting how to promote a follow-ups entry to a TDE. The promotion has two shapes, both recorded literally: promotion of an existing entry (a follow-up that had been on the list for weeks matures into a TDE) and *retropromotion at creation* (when the TDE is created, it's acknowledged it originated in a prior follow-up and noted). The frontmatter field closes the trail: any TDE can now point back to the follow-up that originated it, if one existed.

---

## 4. Three TDEs in Sentinel — the empirical loop closed

Issue #128 didn't close by argument; it closed by evidence. The same day, Sentinel — the same adopter that had been through thirteen Charters without a single TDE — created three:

- A TDE for a **RequireScope architectural gap**, heritage crossing modules. Criteria 1 + 2 + 3.
- A TDE about **missing test coverage in the HTTP layer**, debt persisting across feature Charters. Criteria 1 + 4.
- A TDE for **review of legacy AILOGs** predating Charter format v3, a piece needing its own cycle. Criteria 1 + 3.

Three documents in hours. The speed matters less than the fact that each one applied *at least one* of the criteria and none landed in gray territory. The criteria worked as an unambiguous heuristic — the operator didn't have to negotiate with himself about whether a piece was TDE or `R<N>`. The empirical loop the Issue had opened in the morning closed with three documents that same afternoon.

PR [#136](https://github.com/StrangeDaysTech/straymark/pull/136) (`fw-4.13.1`, merged the next day) refined two criteria based on that same experience. *Heritage* (1) was explicitly split into *strict heritage* vs *pattern propagation* because one of Sentinel's three TDEs showed the second case clearly: the debt wasn't literally inherited; it was the same pattern reappearing when the same procedure was followed. *Multi-module* (2) was reformulated as *"multiple modules **or Charter execution boundaries**"* because another of the three TDEs was governance-trail (crossing session boundaries without crossing code modules). The criteria grew one single refinement-pass after being exercised on three real cases. The framework validated principle #12 again: no schema crystallizes without a second domain.

---

## 5. The stacked-PR lesson

This is where the post deliberately veers off. The PR chain #129 → #131 → #133 that closed Issue #128 left behind an adjacent operational lesson — not about TDE but about how *not* to stack branches. The lesson is documented literally in `CLAUDE.md` (lines 181-220) of the public repo under the section **"Stacked PRs — avoid, or use merge commits"**.

### How it broke

PR #131 (`cli-3.12.1`, validator fix that was rejecting `status: identified` on TDEs) was opened with `base = #129` because `#131` needed `#129`'s code to compile its tests. Then `#129` was merged to `main` with *squash*. The current `CLAUDE.md` describes it with a precision worth quoting in full:

> *"When you stack PR B on top of PR A's branch (base of B is A's head, not `main`), and A is squash-merged to `main`, B's content gets stranded:*
>
> *— The squash merge of A creates a new commit on `main` with content equivalent to A but a different SHA than A's original commits.*
>
> *— B's branch still points at A's original commits, which now have no descendant on `main`.*
>
> *— When B is merged, GitHub merges it into A's branch (its declared base), not `main`. The 'merge to main' never happens — even though GitHub UI shows B as MERGED."*

The practical result in the #129/#131 cycle: GitHub's UI showed `#131` as *MERGED*, the validator tests were ostensibly on main, but the content never arrived. PR #133 was the procedural sync that had to be opened later to carry the content to its destination. Time lost in diagnosis: about three hours. Time of the recovery operation once understood: five minutes.

### How to prevent it

The `CLAUDE.md` documents two strategies, one conservative and one pragmatic:

> *"1. Sequential, not stacked. Wait for PR A to merge to `main`, then rebase B onto `main` and open B as a standalone PR with `base = main`. Slower but bulletproof.*
>
> *2. If you must stack, use merge commits (not squash) for the parent. A merge commit preserves shared history, so subsequent merges of stacked PRs into `main` resolve cleanly. Pay the cost of a noisier `git log` for the stacked-PR safety."*

Sentinel and StrayMark default to squash. That choice is good for keeping `git log` clean, one decision per feature, one commit per PR. But it costs exactly this: incompatibility with stacked-PRs. The rule `CLAUDE.md` now codifies is honest: either don't stack, or pay in commit history to be able to stack. There's no third option that doesn't break something.

### How to recover (if it already happened)

The recovery section of the `CLAUDE.md` is the most operational piece of the document. Four steps:

> *"1. `git checkout -b chore/sync-<B-content>-to-main main`*
>
> *2. `git cherry-pick <B's merge commit SHA>` — should be clean because B touches files outside A's conflict zone (which is why it could be stacked in the first place).*
>
> *3. Push, open PR with `base = main, head = chore/sync-...`. Cherry-pick produces a fresh commit on top of `main`, so no conflicts.*
>
> *4. The branch protection may require `--admin` merge if the content was already reviewed in B (the original PR) — sync PRs are purely procedural."*

The last step is editorially important: explicit note that an `--admin` merge is authorized *only* for procedural PRs like this — where the content was already reviewed in the original PR. It's the only exception to the *"content goes through human review"* rule of the project.

The reason it's worth registering the lesson as a sub-section of the TDE post — and not as a separate post — is that they share a trait: both are pieces of **transversal process debt** the framework now explicitly names. One is code debt (TDE as a new type); the other is workflow debt (stacked-PR as a documented anti-pattern). They share an origin: real operational friction was what triggered the document.

---

## 6. What the episode left in the repo

Four PRs in under twenty-four hours, three recorded lessons:

- **PR [#129](https://github.com/StrangeDaysTech/straymark/pull/129)** (`fw-4.13.0`, 11 May 19:38 UTC) — TDE activation trigger. Four criteria + new section in `AGENT-RULES.md §3` + FU → TDE path.
- **PR [#131](https://github.com/StrangeDaysTech/straymark/pull/131)** (`cli-3.12.1`, 11 May 19:39 UTC) — validator accepts `status: identified` on TDEs. Without the fix, the first TDE created under the new trigger failed validation.
- **PR [#134](https://github.com/StrangeDaysTech/straymark/pull/134)** (12 May 04:54 UTC) — the stacked-PR lesson documented in `CLAUDE.md`. It didn't change framework code; it changed the repo's operational rules.
- **PR [#136](https://github.com/StrangeDaysTech/straymark/pull/136)** (`fw-4.13.1`, 12 May 04:54 UTC) — TDE trigger refinements based on Sentinel's three real cases.

What I most want to flag about the chronology: `CLAUDE.md` (PR #134) was updated in the same arc that closed Issue #128. The stacked-PR lesson didn't wait to be forgotten. If the framework's discipline is *"every significant change leaves a documented trace"*, that applies to operational friction too, not only to design. The framework exists, in part, so expensive learnings don't evaporate.

---

## 7. Closing

What I took from the process, in four claims:

1. **A document type without a trigger is an invisible type.** Same as in Issue #113, what the agent doesn't see on entering the repo doesn't exist for it. TDE existed technically from January, but only started being used when the four criteria appeared in `AGENT-RULES.md §3`. Structure without an activating verb is decoration.

2. **Transversal debt deserves its own type.** Fragmenting debt into per-Charter `R<N>` destroys the most important property of a governance system: the consolidated view. Four criteria, any one is enough, holds the line between *this is risk of the Charter* and *this is debt of the project*.

3. **Operational learnings get documented too.** PR #134 updated the project's `CLAUDE.md` the same day the lesson surfaced. It isn't archival paranoia; it's the same rule applied to process: if the friction cost you three hours today, write the lesson today.

4. **The framework grows from features as well as from anti-patterns.** This episode left behind a new type (TDE as operational entity) *and* a documented anti-pattern (stacked-PRs incompatible with squash). Both are framework material, even if only one shows up as a version bump.

Next, in the following post, a brief but structural episode: *"The Spanish audit-prompt was the outlier"* (`H-10`). An i18n convention detail that revealed which language had been the canonical one — and why fixing it changed, without meaning to, how the framework thinks about translation.

---

*Anchors: [Issue #128](https://github.com/StrangeDaysTech/straymark/issues/128). PRs [#129](https://github.com/StrangeDaysTech/straymark/pull/129) · [#131](https://github.com/StrangeDaysTech/straymark/pull/131) · [#134](https://github.com/StrangeDaysTech/straymark/pull/134) · [#136](https://github.com/StrangeDaysTech/straymark/pull/136). Releases: `fw-4.13.0` / `cli-3.12.1` / `fw-4.13.1`. Stacked-PR lesson codified in [`CLAUDE.md` lines 181-220](https://github.com/StrangeDaysTech/straymark/blob/main/CLAUDE.md) of the repo.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
