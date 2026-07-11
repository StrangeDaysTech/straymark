---
slug: what-a-million-assertions-cant-settle
title: What a million assertions can't settle
authors:
  - jose
tags:
  - straymark
  - audit
  - attribution
  - governance
  - sentinel
  - data
date: 2026-07-10T00:00:00.000Z
description: Bun's Rust rewrite ran AI adversarial review at a scale worth studying — one model family, split context windows, and 1,386,826 test assertions as the final judge. StrayMark's audits look different because the target is different, and the difference isn't stylistic. When a passing test suite can be the judge, you isolate reviewers by context. When the judge is a human judgment call, you diversify by model family — and 25 audit cycles of real data show exactly why, including one family that went quietly blind.
---
*Bun was rewritten from Zig to Rust in eleven days on the back of AI adversarial review — a genuine landmark, and one worth reading closely for what it says about how you disbelieve a machine's work. Their method and ours look different, and the difference isn't taste: it's dictated by what gets to be the judge. Bun had 1,386,826 test assertions and six-platform CI, so a passing suite could settle correctness, and they isolated reviewers by context window within a single model family. StrayMark audits changes where the "right answer" is a judgment with no test to settle it — so we diversify across model families and treat their independent convergence as the signal. Twenty-five real audit cycles in the Sentinel project show what that buys, what it catches that green tests miss, and — as a curiosity — how seven different model families actually behaved, including one that had to be demoted for going quietly blind.*

<!-- truncate -->

> *The v0 cycle on this same Charter produced 0 substantive findings. The v1 cycle on the same code, with the same auditor model families, produced 1 verified runtime-fatal bug plus 3 deeper findings. Filesystem tool use and the full git range turn paste-based theatre into real engineering review.*

This is a coda to [Who the audit thought it was](who-the-audit-thought-it-was) — that post was about making the audit's attribution un-fakeable; this one is about what the audits actually did, with numbers. It was prompted by [Bun's rewrite-in-Rust post](https://bun.com/blog/bun-in-rust), which describes an AI-audit methodology at a scale worth taking seriously, and worth contrasting honestly.

## Two ways to distrust a machine

Bun's process is beautifully engineered. Jarred Sumner rewrote Bun with roughly 50 dynamic Claude workflows over 11 days: **one implementer, two or more adversarial reviewers, and a fixer**, the reviewers running in **separate context windows** and told to assume the code is wrong — because "the Claude that wrote the code wants the code to get accepted; the Claude that reviews wants to find bugs." At peak, ~64 Claudes across 4 git worktrees; 6,502 commits; ~$165,000 in tokens. And the load-bearing part: **all of it stood on an objective oracle** — Bun's test suite is written in TypeScript, so it doesn't depend on the runtime's language; **1,386,826 `expect()` assertions across 60,624 tests** passed on all six platforms before Sumner "pressed the merge button," having manually verified the tests weren't being skipped.

That last fact is what makes the single-family, split-context design exactly right for the job. A Zig→Rust rewrite is a **behaviour-preserving port**: "correct" has a clean, machine-checkable definition. The adversarial reviewers don't have to be the final word, because the million-assertion suite is. Independence-by-context is enough, because anything the reviewers miss, the oracle catches.

StrayMark audits the case where that oracle doesn't exist. Most engineering isn't a port; the question is *is this the right design, does it hold under concurrency, does this contract still match the decision that defined it* — judgments with no suite that can settle them. When the review **is** the oracle, a single mind's blind spot is invisible: if your one reviewer is systematically blind to a class of bug, nothing downstream will catch it. So StrayMark's audit cycle diversifies along the axis Bun didn't need to — **model family** — and treats *independent cross-family convergence* as the strongest signal, with a separate calibrator model reconciling the reports and rating the auditors.

| | Bun's rewrite audit | StrayMark's audit cycle |
|---|---|---|
| **What settles "correct"** | An objective oracle — 1,386,826 test assertions, 6-platform CI | A human-directed judgment; no test can settle it |
| **Independence comes from** | Separate context windows (author vs. adversarial reviewer) | Separate **model families** auditing blind, plus context separation |
| **Model diversity** | One family (Claude / Fable 5), many instances | Seven families across the corpus — gpt, gemini, glm, qwen, claude, deepseek, kimi |
| **Reconciliation** | A fixer applies reviewer notes | A calibrator model consolidates, dedups, and rates each auditor |
| **The human** | Monitors the loop, verifies tests ran, presses merge | Directs identity, adjudicates convergence, decides closure |

Neither is "better." They're calibrated to different targets. Bun's is the right shape when a suite can be the judge; ours is the shape you need when it can't.

## Why family diversity, not just context

Here's the argument, and then the data that backs it. If the reviewer is the final judge and you run one model family, you are betting that family has no systematic blind spot. That bet is uncheckable from the inside — a blind spot is, by definition, the thing you can't see. The only way to surface it is to ask a *different* mind and notice the disagreement. Cross-family diversity isn't about redundancy; it's about making blind spots *representable*.

Sentinel — the project StrayMark governs and dogfoods on — has now run **25 audit cycles across 21 Charters, 68 auditor reports, seven model families**, from 2026-05-05 to 2026-07-10. And the data says independence is doing real work. Of the consolidated findings, roughly **70% were raised by only one auditor** — different families genuinely surface different things — while the ~**19% that converged across families** are exactly what the calibrator flags as highest-confidence. From `CHARTER-40`'s review: *"the budget cap is a non-atomic check-then-act — found independently by gpt-5-codex AND claude-fable, the strongest convergent signal."*

## What the audits caught that green tests didn't

The point of all this is software quality, so here is the quality, quantified. Across those 25 cycles: **211 raw findings**, consolidated to **169**, breaking down by category as **92 implementation-gaps, 89 real-debt, 23 false-positives, 6 hallucinations**. Of Sentinel's **188 follow-up entries**, 55 cite an audit; **three of its four technical-debt entries trace directly to audit reviews** (one — a `CommsHub` scope guard that was declared but never actually invoked, an auth hole — was promoted to a TDE and then resolved).

But the counts undersell it. The recurring theme is defects that a **green test suite and human review both passed**, because they weren't the kind of thing a test was watching for:

- **A runtime-fatal SQL bug (`CHARTER-07`).** An `ON CONFLICT` clause over a `COALESCE(col, '')` expression that Postgres rejects at runtime — *"the first consumer Charters would hit this on first write attempt."* The compile gate structurally cannot see a SQL semantic error. `gpt-5.3-codex` caught it; `gemini` missed it.
- **A concurrency race under a green suite (`CHARTER-17`).** The implementation shipped *"build + vet + unit + integration suite green, 12/12 declared tasks."* The auditor still found a `SELECT`-then-`INSERT` under READ COMMITTED where two replicas both see `isNew=true` — *"duplicate side-effects: admin emails, legal-text emissions"* — confirmed against the actual `cloud-run.yaml` multi-instance topology. No single-node test would ever reproduce it.
- **A production-blocking bug masked by its own tests (`CHARTER-42`).** The `SafeMode` tests *"masked it by injecting a permissive fake MCP client."* Green, and wrong.
- **LLM-safety findings (`CHARTER-40`).** An `AUTO` action letting the model perform a global suppression on a model-chosen email; free-text model reasoning persisted into an audit row *"flagged `PIIRedacted: true` without ever passing the redactor."*

And the single most convincing data point is a natural before/after on the *same Charter*: the legacy v0 audit cycle — paste-based, no tool use — produced **0 substantive findings**; the v1 cycle, same code and same model families but with filesystem tool use and the full git range, produced **the runtime-fatal SQL bug plus three deeper findings**. The methodology, not the models, is what turned "looks fine" into a caught production defect.

## A curiosity: how the seven families actually behaved

Because every review scores its auditors on a fixed rubric — **scope precision (25%), technical depth (25%), bug detection (30%), false-positive rate (20%)** — the corpus doubles as a candid, if small and time-bound, behavioural record. Aggregated across all 25 cycles:

| Model family | Audit runs | Findings raised | Rating range | In one line |
|---|--:|--:|---|---|
| **gpt** (5.2 / 5.3 / 5-codex / 5.5) | 26 | 99 | 7.6 – 9.8 | The workhorse — ran `go build`/`go test`, caught the material bugs, "carried the audit." |
| **glm** (5.1 / 5.2) | 6 | 37 | 7.9 – 9.4 | Strong newcomer; high volume, occasional false-positive inflation. |
| **claude-fable-5** | 2 | 24 | 8.4 – 9.2 | The only auditor that *ran the declared verification commands*; tends to under-severize. |
| **qwen** (3.7-max / plus) | 6 | 23 | 5.5 – 9.0 | Variable; strong on the backend and key-management batches. |
| **gemini** (2.5-pro, 3.1-pro, cli) | 24 | 19 | 2.1 – 10 | See below — the cautionary tale. |
| **deepseek-v4-pro** | 2 | 5 | 6.4 – 6.7 | Moderate depth. |
| **kimi-k26** | 1 | 4 | 5.4 | Inflated two severities, missed the material findings. |

The gemini line is the one that vindicates the whole design, and it's documented in-corpus as a tracked pattern (`FU-030-003`). Across **24 runs it raised 19 findings total** — and more to the point, it kept *missing the real ones*. The calibrator escalated it over four cycles from an observation to a formal recommendation: *"demote to advisory-only… unreliable on integration-gap class findings — RLS bypass, missing wire-up, convention drift."* In one safety audit it scored **2.1/10**, having *"detected none of the nine real findings and affirmatively mis-described the guard as enforcing precedence — a hallucinated false guarantee that is actively misleading for a safety audit."* Its measured catch-rate on that class hovered around **one in four**.

Sit with what that means. Had Sentinel run a single-family audit on that family, those findings would simply not exist — no test would have caught them, and the review that was supposed to would have returned a confident PASS. The blind spot was invisible from inside the family and obvious the moment a *different* family looked at the same diff. That is the entire argument for cross-family independence, and here it is as a measured fact rather than a principle.

## What we deliberately don't claim

This is not a model leaderboard. The sample is one project, over about two months, on specific model versions that will be stale within weeks; `gemini` had a genuine bright spot (a clean 10/10 catch on `CHARTER-31`), and a different corpus or a newer version could reorder the whole table. The honest reading isn't "family X is bad" — it's the **methodological** one: *any* single mind can be systematically blind, the blindness is uncheckable from inside, and diversity is what makes it visible. That claim survives whichever model is on top next month.

Nor is this a knock on Bun. Their single-family, split-context, oracle-backed design is *correct for a port*, and at a scale — a million assertions, six platforms, zero skipped tests — that most teams never reach. The distinction is only this: when the test suite can be the judge, isolate reviewers by context; when the judge is a human calling a judgment, isolate them by family, and keep the human directing the call. Everything in StrayMark's audit — including the [operator-provided identity fix](who-the-audit-thought-it-was) that made cross-family convergence un-fakeable — exists to protect that second case.

## If you've read this far

The portable question is about your own review process, human or automated. Ask what your reviewers would fail to catch *in a way nothing downstream would catch either* — the class of defect that has no test, no linter, no type to stop it. For that class, redundancy within one perspective buys you almost nothing; two of the same reviewer share the same blind spot. What buys you something is a *different kind* of reviewer, and a way to notice when they disagree. Bun could lean on a million assertions to be the judge. Most of what you ship can't. For that, the only oracle you have is a second mind that sees differently — so make sure the second mind is actually different, and that a person is still reading the disagreement.

---

*Data from the Sentinel project's `.straymark/audits/` — 25 audit cycles, 21 Charters, 68 auditor reports, 7 model families, 2026-05-05 → 2026-07-10 (read-only). Bun figures from [bun.com/blog/bun-in-rust](https://bun.com/blog/bun-in-rust). Related: [Who the audit thought it was](who-the-audit-thought-it-was).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*