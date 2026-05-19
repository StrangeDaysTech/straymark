---
slug: opening-the-framework
title: Opening the framework
authors:
  - jose
tags:
  - straymark
  - i18n
  - cla
  - governance
  - opening
date: 2026-05-15T00:00:00.000Z
description: Seventy-two hours of gestures that only make sense together
---
*Two gestures in seventy-two hours: closing the last 5% of i18n parity (20/20/20 governance docs across EN + ES + zh-CN) and publishing the CLA badge in the README. Small separately; together, the moment the framework stops talking to itself and starts talking to whoever wants to read it.*

<!-- truncate -->

## 1. Two gestures in seventy-two hours

On 13 May 2026, at 06:20 UTC, `fw-4.13.4` merged. It closed two small i18n coverage gaps — the ISO-25010 reference document that was only in English, and the Charter template that was missing in Simplified Chinese. Twenty-two canonical framework files reached structural EN/ES/zh-CN parity.

On 15 May, at 00:14 UTC, sixty-five hours later, PR [#154](https://github.com/StrangeDaysTech/straymark/pull/154) merged. It added a badge to the README — the CLA Assistant one, that small visible square that tells the visitor the repo accepts contributions under a Contributor License Agreement. The CLA policy had existed for months already in `CONTRIBUTING.md`; the only thing that changed was that it now appears in the README at first glance.

This post covers the two gestures. Seen separately, they're small — one completes the last 5% of a technical job, the other publishes something that was already there. Seen together, they're the moment the framework decides to present itself to the outside. The piece an open-source project does when it stops talking to itself and starts talking to whoever wants to read it.

---

## 2. The last 5% of i18n coverage

`fw-4.13.4` isn't a big release. PR [#144](https://github.com/StrangeDaysTech/straymark/pull/144) has 511 addition lines and almost all are translations. But the PR summary says something worth recording:

> *"Brings governance docs to full 20/20/20 parity across EN + ES + zh-CN."*

Twenty over twenty over twenty. Twenty-two canonical files in `dist/.straymark/00-governance/` (the count grew later; in May it was twenty), all with their counterpart in `i18n/es/` and `i18n/zh-CN/`. Full parity, not approximate.

The two files that closed parity are specific:

- `ISO-25010-2023-REFERENCE.md` — the normative reference to the international software quality standard, cited from framework principles. Up to `fw-4.13.4` it existed only in English. The release added `i18n/es/ISO-25010-2023-REFERENCE.md` and `i18n/zh-CN/ISO-25010-2023-REFERENCE.md`, 150 lines each.
- `charter-template.md` in zh-CN. The Charter template was already translated into Spanish; Simplified Chinese was missing, 211 lines.

It's not glamorous work. It's the piece that closes a frame. But it matters for two things. First: until that release, a Chinese adopter cloning the framework was going to find essential templates in English only, which contradicts the promise that the framework is available in three languages. Second: the blog already documented in Post 8 that the framework's i18n coverage was nearly complete, with two minor gaps remaining as recorded debt. `fw-4.13.4` closed that debt. What looked like a side note in a post about the audit-prompt outlier turned out to be a release of its own, with its own CHANGELOG entry.

---

## 3. What gets translated and what doesn't

There's an operational rule that PR #144 codifies explicitly and that's worth recording, because it's the framework's clearest rule on i18n:

> *"LLM-processed assets (skills, workflows, schemas) stay EN-only; human-primary artifacts get translated."*

That is: what AI agents read — skills, workflows in `.claude/`, `.gemini/`, `.agent/`, schemas in `dist/.straymark/schemas/`, prompts in `dist/.straymark/audit-prompts/` — lives in English only. What humans read — documentation, principles, contribution guides, templates with human-facing copy — gets translated into the three languages.

It's the same principle Post 13 documented as the intellectual ancestor of the universal `AGENTS.md`: agents are a single technical audience that doesn't benefit from translation. What's recordable here is that the rule, in May, is written into a PR as an operational criterion, not as an intuition. Technical proposals cite it; reviewers use it to decide whether a new file goes into `i18n/` or not.

One additional detail from PR #144 is worth recording literally:

> *"docs/contributors/TRANSLATION-GUIDE.md gap was left intentional — its target audience already reads English to read the guide."*

The translation guide stayed intentionally in English because its audience — contributors translating the framework into other languages — necessarily already reads English. It's operational honesty: 20/20/20 parity isn't 22/22/22 for every repo file. It's 20/20/20 for files whose translation adds user value. The difference between total coverage and applicable coverage.

---

## 4. The badge that introduced nothing new

PR #154 is the pedagogical opposite of `fw-4.13.4`. Where `fw-4.13.4` closes technical work, PR #154 publishes something that was already there. Its 12 lines of change add exactly this to the README (in English, Spanish, and Chinese):

```markdown
[![CLA assistant](https://cla-assistant.io/readme/badge/StrangeDaysTech/straymark)](https://cla-assistant.io/StrangeDaysTech/straymark)
```

A badge. A small image that appears above the README, next to the other project badges (license, version, downloads). Visually unremarkable.

But the badge isn't decoration. It's a signal. And the signal says something concrete: *"this repo accepts external contributions, and here are the rules"*. Any visitor who opens the README can click the badge and land at CLA Assistant — the service that automatically comments on any contributor's first pull request, asking them to sign the Contributor License Agreement. One signature covers all future contributions to the project.

What matters is that the CLA, the `CONTRIBUTING.md`, the full review policy, all of that existed before PR #154. It was coded, alive, operational. The framework already accepted external contributions — it's just that the casual repo visitor had to open `CONTRIBUTING.md` to find out how it worked.

PR #154 makes a distinction worth naming: **having the policy** and **publishing the policy** are two different things. The first is internal discipline; the second is a public invitation. Until 15 May, the framework had the policy. From 15 May, it publishes it.

It's an editorial change, not an architectural one. But the day an open-source project decides to put the CLA Assistant badge above the README is the day it decides to formally present itself as adoptable by third parties. It's the moment the framework stops being a one-developer-and-its-adopter (Sentinel) project and becomes a project that *accepts* having more adopters, more contributors, more hands.

---

## 5. When a framework feels ready to be seen

These two milestones, read together, record a specific moment in a young framework's life: the moment it feels ready to be seen by people who didn't write it.

`fw-4.13.4` closes the technical promise. If the framework says it's bilingual Spanish-English with experimental Simplified Chinese support, the twenty-two canonical files must be in all three languages. Not nineteen out of twenty. Not twenty out of twenty-one with two pending. Twenty over twenty over twenty. Full parity is what makes the promise credible when the visitor checks.

PR #154 closes the social promise. If the framework says it accepts external contributions, the CLA badge in the README is the industry-standard way of saying so. Without the badge, the openness exists but you have to discover it; with the badge, it's signposted. The visitor's first click finds the path.

It's worth naming the asymmetry of the two gestures. Closing the last 5% of technical work costs exactly that — a dedicated release, two translated files, a CHANGELOG entry. Publishing a preexisting policy costs twelve lines of Markdown. The cheap one is sometimes the operationally most significant gesture, because it changes how the outside world perceives the project without changing anything about the project itself.

It's the "opening the framework" version of the pattern Post 5 documented as *structural visibility*: what exists but isn't named, doesn't exist for the agent. Here: what exists but isn't published, doesn't exist for the external adopter. Seventy-two hours in May, two small milestones — one technical, one editorial — and the framework crosses the line between *"it works but whoever wrote it knows it"* and *"it works and anyone can see it"*.

---

## 6. Symbolic closing of the arc

With this post, the blog covers the last pending milestones from the second batch that `RETOMAR-AQUI.md §5` listed after the closing of the main H-01 → H-13 arc. The four identified candidates — `H-?-C` (validate + schemas), `H-?-A` (universal AGENTS.md), `H-?-B` (full i18n coverage), `H-?-E` (CLA badge) — are all covered: three in their own posts (Posts 12, 13, this one) and one combined.

What I learned writing the second batch, looking at the four milestones together:

- **Phase 2 / validate** (Post 12) is structural: the framework starts to verify what it had been promising.
- **TUI explore** (Post 11) is about visibility: the framework becomes navigable.
- **Universal AGENTS.md** (Post 13) is about reach: the framework becomes discoverable by any CLI.
- **Full i18n coverage + CLA badge** (this post) is about opening: the framework becomes visible to the external visitor.

Each one covers a distinct dimension of the same idea, which the blog already encoded in earlier posts: what a framework says it is isn't enough for it to be; it has to be present on every surface where someone — agent, external adopter, visitor — can build a model of the project. The second batch's milestones are the mature version of the *structural visibility* Post 5 named with a governance Issue.

Now, after Post 14, there are no more candidates in the queue. If new framework milestones emerge between today and when the blog's next session activates, RETOMAR-AQUI will be used again as the resumption point. If none emerge, this post is the real closing — the closing that completes the inventory the blog set out to cover.

---

## 7. Closing

What I took from the process, in four claims:

1. **Total coverage costs little at the end of the work and a lot at the beginning.** The last 5% — those two files in Chinese — is a dedicated release. But it's only trivially closable because the other 95% was already done. 20/20/20 parity isn't born at the start; it's pursued for months and closed at the end with a small PR.

2. **Having a policy and publishing it are two different things.** The framework's CLA existed months before the badge. Twelve lines of Markdown turned an internal policy into a public invitation. The day you put the badge you invent nothing; you declare you're ready to be found.

3. **The operational rule of what gets translated is archival.** What humans read gets translated; what agents read lives in English. That distinction, intuited from January's first AIDEC, became an explicit review criterion in May. Every new framework file passes through that question before choosing its destination.

4. **When a framework becomes adoptable by third parties is a decision, not a state.** There's a day when the operator decides the framework can be seen without embarrassment by people who didn't write it. That day materializes in small gestures — completing translations, publishing the badge — that don't change functionality but change posture. The framework stops talking to itself.

With this the blog closes the second batch. The arc that started retroactively with the project's prehistory (Post 2, January) and completed with the meta-patterns (Post 10, May) now also has its corollary: the milestones that made the framework adoptable. Fourteen posts, twenty-eight files (fourteen ES + fourteen EN), between twenty-five and twenty-six thousand words per language. As the closing of Post 10 said: *the blog finishes telling the story. The framework goes on*.

---

*Anchors: PR [#144](https://github.com/StrangeDaysTech/straymark/pull/144) — `fw-4.13.4` (merged 2026-05-13 06:20 UTC, full i18n coverage). PR [#154](https://github.com/StrangeDaysTech/straymark/pull/154) — CLA Assistant badge in READMEs (merged 2026-05-15 00:14 UTC). Full policy: `CONTRIBUTING.md` (repo root + translations in `docs/i18n/{es,zh-CN}/CONTRIBUTING.md`).*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*

*— End of the second batch of the StrayMark blog.*
