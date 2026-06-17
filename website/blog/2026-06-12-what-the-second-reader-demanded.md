---
slug: what-the-second-reader-demanded
title: What the second reader demanded
authors:
  - jose
tags:
  - straymark
  - loom
  - knowledge-graph
  - architecture
  - rust
  - cli
  - sentinel
date: 2026-06-12T00:00:00.000Z
description: Before Loom could draw anything, StrayMark's document model had to stop being the CLI's private property. The refactor that shipped zero user-facing change was the precondition for everything visual that followed — and a small lesson the framework had been preaching to adopters all along.
---
*StrayMark documents have always formed a graph — every `related`, `supersedes`, `originating_ailogs` link is an edge. The CLI built that graph internally for `straymark audit`, and a human could read it one document at a time in the `explore` TUI. Then a second consumer showed up — Loom, an experimental browser view of the whole corpus — and asked to parse the same documents. The honest answer to "can it reuse the CLI's parser?" was no, because the parser wasn't a library; it was buried in `cli/src/document.rs`. The fix shipped as `cli-3.23.1` with **zero user-facing behavior change**. It was also the most important release of the month.*

<!-- truncate -->

> *"You cannot have two parsers of the same corpus."*

That sentence is the whole post, but it took a new component to make it concrete. For most of StrayMark's life there was exactly one program that read StrayMark documents: the CLI. One parser, one consumer, no possibility of disagreement — because there was nothing to disagree with. The graph that links Charters to AILOGs to TDEs to ADRs existed, but it lived in the CLI's private memory, assembled on demand for `straymark audit` and never shown to anyone whole.

Three independent framework reviewers flagged the same gap: as a corpus grows — the reference adopter, [Sentinel](https://github.com/StrangeDaysTech/sentinel), is the case that keeps surfacing these — reading the document web one card at a time in a TUI stops conveying *shape*. You can't see the cluster, the bridge document, the orphan, the cycle. The answer was going to be a graphical view. And the moment that view became real, StrayMark would have its first second reader.

## Clearing the deck first

A new component is a new front. Opening one on top of unfinished business is how a project accumulates the kind of debt it then writes blog posts apologizing for. So the week before Loom's first line of code, three deferred items closed — each small, each on the "should have done this already" list.

- **Portable installed skills** ([`fw-4.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.24.0), [#232](https://github.com/StrangeDaysTech/straymark/issues/232)). A Codex (`gpt-5.5`) review of StrayMark's own skills running under Codex CLI in Sentinel found five portability bugs — the sharpest being that skills generated for Codex still wrote `agent: claude-code-v1.0` into the frontmatter they produced, *distorting provenance telemetry on a tool whose entire job is provenance*. Skills now resolve their own runtime identity from `AGENT-RULES.md §1` instead of hardcoding one.
- **`charter close` closes the follow-ups loop** ([`cli-3.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.22.0), [#135](https://github.com/StrangeDaysTech/straymark/issues/135) Tier 3). The last open tier of the follow-ups roadmap — closing a Charter now scans its AILOGs for deferred work, extracts what isn't in the registry, and offers per-entry TDE promotion. It had been gated until drift detection was reliable; it was, so it shipped.
- **`charter drift` is native Rust now** ([`fw-4.26.0` / `cli-3.23.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.0), [#237](https://github.com/StrangeDaysTech/straymark/issues/237)). The command used to shell out to a bash script, which meant it simply didn't run on Windows without WSL or Git Bash. Porting the declared-vs-modified set-difference in-process closed the last functional Windows-native gap — and deleting the intermediate that parsed the script's stdout removed a whole bug class. The integration suite, which already pinned the behavior, now runs on every platform and doubles as the script-equivalence guarantee.

None of these is a headline. Together they're the difference between starting a new component from a clean base and starting it from a base you're quietly ashamed of.

## One parser, or two truths

Then the actual pivot: [`cli-3.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.1), recorded in [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md), Loom milestone M0.

The repository root became a Cargo **workspace** (`core` + `cli`). The document model — `DocType`, `Frontmatter`, `StrayMarkDocument`, `parse_document`, `discover_documents` — moved verbatim out of `cli/src/document.rs` into a new crate, **`straymark-core`**, published to crates.io. And `audit_engine::build_traceability`, the CLI's private graph assembler, generalized into `core::graph`: a typed, bidirectional, orphan-preserving knowledge graph over the frontmatter cross-links, with one deliberate design choice that matters later — **dangling references are kept as first-class `resolved: false` edges**, not dropped. A link to a document that doesn't exist is data, not an error to swallow.

Why go to the trouble of a workspace and a published crate instead of just letting Loom copy the parsing logic? Because copying is how you get two truths. If Loom parsed StrayMark documents with its own code, the two parsers would drift — a new frontmatter field recognized by one and not the other, a heading variant matched here and missed there — and the visualization would quietly disagree with the audit. That is *precisely* the failure mode StrayMark exists to prevent in adopters' projects: two sources of truth for the same fact, diverging silently. Shipping it inside our own tooling would have been malpractice with a straight face.

So the bar for M0 was the bar the principle demands: **the CLI's behavior had to be byte-for-byte identical before and after.** The full test suite passed unchanged; `straymark audit` produced identical output. The release notes say "no user-facing behavior changes," which reads like a non-event and was in fact the entire point. A refactor that changes nothing the user can see, in service of guaranteeing two programs can never disagree, is the most StrayMark-shaped release imaginable.

## What we deliberately didn't do

The restraint section, because every post in this series has one.

`straymark-core` is published to crates.io — it's a real shared library now. **Loom is not.** It ships GitHub-release-only under `loom-*` tags, marked experimental (v0 / N=1), behind an opt-in download gate in `straymark loom serve`. The CLI gained **no** `axum` or `tokio` dependency from any of this — the server lives entirely in `experiment-loom/`, and the CLI's only knowledge of it is a launcher that fetches a binary on first use. The shared *gramática* graduated to a published crate the moment a second reader needed it; the experimental *component* stays firewalled until it earns its way out. One of those decisions is permanent and the other is reversible, and they're kept on opposite sides of the line on purpose.

## If you've read this far

The portable exercise: find the format your project parses in more than one place. Config files read by both the app and a CI script. A log shape consumed by a shipper and a dashboard. An API contract serialized on one side and deserialized on another by hand-written code. Now ask the one question M0 was built to answer: do those two readers share a parser, or do they merely *resemble* each other today? Resemblance is not a guarantee — it's a coincidence with an expiration date, and the date is whenever someone edits one side and forgets the other. The only durable fix is the unglamorous one: extract the grammar, publish it once, and make divergence impossible instead of merely unlikely.

---

*StrayMark `fw-4.24.0` → `fw-4.26.0`, `cli-3.22.0` → `cli-3.23.1` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) · Charter [`CHARTER-01-loom-server`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-loom/CHARTER-01-loom-server.md) · Issues [#135](https://github.com/StrangeDaysTech/straymark/issues/135) · [#232](https://github.com/StrangeDaysTech/straymark/issues/232) · [#237](https://github.com/StrangeDaysTech/straymark/issues/237) · PRs [#238](https://github.com/StrangeDaysTech/straymark/pull/238) · [#239](https://github.com/StrangeDaysTech/straymark/pull/239). Predecessor: [What the bash script said was in sync](what-the-bash-script-said-was-in-sync). Next: [What the graph couldn't draw yet](what-the-graph-couldnt-draw-yet).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
