---
slug: emergent-observation-design
title: The day the agent saw something nobody asked it to see
authors:
  - jose
tags:
  - straymark
  - ai-agents
  - governance
  - emergent-design
  - charter
  - sentinel
date: 2026-05-16T00:00:00.000Z
description: Naming a design property that was already silently doing its job
---
*An agent flagged a stale spec before being asked to. The reconstruction of that morning, and the design property that surfaces it — codified after the fact as Principle #8 of StrayMark.*

<!-- truncate -->

> *"Hi, did we already deal with Issue #153?"*

That's how the conversation started, today, May 16th, in the morning. A short, almost-housekeeping question, before getting into something else. The answer was no — Issue #153 was open on purpose, waiting for a second domain to exercise the mechanic before crystallizing it (StrayMark's Principle #12: validation against a second domain before canonization). But the question opened another door, and by the end of that conversation I had codified a design property of the framework that had been doing its job in silence for months, surfaced by an episode from just two days ago.

This post is the reconstruction of that arc, because it's worth keeping. Not for the outcome — a new canonical doc, a Principle #8, a PR already merged — but for the mechanism it revealed. StrayMark has a human project behind it: making sure that agent-assisted development doesn't slip through the fingers of the engineers who are responsible for the software being produced. And this episode exposed a concrete design property pushing in that direction.

A note on craft: StrayMark gets tested first on my own projects. Sentinel is one of them — private, but its interaction with StrayMark is documented in the public Issues of the StrayMark repo, so the context is linkable. I'm the author and first adopter of the framework. Principle #12 obliges me to wait for a second domain before declaring any new mechanic canonical; but the first domain is me, and this post acknowledges that without disguise.

## The incident: an agent that flagged what nobody asked it to flag

On **May 14, two days ago**, in Sentinel, an agent brought to my attention an observation I hadn't requested. It was about to help me fill out CHARTER-18 — a concrete piece of work inside an already-long chain — and before writing a single line of code it said, in essence:

> *(translated from the original in Spanish)* There's a problem. The `specs/002-commshub/plan.md` was frozen on April 21. Since then seven consecutive Charters have passed, and across the AILOGs of that chain there are twelve empirical learnings unreflected in the plan that materially affect the scope of US5. If we fill out CHARTER-18 reading the stale plan, the next audit cycle is going to remediate divergences atomically pre-close — with approximately 50% probability of at least one critical or high finding from stale-premise inheritance.

And it cited the specific AILOGs by ID. And the concrete code references.

Nobody asked for that analysis. There was no trigger configured. There was no CLI command whose purpose was to produce that output. The agent was there to help me with the next task, not to audit the project's state. But while positioning itself for that next task, it saw a divergence between two sources that StrayMark's documentation explicitly connects, and it chose to flag it before proceeding.

I filed Issue [#150](https://github.com/StrangeDaysTech/straymark/issues/150) that same afternoon as an RFC, and between 5pm and the night the manual spec-refresh discipline got discussed and landed in `fw-4.14.3`. The next day, **May 15**, I raised Issue [#156](https://github.com/StrangeDaysTech/straymark/issues/156) to upstream the two patterns the exercise had revealed: a *pre-declare SpecKit refresh* and a *post-close Batch N.4 amendment*. By the early hours of **May 16** they were canonized in `fw-4.16.0` as `CHARTER-CHAIN-EVOLUTION.md`, with dedicated telemetry and a CLI helper (`straymark charter refresh-suggest`). The behavior reproduced, the patterns crystallized, cycle closed.

But when I sat down that same morning of the 16th to open the conversation with Issue #153 — *"did we deal with this?"* — and then bounced back the inverse question — *"hold on, what made that observation possible in the first place?"* — I realized the cycle had only half closed. What got codified in `fw-4.16.0` was the **application** of the pattern. What remained to be codified was the **mechanism** that produced it.

## The human question, said with technical words

I phrased it like this, in the conversation with the agent that generated this post:

> *(translated from the original in Spanish)* I was somewhat surprised by the agent's behavior — that it noticed the accumulated knowledge that, if not applied, would cause problems with the implementation of Charter 18. That is, it wasn't a task I asked it to analyze, nor was it the result of a prompt trigger being activated, and it seems to me that it was thanks to the StrayMark documentation. I'd like to dig deeper into how this happened, which I think is positive, so that we can take advantage of the experience and hook it into StrayMark's habitual behavior — this emergent pattern that came from the agent and its relationship to the generated documentation was very interesting.

What I'm asking there isn't to implement a feature. It's to understand a mechanism in order to reproduce it. The difference matters. When you ask to implement something, the path is clear: spec → tasks → code. When you ask to understand why something worked, the path is rarer: you have to diagnose a working system, identify which pieces made it work, and separate the accidental from the structural.

There's a particular urgency to this kind of question when you're building tools to collaborate with AI agents. Because if what worked was an accident, the next agent — or the next version of the same agent — may not reproduce it. If what worked was structure, then it can be preserved deliberately. The difference between those two things, for a project that intends to give engineers control over AI-assisted development, is the difference between luck and design.

## The audit: which pieces of StrayMark made the observation possible?

I asked an agent to systematically map StrayMark's documentation with three concrete questions:

1. Which documentary mechanisms connect sources explicitly to each other? (Frontmatter, canonical sections, stable IDs, CLI commands that cross sources.)
2. Where does the documentation give the agent *explicit permission* to flag things beyond the task asked?
3. What pairs of sources exist, beyond the one that originated this case, that could produce similar emergent observations if the infrastructure is present?

The report came back structured, with file:line for each finding. What mattered wasn't the individual details (frontmatter with `originating_charter:`, sections `§Risk: R<N>`, conventions of stable IDs, commands like `charter drift`) but the pattern that emerged when seeing them together. Two properties consistently coexisted:

**Property 1: mandatory structural cross-referencing.** Each document type has *required fields* and *canonical sections* that declare, in the document's own structure, which other documents it points to. When an agent reads an AILOG, it doesn't have to guess which Charter it belongs to — the `originating_charter:` tells it. When it sees a risk, it isn't an observation in free prose — it's an `R<N>` entry in a canonical, countable section. When the spec points to Charters, it does so formally; when Charters point to AILOGs, the same. It's an explicit linkage graph the agent can triangulate without creative work.

**Property 2: cultural permission without blocking gatekeeping.** AGENT-RULES §6 tells the agent: *"Be Proactive — Identify potential risks, Suggest improvements when evident, Alert about technical debt"*. PRINCIPLES §2 tells it: *"Not hide relevant information"*. And critically, AGENT-RULES §3 gives it autonomy to *create* documents (AILOG, AIDEC, TDE) without operator pre-approval. The operator retains prioritization, not creation. The agent doesn't have to ask permission to flag something: flagging it is rule execution, not a value judgment.

What's interesting is that neither property alone produces the behavior. If you only had formal linkage (Property 1) without cultural permission, you'd have a queryable corpus no agent dares to query proactively. If you only had cultural permission (Property 2) without structure, you'd have vague flags — *"I think something might be wrong somewhere"* — that operators can't act on.

Combined, they produce something that deserves its own name: the agent externalized *"should I say something?"* as *"is there a canonical section where this fits?"*. If the answer is yes, flagging stops being an emotional decision and becomes mechanical execution. The cost of flagging drops because the destination — the canonical section — already exists.

## Why this matters beyond StrayMark

We're in a moment where AI agents can write code as fast as we think, and the question of how *not to lose control* of the process is genuinely urgent. It's not an apocalyptic question — it's an operational, craft-level question, asked by people who are building software right now and have to ship. The very pace of this episode is evidence: from the agent's diagnosis to the canonized meta-pattern less than **72 hours** passed. That speed is a gain, but it's also why visibility mechanisms matter so much.

The dominant response in the industry today is some variant of *stricter prompts, more rules in context, more gates in CI*. Those approaches are valid but have one characteristic worth naming: they turn the agent into something closer to a worker executing orders in verifiable pipelines. That solves reliability but at the cost of emergent observation. An agent that only does what it's ordered to do isn't going to flag something nobody asked it to flag, no matter how good its reasoning.

What the Sentinel episode showed is a different, complementary response: **build the documentary apparatus such that important divergences are structurally visible, and give the agent cultural permission to flag them without asking approval.** That preserves emergent observation as a property of the system, not as luck depending on how good the agent is or how long the prompt is.

There's something almost Taoist about this stance: we don't tell the agent what to look for; we build an environment where what needs to be seen is visible. The pair "visible structure + permission to name" does the work. The agent becomes, paraphrasing some colleagues, an *honest broker* between living documentation and the state of the code.

And this is what the human project behind StrayMark pursues, even if it's rarely said in those words: that the dizzying change of AI-assisted development doesn't throw us into a world where engineers lose visibility over what's being built. Visibility isn't preserved with more bureaucratic controls; it's preserved by designing the environment the agent works in so that what matters is naturally visible. The engineer doesn't become a reviewer of endless Pull Requests — they become a prioritizer of observations the agent brings structured and named.

## What we did about it

With the diagnosis clear, the question was how to hook it into StrayMark's habitual behavior without killing the emergent quality. This tension is real: every time you turn a spontaneous behavior into an obligation, you turn it into something else. A hard rule can be robust, but it can also generate agent resistance or false signal when triggered out of context.

The decision was conservative: **name the meta-pattern, don't mandate its use.**

Concretely, in PR [#160](https://github.com/StrangeDaysTech/straymark/pull/160) released as `fw-4.17.0`:

- A new canonical doc — `EMERGENT-OBSERVATION-DESIGN.md` — articulates the design property with its two components, presents the Sentinel empirical case as anchor, and builds a *pyramid of instances*: the meta-pattern at the top, and underneath all the applications already canonized (Pattern 1 and Pattern 2 of chain evolution; charter drift; follow-ups backlog drift; TDE-vs-`R<N>` escalation; external audit checkpoint). What previously seemed like ad hoc patterns reveal themselves as applications of the same underlying principle. This visualization alone is worth it: when you see seven things that seemed different and suddenly recognize they share a shape, the framework gains internal coherence without having added anything.

- A new Principle #8 — *Cross-Source Dissonance Surfacing* — in `PRINCIPLES.md`. It's the cultural rule condensed in five lines. The agent reads it when onboarding into the project and applies it recursively.

- An expansion of `AGENT-RULES.md §6 "Be Proactive"` with concrete examples of divergence to watch for: stale spec, accumulated `R<N>` not escalated to TDE, ADR contradicted by implementation, follow-ups crossing the backlog-pattern threshold, audit findings emerging post-close. They're not obligations; they're *examples of what a proactive agent would notice*. The difference matters for preserving the emergent quality.

- Four open axes identified as gaps — places where the cross-referencing infrastructure exists partially but the pattern isn't named: MCARD ↔ deployed model code, SBOM ↔ lockfiles, ADR vigente ↔ contradicting implementation, Constitution Check ↔ framework version bump. Each one needs N=1 empirical validation before crystallizing — Principle #12 applies. They got recorded in Issue [#161](https://github.com/StrangeDaysTech/straymark/issues/161) as a tracking RFC, waiting for a second domain to push one of them.

- Explicit anti-patterns: how the meta breaks. If a new document type ships with optional frontmatter linkage, cross-referencing develops blind spots. If a canonical section is replaced by free prose, queryability evaporates. If blocking gatekeeping is introduced into AILOG/AIDEC/TDE creation, the cultural permission breaks. If telemetry evolves without preserving signals like `r_n_plus_one_emergent_count`, the feedback loop breaks. These anti-patterns are cheap but effective protection against accidental erosion: any future change proposal can be checked against the list to detect whether it's regressing the meta-pattern.

All of this, from diagnosis to merge and the published `fw-4.17.0` release, took place between 9am and 4am the same day. Fifteen hours. Which is exactly the kind of pace the blog admits: you have to learn to operate under this clock, or control slips away.

## What I learned from the process

Three things I didn't expect to learn when I started by answering *"no, Issue #153 is still open"*:

**First, that codifying a meta-pattern is different from codifying a pattern.** The meta-pattern doesn't add obligations; it adds *vocabulary* and *visibility of the underlying property*. That property was already doing its job; what was missing was giving it a name to protect it under future framework evolution. Whoever reads `EMERGENT-OBSERVATION-DESIGN.md` doesn't learn to do something new — they learn to *recognize* something that was already there. That's what allows preserving it deliberately.

**Second, that the emergent property disappears if over-regulated.** There was a moment in the conversation where I chose not to turn Pattern 1 into a hard obligation (*"the agent MUST run `refresh-suggest` before declaring Charter-(N+1)"*) and instead keep it as recommendation plus the naming of the meta. That decision is counterintuitive if you come from the world of *"more controls = more reliability"*, but it's the right call for this kind of property. Agent proactivity is fragile: the moment it becomes a mandate, it stops being proactivity. What scales is the cultural engine ("Be Proactive") plus the structural linkage, not the list of specific obligations.

**Third, that the documentary apparatus we build for agents is also educating us humans.** When I saw the seven patterns scattered across various governance docs and recognized them as instances of the same meta, I understood something about StrayMark I hadn't understood before — and StrayMark is me building it over months. The framework is revealing things to its own author. That property — that structured documentation produces *insights about itself* when audited — is a domestic variant of what the agent does when it flags a divergence. It's the same mechanism, applied at a different scale.

## The human project behind it

I mean it when I say StrayMark has a human project. I'm not building a documentation governance framework because frontmatter fields obsess me. I'm building it because I have the suspicion — increasingly less speculative — that the next five years of software engineering are going to be a constant negotiation between the speed agents offer and the visibility engineers need to keep being responsible for the product. If that negotiation tilts to the agents' side, we end up in a world where code gets written and nobody knows why. If it tilts to bureaucratic control, we end up without the productivity gains that justified the experiment.

The balance isn't theoretical. It's built with concrete decisions: where to put a frontmatter field, which section should be canonical versus free, when to give the agent autonomy and when to retain prioritization. Each of those decisions tilts the balance one way or the other. And the only way to know if the balance is right is to test it — and to name what works when it works, so it survives the next change in the framework.

The CHARTER-18 episode, two days ago, was a test. It worked. What I wrote today was giving it a name. The next test already has candidates: MCARD, SBOM, ADR vigente vs implementation, Constitution Check ↔ framework bump. Four axes where the meta-pattern could reproduce if we close the infrastructure gaps. I'll wait for a second domain to push one empirically before codifying it. Principle #12 — validation against a second domain before crystallizing — remains the guardrail.

If you're reading this and you work with agents to develop software, I invite you to look at your own documentary apparatus with these two questions: *which sources are formally connected, with explicit linkage an agent can triangulate without creative work?* and *how cheap is it for the agent to flag something not asked when it detects a divergence?* If the answer to the first question is "few" and to the second is "expensive", you're probably leaving emergent observations on the table. Which isn't the end of the world — but it's exactly what separates an AI-assisted development process *under human control* from one that slips away.

---

*StrayMark fw-4.17.0 — Issue [#150](https://github.com/StrangeDaysTech/straymark/issues/150) · [#156](https://github.com/StrangeDaysTech/straymark/issues/156) · [#161](https://github.com/StrangeDaysTech/straymark/issues/161) · PR [#160](https://github.com/StrangeDaysTech/straymark/pull/160) · tag [`fw-4.17.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.17.0)*

*This document was produced with assistance from generative AI tools (Claude 4.7); all responsibility for the content rests with the human author.*
