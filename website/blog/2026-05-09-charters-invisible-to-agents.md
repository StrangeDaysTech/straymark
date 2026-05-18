---
slug: charters-invisible-to-agents
title: Charters invisible to the agents
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - charters
  - agents
  - observability
  - governance
date: 2026-05-09T00:00:00.000Z
description: 'Issue #113 — the gap between having an artifact and the agent seeing it'
---
## 1. Six hours not to see it, five minutes to see it

Issue [#113](https://github.com/StrangeDaysTech/straymark/issues/113) opens with one of those sentences that, read back months later, sounds obvious and at the same time cost a lot to reach:

> *"Time to detect: ~6 hours of session work; never autonomously surfaced. Time to resolve once user prompted: ~5 minutes."*

Six hours working with a capable, attentive agent, with the framework's whole canonical onboarding apparatus loaded — `STRAYMARK.md`, the project constitution, the `CLAUDE.md` checklist, the `/straymark-*` skills available, `/straymark-status` run at the start — and not once, over six hours, did the agent suggest using a Charter. When I asked for it directly, the flow was incorporated in five minutes. The asymmetry isn't about capability. It's about visibility.

This post covers a short, specific episode: Issue #113, opened on 7 May 2026, three days after Charters crystallized as a first-class entity in the framework (fw-4.4.0, 2 May). It's the natural counterweight to this blog's first post. That post named a property that appeared when everything was in place. This post documents what happens when a structural piece exists but *isn't on the surface the agent reads first*.

---

## 2. The accidental experiment

The setup wasn't designed as an experiment. It was a real project: a *greenfield* in Rust, CLI+TUI suite, on its first `v0.1` version. The agent was Claude Opus 4.7 with a 1M-token window, enough to keep the whole repo in context without paging. The flow was the canonical SpecKit one: `/speckit-specify`, `/speckit-plan`, `/speckit-tasks`, and between tasks and implement the idea of splitting into Charters should have surfaced — *in theory*.

The Issue formulates it with almost cold precision:

> *"A capable, attentive agent following the canonical project onboarding (`DEVTRAIL.md`, the project constitution, `CLAUDE.md` checklist, available `/devtrail-*` skills, `/devtrail-status`) did not autonomously identify Charters as a workflow concept during plan/tasks generation, even though the project clearly fit their use case (multi-session implementation, multi-phase tasks, audit value)."*

*(The Issue still uses `DEVTRAIL.md` because, given the chronology, the rebrand to StrayMark was being merged almost in parallel. The mixed nomenclature is calendar residue, not oversight.)*

The project met each condition where a Charter is the right unit: multi-session implementation, multiple phases, audit value. The agent did everything right — read the constitution, ran status, identified available skills, generated a reasonable SpecKit plan — and never connected to Charters. As if the concept didn't exist. To the agent, in its model of the repo, it didn't.

The most uncomfortable thing about the Issue, in hindsight, is that the agent didn't fail from incapacity. It failed because the framework, without meaning to, had built a map where Charters weren't marked.

---

## 3. Nine gaps converging on one

When the operator (me) opened the Issue and sat down to audit why it had happened, the problem wasn't one thing. It was nine. And all of them were versions of the same defect:

| # | Gap | Where |
|---|---|---|
| 1 | The framework's canonical document didn't list Charter as a concept | `STRAYMARK.md` (then `DEVTRAIL.md`) §6/9/10/11/13/15 |
| 2 | The `CLAUDE.md` checklist had no trigger to suggest Charter | `dist/dist-templates/directives/CLAUDE.md` (also `GEMINI.md`, `copilot-instructions.md`) |
| 3 | Project constitutions inherited the framework's gap | Any `*-constitution.md` installed via `straymark init` |
| 4 | No `/straymark-charter-new` skill existed | Skills directory |
| 5 | `/straymark-status` didn't list Charters in its output | Skill + CLI subcommand `straymark status` |
| 6 | The audit skills framed the Charter as an *external* artifact, not as a surface to generate | `/straymark-audit-*` skills |
| 7 | Charter templates lived indistinguishable from other templates | `dist/.straymark/templates/` (root) |
| 8 | No conceptual bridge between SpecKit (`plan.md`) and Charter | No explicit document |
| 9 | The mental model the agent ended up building was binary: either SpecKit, or nothing | Compounded effect |

Nine failures and one single defect: the Charter existed as a technical artifact (valid schema, working command, template ported from Sentinel), but wasn't named in any of the places where the agent builds its mental model of the project on entry.

The agent reads `STRAYMARK.md` and the constitution to understand what the project is about. If Charter doesn't appear, it isn't there. The agent reviews which skills are available to use. If there's no `/straymark-charter-new`, it isn't there. The agent runs `/straymark-status` to understand what kinds of files live in the repo. If the output doesn't mention Charters, they aren't there. Each surface the agent passes through on onboarding repeats the same silence. And the sum of silences convinces the agent the concept isn't part of this project.

---

## 4. Why this episode matters beyond the Issue

It's worth pausing here, because this is exactly the episode the fw-4.17.0 meta-pattern — the pattern of *emergent observation* — needed to have happened *first* in order to be formulated at all.

A week after #113, in `EMERGENT-OBSERVATION-DESIGN.md`, the framework named two properties whose composition produces an agent surfacing things nobody asked it to surface: formal links between artifacts (`originating_charter`, `originating_ailog`, `originating_spec` in frontmatter), and cultural permission to flag dissonances between sources. Those two properties, *composed*, produce the agent reading the AILOG, counting the `R<N>(new, not in Charter)` entries, and flagging the delta without being asked.

But there's a precondition that formulation takes for granted: that the agent *sees* the Charters on entering the repo. If the Charters aren't on the visible surface, no formal link between artifacts can compose into emergent observation — because one of the composition's terms doesn't exist for the agent.

Issue #113 is the episode that documented that precondition from the reverse. Fw-4.17.0 named the pattern when it was present; fw-4.12.0 (five days earlier) closed the gap that prevented it from being present at all. Without Issue #113 and its correction, the meta-pattern wouldn't have anywhere to stand. The framework doesn't surface what it can't see, and it can't see what the surface doesn't name.

---

## 5. The response: multiplying surfaces

PR [#122](https://github.com/StrangeDaysTech/straymark/pull/122) (`fw-4.12.0`, 9 May) closed the nine gaps with a single operational strategy: *name Charter in every place where the agent builds its model of the project*. The PR's numbers are honest: +1211 lines, −92 lines, 36 files modified, almost entirely documentation and templates. There was no code refactor. There was a redistribution of visibility.

Main changes, grouped:

- **In the canonical document** (`STRAYMARK.md`): a new section dedicated to Charter as a concept (§15), plus rows in the §6/9/10/11/13 tables that listed document types without Charter.
- **In agent directives** (`CLAUDE.md`, `GEMINI.md`, `copilot-instructions.md`): an explicit trigger teaching the agent when to *suggest* a Charter, not only when to create one if asked.
- **In skills**: new `/straymark-charter-new` (Claude, Gemini, agnostic versions). `/straymark-status` updated to list active Charters.
- **In the CLI**: a `Charters` block added to `straymark status` output, so the agent running the command sees it among the initial signals.
- **In templates**: moved to a dedicated subdirectory `dist/.straymark/templates/charter/`, no longer mixed with other templates.
- **New document**: `SPECKIT-CHARTER-BRIDGE.md` (171 lines, EN/ES/zh-CN). Makes the bridge between SpecKit's `plan.md` and StrayMark's Charter explicit — the four criteria for a SpecKit feature to produce one or more Charters, the three cases where it doesn't apply, the four granularity heuristics, the `originating_spec ↔ originating_charter ↔ originating_ailog` frontmatter linkage.

The epilogue of `SPECKIT-CHARTER-BRIDGE.md` records the case that originated the entire change without metaphor:

> *"Cited the empirical context (issue #113): Greenfield Rust CLI/TUI suite, Claude Opus 4.7 onboarding via canonical entry points. Charters were eventually adopted (2 Charters: foundation + MVP) only after explicit user prompt — confirming the gap was systemic, not session-specific. This document removes the gap."*

*The gap was systemic, not session-specific.* That sentence is the Issue's operational lesson. It wasn't a distracted agent one day; it was the framework that didn't speak to the agent about Charters in any of the places the agent was looking.

---

## 6. What I learned about structural visibility

The lesson, written in prose and not in a table:

Creating an artifact in a framework is not the same as making it visible to the agents that work in repos of that framework. They're two steps. The first — schema, command, template, examples — is the visible step: it shows up in the CHANGELOG, ships in the release, appears in `git log`. The second — anchoring the artifact on every surface where the agent enters the repo — is invisible: nothing about the second step appears as a *feature* in itself; it's cross-references, lines in checklists, items in outputs, sections in docs. But the second step is what decides whether the first one exists for the agent.

The framework governs agents that read the repo from outside the session where the artifact was created. Each agent entering the repo builds its mental model from scratch, reading the canonical documents, the available skills, the status outputs. If the artifact isn't named there, it doesn't exist in the model. And a model without the artifact doesn't propose using it, doesn't audit its absence, doesn't flag related dissonances. The agent's technical capability is irrelevant if the repo's surface doesn't give it the right noun on entry.

That's what I now call, internally, *structural visibility*. It isn't a property of the agent. It's a property of the repo, and by extension, a responsibility of the framework: every time it crystallizes a new concept, it has to multiply the surfaces where that concept is named. One single surface — the schema, the command, the template — isn't enough. There are nine, minimum, and they're the nine in Issue #113.

---

## 7. Closing

What I took from the process, in four claims:

1. **Existing as an artifact isn't the same as being visible to the agent.** Charters had schema, command, template, examples — and were still invisible. The difference between the two states is the nine surfaces of Issue #113.

2. **Six hours vs. five minutes.** A capable agent can spend hours not detecting what, once named, takes minutes to integrate. The asymmetry doesn't diagnose the agent; it diagnoses the repo.

3. **Structural visibility is the framework's responsibility.** When it crystallizes a concept, it doesn't end with the command that creates it. It ends when the concept is named on every surface where the agent builds its model of the repo: canonical document, directive, skill, status, template, bridge to other frameworks.

4. **Without structural visibility there is no emergent observation.** The properties that later made it possible for an agent to surface dissonances between sources — formal links + cultural permission — need, as a precondition, the artifacts to be visible from the first moment of onboarding. Issue #113 documents what happens when that precondition is missing.

Next, in the following post: an episode adjacent to this one — *the rebranding to StrayMark* (`H-08`). Issue #113 was resolved in the same temporal arc as the rename from DevTrail to StrayMark, and they share something worth naming — the responsibility of a framework to declare its identity with clarity, both to its agents and to its adopters.

---

*Anchors: [Issue #113](https://github.com/StrangeDaysTech/straymark/issues/113) (opened 2026-05-07). [PR #122](https://github.com/StrangeDaysTech/straymark/pull/122) — `fw-4.12.0` (merged 2026-05-09). Document generated by the PR: [`SPECKIT-CHARTER-BRIDGE.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md).*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
