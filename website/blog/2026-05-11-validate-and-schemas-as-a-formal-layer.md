---
slug: validate-and-schemas-as-a-formal-layer
title: Validate and the schemas as a formal layer
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - validate
  - schemas
  - governance
  - phase-2
date: 2026-05-11T00:00:00.000Z
description: 'Phase 2: when the framework starts to verify what it had been promising'
---
*464 lines of bash got replaced with a flag. From "I check this looks like a document" to "I check this satisfies the canonical schema, in the mode you ask for, with concrete operational consequences." Phase 2 of the CLI roadmap and the formalization of validate.*

<!-- truncate -->

## 1. Having schemas is not the same as validating against schemas

`straymark validate` had existed since 25 March 2026 — PR #27, part of the first arc of CLI Phase 1. What it did back then was small: verify that documents in `.straymark/` had valid Markdown syntax, parseable frontmatter, and the minimum fields of the correct type. It was useful. It was not *enforcement*.

What changed on 11 May 2026, with the `fw-4.6.0 / cli-3.7.0` release, was something else: Phase 2 of the CLI roadmap closed a full arc of seven PRs (#73 to #79) and consolidated them into one (#80). The CHANGELOG says it without metaphor:

> *"The first feature-bearing release since the repositioning... Phase 2 of `Propuesta/devtrail-cli-roadmap.md` lands as 7 bisect-safe PRs (#73–#79), grouped here for reviewers. The release closes the empirical loop the Sentinel experiment opened: telemetry at Charter close, drift detection at Charter close, and a canonical approval signal for `review_required: true` documents."*

What matters for this post isn't the seven PRs as individual milestones; it's the three that touched the `validate` command and the schemas. What in March was *"I check that this looks like a document"* in May became *"I check that this satisfies the canonical schema of the type, in the mode you ask for, with concrete operational consequences"*.

---

## 2. From artisanal bash to a command flag

One of the most visible pieces of the change is small: the `--staged` flag.

Until Phase 2, the recommended pattern for adopters to validate their documents before a commit was a bash script called `pre-commit-docs.sh`. It lived in `dist/.straymark/scripts/` and had 464 lines. It did the reasonable thing: read `git diff --cached --name-only`, filter files in `.straymark/`, parse their frontmatter with `awk`, check fields. It worked. And it was 464 lines of bash each adopter had to trust executed the right thing, in the right version, with the right YAML parser.

A PR on 28 March (commit `d9ea874`) replaced the 464 lines with a flag:

```bash
straymark validate --staged
```

Reads staged files the same, filters by `.straymark/` the same, validates the same. The difference: today all of that lives in Rust inside the binary, not in bash on disk. The adopter doesn't have to maintain the script; the script can't diverge between versions; the behavior is the same on Linux, macOS and Windows. The framework stopped asking the adopter to trust a loose script and started giving them a command with the implicit promise of a versioned binary.

It's the same pattern Post 6 documented for archival discipline (preserve git history instead of rewriting) or Post 4 documented with decision A1 (orchestrate without invoking APIs): move from *"the operator does X manually with risk of divergence"* to *"the framework canonizes X in a declarative command"*. `--staged` isn't an exotic feature; it's manual debt swept away.

---

## 3. The three v0 schemas

Phase 2 also consolidated the three schemas that live in `dist/.straymark/schemas/`:

| Schema | What it describes | Origin |
| --- | --- | --- |
| `charter.schema.v0.json` | Structure of the Charter as a first-class entity (Post 4) | Sentinel `/plan-audit` (6 cycles, format v1 → v2 → v3) |
| `charter-telemetry.schema.v0.json` | Charter telemetry (Post 3, extended in Post 10) | 5 `PLAN-NN.telemetry.yaml` files from Sentinel |
| `audit-output.schema.v0.json` | Canonical output of external audit (Post 4 + Post 10) | Sentinel dual-audit (Copilot + Gemini + critical Claude) |

Each schema carries a `$comment` field in its root. The one in `charter.schema.v0.json` says literally:

> *"EXPERIMENTAL v0. Crystallized from Sentinel /plan-audit (6 cycles, format v1 → v2 → v3). Will not stabilize to v1.0 until validated in a second domain (frontend, ML pipeline, or infra-as-code)."*

The one in `charter-telemetry.schema.v0.json`, similar:

> *"EXPERIMENTAL v0. Derived from 5 PLAN-NN.telemetry.yaml files in Sentinel (one project, Go backend) — same N=1-domain caveat as charter.schema.v0.json. Will not stabilize to v1.0 until validated in a second domain."*

The one in `audit-output.schema.v0.json`, idem:

> *"EXPERIMENTAL v0.x. Phase 3 of the CLI roadmap. Crystallized from the dual-audit pattern validated empirically in Sentinel."*

All three share an explicit declaration: *no crystallizes without a second domain*. It's framework principle #12, not as a separate paragraph in governance documentation, but **inside the JSON Schema itself, in a field any standard validator reads and any schema-explorer tool renders to the user**. The framework doesn't hide the provisional nature of its schemas; it puts it where nobody can miss it.

This matters more than it seems. The usual practice in schema ecosystems (JSON Schema, OpenAPI, Avro) is to publish `v1` and then bump as things change. Marking `v0` with a `$comment` saying *"this is experimental until a second domain validates it"* is admitting, from the start, that the first crystallization isn't definitive. The schema itself is honest.

---

## 4. Three modes of the command

`straymark validate` is no longer a command with one flow. After Phase 2 it has three operational modes, each coupled to a moment in the life cycle:

- **`all` mode (default)** — `straymark validate` with no flags. Walks `.straymark/` recursively, parses each `.md` file, tries to resolve its type (AILOG, AIDEC, ADR, ETH, REQ, TES, INC, TDE, MCARD, SBOM, SEC, DPIA, Charter), applies the corresponding schema, and reports per-file violations. This is the *full audit* mode. Useful for CI, useful for reviewing a fresh repo, useful when an adopter wants to measure how much formal debt accumulated.

- **`--staged` mode** — `straymark validate --staged`. The one that replaced bash. Only validates files `git diff --cached --name-only` reports as staged within `.straymark/`. Its natural home is `.git/hooks/pre-commit`, and since fw-4.6.0 there's a `straymark init --hooks` that installs it automatically. This is the *operational gate* mode — the one that prevents broken documents from reaching `main`.

- **`--check-pending-reviews` mode** — `straymark validate --check-pending-reviews`. The subtlest of the three, and the one that justifies more paragraphs.

Until Phase 2, a document the agent marked `review_required: true` in the frontmatter (an AIDEC with low confidence, an AILOG with high risk, etc.) sat waiting for *human approval*. But there was no canonical way to *signal* that approval. The operator read the document, mentally approved it, moved on. No structural trace of the approval. This was exactly Issue [#67](https://github.com/StrangeDaysTech/straymark/issues/67).

`--check-pending-reviews` closes that hole. The command walks all documents with `review_required: true` and lists them with their `confidence`, their `risk_level`, their author (human or agent), and their date. Approval is a commit that changes `review_required: true` → `review_required: false` with human co-authorship in the commit message — and `validate --check-pending-reviews` stops listing the document. Approval is a *canonical signal*, not a verbal agreement with oneself.

This is the piece that closes the agent's loop. The framework lets the agent create documents without prior human approval (Post 1, property #2: *"cultural permission without blocking control"*); but marks them for review when appropriate; and now has a command to list what's pending, with no escape hatch. No audit document gets lost silently.

---

## 5. Phase 2 closes the empirical loop

The full CHANGELOG sentence is worth keeping because it condenses what the release means:

> *"The release closes the empirical loop the Sentinel experiment opened: telemetry at Charter close, drift detection at Charter close, and a canonical approval signal for `review_required: true` documents (resolving issue #67)."*

Three pieces that close an arc that started with the six-Plan experiment (Post 3):

1. **Telemetry at Charter close** — the YAML block Sentinel filled by hand during the experiment became the validatable `charter-telemetry.schema.v0.json`.
2. **Drift detection at Charter close** — the 145-line bash script (`check-plan-drift.sh`) became the `straymark charter drift` subcommand, integrated into the validate flow.
3. **Canonical approval signal** — what the operator did mentally became a command with structural consequences: `--check-pending-reviews`.

It's the pattern the blog has seen recur: every time the framework crystallizes a practice Sentinel was doing manually, it does it after empirically validating that the practice works. The difference this time is that **the command that verifies the crystallization is part of the package**. Before Phase 2, Sentinel had templates, declared schemas, and the implicit promise that documents followed the schemas. After Phase 2, there is a command that **checks**. It isn't a capability change; it's a guarantee change.

---

## 6. Principle #12, turned into a schema

The philosophical piece of the post — and the one I most want on record — is this:

The `$comment` of the three v0 schemas encodes a principle the framework had been declaring in prose since April. Post 4 named it like this: *"schema marked `v0` on purpose, not `v1`, because the framework's principle #12 says no schema crystallizes without a second domain validating it"*. Post 7 applied it to the TDE design: *"the framework validated principle #12 again: no schema crystallizes without a second domain"*. Post 10 applied it to Pattern 1 and Pattern 2: *"the document says so without shortcuts: the pattern is empirically validated in Sentinel. Wait for the second domain before crystallizing it higher"*.

Phase 2 does something different. **It doesn't declare the principle; it writes it inside the schema itself, in a place any schema consumer is going to read.** The `$comment` isn't marketing; it's protocol. Any JSON Schema 2020-12 validator exposes it; any IDE that parses the schema shows it. It's the first time the framework formalizes the provisional nature of its formalizations.

If I had to sum it up in one sentence: the framework validates what it has, knowing that what it has is provisional, and puts that in writing *in the place where validation happens*. Validating the provisional is discipline. Crystallizing prematurely and validating the crystallized is theology.

---

## 7. Closing

What I took from the process, in four claims:

1. **Having schemas is not the same as validating against schemas.** The first step is declarative; the second is operational. Phase 2 did the second, not the first. The difference between the two framework modes — *"I have the promise"* vs. *"I have the command that checks it"* — determines whether the adopter can trust what's declared.

2. **The "artisanal bash → command flag" pattern is structural.** `--staged` isn't an isolated feature. It's Phase 2's version of the same pattern Post 4 documented for the external audit cycle, that Post 10 documented for the chain-evolution CLI helpers: verbose imperative → readable declarative → versioned. Every time the framework sweeps a bash script for a flag, it raises a layer of guarantees.

3. **`--check-pending-reviews` closes the agent's loop.** What Post 1 named as cultural permission without pre-approval completes here: the agent can create, but documents marked for review don't get lost — the command lists them until a human approves them with an explicit commit. Without that signal, the cultural permission of Post 1 would be negligence.

4. **Declared provisionality > premature formalization.** The `$comment` of v0 schemas makes principle #12 protocol, not doctrine. Any tool that consumes the schema knows the formalization is provisional. It's the strongest structural honesty a young framework can have: validate what you have, without pretending what you have is definitive.

---

*Anchors: PR [#27](https://github.com/StrangeDaysTech/straymark/pull/27) — original `validate` version (March 2026). Phase 2: PRs #73 to #79 + PR [#80](https://github.com/StrangeDaysTech/straymark/pull/80) — `fw-4.6.0 / cli-3.7.0` (merged 2026-05-11). Schemas: `dist/.straymark/schemas/{charter,charter-telemetry,audit-output}.schema.v0.json`. [Issue #67](https://github.com/StrangeDaysTech/straymark/issues/67) — origin of `--check-pending-reviews`.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
