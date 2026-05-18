---
slug: charters-and-the-external-audit-cycle
title: 'Charters as a first-class entity, and the external audit cycle'
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - charters
  - audit
  - cli
  - governance
date: 2026-05-06T00:00:00.000Z
description: From manual ritual in Sentinel to canonized CLI command
---
## 1. When discipline starts to feel like ceremony

By late April, Sentinel closed its fourth Charter of the month with a feeling that, in writing, sounds slightly pathetic: external audit was working — Copilot 9.25, Gemini 9.5, the drift script with zero false positives — but every close had me opening three files by hand, copying three prompts into three different windows, waiting for replies, pasting three calibration YAMLs back into the telemetry file, and eyeballing the hashes to make sure they matched. Forty minutes of copy-paste per Charter on top of work that took two hours. Discipline was working; the ritual was becoming a problem.

The `audit-skills-design.md` proposal, written on 3 May, named it without softening:

> *"If at every close the operator had to manually move data between files, throughput would collapse and discipline would stop being virtuous friction and turn into ceremony. Whatever can be automated reversibly should be automated; the documents are kept for ex-post human audit."*

This post covers three days — from 2 to 6 May 2026 — during which two parallel arcs finished at the same time. One: the Charter stopped being artisanal practice and became a first-class entity of the CLI. Two: the external audit cycle stopped being a manual ritual with ad-hoc prompts and became a canonical command (with a counterintuitive architectural decision behind it that deserves its own section).

---

## 2. What Sentinel was already doing, and what StrayMark didn't have

The 3 May proposal puts it in a sentence that's barely elegant but precisely accurate about what was happening:

> *"Sentinel has the skills, StrayMark doesn't."*

What Sentinel had: `sentinel/.claude/skills/plan-audit/` and `plan-audit-review/` — local skills that generated a prompt, calibrated the response on return, and merged it into telemetry. They worked. They had been validated across six cycles. But they were coupled to Sentinel-specific paths (`docs/plans/`, `internal/modules/`, `go vet`) and to the *"Plan"* vocabulary, which had already been renamed to Charter a week earlier.

What StrayMark didn't have: nothing equivalent. The framework, up to April, was documentation + skills + a CLI with `init`, `update`, `remove`. The unit that in Sentinel was called *Plan* — the bounded unit, declared *ex-ante*, audited *ex-post* — didn't exist as a CLI artifact. It existed as practice.

The arc of fw-4.4.0 (2 May) and of fw-4.7 → 4.9 (3-5 May) consists, almost verbatim, of porting what Sentinel did by hand into generic commands that any framework adopter can invoke. The migration table, copied from `cli-roadmap.md`, is honest about the origin:

| Sentinel artifact *(April 2026)* | StrayMark equivalent *(May 2026)* |
|---|---|
| `TEMPLATE.md` (v3) in `docs/plans/` | `dist/.straymark/templates/charter-template.md` |
| `scripts/check-plan-drift.sh` (145 bash lines) | `straymark charter drift` (Rust subcommand) |
| Local skill `plan-audit` | Generic skill `straymark-audit-prepare` |
| Dual reports in `audit/plans/05,06/{copilot,gemini,claude}.md` | Canonical output of `straymark charter audit` |
| Hand-edited telemetry YAML | `dist/.straymark/schemas/charter-telemetry.schema.v0.json` |

The fw-4.4.0 CHANGELOG frames it without metaphor: *"Crystallizes the Charter pattern — bounded, auditable units of work declared ex-ante and validated ex-post — that emerged from the 6-cycle Sentinel /plan-audit experiment."* The crystallization is the point. What in April was custom-with-a-bash-script, in May is a command with a validatable schema.

---

## 3. What happens when something becomes "first-class"

PR #65 (fw-4.4.0 / cli-3.6.0, 2 May) does three concrete things:

- It creates the `straymark charter new` command. Auto-increments the number (`NN-slug.md`), pre-populates the origin (`originating_ailogs` if it's born from a prior AILOG; `originating_spec` if it's born from a SpecKit `plan.md`), and supports the `--type X|S|M|L` flag to fix the size from the first moment.
- It introduces `charter-template.md`, ported from Sentinel's `TEMPLATE.md` v3. It carries embedded the six conventions that the April experiment was crystallizing cycle by cycle: Local/Production checks separation, effort in time (not story points), structured sub-sections, `R<N>` risk documentation, closure with post-merge reconciliation via AILOG, and auto-checklist drift.
- It ships `dist/.straymark/schemas/charter.schema.v0.json` — the schema marked `v0` on purpose, not `v1`, because the framework's principle #12 says no schema crystallizes without a second domain validating it. Sentinel is one domain. The second is missing.

PR #68 (3 May) does something that looks small but matters: the *atomic Charter closure pattern*, format v4. In Sentinel, the step *"update the Plan-doc post-merge if the AILOG documents divergences"* existed as a note in the TEMPLATE but had no systematic trigger. In practice, that meant when there was divergence between what was declared and what was delivered, the operator (me) relied on memory to reconcile the document — and memory failed. The drift stayed in the repo for days, sometimes weeks, until the next cycle caught it. Format v4 makes the reconciliation a mandatory step of closure, not a marginal note.

PR #69 (same day) closes one friction detail: manual numbering. Before, you had to decide whether the next Charter was 11 or 12; now `straymark charter new` reads the directory and proposes the next number. Pure UX. But it's the class of friction that, summed across twenty Charters, decides whether the framework gets used or abandoned.

The three PRs shipped in the same commit storm of a Sunday and a Monday. That cadence is deliberate: once the pattern crystallizes, UX details have to close *together*, not in separate bumps that leave the adopter with half the flow.

---

## 4. Three releases in a day

What came next — fw-4.7.0, fw-4.8.0, fw-4.9.0 — are consecutive releases of the external audit cycle. The `audit-skills-rollout.md` proposal records the pace without disguise:

> *"Phase 1 executed in 1 calendar day (5 sequential PRs on 3 May 2026), substantially faster than the 1.5-2 weeks focused estimate. The throughput is due to the design in `audit-skills-design.md` already having the decisions crystallized (D1/D2/D3) and the arborist heuristic with explicit graceful-degradation — there were no pending decisions during implementation. It serves as additional operational evidence for principle #6 (when the proposal is well-written, implementation is execution, not design)."*

The concrete external audit cycle consists of three chained commands:

1. `straymark charter audit prepare <CHARTER-NN>` — generates canonical prompts for external auditors from the closed Charter, its associated AILOG, and the diff of touched files. Output: three `.prompt.md` files in `audit/<CHARTER-NN>/{copilot,gemini,claude}.prompt.md`.
2. *(The human carries those prompts to their auditor of choice, receives replies, pastes them back.)*
3. `straymark charter audit collect <CHARTER-NN>` — takes the responses, validates each one against the audit schema, merges calibrations into the Charter's telemetry, and produces a summary with the convergence (or divergence) between auditors.

Decision D1 in the proposal is what holds everything else up:

> *"Skills delegate via `Bash(straymark charter audit *)` to the canonical implementation. Templates live only in `dist/.straymark/audit-prompts/`. Zero drift possible between skill and CLI."*

There's only one place where the prompts live, one single implementation of the flow, and skills (Claude, Cursor) invoke it via Bash. It's a humble pattern — the CLI is the single source — but it avoids what in any framework with two surfaces (skill + CLI) ends up being the chronic problem: that the skill and the CLI say different things depending on who updated which last.

---

## 5. Why the CLI orchestrates but doesn't invoke APIs

This is the strangest decision of May, and the one that took me the most work to make. It's documented literally in `cli-roadmap.md` §0 as *"architectural decision A1"*:

> *"A1 (Phase 3): orchestration-only, no HTTP API clients in v0. Roadmap §5.4 originally suggested 'support OpenAI/Google/Anthropic in v0' with API key handling. The implemented reality is that the CLI prepares prompts, validates outputs against schema, and integrates with telemetry — but does NOT invoke APIs. The operator pastes the prompts into their auditor of choice manually."*

The reasoning, also literal:

> *"Implementing 3 HTTP clients is 1-2 weeks + perpetual maintenance when the APIs change (premature for an experimental v0); the human-in-the-loop pattern matches Sentinel's `/plan-audit`; complies with principle #10 ('it is not an LLM gateway'); closes RFC #82 by design. HTTP clients reopen in v1 when a real adopter justifies it with data."*

Three arguments, each with its own weight.

**The first is pragmatic.** Three HTTP clients — OpenAI, Anthropic, Google — are between one and two weeks of implementation, plus permanent maintenance every time any of the three changes something. For a command whose manual form was already working empirically in Sentinel, that's an absurd cost. It's engineering to solve a problem no adopter had asked to solve.

**The second is about continuity.** What Sentinel validated in April was precisely the human-in-the-loop pattern: the operator pastes the prompt into Copilot/Gemini/Claude *by hand*, reads the response, pastes it back. The six-Plan experiment that gave rise to this whole arc *never* invoked APIs. If the CLI now invoked APIs on its own, it would be replacing a validated pattern with an unvalidated one, with no empirical reason.

**The third is about identity.** StrayMark is not an *LLM gateway*. Framework principle #10 says it explicitly. There are dozens of products that *are* — LangChain, LiteLLM, LiteLLM Proxy, OpenRouter, every wrapper — and they're useful for what they do. StrayMark does something else: it structures discipline around the work done *with* those models, whichever model that is. Whether the CLI invokes or doesn't invoke APIs changes what the framework *is*; keeping it *orchestration-only* keeps the identity clean.

The part of the paragraph I care about most: *"HTTP clients reopen in v1 when a real adopter justifies it with data."* The decision wasn't closed by dogma; it was deferred behind an evidence threshold. If in six months an adopter shows that the human-in-the-loop flow breaks their throughput, the HTTP clients land. Until then, the forty seconds of manual friction it takes to paste the prompt into another window is trivial compared to the cost of maintaining three live SDKs.

---

## 6. What the framework decided *not* to automate

It's worth closing with the other face of the same decision. The `audit-skills-design.md` proposal says:

> *"Whatever can be automated reversibly should be automated; the documents are kept for ex-post human audit."*

The first half explains May's three releases: prompts generated automatically, calibrations merged automatically, drift detected automatically. The second half — *"the documents are kept for ex-post human audit"* — explains what was chosen to stay manual on purpose.

What the framework does *not* automate:

- **The operator's reading of the AILOG before closing the Charter.** The `straymark charter audit prepare` command generates the prompts, but the operator *has* to open the AILOG and read it. If we automated that — an agent that reads, summarizes, decides — we'd lose the moment of human judgment that justifies the Charter existing at all.
- **The decision to accept or reject the external audit.** The CLI merges calibrations into the telemetry, but doesn't act on them. If an auditor scores the Charter a 4, the Charter doesn't auto-reopen: the operator decides whether to reopen, to argue, or to declare it *acceptable-with-known-debt*. That decision is the discipline; automating it would destroy it.
- **The invocation of the models.** As argued above: the pause where the operator copies the prompt is exactly where they decide which model to use, which prompt to tweak, whether the question as phrased seems fair. That pause is feature, not bug.

The criterion running through all three is one: *automate what's reversible and mechanical; preserve as manual what's judgment*. Forty seconds of copy-paste aren't too many if they buy keeping the judgment human.

---

## 7. Closing

What I took from the process, in four claims:

1. **A practice that works empirically doesn't need to be reinvented to be canonized.** Sentinel did the validation work in April; May only ported it into the framework. The crystallization is transfer, not design-from-scratch.

2. **When the proposal is well-written, implementation is execution.** Three releases in a day are only possible when all the decisions were closed before code was touched. Principle #6 says it more prosaically, but this is what it means.

3. **Not everything automatable should be automated.** Decision A1 — the CLI orchestrates but doesn't invoke APIs — is the technical version of a philosophical decision: human-in-the-loop isn't legacy to get rid of, it's a feature that holds the discipline up.

4. **A framework's identity is defined as much by what it does as by what it doesn't.** *"It is not an LLM gateway"* is one of the few principle #10 sentences worth remembering verbatim. What StrayMark does, it does well because there are many things it refuses to do.

Next, in the following post: the methodological episode that connects with the very start of this blog — *Issue #113: Charters invisible to the agents* — where the framework discovers that creating commands and schemas isn't enough if the repo's *surface* doesn't speak to the agent. It's the natural counterweight to this post: here we talk about what got canonized; there we talk about what didn't get seen.

---

*Anchors: PRs [#65](https://github.com/StrangeDaysTech/straymark/pull/65) · [#68](https://github.com/StrangeDaysTech/straymark/pull/68) · [#69](https://github.com/StrangeDaysTech/straymark/pull/69) (Charters first-class). Proposals [`audit-skills-design.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-design.md) · [`audit-skills-rollout.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-rollout.md) · [`audit-cli-flow.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-04-audit-cli-flow.md) · [`cli-roadmap.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-cli-roadmap.md) (decision A1 §0). Releases fw-4.4.0 → fw-4.9.0.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
