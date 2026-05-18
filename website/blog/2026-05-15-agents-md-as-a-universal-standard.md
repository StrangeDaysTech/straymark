---
slug: agents-md-as-a-universal-standard
title: AGENTS.md as a universal standard
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - agents-md
  - governance
  - open-standards
  - i18n
date: 2026-05-15T00:00:00.000Z
description: Four months between the intuition and the open standard
---
## 1. Four months between intuition and standard

On 15 May 2026, at 19:05 UTC, PR [#155](https://github.com/StrangeDaysTech/straymark/pull/155) closed the framework as `fw-4.15.0 / cli-3.13.2`. What it brought is a single new file in the list of directives the CLI injects: `AGENTS.md`, at the adopter's repo root, parallel to the ones already there (`CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules`, `.cursor/rules/straymark.md`).

This would be trivial if it weren't for the fact that exactly four months earlier, on 27 January 2026, the project's first AIDEC — the one Post 2 covered with the typo'd-year anecdote — had declared:

> *"AI agents (Claude, Gemini, Copilot, Cursor) process instructions equally well in any language, so translating their config files provides no functional benefit."*

That sentence, written six hours after the project's *initial commit*, contained an intuition the framework had been operationalizing ever since: agents are a single technical audience. Config files (`CLAUDE.md`, etc.) stayed in English (while human documentation got translated into three languages) because the "AI agents" audience didn't benefit from translation. What was missing to close that intuition was giving it a single place to point at, without having to clone the file four or five times for each different vendor.

This post covers exactly that: how `AGENTS.md` — an open standard donated to the Agentic AI Foundation in December 2025 — completed four months later the decision the first AIDEC had opened in January.

---

## 2. The fragmentation `AGENTS.md` resolves

Between 2024 and 2026, while AI agent CLIs proliferated, each one invented its own instruction-file format:

- Anthropic published `CLAUDE.md` with the first version of Claude Code.
- Google adopted `GEMINI.md` for Gemini CLI.
- GitHub established `.github/copilot-instructions.md` for Copilot CLI.
- Cursor started with `.cursorrules` and later migrated to `.cursor/rules/*.md`.
- OpenAI used another path for Codex CLI; Aider, Devin, Continue, Roo Code, Factory Droids, Sourcegraph Amp, Zed AI, Windsurf, Amazon Q each had their own.

A StrayMark adopter using two or three of those CLIs had to maintain two or three copies of the same content, hand-synced, hoping none diverged. Up to `fw-4.14.x`, the framework acknowledged that fragmentation in its `straymark init` command: it injected into the five most common vendor-specific files. But the other ten or fifteen CLIs were left out. The adopter handled them by copying content by hand.

In late 2025, the community of open-source projects producing agent CLIs did what gets done when a fragmentation becomes costly: they agreed on a standard. `AGENTS.md` at the repo root, read by everyone. The donation to the Agentic AI Foundation (Linux Foundation, December 2025) gave it institutional governance. The specification is deliberately minimal: a Markdown file, no imposed schema, in a predictable location. What each CLI does with that file is its own business; the canonical question — *"where is the agent configuration for this repo?"* — now has a single answer.

By May 2026, the list of CLIs that read `AGENTS.md` includes (cited literally from the PR #155 body):

> *"Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others."*

Fifteen CLIs. Some still read *also* their vendor-specific files for historical compatibility. But all fifteen know to look for `AGENTS.md` first.

---

## 3. Adopt without killing your own

The most interesting decision of `fw-4.15.0` isn't adopting `AGENTS.md`; that was inevitable once the standard had critical mass. What's interesting is *how*. The PR body articulates it without softening:

> *"Parallel to CLAUDE.md / GEMINI.md: marker block points to STRAYMARK.md, with a minimum-viable body below for readers that cannot follow relative links."*

Three key words: **parallel**, **marker block**, **minimum-viable body**.

*Parallel* means `AGENTS.md` doesn't replace vendor-specific files. It coexists. The framework still injects `CLAUDE.md`, `GEMINI.md`, `.cursorrules` and the others. The reason is pragmatic: some CLIs (notably Cursor in its legacy version) require the full content to be embedded, not behind a relative link. Keeping the sibling files preserves that compatibility without each adopter having to wrestle with their specific CLI.

*Marker block* is the convention the framework already used in the other vendor-specific files: an HTML-commented block between `<!-- straymark:begin -->` and `<!-- straymark:end -->` pointing to `STRAYMARK.md` as source-of-truth. Any `straymark update-framework` replaces only what's between those markers. Whatever the adopter wrote *outside* them — for instance, project-specific instructions — stays intact. The framework respects the adopter's file; it only governs its own section.

*Minimum-viable body* is the concession to CLIs that don't follow relative links. Below the marker block, `AGENTS.md` carries a short body with the essentials: agent identity, review requirements, pre-commit checklist, regulatory frontmatter snippet, NIST AI 600-1 risk categories, observability rules, naming convention. It's enough for an agent that can't read `STRAYMARK.md` to operate correctly; it's insufficient to replace the canonical document. The asymmetry is deliberate: we want the agent to *want* to open `STRAYMARK.md`.

The defensive CLI detail (`cli/src/commands/remove.rs:13`) closes the operational flank: `AGENTS.md` was added to `LEGACY_DIRECTIVE_TARGETS`. If an adopter loses their `.straymark/dist-manifest.yml` and runs `straymark remove`, the legacy fallback cleans `AGENTS.md` too instead of leaving it orphaned. It's boring housekeeping, but it's the difference between a framework that uninstalls completely and one that leaves debris.

---

## 4. No ADR

It's worth naming an absence. The decision to adopt `AGENTS.md` did not get its own ADR.

It's a deliberate contrast with Post 6's StrayMark rebrand, which did have a public `ADR-2026-05-08-001` with three evaluated alternatives and enumerated consequences. It's also a contrast with Post 12's Phase 2, which had a dense CHANGELOG entry with architectural justification. `fw-4.15.0` had only the PR body as documentary justification — a good body, with a clear argument, but not an ADR.

Why? The operational answer: the decision was perceived as **additive**, not structural. Adopting an open standard that coexists with existing formats doesn't change the framework's model; it only extends it. There's no alternative whose dismissal would require formal justification: not adopting `AGENTS.md` would mean leaving fifteen CLIs out for aesthetic preference, and nobody had that argument to make.

But there's a recordable lesson here. The blog has seen the pattern before — Phase 2 closes an empirical loop, the rebrand changes identity, Post 10's meta-patterns name what was already there — and in each case the question *"does this deserve an ADR?"* has a different answer. The operational rule the framework seems to have adopted, without naming it, is this: **an ADR is justified when the decision closes costly alternatives and leaves a record of those not chosen**. `fw-4.15.0` didn't close costly alternatives — it only added a layer. So the PR was enough.

I leave it on the record here because it's worth acknowledging that not every change deserves an ADR, nor is every PR a historical document. Archival discipline has its own criteria.

---

## 5. Visibility for agents, two layers

Ten days before `fw-4.15.0`, on 9 May, Issue [#113](https://github.com/StrangeDaysTech/straymark/issues/113) was closed — the episode Post 5 documented as *"Charters invisible to the agents"*. The operator had worked six hours with a capable, attentive agent, with the canonical onboarding apparatus loaded, and the agent had never proposed using a Charter. The conclusion that post canonized: *structural visibility* is the framework's responsibility, not the agent's property.

`fw-4.15.0` covers the other face of the same problem.

Post 5 was about **internal structural visibility**: how we ensure an agent already reading our repo discovers the framework's core concepts. PR #122's answer (Post 5) was to multiply the internal surfaces where Charter appears as a concept: `STRAYMARK.md §15`, `CLAUDE.md`/`GEMINI.md` directives, `/straymark-*` skills, `status` output, templates, the bridge to SpecKit. Nine internal surfaces.

This post is about **external structural visibility**: how we ensure any agent entering the repo, whatever its vendor, finds the framework's apparatus. `fw-4.15.0`'s answer is to adopt the canonical meeting point the community chose. One external surface, read by fifteen vendors.

Both layers share the same conceptual axis — *a technical artifact only exists for the agent if the surface names it* — but a different scope. Internal visibility guarantees the agent sees the Charters while reading `STRAYMARK.md`. External visibility guarantees the agent sees `STRAYMARK.md` when entering the repo. One layer without the other is incomplete.

---

## 6. What the adopter does when running `update-framework`

The operational part of the release, cited literally from the CHANGELOG:

> *"straymark update-framework brings the new template and injects AGENTS.md at the project root. The injection follows the same rules as every other directive target: it creates the file if absent, replaces the marker block on subsequent runs, and appends safely when the file pre-exists without StrayMark markers (very common in 2026 — many adopters already hand-maintain an AGENTS.md)."*

Three important behaviors:

1. **If it doesn't exist**, the CLI creates it with the canonical template.
2. **If it already exists with StrayMark markers**, the CLI replaces only the content between markers.
3. **If it already exists without markers** — very common in 2026, because many adopters were already writing their own `AGENTS.md` by hand — the CLI **appends its markers at the end of the file**, without touching what the adopter had written.

It's the same archival respect the blog already documented in Posts 3 and 6: the framework governs *its section*, not the whole file. If the adopter wants to write project-specific instructions in their `AGENTS.md` — internal policies, naming conventions, instructions specific to a particular codebase — those instructions stay intact at every `update`. The StrayMark block lives within its small marked boundary, and updates within that boundary.

The CHANGELOG note closes with a small operational warning worth recording: *"If your .gitignore excludes AGENTS.md, adjust it before update-framework so the injection lands in version control."* It's honest about edge cases the framework can't resolve on its own — the adopter's choice to ignore the file is sovereignty, not error.

---

## 7. Closing

What I took from the process, in four claims:

1. **Four months between intuition and materialization is what it takes to close a loop well.** January's AIDEC carried the intuition — *"agents are a single audience"*. `fw-4.15.0` closed it with an open standard. Between the two, the framework iterated on vendor-specific archetypes until the universal substitute appeared. The early intuition was right; the timeline was what the ecosystem's maturation required.

2. **Adopting an open standard doesn't require renouncing your own format.** `AGENTS.md` coexists with `CLAUDE.md`, `GEMINI.md`, `.cursorrules`. The decision isn't "one replaces the others" but "one joins the others when it contributes". The framework gains external visibility without losing compatibility with CLIs that require specific formats.

3. **Not every change deserves an ADR.** The StrayMark rebrand closed costly alternatives and left a formal record of the discarded ones. `fw-4.15.0` only added a universal layer: no ADR would have enriched the decision. Archival discipline has operational criteria, not doctrines.

4. **Visibility for agents operates in two layers.** The internal layer (Post 5, `fw-4.12.0`) ensures the agent sees the framework's concepts while reading the repo. The external layer (`fw-4.15.0`) ensures the agent enters the repo through the right door regardless of which CLI orchestrates it. One without the other leaves either capable agents reading in silence or disoriented agents with no entry point.

---

*Anchors: PR [#155](https://github.com/StrangeDaysTech/straymark/pull/155) — `fw-4.15.0 / cli-3.13.2` (merged 2026-05-15 19:05 UTC). Original AIDEC (Post 2): `.chronicle/07-ai-audit/decisions/AIDEC-2025-01-27-001-i18n-strategy.md` (commit `7b7193e`, ID with typo'd year). Injected template: `dist/dist-templates/directives/AGENTS.md`. Standard specification: [agents.md](https://agents.md) — donated to the Agentic AI Foundation (Linux Foundation), December 2025.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
