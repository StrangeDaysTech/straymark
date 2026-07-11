---
slug: what-the-spec-path-only-proved-existed
title: What the spec path only proved existed
authors:
  - jose
tags:
  - straymark
  - baton
  - speckit
  - coherence
  - cost-routing
  - rust
  - sentinel
date: 2026-06-25T00:00:00.000Z
description: A fourth experiment started with an economic problem — when the subscription subsidies end, all-frontier-all-the-time stops being affordable — and immediately ran into a structural one. You can't route work to a cheaper model if the work is drifting from its own plan; a cheaper router over forgotten intent only automates the drift faster. So Baton's first move touches no models at all. It reads the plan StrayMark had only ever proved *existed*.
---
*A fourth experiment, Baton, started from an uncomfortable economic forecast: the monthly AI subscriptions are propped up by a consumption subsidy, and when that ends — pay-per-million-tokens, the way the Chinese providers already bill — running every task on a frontier model stops being affordable for an independent developer. The obvious fix is to route cheap work to cheap models and reserve the frontier for design and adjudication. The non-obvious problem is that a cheaper router operating over forgotten intent only automates the drift faster. So Baton's first deliverable routes nothing. It reads the SpecKit plan that StrayMark, until now, had only ever checked the **existence** of — and reconciles it against what actually got built.*

<!-- truncate -->

> *A cheaper router operating over forgotten intent only automates the drift faster. So: coherence first, routing later.*

This is the first post about **Baton**, the newest sibling experiment — living in `experiment-baton/`, alongside [Loom](where-the-debt-actually-was) and the OKF comparison, with its own charters, its own tags, and its own graduation gate. Like Loom, it's designed from day one to graduate a clean, typed core back into StrayMark if it earns its keep. Unlike Loom, it starts from money.

## 1. Why now

The forecast is boring and hard to argue with. A MAX-tier subscription runs about \$100/month today, and that price is widely understood to sit on top of \$400–600/month of consumption the provider is currently absorbing. The Chinese labs have already moved to metered, pay-per-million-token billing; the Western ones look like a matter of time. When the subsidy evaporates, the real cost of the way we work now — every commit, every PR summary, every atomic task classification handed to the same frontier model that designed the architecture — becomes potentially unaffordable for a solo operator. And the operator, here, was me.

Switching providers wholesale is a high-variance bet. The actionable move from today is to change *how* the compute is spent. Not everything needs a frontier model: commits, PRs, atomic task lists, classification — models a tier down do those well. Reserve Opus / GPT-5.5 for design, architecture, adjudication, and open-ended diagnosis. The thesis is simple: **aim the token spend at the model appropriate to the kind of task**, leaning on the governance discipline StrayMark already produces to know what kind of task each unit is.

That's the whole pitch, and it would be an easy weekend project if it weren't quietly wrong in a way that took a Sentinel bug to see.

## 2. The trap that reordered the work

Here is the failure mode a naive router walks straight into. StrayMark governs *implementation* — charters, AILOGs, follow-ups, TDEs — and Loom projects the *emergent* architecture from those signals plus the code on disk. But the *intended* architecture — the global plan that lives in the SpecKit artifacts — never enters the graph. Verified, not assumed: a Charter can declare `originating_spec: specs/004/spec.md`, and StrayMark validates only that **the file exists**. `validate_spec_path()` never parses its contents. The link between a Charter and its spec is nominal. Loom's architecture projection reads charters, drift, AILOGs, TDEs, on-disk inventory — and never once opens `spec.md`.

So the intended architecture and the emergent architecture are two planes nobody reconciles. Intent dilutes into implementation, and the divergence is invisible because no artifact compares them. Now imagine wrapping that with a router that cheerfully sends "implement this task" to an economy-tier model. If the task's own plan has already drifted, you haven't saved money — you've made the drift cheaper and faster to produce. **A cheaper router over forgotten intent only automates the drift faster.** Coherence has to come first, or routing is actively harmful.

### The villain, on real data

This isn't a hypothetical. The canonical case is [issue #304](https://github.com/StrangeDaysTech/straymark/issues/304), a real bug in the Sentinel adopter. A post-MVP decision (PM-002, sitting in a spec's backlog) extended a per-component health contract. The frontend spec — a *different* spec — never referenced that decision. It implemented an **assumed** contract: wrong fields, wrong enums, metrics that didn't exist. The mocks encoded the assumption. The tests passed, green. And it only detonated in staging, as `TypeError: t.find is not a function`.

The same mechanism produced two more Sentinel scars. The **mockup telemetry**: an agent, needing a data source that a planned-but-unbuilt module was supposed to provide, quietly emitted mockups where it needed them — so the whole telemetry surface came up fake, built on a foundation that didn't exist. And the **PolicyEngine dispersion**: a module documented in `.specify/memory/` as a clean, dedicated component whose functions instead scattered across other modules, because the agent making local decisions never consulted the global plan that said otherwise. None of these are isolated human slips. They're one structural hole: *an agent that never read the global SpecKit plan makes local decisions that contradict the design, and nothing tells it.*

## 3. Phase 1 — the read seam

There are three seams between SpecKit's intent and StrayMark's governance. Baton's first Charter — [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md), effort L, closed the same day it was reported — builds only the **read** seam. Ingest the intent, reconcile it, emit a diagnostic. Read-only: it diagnoses, it does not mutate SpecKit and it does not run agents. It touches no models. That last constraint is the point — the economic driver is real, but the first deliverable pays it exactly zero attention, because the coherence problem has to be solved before a router is safe to build.

Four pieces, built in order across five batches:

- **A versioned, read-only SpecKit adapter** ([#308](https://github.com/StrangeDaysTech/straymark/issues/308)). Anchored to `speckit_version` (0.11.3, the one Sentinel runs), it parses `specs/**/{spec,plan,tasks}.md`, post-MVP backlogs, `contracts/**`, and the free-form `.specify/memory/`. The memory files are treated as a *hint*, never a hard contract — tolerant parsing, low-confidence findings, so human prose doesn't produce false alarms.
- **An IntentModel — the third plane** ([#309](https://github.com/StrangeDaysTech/straymark/issues/309)). A typed representation of the components, modules, and contracts SpecKit *declares*, including post-MVP backlog decisions. It reuses Loom's architecture-model types where it can and does **not** re-implement `glob_match` or drift; the whole design principle is *don't duplicate the matcher, share it*.
- **Cross-spec contract-provenance edges** — the heart of #304. They link a consumer (a spec, an FR) to the producer or decision that defines the contract it depends on. Inferred conservatively in this phase; a silent false negative is preferred to a noisy false positive.
- **A coherence engine, C1–C4** ([#310](https://github.com/StrangeDaysTech/straymark/issues/310)), plus a Loom-consumable intent overlay ([#311](https://github.com/StrangeDaysTech/straymark/issues/311)) that extends the projection to a third plane: *intent vs. governance vs. code*. High-confidence finding classes first: a component intended in memory with no files implementing it; a contract field a consumer requires with no producing source; a consumer built against a shape a later decision changed. The CLI mirrors `architecture validate` — `text|json|markdown`, non-zero exit on findings, CI-gateable.

The prototype lives in a `straymark-baton` crate, mirroring how `straymark-loom` sits apart from `core`. The pure, typed logic is exposed so it can graduate to `straymark-core` later — but not yet, so an experiment can't couple the shared core to code that might be thrown away.

## 4. What the dogfood actually caught

The graduation gate was deliberately concrete: *Phase 1 succeeds if the diagnostic, run read-only against Sentinel, catches at least one real architectural drift that human review had let through.*

Run against Sentinel at HEAD `24d5a66`, `git status --porcelain` empty before and after both commands — the bridge mutates nothing in the target — the first raw pass emitted **90 findings**, mostly noise. That's the expected shape of a first dogfood: real data surfaces precision bugs that no fixture would. Four targeted calibration fixes brought it to **6 findings, 0 blocking**, each fix a genuine bug caught only by reality — a backlog decision was claiming every endpoint its spec mentioned rather than only the ones its own section cited (that alone collapsed one finding class from 84 to 1); test files were being counted as contract producers; a memory-derived match was firing on a substring.

The headline that survived is one precise C4:

```
[C4] spec '005-frontend-dashboard' consumes contract 'services.public-visibility'
     but never references its defining decision(s): PM-001, AILOG-2026-04-21-002
```

That is the #304 pattern, structurally, on live data: a consumer spec depends on a contract that a decision in *another* spec defined, without acknowledging it — the exact cross-spec decision-propagation drift that nothing had ever surfaced, and that human review had not flagged. Caught read-only, on real code, in one legible line.

The intent overlay added the rest of the legible picture — `DevPortal` and `UsageGuard` designed but with no module (real gaps); `SentinelAgent` in the memory docs versus an `agents/` directory on disk (a naming drift). And a detail I've come to trust the tool for: **PolicyEngine now shows `intended & implemented`.** The module the team once forgot and dispersed has since been built, and the bridge correctly does *not* flag it. It reflects current reality, not a stale anecdote — which is exactly what you need from something you'll wire into an authoring flow.

## 5. What we deliberately didn't do

The restraint section, which Baton needs more than most, because the economic pitch is seductive.

We touched **no models.** No tier, no token, no cost, no routing — Phase 1 imports none of it, on purpose. The whole point was to prove that coherence is a prerequisite, and building the router first would have been the mistake the experiment exists to avoid.

We stayed **read-only.** The bridge diagnoses; it never writes a patch back to SpecKit. Silent auto-correction of a contract is precisely the kind of confident wrongness that got Sentinel into this.

And we shipped an **honest limitation**, not a clean win. Contract keying over generated type files (`types.gen.ts` held every API type in one file) collapses distinct contracts together, so the field/enum-mismatch findings (C2/C3) don't isolate on Sentinel yet — the run is 0 blocking *by design*, not because the drift is fully caught. The consumer side of that got a call-site binding fix ([#313](https://github.com/StrangeDaysTech/straymark/issues/313)); the symmetric producer side is a tracked sibling follow-up ([#319](https://github.com/StrangeDaysTech/straymark/issues/319)). All of this is **EXPERIMENTAL (Baton v0 / N=1)** and may change without a deprecation cycle. The value proposition of a coherence tool is trust, and trust is lost far faster by one confident false positive than it's earned by ten true ones — so the finding classes start narrow and the severity starts low.

## 6. If you've read this far

The portable question is the one the nominal spec-path check handed us. Find a place in your own system where one artifact *references* another — a config that names a schema, a service that declares a dependency, a ticket that links a design doc — and ask what the reference actually verifies. Does the check confirm the linked thing *exists*, or does it confirm the two things still *agree*? StrayMark validated that `originating_spec` pointed at a real file for months, and felt like a link the whole time. It was proving existence and implying coherence, and the gap between those two is where a frontend gets built against a contract nobody updated. Every "it's linked, so it's fine" you didn't personally reconcile deserves the same suspicion — not *is the link there*, but *does the link check anything past the filename?*

The next post is where Baton finally looks at money — and finds that the thing it assumed would be the lever isn't.

---

*Baton Phase 1 — [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md) · concept [`01-baton-concept.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/01-baton-concept.md) · dogfood [`03-sentinel-dogfood-report.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/03-sentinel-dogfood-report.md). Issues [#304](https://github.com/StrangeDaysTech/straymark/issues/304) · [#308](https://github.com/StrangeDaysTech/straymark/issues/308) · [#309](https://github.com/StrangeDaysTech/straymark/issues/309) · [#310](https://github.com/StrangeDaysTech/straymark/issues/310) · [#311](https://github.com/StrangeDaysTech/straymark/issues/311) · [#313](https://github.com/StrangeDaysTech/straymark/issues/313). Sibling arc: [Where the debt actually was](where-the-debt-actually-was).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
