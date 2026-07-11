---
slug: what-the-dry-run-would-have-spent
title: What the dry run would have spent
authors:
  - jose
tags:
  - straymark
  - baton
  - cost-routing
  - speckit
  - telemetry
  - rust
  - sentinel
date: 2026-06-26T00:00:00.000Z
description: Baton's second phase finally looked at money — but it routed nothing. It classified the 762 work units StrayMark had already recorded in Sentinel, recommended a tier for each, and printed what a routing policy would have spent versus all-frontier. The headline was a ~93% illustrative ceiling. The honest reading inverted the experiment's own hypothesis — granularity was never the lever. Signal coverage was.
---
*The coherence bridge touched no models on purpose. Baton's second phase finally does the economic thing — and still runs nothing. It reads the 762 work units StrayMark already recorded in the Sentinel adopter (charters, batches, follow-ups, tasks), classifies each with cheap deterministic signals, recommends a model tier, and emits **economic telemetry**: what would have gone where, and what it would have cost under a routing policy versus sending everything to a frontier model. No agent is dispatched; no model is invoked. The point isn't to save yet — it's to make the potential saving visible and test one hard rule: the cost of deciding must not eat the saving. It didn't. But the data answered a different question than the one we asked.*

<!-- truncate -->

> *If the overhead erases the saving at some granularity, that's a valid result, not a failure to hide.*

This is the second post about **Baton** (after [the coherence bridge](what-the-spec-path-only-proved-existed)). Phase 1 established that you can't route safely over drifting intent. Phase 2, [`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md), builds the classification-and-recommendation layer as a **retrospective dry run** — and, one Charter earlier, closes the loop on *when* the coherence check should fire.

## 1. The activation seam — detection, moved earlier

Phase 1 shipped `coherence` and `overlay` as on-demand, read-only commands. But the #304 drift — the frontend built against an assumed contract — is born the moment an agent **codes before** anyone runs the diagnostic. On-demand detection arrives late.

So a small Charter in between, [`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md) ([#316](https://github.com/StrangeDaysTech/straymark/issues/316)), moves the signal to authoring time. It's a **SpecKit extension** that hooks the `before_implement` lifecycle event and runs the Phase 1 engine — scoped to the active feature via a `--spec` flag, for speed — surfacing the high-confidence findings to the agent *before* it writes code. The precedent already existed and was verified in Sentinel: SpecKit's `agent-context` extension runs `after_specify`/`after_plan` to refresh context. Baton ships an analogous extension that, on `before_implement`, checks coherence instead of just refreshing.

The design bias is the one Phase 1 earned: **advisory by default**, not a hard gate. A false positive that interrupts authoring costs more trust than one that shows up in an on-demand report, so the hook surfaces blocking-and-high-confidence findings and lets the human decide, with a hard gate available only by config. And it degrades gracefully — if the `straymark-baton` binary isn't found, the hook warns and steps aside; it never breaks the SpecKit flow. Dogfooded in Sentinel, `before_implement` on feature `005-frontend-dashboard` surfaced the real C4 (`services.public-visibility` consumed without referencing its defining decision) in advisory mode, repo untouched. That's issue #304's third ask — *"drift signal at authoring time, not just audit time"* — in its strongest form.

## 2. The dry-run router — three framing calls first

Then the economic layer. Three framing decisions, resolved with the operator before a line was written, because the concept doc had deliberately left them open:

1. **Routable unit: instrument what exists, and measure.** No invented vocabulary. An earlier consultant draft had proposed a "Work Unit / WU-NNN" concept that doesn't exist in the repo; we threw the *name* out but kept the real problem it pointed at. Baton classifies at the granularity StrayMark **already records** — Charter, Batch, Follow-up, Task — measures how heterogeneous each level is, and lets the telemetry reveal which granularity "pays." The open sub-decision gets answered with data, not by decree.
2. **Cost model: illustrative tiers in config.** A `baton:` block in `config.yml` (the same config-driven pattern as [#279](https://github.com/StrangeDaysTech/straymark/issues/279)) declares an *illustrative* cost-per-million-tokens per tier. That unblocks the dry run without solving real provider-cost identity (deferred to Phase 3). We measure **relative** saving, not a bill.
3. **Classifier: cheap signals only, no LLM.** Deterministic rules over signals StrayMark already computes. This satisfies the hard economic corollary directly: classifying must be cheap, or the classifier becomes the cost it was meant to avoid. An LLM classifier stays a future escalation, used only when the cheap signals are ambiguous *and* the saving justifies it.

The routable units are the work StrayMark already recorded: 45 Charters, 82 Batches, 135 Follow-ups, 500 Tasks — **762 units**. Each is classified into a task class — Planner/Architect, Implementer, Auditor, Operator — from cheap signals: `effort_estimate`, `analyze` complexity, the Charter's `risk_level`, a follow-up's bucket and severity, the Loom projection's component state, Phase 1's coherence findings, the file surface touched. Typed, pure, no I/O. A tier policy maps class → tier with a conservative fallback, and `route --dry-run` prints the economics. It recommends. It never executes.

### The hard rule, stated up front

The constraint that governs the whole design:

> The cost of classifying, atomizing, and routing must not equal or exceed the saving of not loading everything onto a frontier model. If the complexity of the solution matches the cost of not having it, the solution doesn't exist.

The telemetry measures this **explicitly** — overhead versus saving, per granularity. If a granularity's overhead meets or beats its saving, it's reported as *non-routable*, not forced. That's the honesty guard: a negative result is a result.

## 3. The headline economics (illustrative)

Run read-only against Sentinel's real governance corpus, `git status` unchanged:

```
ALL (762 units) — routable
  tiers: economic 617 · frontier 14 · local 131
  cost: all-frontier 1293.60 → routed 93.68  (gross saving 1199.92 ≈ 93%)
  overhead 15.24 → net saving 1184.68
  caveats: 57% low-confidence · 57% of saving on low-confidence routing · 12% conflicted
  sensitivity: breakeven overhead/unit 1.575 · robust at 2× overhead: true
```

Every granularity comes out net-positive and robust to twice the illustrative overhead. The classification overhead is **1.2% of the gross saving** — so the hard rule is not violated by the cost of deciding. By the letter of the gate, routing pays everywhere. If I'd stopped reading there, I'd have shipped a triumphant number.

## 4. The honest reading — confidence, not granularity

The §4.2 verdict says "routable everywhere." The honesty guards say the saving is **fragile**, and they say it in a way that contradicts the experiment's founding conjecture.

| Granularity | conflict % | high+med confidence % |
|---|---|---|
| Charter | 6% | 46% |
| Batch | 8% | 39% |
| Follow-up | 5% | 37% |
| Task | 15% | 44% |

Two findings, both against the hypothesis:

**Heterogeneity is *not* the blocker.** The conjecture was that a coarse unit — a whole Charter — bundles mixed work (design + code + audit + cleanup) and so resists clean routing, and that a finer unit would route better. The data inverts it: **Task** has the *highest* conflict (15%), Charter among the lowest (6%). The reason is deflating — the conflict metric is confounded by **title verbosity**. Task titles are descriptive, so they surface more cues, so they detect more "conflict"; charter titles are terse. Conflict was a weak proxy for heterogeneity, and there's no granularity that's meaningfully more homogeneous to prefer.

**Signal coverage is the binding constraint.** High+medium confidence sits at **37–46% across *all* granularities** — flat. **57% of every level is low-confidence**, and not because of conflicts (5–15%) but because of the *no-cue default*: 45% of units surface no classification cue at all. High confidence needs `effort_estimate`, which only Charters carry; most units get classified on a terse title alone. So **57% of that 93% gross saving rests on low-confidence guesses.**

The empirical answer to the routable-unit question: *which granularity is routable?* By net saving, all of them; by trust, none is cleaner than another. **Granularity is not the lever on this corpus. Signal coverage is.** Introducing a finer sub-unit — the thing the discarded "Work Unit" vocabulary was reaching for — would not help: Task is already the finest, and it's no more trustworthy than a Charter.

## 5. Graduating knowledge, not a saving

The gate allowed for exactly this: *net-positive relative saving after overhead at some granularity, naming which; and a net-negative result with evidence still graduates the knowledge.*

It's MET — and what it graduates is the knowledge, which turned out to be the more valuable outcome. The dry run establishes routing's **ceiling** (~93% illustrative, surviving 2× overhead) and its current **trust floor** (~43% of units route at high+medium confidence, so ~57% of the saving is a guess). And it **reframes Phase 3**, on data rather than assertion: the path to a *trustworthy* saving is not a different routable unit. It's **wiring the deferred signals** — per-function complexity (which needs `analyze` graduated from the `cli` crate into `core`), architecture state from the Loom projection, coherence findings from Phase 1 — to lift confidence *before* anything executes. B2 deferred those heavier signals under the cheap-first rule; the dogfood just justified them retroactively as the next step. That's the empirical loop working exactly as intended.

## 6. What we deliberately didn't do

Still **recommend-only.** `route` requires `--dry-run`; no model client is linked, no agent is dispatched, no network call is made. Real execution is Phase 3.

The costs are **illustrative**, and labeled as such everywhere. The 93% is *shape*, not money — real provider-cost identity is deferred until there's a unified contract to model, which there isn't yet. Reporting a fake dollar figure would be its own kind of mockup telemetry.

And we resisted the tempting patch. When the cheap signals under-covered 45% of units, the obvious move was to reach for an LLM classifier to fill the gap. We didn't — that would violate the cheap-first rule and bury the real finding (signal coverage) under a more expensive guess. The under-coverage is filed as a follow-up, not hidden. All of this stays **EXPERIMENTAL (Baton v0 / N=1)**.

## 7. If you've read this far

The portable lesson is what the honesty guards did to a beautiful number. Baton could have shipped "93% cheaper" and been telling the truth — the gross saving is real. It would also have been deeply misleading, because more than half of it rides on the tool guessing when it had nothing to go on. The number that matters wasn't the saving; it was the *confidence distribution behind* the saving, and the two point in opposite directions. Find a headline metric in your world that's carrying good news — a conversion lift, a latency win, a cost reduction — and ask what fraction of it rests on cases the measurement was actually sure about. A true average over a pile of guesses is still a pile of guesses. The dry run's most useful output wasn't the 93%. It was the 57% next to it.

The next post is where the fix for that 57% turns out to cost almost nothing — because the author knew the answer all along.

---

*Baton Phase 2 — [`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md) · [`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md) · dogfood [`04-phase2-dry-run-dogfood.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/04-phase2-dry-run-dogfood.md). Issues [#316](https://github.com/StrangeDaysTech/straymark/issues/316) · [#323](https://github.com/StrangeDaysTech/straymark/issues/323) · [#324](https://github.com/StrangeDaysTech/straymark/issues/324) · [#325](https://github.com/StrangeDaysTech/straymark/issues/325) · [#326](https://github.com/StrangeDaysTech/straymark/issues/326). Predecessor: [What the spec path only proved existed](what-the-spec-path-only-proved-existed).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*