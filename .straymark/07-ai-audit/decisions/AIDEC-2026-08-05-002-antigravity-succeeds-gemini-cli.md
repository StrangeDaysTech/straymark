---
id: AIDEC-2026-08-05-002
title: Antigravity CLI succeeds Gemini CLI — retire the channels, keep GEMINI.md, and build a retirement mechanism
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
tags: [multi-agent, antigravity, gemini-cli, deprecation, dist-manifest, update-path]
related:
  - AILOG-2026-08-05-004
  - AIDEC-2026-08-05-001
---

# AIDEC: Antigravity CLI succeeds Gemini CLI

## Context

Google retired Gemini CLI. The operator asked to discontinue StrayMark's support
for it and prepare support for its successor, **Antigravity CLI** (`agy`).

The obvious reading of "discontinue Gemini CLI support" — delete `GEMINI.md`,
delete `.gemini/skills/` — turns out to be half wrong. Verified against `agy`
1.1.10 installed on this machine and against `agy-customizations`, the skill
Google ships *inside the binary* as the authoritative spec of its customization
system:

| Surface | What Antigravity actually does |
|---|---|
| `GEMINI.md` | **Reads it.** Listed as a first-class Rules file alongside `AGENTS.md` and `.agents/rules/*.md`. |
| `~/.gemini/` | **Its own global config directory.** The binary contains a `resolve_gemini_dir` symbol; `~/.gemini/skills/`, `~/.gemini/settings.json` and `~/.gemini/GEMINI.md` are live. |
| `<project>/.gemini/skills/` | **Not a discovery location.** Workspace roots are `.agents/`, `.agent/`, `_agents/`, `_agent/`; the global one is `~/.gemini/config/`. |
| `<project>/.agent/workflows/` | **Not read.** The root is valid, but the subdirectory it looks for is `skills/<name>/SKILL.md`. The binary contains no `agent/workflows` string. |

So the product was retired while the on-disk contract was kept. Two of
StrayMark's four Gemini-era surfaces were dead; one was alive and mis-labeled;
and the fourth — `.agent/workflows/` — had been advertised in the README as the
Antigravity channel while being read by nothing at all.

## Problem

Three decisions, and one of them turns out to gate the others.

1. **`GEMINI.md`**: retire it with the product, or keep it?
2. **The skills channel**: where does the Antigravity-readable channel live, and
   what happens to the two dead ones?
3. **How does a retirement reach anyone?** `update_files` only ever copies.
   Dropping a directory from `files:` leaves it in every existing installation
   until someone runs `straymark remove`. Without solving this, "discontinuing"
   a channel means new installations stop getting it while every existing
   installation keeps it forever — the worst of both.

## Alternatives Considered

### Alternative 1: Delete `GEMINI.md` along with the product

**Pros**: Consistent with the headline ("discontinue Gemini CLI"); one fewer
file in the adopter's project root.

**Cons**: Directly contradicted by the runtime. Antigravity loads `GEMINI.md`
as Rules; deleting it removes a governance source from the very CLI we are
adding support for. The name is stale, the file is not.

### Alternative 2: Keep `GEMINI.md`, re-framed to the Antigravity lineage

**Pros**: No regression. The template retitles to Antigravity, the identity
becomes `antigravity-v{version}`, and a short note explains *why* the file keeps
the `GEMINI.md` name — which matters, because the next person to read the tree
will otherwise conclude it is a leftover and delete it.

**Cons**: A file named after a retired product persists. Accepted: the filename
is Google's contract, not ours.

### Alternative 3: Add `ANTIGRAVITY.md` next to it

**Cons**: Verified against the binary — no `ANTIGRAVITY.md` or `AGY.md` string
exists. It would be a file nothing reads. Rejected on evidence.

### Alternative 4 (channel): `.agents/skills/` (canonical) vs `.agent/skills/` (alias)

Google's shipped doc names both: *"Path: `.agents/` (or `.agent/`, `_agents/`,
`_agent/`)"*. The binary's error templates hardcode `.agents/` only, which left
open whether the alias was implemented or aspirational.

**Settled empirically (2026-08-06).** A probe project carrying one skill under
each root was opened in an interactive `agy` session: it listed *both*
`straymark-probe-alias` (`.agent/`) and `straymark-probe-canonical` (`.agents/`)
among its available skills. `.agent/` is a real alias, so the channel keeps the
root StrayMark already ships and no migration is needed.

### Alternative 5 (retirement): document `straymark repair` instead of building a mechanism

**Pros**: No new code.

**Cons**: Makes every future retirement depend on adopters reading a release
note and running a second command. The failure is silent — the dead directory
just sits there. And this is the second release in a row where the update path
turned out to be the thing standing between a decision and the adopters
(see [[AIDEC-2026-08-05-001]], where `update-framework` refused to create new
injection targets). Fixing the shape once beats documenting around it twice.

## Decision

**Chosen**: keep `GEMINI.md` re-framed (Alt 2); move the skills channel to
`.agent/skills/` generated from the Claude source; retire `.gemini/skills/` and
`.agent/workflows/` through a new declarative `retired:` mechanism (rejecting
Alt 5).

**Justification**: The runtime evidence settles (1) and (2) with no room for
preference — one surface is alive, two are dead. For (3), the mechanism is
small, declarative, and provenance-gated: a file is deleted only when its hash
still matches what `.checksums.json` recorded, so a retirement notice from
upstream never becomes a licence to delete work the operator did inside the
path. Files the operator edited and files StrayMark never installed are kept
*and reported separately*, because telling someone they "modified" a file they
wrote themselves is wrong in a way that erodes trust in the tool's reports.

A secondary finding pushed the channel from hand-mirrored to generated:
`.gemini/skills/` had **drifted behind `.claude/skills/` in 7 of 15 skills**,
precisely because nothing regenerated or gated it — while `.codex/skills/`,
which is generated and CI-checked, was byte-perfect. So `gen_codex_skills`
became `gen_minimal_skills` and now emits both minimal-frontmatter channels
under the existing CI gate. A `git mv` would have imported the stale files.

## Consequences

### Positive
- Antigravity users get the skills from the project tree with no install step,
  plus `GEMINI.md` and `AGENTS.md` as Rules.
- The framework can now retire *any* distributed path and have it actually
  disappear from existing installations — reusable, not a one-off.
- One less hand-maintained channel; the new one cannot drift.
- StrayMark stops advertising Antigravity support via a directory Antigravity
  does not read.

### Negative
- A file named `GEMINI.md` persists after the product's death, which will look
  wrong to every reader until they hit the note explaining it.
- `.agent/` is now described as an Antigravity customization root rather than a
  vendor-agnostic standard. That claim was aspirational anyway — the channel it
  described was read by nothing — but the README loses a tidy story.

### Risks
- **R1 — RESOLVED (2026-08-06).** The `.agent/` vs `.agents/` root was the one
  assumption the code could not settle. Three headless `agy -p` probes failed:
  workspace customizations did not surface in print mode at all, even in a
  trusted directory with a git root, so the probe could not distinguish the two
  roots rather than showing one failing. The operator ran it interactively
  instead and `agy` listed **both** probe skills, confirming `.agent/` as a real
  alias. No change required. *Durable lesson: `agy -p` is not a substitute for
  an interactive session when the question is customization discovery.*
- **R2 — pruning deletes files.** Gated on hash provenance and reported in full,
  but it is the first code path in StrayMark that removes adopter files outside
  `straymark remove`. The conservative default is deliberate: when in doubt, keep.
- **R3 (new, not in Charter) — `AGENTS.md` remains the fallback for both.**
  If Antigravity ever drops `GEMINI.md`, adopters degrade to `AGENTS.md` rather
  than to nothing. No action needed; noted so the coupling is visible.
