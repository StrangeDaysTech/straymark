---
slug: what-the-follow-up-took-for-granted
title: What the follow-up took for granted
authors:
  - jose
tags:
  - straymark
  - governance
  - followups
  - charters
  - cli
draft: false
date: 2026-07-18T00:00:00.000Z
description: A greenfield adopter cleared a backlog of follow-ups before shipping and found that three of them — the three that were real work — each carried a premise that was false the moment you checked it. Not because the code had changed under them, but because a follow-up is written at the one moment you are least equipped to verify it. The lesson isn't "write better follow-ups." It's that a follow-up is a dated hypothesis, and the cheap place to test it is when you read it, not when you write it — so that is where StrayMark now puts the check.
---

*An adopter cleared a follow-up backlog before shipping — seven entries down to one. Four were already-resolved noise. Of the three that were real, unresolved work, every single one carried a false premise: a test to "replicate" that never existed, a gate to build against a reference that can't exist, an optimization to fix a cost that wasn't there. None of the three was wrong because the code drifted under it. They were wrong the moment they were written, and the falseness only surfaced on a thirty-second check performed months later. That timing is the whole story. A follow-up is authored at the exact moment you are least able to verify it — and read at a moment when verifying is nearly free. StrayMark's registry used to treat entries as instructions to execute. They are better understood as dated hypotheses to re-test — and, as of this release, that is what the tool says and where it puts the check.*

<!-- truncate -->

## The setup

The adopter is the same greenfield .NET 10 / Rust CRDT library behind [#345](https://github.com/StrangeDaysTech/straymark/issues/345)/[#346](https://github.com/StrangeDaysTech/straymark/issues/346)/[#355](https://github.com/StrangeDaysTech/straymark/issues/355)/[#360](https://github.com/StrangeDaysTech/straymark/issues/360) — the [Adopter] Weft discussion. Milestone 3 was code-complete except for the operator-gated publish. Before pulling that irreversible lever, the plan was to drain the follow-up backlog: seven open entries, so that whatever shipped wasn't shipping with known-but-unwired gaps frozen into a public package.

Triage split them fast. Four were not work: two were already-resolved risks the extractor had scraped, one was a decision deferred to a trigger that hadn't fired, one was blocked on an upstream merge outside the project's control. That left **three follow-ups that were genuine, unresolved, actionable work.** All three got a Charter. All three, on inspection, turned out to rest on something that wasn't true.

## Three follow-ups, three false premises

**The parity test that didn't exist.** One follow-up read, in effect, "replicate for the Loro shim the header↔binding parity test that the yrs shim already has." Reasonable — except the yrs test did not exist. What existed were two *comments* — one in the C# binding, one in the C header — both stating "a CI test validates that these declarations match this header." No such test was ever written; the line was aspirational, traceable back to a research note that said the check *could* be generated. The follow-up's author had read a comment as a fact. The follow-up inherited the comment's falseness and added a hop of authority. Building it "as written" would have meant porting a test that had no original.

**The gate against a reference that can't exist.** Another asked for a determinism gate for the Loro engine, mirroring the existing yrs↔Yjs parity gate. The symmetry is seductive: yrs has an independent reference implementation (Yjs) to check byte-for-byte against, so surely Loro should have its equivalent. It can't. There is no independent second implementation of Loro's format — the npm package is a WebAssembly build of the *same* Rust core, so comparing against it is comparing the crate to itself. The follow-up reasoned by analogy, and the analogy quietly failed. The realizable gate was a different, more modest thing (self-determinism across runs, a regression witness rather than a parity proof) — which is what got built, but only because the premise was caught first.

**The optimization for a cost that wasn't there.** The third feared that reordering the relay to persist-before-broadcast would push I/O onto the actor's hot path and hurt throughput. Tracing the actual await chain showed the persistence call was *already* awaited on the receive loop, before the connection reads its next frame — reordering added no I/O to any hot path it wasn't already on. A load harness confirmed it: the "safe" ordering cost effectively zero at p50/p99. The follow-up had encoded a *mental model* of the architecture, not the architecture as built. This is the subtlest of the three: no lying comment, no broken analogy — just a map in someone's head that had drifted a notch from the territory, and a note that faithfully recorded the map.

Three follow-ups. A comment believed, an analogy over-trusted, a mental model slightly stale. Different failure modes, one shared shape: **each premise was false at the moment of writing, and cheap to falsify at the moment of reading.**

## The two moments

There are exactly two places you could put a "verify the premise" rule: when the follow-up is written, or when it is read.

Writing happens at the worst possible moment for verification. A follow-up is a note-to-self jotted while *finishing something else* — closing Charter N, head full of the current subsystem, glancing sideways at a different one you're about to leave. Verifying the sideways glance means a full context switch away from the work you're trying to land. It's expensive precisely *then*.

Reading happens at the best possible moment. When you finally act on the follow-up, you're already inside that subsystem, with the code open. Checking "does this test actually exist / does this reference actually exist / does this cost actually exist" is a `grep`, a file read, a traced call chain — seconds. The three false premises above collapsed on exactly these checks.

So the verification is not just cheaper at read time — it's *categorically* cheaper, because at read time you've already paid the context-switch cost for other reasons. The economics point in one direction.

## The reframe: a backlog is a speculative buffer

Here's the part that changes how the whole feature should be understood. It's tempting to conclude "authors should verify follow-ups harder before writing them." That's the wrong lesson, and it would make the tool worse.

A follow-up backlog is a *speculative buffer*. Its job is to capture, cheaply, that something *might* be worth doing — so the signal isn't lost when attention moves on. If you demanded verification at capture time, you'd spend the close of every Charter spelunking subsystems you're abandoning, and the rational response would be to stop writing follow-ups at all. **Eager verification defeats the purpose of the buffer.** The under-verified entry isn't a defect of the author; it's the *expected epistemic status* of anything in a speculative buffer.

Which means the false premises weren't bugs in how the follow-ups were written. They were the natural state of a hypothesis that had never been tested — and a follow-up is a hypothesis. The only real bug would have been *executing one without re-testing it*. Which is exactly the trap the registry's framing set up: it presented entries as a to-do list, an instruction set, a plan. Read as instructions, false premises become wasted Charters. Read as **dated hypotheses**, they become what they are — cheap bets to re-check the moment you're positioned to.

## What we shipped

The field report landed as [#365](https://github.com/StrangeDaysTech/straymark/issues/365). It split cleanly into two changes, and — fittingly — the reframe itself was handled as a *dated hypothesis*: recorded as a decision ([`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md)), reviewed by a human, and signed before a line of the shipped doc changed. A claim about how to treat claims deserved the same discipline it was arguing for.

### Frame entries as hypotheses, and move verification to execution

Released as [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0) ([#369](https://github.com/StrangeDaysTech/straymark/pull/369)), in three layers:

**The words.** The follow-ups pattern doc gains a first-class *"Epistemic status"* section that says the quiet part out loud: the registry is a speculative buffer, an entry is a dated and decaying hypothesis rather than an instruction, an under-verified entry is the *expected* state and not an authoring defect — and the only real bug is executing one without re-testing its premise. The agent directives (`AGENT-RULES.md §13`) gain the matching rule: **write cheaply at capture; re-verify the premise when you promote or act — never at capture.** The framing was the load-bearing change. Everything else just gives it teeth.

**The fields.** An entry can now carry an explicit `Premise` — the load-bearing assumption it rests on — and a `Verified-at` date. Both are optional and the schema stays `v1`, so no existing registry changes. Stating the premise is what turns "re-verify" from a vague nudge into a concrete target: *"the yrs shim already has a parity test"* is a sentence you can falsify in one `grep`. `Verified-at` absent means "never re-checked since capture" — the honest default; its presence is provenance that the hypothesis was tested against reality before anyone spent a Charter on it.

**The checkpoint.** Two CLI affordances put the check exactly where it's cheap:

- `straymark followups verify FU-NNN` surfaces the premise, optionally records or updates it (`--premise "..."`), and stamps `Verified-at` when you confirm the re-check (`--verified`). With no flags it's read-only — it just shows you the assumption and asks whether it still holds. This is the common path: an entry acted on as a chore that never becomes a formal debt document.
- `straymark followups promote FU-NNN --premise-verified` does the same at the moment a follow-up graduates into a TDE: it prints the premise with a *"is this still true? re-verify against the code"* reminder, and stamps `Verified-at` on confirmation.

The design rule under both: **the CLI reminds and records; it never gates.** Promotion proceeds with or without the flag; `verify` never blocks anything. It won't decide whether your premise is true — that's the human's job, standing in the one spot where the check is nearly free. Anything stricter would recreate the capture-time tax the reframe exists to avoid.

### The title the machine wrote

The secondary finding was smaller and more concrete, and it shipped first, in [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2) ([#366](https://github.com/StrangeDaysTech/straymark/pull/366)). When `followups drift --apply` auto-extracts an entry from an AILOG's `## Follow-ups` section, it took the *first physical line* of the bullet as the title. AILOG bullets are hard-wrapped prose, so a lead sentence got sliced at the author's wrap column — three of this session's entries were literally titled things like *"**Footgun of a local pack contaminated with `test-hooks`** — the pack reads from"*, cut mid-thought. A machine grabbing a line-fragment loses the nuance a hand title carries, and a follow-up whose *title* misrepresents it starts life already a little wrong — which compounds the exact "read as instruction" hazard the rest of this post is about.

The fix un-wraps the bullet, prefers a substantial leading `**bold**` span as the title (the convention authors already reach for), and otherwise takes the first *sentence*, capped at a word boundary. The subtle part was keeping it **hash-neutral**: the registry dedupes entries by a content hash derived from the raw first line, so a nicer title had to be decoupled from the dedup key — otherwise every already-extracted entry in every adopter's registry would re-appear as a duplicate on the next scan. Titles got sharper; nothing re-duplicated.

## The portable version

If you keep any backlog of deferred work — a follow-ups registry, a `// TODO(later)`, an issue tagged `someday` — you're keeping a buffer of hypotheses, whether or not you call it that. The entries are cheap to write and were written when you couldn't check them. The mistake is not writing them loosely; that's correct, and demanding rigor at capture would just make you stop capturing. The mistake is reading them as a plan and executing on faith. Re-test the premise when you act — you're standing in the one spot where it's nearly free — and let a false premise cost you a `grep`, not a Charter.

---

*Empirical basis: three Charters clearing a follow-up backlog in the [Adopter] Weft project, 2026-07-16 → 2026-07-18 (7 open → 1). Shipped in StrayMark [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0) (the reframe, [#365](https://github.com/StrangeDaysTech/straymark/issues/365)/[#369](https://github.com/StrangeDaysTech/straymark/pull/369), [`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md)) and [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2) (title fidelity, [#366](https://github.com/StrangeDaysTech/straymark/pull/366)). Related: [#360](https://github.com/StrangeDaysTech/straymark/issues/360), [#355](https://github.com/StrangeDaysTech/straymark/issues/355), [#346](https://github.com/StrangeDaysTech/straymark/issues/346).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
