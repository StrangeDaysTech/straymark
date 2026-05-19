---
slug: four-names-in-four-months
title: Four names in four months
authors:
  - jose
tags:
  - straymark
  - prehistory
  - identity
  - governance
date: 2026-04-05T00:00:00.000Z
description: Looking for the concept by trying out names
---
*Chronicle → Monimen → DevTrail → Strange Days Tech → StrayMark. The project's prehistoric arc of four renames in four months — three rushed ones in January and a disciplined one in May. The idea didn't move; the name did.*

<!-- truncate -->

> **Editorial note**: this is the first post published retroactively on the blog. The `date` in the frontmatter corresponds to a point inside the arc the post narrates, not the day it was written. The entire blog is being built this way, backwards, starting from the episode that triggered me to write it at all (`emergent-observation-design`, 2026-05-16). Pretending otherwise wouldn't help anyone.

---

## 1. The day after

On 28 January 2026, at 17:29 Central Time, this commit landed in the repo:

> `chore: rebrand Chronicle to Monimen`

One day. Twenty-nine hours, to be exact, after the project's *initial commit*. The framework barely existed, and it was already changing its name.

I'm telling this in the past tense, but it happened three months ago. And three months ago, two more renames later, it still wasn't called StrayMark. The project that today bills itself as *Documentation Governance for AI-Assisted Software Development* carries a short, disorderly prehistory that's worth naming before moving on.

---

## 2. Five events, four months

| Date (UTC-6) | Time | Event | Anchor |
|---|---|---|---|
| 2026-01-27 | 12:14 | *Initial commit*: **Enigmora Chronicle Framework v1.0.0** | [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) |
| 2026-01-28 | 17:29 | Rebrand: Chronicle → **Monimen** | [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) |
| 2026-01-29 | 19:30 | Rebrand: Monimen → **DevTrail** (`BREAKING CHANGE`) | [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) |
| 2026-03-01 | 19:05 | Org rebrand: Enigmora → **Strange Days Tech** + the Rust CLI is born | [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) |
| 2026-05-08/09 | — | DevTrail → **StrayMark** (ADR + arc of 5 PRs) | ADR `2026-05-08-001`, PRs #114-#118 |

Three renames in three days. Then a month-long pause. Then the org rebrand with the Rust CLI bundled in. Then another two-month pause. Then StrayMark, this time with a public ADR anchoring the decision.

A small detail worth flagging: the 28 January commit (Chronicle → Monimen) was not marked `BREAKING CHANGE`. The 29 January one (Monimen → DevTrail) was. *"BREAKING CHANGE: Complete rebrand from Monimen to DevTrail"*, the commit body reads. The difference is small but telling: the second rename felt more definitive. It was, for two months.

---

## 3. What doesn't move

This is the section that interests me most. The project changes names four times in four months. The idea doesn't.

The README of the *initial commit* — the one that lived in the repo when it was called *Enigmora Chronicle Framework* — opens with this line:

> **Documentation Governance for AI-Assisted Software Development**

It's the same line that opens the StrayMark README today. Four names, one tagline.

A bit further down, that same v1.0.0 README states the project's thesis in an explicit block quote:

> *"No significant change without a documented trace."*

That sentence is, today, Principle #1 of the framework. It lives in `PRINCIPLES.md §1 — Total Traceability`. The wording shifted slightly when translated into Spanish, but the weight of the claim stayed intact: no significant change without a documented trace.

The eight document types listed in the v1.0.0 README — REQ, ADR, TES, INC, TDE, AILOG, AIDEC, ETH — are all still alive. Others were added later (MCARD, SBOM, SEC, DPIA), but none displaced the originals. The January taxonomy held.

**The name moves four times; the idea doesn't.** What changed across those four months wasn't the *what* — it was the label that fit on it.

---

## 4. The AIDEC with a typo'd year

There's a small anecdote I keep coming back to.

The project's first structured document — the first AIDEC, the document that, in the framework's own taxonomy, records a decision made with an agent's assistance — is `AIDEC-2025-01-27-001-i18n-strategy.md`. It lives in commit [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e), added at 18:01 on 27 January 2026, six hours after the *initial commit*.

The ID says **2025**. The commit is from **2026**.

The very document that established the `[TYPE]-[YYYY-MM-DD]-[NNN]` identifier convention has a typo in its own year. The framework started imperfect, and nobody fixed it in time. Anyone who clones the repo and looks at the ID closely will notice it — and will also see that the commit which introduced it is dated correctly. That's the best evidence I have that audit discipline is something that arrives later; it doesn't ship with the repo.

But more interesting than the typo is what that AIDEC decides. The question José posed in January, with help from Claude Opus 4.5, was: how do you internationalize a framework that exists for humans but whose configuration is read by agents? The answer, from the justification section of the AIDEC itself:

> *"AI agents (Claude, Gemini, Copilot, Cursor) process instructions equally well in any language, so translating their config files provides no functional benefit."*

The decision that document made — translate what humans read (documentation, templates); keep what agents read (CLAUDE.md, GEMINI.md, .cursorrules) in English — was an early intuition that the framework has two distinct audiences. The blog itself ratifies that intuition today: bilingual Spanish/English for the humans who'll read it, while the skills and prompts that orchestrate agents stay in English only. Three months later, it's still the right call.

---

## 5. The birth of the CLI — and the missing number

### 5a. The day the project became software

On 1 March 2026, at 19:05, commit [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) landed:

> *"feat: rebrand to Strange Days Tech, add CLI scaffolder, restructure repo"*

Until that day, the project was prose. It was templates, skills, governance rules, and a pile of Markdown that depended on the operator (me) copying it by hand into the repos where I wanted to use it. There was a bash script called `copy-devtrail.sh`. It was a toy.

That 1 March commit introduced `cli/Cargo.toml` and `cli/src/main.rs`. The first CLI was called `devtrail-cli`, version `2.0.0`, and had three commands:

```rust
enum Commands {
    Init { path: String },
    Update,
    Remove { full: bool },
}
```

`init`, `update`, `remove`. Three operations. Everything else came later: `status`, `repair`, `validate`, `new`, `compliance`, `metrics`, `analyze`, `audit`, `explore`. But the original three are still there, almost untouched.

The important thing about that commit isn't the three commands: it's that, on that day, the project stopped being a body of documentation living in its own repo and started being an executable tool that gets installed into other repos. The boundary between *"documentation governance project"* and *"framework with tooling"* was crossed that Sunday in March. It matters more than any of the renames that came before.

It was also, incidentally, the day the project moved GitHub accounts: from `enigmora/devtrail` to `StrangeDaysTech/devtrail`. The organization renamed itself from Enigmora to Strange Days Tech, S.A.S., in the same commit that shipped the CLI. Simultaneous renames: the product's (Monimen to DevTrail) had happened in January; the company's, in March. Two distinct layers of identity moving on different clocks.

(A side note, important for archival accuracy: agent co-authorship — the `Co-Authored-By: Claude Opus X.Y` line that appears in commits — does not start in March. The *initial commit* of 27 January is already signed by Claude Opus 4.5. Transparency about AI co-authorship was a rule from the first line of code. What changes on 1 March isn't the practice, it's the scale: the CLI is the first artifact where co-authorship produces *executable code*, not documentation.)

### 5b. The missing number

There's a detail about framework versioning that makes more sense if you just look at `git tag`:

```
fw-2.0.0
fw-2.1.0
fw-4.0.0
fw-4.1.0
...
```

There is no `fw-3.x.x`. The project deliberately jumped from 2 to 4.

The jump syncs with commit [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2), dated 27 March 2026, whose body describes the change like this:

> *"Reposition DevTrail from 'documentation helper' to 'ISO 42001-aligned AI governance platform' across all user-facing docs. Lead with regulatory urgency (EU AI Act Aug 2026) and compliance value proposition."*

Until March, DevTrail presented itself as a *documentation helper*. From that commit onward, it presented itself as an *ISO 42001-aligned AI governance platform*. The version-number jump (2 → 4, skipping 3) marked the magnitude of the thesis shift. It was a conscious call: one major bump wasn't enough to signal the difference.

And this is where the prehistory starts to make retrospective sense. The intuition that had been in the repo since January — the idea of a robust, normative, standards-alignable record-keeping system — was only named explicitly in March. We'll see this more clearly in the next section, when we talk about the second name.

---

## 6. About the names

Three of the four renames have no ADR. The justification lives only in commit messages, and sometimes not even there: the commit `chore: rebrand Chronicle to Monimen` doesn't say why Monimen. The next one, `chore: rebrand Monimen to DevTrail`, doesn't say why DevTrail either. The practice of the ADR as a disciplined artifact came later; in January the name would change with a commit and the justification would evaporate into the conversation that produced it.

But some etymology is rescuable.

- **Chronicle** *(27 Jan)*. Historical record. Logbook. The name speaks for itself: what you do with a *chronicle* is write down what happened, in order, so it can be reviewed later. No ADR, but the word carries its own meaning.

- **Monimen** *(28 Jan)*. This is the only name whose intuition I do remember and that's worth telling. *Monimen* came from a wordplay on *Monumento* (Monument in English). The analogy that motivated the suggestion: the norms already being intuited at that point — AIDEC, AILOG, alignable with the emerging ISO 42001 governance regime — represented something solid and durable. A *monument* is something erected to last; an immovable reference point for a community. That's what the framework wanted to be. The clipped suffix — *Monimen* instead of *Monument* — was an attempt to give the term a more agile, software-like feel, less marble. It lasted twenty-five hours. But the intuition didn't die: what would, in March, get named explicitly as *ISO 42001-aligned AI governance platform* is the mature version of that same impulse. Monimen was a name with the right thesis and the wrong tongue.

- **DevTrail** *(29 Jan)*. The developer's trail. Less solemn than Monimen, more concrete. The commit marked it `BREAKING CHANGE` — which, viewed from May, carries some irony: the most definitive of the first three rebrands was precisely the one that lasted three and a half months before being replaced.

- **StrayMark** *(8-9 May)*. The mark, the trace that's left behind. But this rebrand deserves its own post: there's already a public ADR (`2026-05-08-001`), an arc of five core PRs (#114-#118), and a *"Why StrayMark?"* manifesto in the README that earns separate treatment. Here, I just close it as the rebrand that finally came with documentary discipline behind it.

The point isn't the etymology itself. It's that each name ratified a distinct intuition about what the product was: **logbook** (Chronicle), **normative pillar** (Monimen), **developer's trail** (DevTrail), **residual mark** (StrayMark). Four names are four hypotheses about the concept. The search ends when the concept stabilizes — and only then can the name sit still.

---

## 7. Closing

What I took from the process, in four claims:

1. **We weren't looking for a name. We were looking for the concept.** The four renames are evidence of searching, not of indecision. The concept got sharper as it cycled through names.

2. **Three of four renames without an ADR.** The practice of the ADR as a disciplined artifact came later. In January there was no habit yet. That isn't debt — it's honesty about pace. Governance habits are the result of wanting to record what was already happening, not the precondition for starting.

3. **The project became software on 1 March 2026. Before that, it was prose.** The Rust CLI is the boundary. Any conversation about StrayMark *as a framework* has to acknowledge there's a before and an after that commit.

4. **There probably won't be another rebranding.** But the blog isn't promising it. The project is honest about its own mutability, and promising otherwise would betray the first claim of the first post.

When I started writing this blog, two weeks ago, I didn't intend to cover the prehistory. The idea was to talk about Charters, about emergent observation, about the things the framework actually does today. But the blog itself is a retroactive act — it exists because something surprised me enough to make me start writing. And once I decided to write backwards, getting to January was inevitable. To the three renames in three days. To the AIDEC with the typo'd year. To *Monimen*.

Next, in the following post: the framework's first real methodological experiment — Sentinel's six Plans, and the day *Plan* got renamed to *Charter*. That's where the story the blog came to tell actually begins.

---

*Anchors: commits [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) · [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) · [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) · [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e) · [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) · [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2). Original README: `git show v1.0.0:README.md`.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
