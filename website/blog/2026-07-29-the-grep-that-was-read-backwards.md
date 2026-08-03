---
slug: the-grep-that-was-read-backwards
title: The grep that was read backwards
authors:
  - jose
tags:
  - straymark
  - audit
  - governance
  - verification
  - multi-model
date: 2026-07-29T00:00:00.000Z
description: A four-model audit cycle reviewed a Charter that added a new public method — the only method that made a feature work. Three of four auditors reported no critical or high findings. The feature had zero production callers, 3,376 passing tests, and did not work end to end. The defect was mechanically detectable with a single grep; one auditor ran that exact grep and read the result backwards. The post-mortem produced a one-rule addition to the audit prompt — and a more uncomfortable finding about the implementer's own verification, which could not produce a red result.
---

*A four-model audit cycle missed a defect that made the audited feature completely non-functional. Three of four auditors reported "no critical or high findings." The one who found it rated it Medium. The defect was mechanically detectable with a single grep — one auditor ran that exact grep, got the correct output, and read it backwards. The feature had 3,376 passing tests and did not work end to end, because the Charter added a new public method and nothing called it. When we asked the most thorough auditor what happened, its answer was more valuable than the finding itself: a named, reproducible failure mode that three of four auditors shared. This post is about that failure mode, the one-rule prompt fix it produced, and the second-order discovery that the implementer's own verification method was structurally incapable of producing a red result — a verification that summed passes and never checked failures, trusted seven times in a row.*

<!-- truncate -->

## The case

The project is a private adopter — we'll keep identifiers generic. A Charter implemented a break-glass elevation feature: an operator with temporary elevated permissions should be able to open records that their base role normally can't reach. The previous Charter had built the elevation-resolution machinery and left it unused. This Charter's own context section stated its purpose plainly: *"the elevation type is already written and tested, and today nobody constructs it. This Charter puts it into use."*

The Charter added an async resolver method — the only method in the codebase that consults elevation state — to the resolver interface. Tests were written. 3,376 tests passed. The audit ran.

Nothing calls the new method. All eight consumers still call the synchronous resolver, which resolves by role only and returns *none* for the operator role the feature exists to serve. An operator with an active elevation cannot open a single record. The feature has 3,376 passing tests and does not work.

Two greps tell the whole story:

```
$ grep -rn "resolver.Resolve\(" src/
Page1.razor:124    Page2.razor:66     Page3.razor:86
Page4.razor:120    Page5.razor:223    Page6.razor:347
Modal.razor:159    Controller.cs:311

$ grep -rn "ResolveAsync" src/
(no results)
```

Eight call sites, all routing through the old path. Zero through the new one. The async method exists, is correct, has tests — and is unreachable.

## What the four auditors did

| Auditor | Findings | Found it? |
|---|---|---|
| gpt-5-6-terra-high | 2 | yes — rated Medium (undercalibrated) |
| kimi-k3 | 15 | no |
| glm-5.2 | 8 | no — explicitly wrote "no critical or high findings" |
| qwen3-8-max-preview | 3 | no |

Three of four auditors detected *symptoms* and did not reach the cause:

- A DI-parity guard test failing deterministically — meaning the E2E fixture couldn't boot the feature. Found by two auditors. Reported as a config defect.
- A coverage gap: the expiry test verified at service level, not at the HTTP/cookie level the Charter declared. Found by one.
- The exact *shape* of the defect — "the caller decides the authorization value and the layer below accepts it" — identified precisely, in a different file. Found by one.

Every symptom was real. Every symptom was a thread that led to the same root. Nobody pulled.

## The post-mortem that mattered more than the finding

We asked kimi-k3 — the auditor with the most thorough report and the most evidence citations — what happened. Its answer is the most useful artifact of the whole cycle, because the failure is reproducible and has a name.

**1. It ran the right search and read the result backwards.**

> *"I ran a grep whose output showed only injection lines — the absence of the async call was visible there, and I read it as 'injection confirmed' instead of 'no calls confirmed.' I grepped for the presence of the wiring and read the result as confirmatory when it was disconfirmatory."*

Nothing failed except the reading of a negative result. The grep returned injection-site lines (the DI container wiring) and the auditor read them as proof the method was *used*, when they only proved it was *available*. Presence of wiring ≠ presence of calls.

**2. It verified the mechanism instead of the adoption.**

> *"I verified the mechanism (exists, is correct) instead of the adoption (is used)."*

It had read the method body — it filed a finding about a missing cache *inside the new async method*. The adjacent question — who calls this? — never got asked. This is the failure mode in its purest form: the auditor was *inside the right method*, verified its correctness, and never looked up to see if anything reached it.

**3. It treated prior-Charter context as narrative, not as falsifiable claims.**

> *"The prompt has 10,598 lines and I absorbed the material from the original AILOGs as background narrative, not converting each one into a falsifiable hypothesis ('this Charter must make X true — where?'). That's the process failure I can name precisely: I treated prior-Charter context as context, not as a list of claims-pending-to-be-made-true."*

The prompt *did* contain the information needed. The Charter states its own purpose is "put it into use." Verifying "put into use" means verifying the call sites changed. The claim was there; it was read as background, not as a checkable assertion.

**4. Its structural diagnosis.**

> *"Every layer was tested and the seam between layers was not — the resolver works (16 tests), the pages work (bUnit with the permission injected), the services reject (14 tests). Three of four auditors verified layers; the defect lived in the overload choice inside files that were not in the diff."*

This is the part that generalizes: auditing a diff is not enough when a Charter claims to put existing code into use. The eight files with the wrong call were modified in the *previous* Charter.

## Convergence measures the visible, not the severe

This is the second consecutive occurrence of a pattern worth naming. In a previous audit cycle, two of three auditors validated a guarantee by citing a test without opening its body — and the test was empty. Here, three auditors converged on the red guard test (visible, mechanical, easy to report) while zero of the three found the defect that made the feature useless.

Convergence among auditors is the signal StrayMark's audit cycle is designed to produce. But convergence measures what multiple minds *can see*, and what they can see is systematically biased toward the *checkable*: test failures, config errors, type mismatches. The defect that matters most — a capability nothing reaches — is invisible to every auditor that doesn't explicitly ask the question. Asking it requires no judgement; it requires a rule.

## The fix: one rule, no judgement required

We offered the auditor four candidate remedies. It picked this one without hesitation:

> *"(a), without a doubt, and it's not hindsight: the async resolver is a new public method; 'enumerate its callers' is a grep I already had in my hand that returns '0 in production' — mechanical High finding, no judgement."*

The rule, now part of the audit prompt as a mandatory step:

> **Enumerate callers of new public entry points.** For each public method, endpoint, or component the Charter ADDS, run a call-site search across production code (excluding tests) and state the count explicitly. Zero production callers is a High finding by default — the Charter added a capability that nothing reaches. When the count is non-zero, check that the callers are the intended ones — an existing overload or a legacy path may still be winning.

The rule is attractive precisely because it requires no judgement. It converts an invisible absence into an explicit count. "Zero callers" is a finding that writes itself.

Two secondary rules from the same post-mortem also shipped:

1. **Consolidated test seam check.** When a test is documented as "consolidated" into another, verify the replacement exercises the same seam, not merely the same unit. In this cycle, the auditor investigated exactly this and dropped it because the Charter's closing notes declared the coverage equivalent. It was not. The Charter's own closing notes switched off an auditor's line of inquiry — and the audited party writes those notes. That is a structural conflict of interest the audit must correct for.

2. **Red gate enumeration.** When a verification gate is red, enumerate what only that gate could have caught. The broken DI-parity test was reported as a config defect; nobody asked what it was protecting.

## The second-order finding: verification that cannot fail

The implementing agent declared "3,376 tests passing, 0 failures" in the AILOG, the PR body, and seven commit messages. A guard test had been failing deterministically since batch 4. The verification command was:

```bash
dotnet test ... | grep -E "Passed!|Failed!" | awk -F'Passed: *' '{s+=$2} END {print s}'
```

It summed the passed counts from all five test projects without checking whether any reported failures. A project in the red got summed and never surfaced.

This is arguably the more useful half of the case. An agent built a verification method that *could not produce a red result*, then trusted it seven times in a row. Self-verification never could have caught this. The audit did — eventually, through a different auditor, on a different finding, by reading the guard test output that the implementer's own pipeline had been silently swallowing.

The fix here is not in the audit prompt but in the AILOG template — the document the implementer writes. The "Tests pass" checkbox now requires declaring the exact command run, with explicit guidance: *a verification that cannot produce a negative result is not verification.* Summing pass counts without checking failure output is the canonical anti-pattern.

## What shipped

Released as [`fw-4.38.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.38.0) / [`cli-3.40.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.40.0), tracked in [#382](https://github.com/StrangeDaysTech/straymark/issues/382):

**Audit prompt v1.2** (EN + ES). Three additions:

- **New mandatory Step 3** — enumerate callers of new public entry points. Zero production callers = High, no judgement required. Non-zero: verify the callers are the intended ones.
- **Enhanced Step 2.6** — consolidated-test seam check: the Charter's own closing notes are a claim by the audited party, not evidence.
- **Enhanced Step 4** — red gate enumeration: what was the broken gate protecting?

**AILOG + Charter templates** (EN/ES/zh-CN). "Tests pass" checkbox now requires declaring the exact command run. A verification that cannot produce a negative result is not verification.

## The portable version

Ask your review process — human or automated — this question: *for each new public capability, does anyone check that something calls it?* Not that it exists, not that it's correct, not that it has tests. That something *calls* it. The answer is usually no, because the question feels too simple to ask. It is. That's the point. The simplest checks are the ones nobody writes down, and the ones nobody writes down are the ones that get skipped by every reviewer, human or machine, in every cycle, forever — until a Charter adds a feature that nothing reaches, three auditors report green, and the only thing that catches it is luck.

Write the check down. Make it mechanical. "Zero production callers" is a number, not a judgement. If your audit prompt, your code review checklist, or your CI gate doesn't produce that number for every new public entry point, you are running the same process that produced 3,376 passing tests on a feature that does not work.

---

*Case documented in [#382](https://github.com/StrangeDaysTech/straymark/issues/382) — four auditor reports, consolidated review, auditor post-mortem, and full source artifacts in the private adopter's `.straymark/audits/` directory. Shipped in [`fw-4.38.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.38.0) / [`cli-3.40.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.40.0). Related: [What a million assertions can't settle](what-a-million-assertions-cant-settle) (cross-family diversity as blind-spot insurance), [Who the audit thought it was](who-the-audit-thought-it-was) (audit attribution).*

*This document was produced with assistance from generative AI tools; all responsibility for the content rests with the human author.*
