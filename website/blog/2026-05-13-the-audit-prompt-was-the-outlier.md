---
slug: the-audit-prompt-was-the-outlier
title: The audit-prompt was the outlier
authors:
  - jose
tags:
  - straymark
  - i18n
  - audit
  - governance
date: 2026-05-13T00:00:00.000Z
description: The only file in the framework that lived in another language
---
*One file among dozens lived backwards — Spanish-canonical in an EN-canonical framework, for six weeks, without anyone noticing. A maintenance fix that records something the blog keeps running into: outliers teach you the convention by contrast.*

<!-- truncate -->

## 1. One line from the commit

This is the post's operational opening. Commit `a8a1ac5` from 12 May 2026 carries a body that begins with this sentence:

> *"The unified audit-prompt template at `dist/.straymark/audit-prompts/` was the only framework artifact whose canonical content lived in Spanish — every other template, skill, workflow, and governance doc follows the EN-canonical + `i18n/<lang>/` overlay pattern resolved by `cli/src/utils.rs:146`."*

One single file. Among dozens. It lived backwards. And it had been living that way since May of the previous year, without anyone noticing.

This post is deliberately short. The fix it covers — PR [#142](https://github.com/StrangeDaysTech/straymark/pull/142), `fw-4.13.3` and `cli-3.12.3`, merged on 13 May — is not a major narrative milestone. It's a maintenance piece. But it records one of the regularities I keep running into in this craft: outliers are useful. They teach you the convention by contrast.

---

## 2. Where the outlier came from

The audit-prompt wasn't born in the framework. It was born as a local *skill* of Sentinel's — `sentinel/.claude/skills/plan-audit/` — during the six-Plan experiment Post 3 covered, in April. It was a *context-specific* skill, written by the operator (me) in Spanish for reasons that can now be enumerated honestly: it was a private experiment, the operator speaks Spanish, and nobody was going to read the skill ever again. I improvised it in my language because it was for me.

When the experiment crystallized into the framework — the fw-4.4.0 to fw-4.9.0 arc Post 4 covered — the skill was ported to the canonical path `dist/.straymark/audit-prompts/audit-prompt.md`. The port was mechanical: copy the file, generalize it enough that it didn't mention Sentinel-specific components, adjust the *Plan* → *Charter* nomenclature. What wasn't done: translate it to English. The file entered the framework *in the language it had been written in*. Without anyone deciding it, Spanish became the canonical version of that one artifact.

The rest of the framework — every other template, every skill, the canonical governance docs — had been written in English since January, with Spanish and zh-CN as overlays under `i18n/<lang>/`. The audit-prompt was the silent exception, for six weeks, until 12 May.

---

## 3. How it was found: by visual inspection

It wasn't emergent observation from the agent. The operator noticed it, visually. The CHANGELOG entry records it without metaphor:

> *"Surfaced empirically when Sentinel audited the `straymark explore` rendering and the user noted that audit-cycle templates were ES while all other templates and skills were EN."*

`straymark explore` is the framework's TUI — the interactive documentation browser Post 14 will cover. Its value is rendering all templates side by side, in the same view. And when all templates are side by side, one in another language is obvious. That's the observation.

It's worth leaving the data because it matters for the blog's honesty. Post 1 articulated the pattern of *emergent observation* — an agent flagging something nobody asked it to flag — and Post 5 documented what happens when that pattern is missing because of *structural visibility*. This episode is neither. It's a more pedestrian observation: a new surface of the framework (the TUI) put all the files in simultaneous view, and the inconsistency became as obvious as when you tidy the bookshelf and notice one book is shelved with the spine backwards. Tools also produce visibility.

---

## 4. The fix: three moves in one commit

PR #142 did three things in the same commit:

- **`dist/.straymark/audit-prompts/audit-prompt.md`** rewritten in English as the canonical version. 312 lines. The translation took care to preserve the severity structure and calibration examples of the original.
- **`dist/.straymark/audit-prompts/i18n/es/audit-prompt.md`** created as an overlay. 318 lines. It's the Spanish content that used to live in root, now in its correct place.
- **`cli/src/commands/charter/audit.rs`** wired to the i18n resolver. Until then, the subcommand hardcoded the `dist/.straymark/audit-prompts/audit-prompt.md` path without consulting the adopter's `language` field in `.straymark/config.yml`. After the PR, it reads the project config and resolves the localized path — if the adopter has `language: es`, they get the Spanish overlay; if they have `language: zh-CN`, it falls back to EN canonical (there's no zh-CN translation of the audit-prompt yet); if they have `language: en` or don't specify, they get the canonical.

Three new tests cover the three paths. Zero functional change for English users (who are the majority). For Spanish users, an improvement: before, they received the prompt in Spanish by accident (because the canonical path pointed to ES); now they receive it in Spanish by convention (because the resolver decides it).

---

## 5. The lateral move

While doing the language switch, the operator took the chance to extract a specificity that had carried over from the Sentinel origin. The §Step 5 section of the prompt — about severity calibration of findings — cited a literal case from the April experiment: *"Etapa 12 Pub/Sub stub vs gochannel"*. A perfectly didactic example, but specific to a Sentinel component no other adopter would recognize.

The English translation replaced that example with a vendor-neutral formulation: *"declared deferral, not a defect — a charter that introduces a thin adapter slated for replacement in a future Charter"*. The idea — a *stub* that is declared debt, not a defect — is preserved. What goes is the component's name.

The move is small but coherent with the blog's principle. When there's an opportunity to detach the framework from a specific adopter without losing pedagogy, take it. It's the same generalization operation Post 3 documented for the six Plans and Post 4 for the first Charter: empirical content stays *in* Sentinel; what enters the framework is the vendor-neutral abstraction. The audit-prompt, almost a year later, finally finished honoring that rule.

---

## 6. Closing

Three short claims:

1. **In a framework with a convention, a single file violating it is a sign of legacy, not of design.** The audit-prompt was the outlier because it was born earlier, and nobody migrated it when the other artifacts aligned. Outliers accumulate provenance.

2. **New tools produce visibility.** The ES → EN switch wasn't decided by systematic repo review. It was decided because a new TUI put all the templates on the same screen and the inconsistency became obvious. The same applies to `grep`, to aggregate views, to `status` outputs: every new surface exposes what was misaligned.

3. **Small rebrands are opportunities to de-identify.** When you touch a file to change the language, the marginal cost of also extracting the adopter specificity left from origin is zero. The blog has recorded this pattern before (Plan → Charter, DevTrail → StrayMark); this is the miniature version. The framework becomes more vendor-neutral one file at a time.

Next, in the following post, an episode that already brushed Post 1 but deserves its own treatment: *"Manual discipline before the pattern"* (`H-11`). How Sentinel handled `CHARTER-18` before the `Pattern 1` of chain evolution existed — the empirical learning that Post 1 later named as meta-pattern.

---

*Anchors: PR [#142](https://github.com/StrangeDaysTech/straymark/pull/142). Releases: `fw-4.13.3` / `cli-3.12.3`. Key commit: `a8a1ac5`. Files: `dist/.straymark/audit-prompts/audit-prompt.md` (EN canonical) + `dist/.straymark/audit-prompts/i18n/es/audit-prompt.md` (overlay). CLI: `cli/src/commands/charter/audit.rs` wired to the i18n resolver.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
