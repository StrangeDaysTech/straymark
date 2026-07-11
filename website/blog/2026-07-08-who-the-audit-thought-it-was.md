---
slug: who-the-audit-thought-it-was
title: Who the audit thought it was
authors:
  - jose
tags:
  - straymark
  - audit
  - attribution
  - framework
  - governance
  - sentinel
date: 2026-07-08T00:00:00.000Z
description: StrayMark's audit is only worth anything if independent model families converge on a finding by themselves — agreement is the signal. A run of releases hardened that guarantee against three ways to fake it. The freshest is the sharpest — a router CLI injects its own product name, so an auditor stamped its report "qwen-code" even after the operator had switched the backend model. The convergence math was being fed a lie about who did the work.
---
*The value of StrayMark's external audit isn't that a model found a defect. It's that **independent model families converged** on it — a finding is signal only when auditors from genuinely different backends reach it separately, and the review step dedups and rates on that basis. Which means the whole apparatus rests on one quiet assumption: that the recorded identity of each auditor is true. A run of recent releases found three ways that assumption breaks, and closed them. The freshest is the most unsettling, because nothing looked wrong: an auditor confidently stamped its own report with the name of the CLI wrapper it ran inside, not the model that actually did the reasoning.*

<!-- truncate -->

> *Router CLIs inject a product identity by system prompt, so auditors would write `auditor: qwen-code` even after the operator confirmed a different backend was selected via `/model` — corrupting attribution and faking cross-family agreement.*

This is a coda, from a different corner of the framework than the [Baton trilogy](what-the-author-already-knew) that precedes it — but it rhymes with it. Baton's arc was about convergence in service of *routing*; this one is about convergence in service of *trust*. Both come down to the same discipline: a signal is only worth what its provenance is worth. StrayMark's [external audit cycle](charters-and-the-external-audit-cycle) has been a first-class command since May; these releases are about making its central guarantee un-fakeable.

## 1. Why identity is load-bearing

The audit works by disagreement and agreement. You run the same charter past auditors on different model families; a defect that only one family raises is weak signal, a defect several families raise independently is strong. The consolidation step (`straymark-audit-review`) dedups findings and rates auditors on that convergence. For any of that to mean something, two things must be true: an auditor must not read another auditor's report before writing its own, and each report must be **honestly attributed** to the family that produced it.

Break either, and the convergence math doesn't just degrade — it *lies*. Two "independent" runs that are secretly the same model look like agreement. Two genuinely different backends both stamped with the same wrapper name collapse into one in the dedup. The rating rewards or punishes the wrong family. The audit keeps producing confident numbers; they've just stopped meaning what they say.

Three releases, three distinct ways to fake convergence.

## 2. Reading the sibling's homework (fw-4.27.0, the v1 hardening)

The first and most obvious: an auditor peeking at another auditor's findings before writing its own. That was closed earlier, in the v1 audit-prompt hardening ([#261](https://github.com/StrangeDaysTech/straymark/issues/261)) — auditor independence plus a contamination guard. It's the baseline: convergence is only evidence if it's arrived at blind. Worth naming here because the two newer fixes are the same principle applied to subtler leaks.

## 3. Mistaking the mock for the territory (fw-4.32.0)

The v1.1 audit-prompt pass ([`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0), [#303](https://github.com/StrangeDaysTech/straymark/issues/303)/[#306](https://github.com/StrangeDaysTech/straymark/issues/306)) tightened *what* an auditor is allowed to trust as reality. Two additive rules:

- **Audit object vs. truth oracle.** The prompt now separates *where you report defects* (the audit object — the code in the git range) from *what you may read to validate them* (a truth oracle — including things outside the range). A client-auditing run must cross-check API/IPC/contract calls against the **real server-side definition**, even when the server is outside the diff. A client↔server contract mismatch is an auditable defect of the client, and **green client-side tests do not absolve it** — the mocks encode the client's own assumption, so a passing test is just the assumption agreeing with itself. (If that failure mode sounds familiar, it's exactly the #304 drift the [Baton coherence bridge](what-the-spec-path-only-proved-existed) was built to catch, seen from the audit side.)
- **Verification fidelity.** For every "verified / resolved / done" claim, the auditor now asks *against which reality* it was checked — the condition that actually matters, versus a convenient proxy (a local test, a mock, the document's own assertion) — and opens the artifact instead of trusting a downstream summary. The `real_debt` finding category was also repointed at the first-class follow-ups registry with TDE promotion, so a confirmed defect lands somewhere durable instead of a loose post-audit note.

This is the audit learning to distrust convenient evidence. The next two releases make it distrust something it never thought to question: itself.

## 4. Who did the verifying (fw-4.33.0 and fw-4.34.0)

Here's the bug, and it's a good one because everything about it looked correct.

Router CLIs — Qwen Code, Gemini CLI, and their kin — inject a **product identity** through the system prompt. So when an auditor filled in its report frontmatter, it self-detected and wrote `auditor: qwen-code` — the *wrapper's* name — **even after the operator had switched the actual backend model via `/model`.** The report was well-formed. The field was populated. It was simply naming the CLI, not the mind.

That does two bad things at once. It **corrupts attribution** — you can no longer tell which model family actually produced the finding. And it can **fake cross-family agreement**: two runs on genuinely different backends both stamp themselves with the same CLI product name, or one run stamps the wrong family, and the convergence-and-dedup math the whole audit rests on gets poisoned at the source. The auditor rating — the thing that's supposed to tell you which families are reliable — rates a phantom.

[`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) makes the **operator-provided identity authoritative** for the auditor. Step 2 of `straymark-audit-execute` was rewritten to take the identity from an optional second argument — `/straymark-audit-execute <CHARTER-ID> <AUDITOR-SLUG>` — or an in-chat statement, to **forbid substituting the CLI product name**, and to add a mandatory **post-write guard** that verifies the `auditor:` frontmatter and the report header both match the provided slug. Self-detection survives only as a last-resort fallback when the operator provides nothing.

[`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0) applies the identical fix to the **calibrator** — the identity on the consolidation step (`straymark-audit-review`), the `calibrator:` / `**Reviewer:**` fields — with the same optional second argument (`/straymark-audit-review <CHARTER-ID> <CALIBRATOR-SLUG>`) and the same post-write guard. Because the reviewer who *judges* the convergence has to be as honestly attributed as the auditors who produced it; a mislabeled referee is no better than a mislabeled player.

Both fixes shipped across all four runtime copies — the `.agent` workflow and the `.claude` / `.gemini` / `.codex` skills — because the bug lives wherever the prompt runs, and a fix in one copy is a fix that fails the moment the operator switches CLIs.

## 5. What this deliberately isn't

It isn't a **detector.** The framework can't reach into a CLI and read which backend is really selected; the operator can, and the fix makes the operator the authority instead of pretending the tool can tell. The guard verifies *consistency* — that what got written matches what the operator declared — not ground truth about the silicon.

It isn't **auto-detection dressed up.** Self-detection stays only as a fallback, and it's explicitly the least-trusted path. The whole move is to demote the tool's guess about itself below the human's statement, exactly as the [Baton work-verb graduation](what-the-author-already-knew) demoted a title-scan below a declared verb. Same lesson, different corner: the authoritative signal is the one the operator states, not the one the machine infers about its own identity.

## 6. If you've read this far

The portable question is about provenance in any system that aggregates independent judgments — code review sign-offs, red-team findings, model-ensemble votes, second opinions. The aggregate is only as trustworthy as the *identity* attached to each input, and identity is exactly the field nobody double-checks, because it's populated automatically and it always looks plausible. `auditor: qwen-code` is a perfectly well-formed value. It's just answering "what tool ran this" when the math needed "what mind produced this," and those diverged the instant a wrapper let you swap the backend underneath it. The audit prompt spent releases learning to trust *what* was verified. These two make it trust *who* did the verifying. Go find the identity field in your own pipeline that gets filled in for free — and ask whether it names the thing you're actually counting on, or just the thing that happened to be holding the pen.

---

*StrayMark Framework [`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0) → [`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) → [`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0), building on the v1 hardening in fw-4.27.0. Issues [#261](https://github.com/StrangeDaysTech/straymark/issues/261) · [#303](https://github.com/StrangeDaysTech/straymark/issues/303) · [#306](https://github.com/StrangeDaysTech/straymark/issues/306). Related: [Charters as a first-class entity, and the external audit cycle](charters-and-the-external-audit-cycle) · [What the author already knew](what-the-author-already-knew).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*