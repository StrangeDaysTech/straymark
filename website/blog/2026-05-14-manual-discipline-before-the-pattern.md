---
slug: manual-discipline-before-the-pattern
title: Manual discipline before the pattern
authors:
  - jose
tags:
  - straymark
  - charters
  - speckit
  - discipline
  - governance
date: 2026-05-14T00:00:00.000Z
description: >-
  Twelve learnings in an Issue, three gates in a PR, and a Charter that closed
  cleanly before the name existed
---
*Seven consecutive Charters on a one-month-old plan in Sentinel, executing CHARTER-18 by hand with the feeling that any false step would bury six Charters of work. Five hours to canonize the manual discipline as upstream governance — without naming the pattern yet.*

<!-- truncate -->

## 1. The right question

[Issue #150](https://github.com/StrangeDaysTech/straymark/issues/150), opened on 14 May at 16:59 UTC, wasn't looking for a *yes/no*. What the operator was asking was for the question to be reframed:

> *"The real question the bridge doc should address is how, not whether: what discipline ensures spec stays in sync with code during multi-Charter execution without the regeneration step lying about the parts that already shipped."*

The discipline already existed. It was being applied in Sentinel at that very moment, on `CHARTER-18`, by hand, with the feeling that any false step would bury the work of six previous Charters. What was missing — what the Issue documented with uncomfortable precision — was the name. The *when*. The rules a future adopter could follow without going through the pain of inventing them.

This post covers the five hours between that Issue and the PR that canonized the discipline as upstream governance, the 12 empirical learnings that justified the canonization, and the concrete operational episode — Sentinel's `CHARTER-18` — that exercised the pattern before the pattern existed. The next post will cover what came 27 hours later: the name.

---

## 2. Seven Charters on the same plan

Context, in one line: Sentinel executed `specs/002-commshub/plan.md` — written in commit `be29421` on 21 April 2026 — through **seven consecutive Charters** (`CHARTER-07` through `CHARTER-17`), one calendar month, without refreshing the plan once.

The original plan was good. It was carefully written, with four user stories (US1-US4) detailed, the data-model declared, the contracts sketched. But the plan was a snapshot of how the operator imagined, back in April, the code would behave. Execution discovered things the plan couldn't have anticipated:

- Infrastructure patterns that appeared *during* execution and turned out to be reusable (a namespace in `core.processed_events`, a `withRLS` helper, PL/pgSQL triggers for invariants).
- Code gaps the plan assumed resolved (`AnomalyDetector` *half-built*, missing idempotency dedup, `Recipient.TenantID` schema gap).
- Charter/audit patterns that crystallized over the month (cross-family audit cycle, HTTP layer test coverage gap, cursor pagination with strict-greater-than).

When the time came to plan `CHARTER-18` for US5, the plan was a month old and the code had a month of empirical learning on top. The two weren't in sync. And `SPECKIT-CHARTER-BRIDGE.md` — the framework document that would normally answer *"what do I do here?"* — was silent on the case. The rule *"how spec stays in sync with code during multi-Charter execution"* didn't exist yet.

---

## 3. The twelve empirical learnings

Issue #150 enumerates the twelve learnings that had accumulated without reflection in the plan. The literal table, grouped by type, is worth keeping in view:

| # | Type | Learning |
|---|---|---|
| 1 | Infra pattern | `core.processed_events` with namespace `module='commshub'` is reusable — plan describes its own table. |
| 2 | Infra pattern | `EnvSecretLoader` + Secret Manager projection (PRs #70/71) — plan describes direct access. |
| 3 | Infra pattern | `withRLS` helper as single-source-of-truth for tenant isolation — plan doesn't anchor it. |
| 4 | Infra pattern | FR-005 PL/pgSQL triggers as invariant enforcement — plan describes app-level checks. |
| 5 | Code gap | `AnomalyDetector` *half-built* — plan implies functional. |
| 6 | Code gap | Idempotency dedup at `delivery_log` layer missing — plan implies coordinated. |
| 7 | Code gap | `Recipient.TenantID` schema gap — plan references `tenant_id` as if it existed. |
| 8 | Code gap | Per-tier rate limit overrides absent — plan says *"3-tier hierarchy"* without mentioning US1 implemented a uniform tier. |
| 9 | Charter pattern | Cross-family external audit cycle mandatory (6 consecutive) — plan silent. |
| 10 | Charter pattern | HTTP layer test coverage gap — plan silent on test layering. |
| 11 | Charter pattern | Cursor pagination tuple with strict-greater-than — plan silent. |
| 12 | Charter pattern | Dispatcher interface-stable swap pattern — plan silent. |

Four infrastructure patterns discovered or refined during execution. Four code gaps the plan didn't anticipate. Four Charter/audit patterns crystallized on the fly. Twelve pieces, in a single month, all real. The plan didn't have them. And the next Charter — `CHARTER-18` — was going to read the plan as if they didn't exist.

The estimate the operator documented in the Issue is honest:

> *"Estimated probability of ≥1 critical/high finding from stale-premise inheritance: ~50%."*

Coin flip, in one word. Half of Charters started on a stale plan end with a critical finding a later audit cycle has to remediate atomically pre-close. And the other half escape only by luck.

---

## 4. Alternative 4: the discipline applied by hand

What happened inside Sentinel, that same day, before the discipline had a name: the operator evaluated five concrete alternatives and chose the fourth. Sentinel's private AIDEC — `AIDEC-2026-05-14-001-speckit-plan-scope-limited-us5-refresh.md` — records the analysis without softening. I paraphrase it here because the detailed content lives in the private repo, but the structure of the reasoning is what matters:

- **Alternative 1 — *"Read it as before"*.** Keep the plan stale and read it as the previous six times. Cost: pay the debt in a later audit cycle, ~50% probability of critical finding.
- **Alternative 2 — Regenerate everything with `/speckit-plan`.** Risk: overwriting assertions about US1-US4 that the real code *doesn't* satisfy; the regenerator doesn't know the current state.
- **Alternative 3 — Refresh in another PR without restrictions.** Same risk as Alt 2 plus review friction.
- **Alternative 4 — Scope-limited refresh + 3 gates.** The chosen one.
- **Alternative 5 — Defer CHARTER-18, do a refactor Charter first.** High calendar cost, marginal benefit over Alt 4.

The fourth consisted of three concrete mechanisms:

**First mechanism — *scope-limited prompt*.** Run `/speckit-plan` with an explicitly restrictive *prompt*: regenerate only Phase 7+8 corresponding to US5; leave US1-US4 byte for byte as they are, marked with `<!-- LOCKED: prior US shipped -->` comments the agent must respect. Without that explicit restriction, SpecKit's regeneration is destructive: it rewrites the whole plan, overwriting descriptions the real code already contradicts.

**Second mechanism — *Gate (a): validation against code reality*.** An ad-hoc script, ~30 lines of bash + grep, that diffs each non-US5 entity in `data-model.md` against the real schema in `db/migrations/*.sql`, and each non-US5 endpoint in `contracts/*.md` against the actual handler signatures in `internal/modules/commshub/handler_*.go`. Any divergence blocks merge. The script exists in Sentinel's repo; it isn't elegant, it's operational. What the framework would canonize later as gate (a) was already there, as artisanal bash.

**Third mechanism — *Gate (b): granular hunk-by-hunk review*.** Review `git diff` file by file, hunk by hunk, accepting no changes to *locked* sections without justification. The operator explicitly notes the expected friction in R1 of the AIDEC: if the agent regenerates more hunks than necessary, the human reviewer tires and the discipline collapses. Mitigation: hunks over 50 lines trigger P1 review.

**Fourth mechanism — *Gate (c): two-PR split*.** The refresh is an independent PR, reviewed against **current code**. The Charter-fill that follows is another PR, reviewed against the already-refreshed plan. Don't mix the two reviews — because if they're mixed, the refresh's debt hides behind the new work.

Four mechanisms. Applied manually, with no upstream documentation backing them. A single adopter (me), a single session, and the feeling the whole time of solving a case no one had solved before.

---

## 5. The result

`CHARTER-18` closed the same day. The Charter Telemetry metrics, recorded literally in the closure YAML:

- `estimation_drift_factor: 1.0` — the Charter finished exactly in the estimated hour range.
- `pre_work.items_discovered_during_planning: 0` — no unexpected items appeared during planning.
- `overall_satisfaction: 5/5` — the operator's best subjective calibration in the entire chain.

More important than the metrics: `CHARTER-18` was the **first Charter in the chain of seven to close without a mid-flight remediation Charter**. The previous six had each needed, at least one, an atomic pre-close remediation to cover divergences between what was declared and what was delivered. CHARTER-18 didn't. The operator's closing statement, cited literally from the later `CHARTER-CHAIN-EVOLUTION.md`:

> *"The SpecKit refresh from PR #76 eliminated most ambiguity that drove drift in prior Charters. No mid-flight remediation Charter required — the EC1..EC15 empirical-corrections inventory in research.md absorbed what would have been pre-execution risk into in-execution awareness."*

The manual discipline worked. It didn't just close the Charter cleanly; it changed the nature of the risk. What used to be pre-execution risk (discovering divergences during Charter execution, remediating atomically) became pre-planning awareness (the `EC1..EC15` inventory enumerates the empirical corrections before declaring the Charter). The risk isn't eliminated; it's moved to a moment where it's manageable.

---

## 6. Four hours and forty minutes

The chronology is worth keeping precise, because it's what the blog comes to record:

- **14 May, 16:59 UTC** — Issue #150 opened. Twelve learnings enumerated, three gates proposed, probability analysis published.
- **14 May, 21:39 UTC** — PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152) merged. `fw-4.14.3` shipped. *"Spec maintenance during multi-Charter execution"* added as a new section of `SPECKIT-CHARTER-BRIDGE.md` with the three gates canonized literally as framework governance. **Four hours and forty minutes** between the question and the upstream answer.

What PR #152 documented as governance is exactly what the private AIDEC had applied by hand hours earlier. The three gates with the same names: validation against code reality (gate a), granular hunk-by-hunk review (gate b), two-PR split (gate c). Plus four heuristics of *when to refresh*: ≥3 closed Charters on the same plan, ≥4 weeks + ≥2 Charters, `R<N>(new)` count > 6, target US touches refined infra. Plus an explicit note on why NOT to re-run `/speckit-tasks` (because it destroys the `[X]` markers and the `*CHARTER-NN:* <sha>` annotations that form the historical trace). Plus a roadmap note mentioning a future `straymark spec-drift` CLI that would mechanize gate (a).

Four hours. It's no exaggeration to say the framework wrote the guidance while the operator still had `git diff` open in another window. The discipline and its canonization were practically simultaneous. And that's worth recording because it's the reverse of the order the academic governance apparatus usually assumes: theory first, then practice. Here it went the other way. Practice first, while it hurt. Theory after, while the pain was still fresh.

---

## 7. Closing

What I took from the process, in four claims:

1. **The pattern is born of the discipline, not the other way around.** The three gates of `fw-4.14.3` weren't designed in the abstract; they were *extracted* from a concrete operational episode, hours after being applied by hand. Governance is post-empirical when the framework respects the order.

2. **Explicit probability is discipline.** *"~50% probability of critical finding from stale-premise inheritance"* isn't Bayesian precision; it's a public claim that forces the decision to be justified. Without that number, the question *"should we refresh?"* gets decided by intuition. With it, it gets decided by risk arithmetic.

3. **Twelve learnings in thirty days is the actual cadence.** It's not noise capturable with *"let me read it later"*. It's empirical density the original plan couldn't have anticipated, and that destroys any long-range planning assumption. The cadence of agent-assisted development is this.

4. **Four hours is not a record; it's a property of the process.** When the discipline is already being applied in one domain, canonizing it upstream is execution, not design. Same as we saw in Post 4 about the external audit cycle (*"five PRs in a day"*), same in Post 6 about the rebrand (*"forty-three minutes"*). The framework didn't invent the discipline; it extracted it and gave it a name.

Next, in the following post, the last episode of the arc this blog came to tell: *"Pattern 1 + Pattern 2 — chain evolution"* (`H-12`). Twenty-seven hours after PR #152, the three governance gates crystallized as two named meta-patterns — *pre-declare refresh* and *amend-on-emergence* — with a telemetry schema and CLI helpers. It's the closing of the arc Post 1 opened from ahead.

---

*Anchors: [Issue #150](https://github.com/StrangeDaysTech/straymark/issues/150) (14 May 16:59 UTC). PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152) — `fw-4.14.3` (14 May 21:39 UTC). Canonized section: `SPECKIT-CHARTER-BRIDGE.md §"Spec maintenance during multi-Charter execution"`. CHARTER-18 metrics reported in `CHARTER-CHAIN-EVOLUTION.md` (fw-4.16.0). Private source paraphrased per `GUIA-EDITORIAL.md §7`: `AIDEC-2026-05-14-001-speckit-plan-scope-limited-us5-refresh.md` in Sentinel's repo.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
