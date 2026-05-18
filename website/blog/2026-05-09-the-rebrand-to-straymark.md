---
slug: the-rebrand-to-straymark
title: The rebrand to StrayMark
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - rebranding
  - governance
  - identity
date: 2026-05-09T00:00:00.000Z
description: The fourth rename — this time with a public ADR and a disciplined arc
---
## 1. The rename that wasn't looking for the concept

ADR `2026-05-08-001` opens with a sentence that, for anyone who read Post 2 of this blog, sounds out of place:

> *"The decision is motivated by legal certainty over the trademark rather than by product strategy or user feedback."*

The three renames of January (Chronicle → Monimen → DevTrail) were rushed searches for the concept. Each one ratified a distinct intuition about what the product was. None had an ADR; commit messages were all the documentary justification that existed.

The fourth rename, the May one, is of a different nature. It doesn't come from looking for the concept. It comes from a trademark conflict investigation — commissioned via a Claude.ai web session in early May — which surfaced *"legal uncertainty about trademark ownership as the project gained adopters."* In January the project had zero adopters and renames could be improvised. In May the project had one adopter (Sentinel), 286 *crate* downloads, two open issues, and a public manifesto promising structured governance. The threshold for improvising another rebrand had already been crossed.

This post covers three things: the disciplined decision made on 8 May, the operational arc of five PRs in forty-three minutes the following day, and the residuals that surfaced two days later. And, underneath all that, a subtler distinction: when a rebrand is purely surface, and when — without quite meaning to — it also changes identity.

---

## 2. The first disciplined rename

What's new on 8 May is the form. This time there was:

- **A prior investigation.** It wasn't decide-rebrand-then-justify; first a conflict was detected, then the decision followed from it.
- **A public ADR.** Three alternatives explicitly considered (keep "DevTrail" and accept the trademark risk; hybrid branding with legacy + new in parallel; reset to v0.1.0 under StrayMark as "new project"). All three dismissed with literal argument.
- **A temporal calculation.** The ADR itself frames the cost this way: *"The rename window narrows over time — every new adopter increases the cost of changing later... The legal risk dominates. Acting now, with one self-owned adopter, is materially cheaper than acting later under pressure."*

The reason the calculation matters is structural. If the rebrand had waited two more months — until the second adopter showed up, or the third — the migration cost would have multiplied. Each adopter repo would have had to run `mv .devtrail .straymark`, update its `CLAUDE.md`/`AGENT.md`, regenerate its pipelines. With a single adopter (the operator himself), the cost was contained and reversible. What the ADR calls *"acting now is materially cheaper"* is exactly that: the rebrand was an exercise in minimizing future debt, not a marketing decision.

There's an unintended echo with Post 2. That one closed with the line *"there probably won't be another rebranding."* This post, written from May, knows there was. The difference between promising and ratifying is that the fourth rename, unlike the first three, was forced by external circumstances no operator can foresee — a trademark conflict — and was faced with the discipline January didn't have.

---

## 3. *In scope*, *out of scope* — the ADR's masterpiece

The ADR section that shapes the rest of this post is the one that distinguishes what gets renamed from what gets preserved. Cited literally:

> *"In scope (the 'live state' of the project): all identifiers in source code, Cargo metadata, source-of-truth path, 30 skill/workflow files, public documentation, CI workflows, GitHub repository name."*

> *"Out of scope (immutable history, preserved verbatim): all commits, commit messages, and git history; all tags published before this ADR; all release titles and bodies of releases published before; all prior CHANGELOG.md sections."*

That distinction is the rebrand's armature. The *live* — paths, commands, URLs, release assets, README — is renamed in one stroke. The *historic* — git log, `devtrail-cli@3.10.0` tags, prior CHANGELOG sections, closed issues — is preserved literally. The project doesn't rewrite itself backwards.

It's the same archival discipline that Post 3 documented for the Plan → Charter rename, now applied to the framework-wide operational rebrand. I lay it out as a table because it helps to see it:

| Layer | Gets renamed | Gets preserved |
|---|---|---|
| Filesystem paths | `.devtrail/` → `.straymark/` | — |
| Rust source | `devtrail-cli` → `straymark-cli` | — |
| Skills/workflows | 30 files `devtrail-*` → `straymark-*` | — |
| Public docs | README, CLAUDE.md, ADOPTION-GUIDE | — |
| CI/CD | asset names, binary paths | Tag prefixes (`fw-`, `cli-`) untouched |
| GitHub repo | `StrangeDaysTech/devtrail` → `StrangeDaysTech/straymark` | — |
| CHANGELOG | New `fw-4.11.0` entry on top | Prior sections literal |
| Published tags | — | `fw-4.10.0`, `cli-3.10.0`, etc. preserved |
| Whole git log | — | Messages with "DevTrail" intact |
| Legacy *crate* | — | `devtrail-cli@3.10.0` on crates.io, not *yanked* |

Version continuity is the most visible proof. The next release wasn't `0.1.0` or `1.0.0`; it was `fw-4.11.0` and `cli-3.11.0`. The numbering declares, without metaphor, that the project is the same project.

---

## 4. Five PRs, forty-three minutes

On 9 May, between 06:05 and 06:48 UTC, five PRs merged in a chain:

| PR | UTC time | What it did |
|---|---|---|
| [#114](https://github.com/StrangeDaysTech/straymark/pull/114) | 06:05 | Merge of the ADR itself. Zero code changes — first you fix the decision. |
| [#115](https://github.com/StrangeDaysTech/straymark/pull/115) | 06:42 | 174 *renames* + 74 modifications. Rust source (38 files), tests (18), `dist/.devtrail/` → `dist/.straymark/` (122 files via `git mv`), 30 skills × 3 platforms. |
| [#116](https://github.com/StrangeDaysTech/straymark/pull/116) | 06:44 | Public docs in three languages (EN/ES/zh-CN). CHANGELOG preamble updated: *"StrayMark (formerly DevTrail; rebranded 2026-05-08)"*. |
| [#117](https://github.com/StrangeDaysTech/straymark/pull/117) | 06:45 | Release workflows. Asset names (`straymark-cli-*`), release titles. Tag prefixes untouched. |
| [#118](https://github.com/StrangeDaysTech/straymark/pull/118) | 06:48 | Bump `fw-4.11.0` / `cli-3.11.0`. New CHANGELOG section. |

The order is deliberate and goes from outside in: first the law (ADR), then internal code, then what adopters see (docs), then distribution (CI), finally the consummated fact (release). This sequence has an archival reading: when someone six months out reviews `git log` chronologically, they'll first see the decision, then the implementation. Causality lives in the repo.

It's worth naming a piece of the arc that broke the original plan. The ADR had contemplated nine distinct PRs (one per logical layer). In practice, the CLI tests depended on *hardcoded* paths — `include_str!("../../dist/.devtrail/...")` — and breaking the path without updating the tests at the same time left CI red. The plan compacted to five because the layers were technically coupled, not because the conceptual discipline failed. The PR #115 note records it without softening: *"The consolidation was forced by tight coupling: tests use `include_str!` on `dist/.devtrail/` paths."*

Forty-three minutes for a rebrand that touched around two hundred sixty files. The pace is only possible because the ADR had closed every decision before code was touched. Framework principle #6 — *"when the proposal is well-written, implementation is execution, not design"* — found another validation case, similar to the external audit cycle one in Post 4.

---

## 5. Why StrayMark does change more than the name

Up to here the ADR is clear: the rebrand is on the surface, the conceptual identity is preserved. But there's a nuance worth naming.

PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120) — merged hours after the main arc, the same 9 May — added the *"Why StrayMark?"* section to the README. It isn't code; it's a manifesto. And it says things no prior document of the project had articulated:

> *"Code is just the fossil trace of a mental battle. Real engineering happens in the chaos of decisions, calculated risks, and the paths you chose not to take. Traditionally, all that human trail is discarded as stray marks (accidental smudges) in a project's history. At Strange Days Tech, we believe those marks are the signal, not the noise."*

The name choice, viewed from the manifesto, isn't arbitrary. The *stray marks* — the erratic traces, the accidental tracks most projects discard — are exactly what the framework drags to the foreground: AIDECs, AILOGs, TDEs, Charters. What in any conventional project would be discardable noise is, in this project, the raw material of audit. The name embodies the product's thesis in a way *DevTrail* never did. *DevTrail* was a neutral path; *StrayMark* is a claim.

The manifesto closes with three lines that later became what an editor would call an operational *tagline*:

> *"Capture the noise. Weave the signal. Humanize the machine."*

Here's the small interesting tension of the rebrand. The ADR says only the surface changed; the manifesto from the same day suggests the identity was also articulated — perhaps for the first time with clarity. It's not a contradiction: the moment of the operational rebrand was used to write down what was already in the project but hadn't been named yet. The identity didn't change. It became legible.

---

## 6. What was left undone

Even with a public ADR and a disciplined arc, *"complete"* remains an approximation.

Two days after the rebrand, while Sentinel was preparing its next external audit cycle, the agent surfaced three classes of residuals. The chronology — captured honestly in `CHANGELOG.md` as inline errata — is worth keeping visible:

- **PR [#137](https://github.com/StrangeDaysTech/straymark/pull/137)** (11 May, 22:55) — *Orphaned skill directories*. The rebrand rewrote the `name:` field inside each `SKILL.md`, but didn't rename the directories of the three platforms (`.gemini/skills/devtrail-*`, `.claude/skills/devtrail-*`, `.agent/workflows/devtrail-*.md`). Thirty orphaned artifacts: new name, old path. Gemini CLI reported it as ten conflict warnings at startup.
- **PR [#138](https://github.com/StrangeDaysTech/straymark/pull/138)** (11 May, 23:32) — *Legacy Charter paths*. Three framework artifacts still referenced `docs/charters/` after `fw-4.12.0` had migrated the canonical path to `.straymark/charters/`. Worst of all: the `pre-pr.sh` hook was gated on `[ ! -d docs/charters ]`, which made for a silent *exit 0* for any post-4.12.0 adopter. The drift check had been installed for seven months but was no-op.
- **PR [#139](https://github.com/StrangeDaysTech/straymark/pull/139)** (12 May, 12:02) — *Broken installers*. `install.sh` and `install.ps1` still pointed to `REPO=StrangeDaysTech/devtrail`, `BINARY=devtrail`, asset name `devtrail-cli-v*`. The README — already rebranded — pointed to the new URL. Result: any user copying the README command from 9 May onward got a GitHub 404 running the installer. *The product was broken in production for seventy-two hours.*
- **PR [#140](https://github.com/StrangeDaysTech/straymark/pull/140)** (12 May, 16:49) — *Micro-residual*. Line one of `.gitignore` still read `# DevTrail - .gitignore`. Plus a new entry for `Aparador/` in internal-dev patterns.

The lesson isn't that the discipline was insufficient. It's that discipline *does not prevent* residuals — it makes them findable, documentable, and repairable in a public inline errata. The first three renames (January) probably had equivalent residuals; we never cataloged them because there was no documentary habit. The fourth rebrand cataloged them. It's the first project rebrand that admits, in the public repo and in the CHANGELOG itself, that the operator didn't catch everything on the first pass.

The most uncomfortable detail — broken installers for seventy-two hours — is the one I most care to record. If instead of one solo adopter (the operator) there had been a team, that window would have hit real users. *Acting now, with one self-owned adopter, was materially cheaper*, says the ADR. That's exactly what the sentence was buying: the right to be wrong for seventy-two hours without hurting anyone.

---

## 7. Closing

What I took from the process, in four claims:

1. **Not all renames are the same type.** The three of January were looking for the concept; the May one was defending the project against future legal debt. Only the May one could be done with a public ADR, because only the May one was born of external evidence and not internal intuition.

2. **The surface vs. identity distinction belongs to the document, not to rhetoric.** The ADR itself declares what gets renamed and what gets preserved. That list — paths yes, git log no; release titles yes, prior tags no — is the rebrand's armature. Without that explicit list, *"rebrand"* means anything you want.

3. **Identity can be made legible without changing.** The *"Why StrayMark?"* manifesto didn't invent what the project was; it articulated it clearly for the first time. The operational rebrand was used to write down what was already in the code but hadn't made it to the README. Identity wasn't rewritten — it was published.

4. **Discipline does not prevent residuals; it makes them findable.** Four *errata* PRs two days later aren't a failure of the rebrand; they're evidence that the rebrand was honest enough to admit what slipped through. The three January renames probably had equivalents; they were never documented.

Next, in the following post, an episode that connects directly with the last point: *TDE as a mechanism for naming transversal debt* (`H-09`). The first real case the framework used to articulate how to surface debt — and which left a hard lesson about stacked-PRs that deserves its own paragraph when we get there.

---

*Anchors: ADR [`2026-05-08-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-05-08-rebranding-straymark.md). Core arc: PRs [#114](https://github.com/StrangeDaysTech/straymark/pull/114) · [#115](https://github.com/StrangeDaysTech/straymark/pull/115) · [#116](https://github.com/StrangeDaysTech/straymark/pull/116) · [#117](https://github.com/StrangeDaysTech/straymark/pull/117) · [#118](https://github.com/StrangeDaysTech/straymark/pull/118). Manifesto: PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120). Residuals: PRs [#137](https://github.com/StrangeDaysTech/straymark/pull/137) · [#138](https://github.com/StrangeDaysTech/straymark/pull/138) · [#139](https://github.com/StrangeDaysTech/straymark/pull/139) · [#140](https://github.com/StrangeDaysTech/straymark/pull/140). Release: `fw-4.11.0` / `cli-3.11.0`.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
