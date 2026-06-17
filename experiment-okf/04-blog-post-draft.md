---
DRAFT — not yet published. Target: website/blog/. Series voice ("What the X…").
Suggested date: pick the publish day. Translations (es/zh-CN) follow the usual
pipeline once the EN draft is approved.
---
slug: what-the-open-format-left-to-the-producer
title: What the open format left to the producer
authors:
  - jose
tags:
  - straymark
  - okf
  - knowledge-graph
  - governance
  - loom
  - interoperability
description: Google published the Open Knowledge Format — markdown files, YAML frontmatter, a graph of links, an agent that maintains them. It's the pattern StrayMark has shipped for months, arrived at independently. The convergence is the validation. The difference is everything OKF deliberately left to the producer.
---
*On 2026-06-12 Google Cloud published the Open Knowledge Format: a directory of markdown files with YAML frontmatter, cross-linked into a graph, written and maintained by AI agents, with a static visualizer and a one-page spec. If that description gives you déjà vu reading this blog, it should — it's the shape StrayMark has shipped for months. Google got there independently, crediting the same Karpathy gist that the whole "LLM-wiki pattern" traces back to. The honest reaction isn't defensiveness. It's that someone with Google's reach just validated the bet, and the only interesting question left is the one their spec answers in a single sentence.*

<!-- truncate -->

> *"OKF requires exactly one thing of every concept: a type field. Everything else is left to the producer."* — the OKF v0.1 spec

That sentence is the whole post. Because **governance is everything else.**

## The convergence

Strip OKF and StrayMark to their mechanics and they're the same machine. A unit
of knowledge is a markdown file with a YAML frontmatter block. The frontmatter
carries one required field — OKF calls it `type`, we call it the document type.
Documents link to each other, and those links form a graph that a tool renders.
The corpus is written and kept current by AI agents, because — in Karpathy's
words, which both projects descend from — *"LLMs don't get bored, don't forget
to update a cross-reference."* Bundles are just files: diffable, git-hostable,
readable without tooling. There's a generator that seeds the corpus by walking
an existing system. There's a graph visualizer.

We didn't copy OKF and OKF didn't copy us. Both projects walked the same path
from the same headwater — Karpathy's gist, itself a callback to Vannevar Bush's
1945 Memex — and arrived at the same primitives. When two independent teams
converge on markdown + frontmatter + graph + agent, that's not a coincidence to
be nervous about. It's the design being right.

So we're not going to argue with OKF. We're going to point at the one sentence
where it hands us our entire reason to exist.

## What got left to the producer

OKF is, by intent, *minimally opinionated*. It standardizes the envelope — how a
concept is shaped, how links work, how a bundle is packaged — and stops there.
*Everything else is left to the producer.* That's the right call for OKF's job,
which is making an organization's **data** legible to agents: what this BigQuery
table means, what this metric measures, the join paths, the runbook. The
envelope is the contribution; the contents are yours.

StrayMark is the opposite kind of project. It is **maximally opinionated about
one thing**: the governance of software built with AI. The document types aren't
left to the producer — they're an ADR (a decision and why), an AILOG (what the
agent actually did), a Charter (what was planned), a TDE (what debt was
knowingly taken on). The lifecycle isn't left to the producer — charters drift
against the code, follow-ups get captured and promoted, audits run across
independent models, compliance maps to the EU AI Act and ISO 42001. StrayMark
*is* the "everything else." It's a fully-loaded answer to the question OKF
correctly declines to answer.

Here's the cleanest way to hold the two in your head:

> **OKF describes the system so an AI can understand it. StrayMark describes how
> the system was built so a human can trust it.**

A team can — and probably should — run both. OKF for the semantic layer over
their data. StrayMark to govern how that data platform gets built. They don't
overlap. They're not even pointed in the same direction: one faces the data, the
other faces the process.

## The part the spec didn't standardize, and we did

There's a sharper version of this, and it's hiding in OKF's link model.

In OKF, links are **untyped**. The spec is explicit: *"the specific kind of
relationship … is conveyed by the surrounding prose, not by the link itself."* A
link from concept A to concept B means *related, somehow* — read the paragraph
to find out how. That's a reasonable default for a minimal format.

But go read the comment thread under Karpathy's original gist. One reply
(pursultani's) makes a pointed critique: the LLM-wiki pattern defaults to
*reconciling* contradictions, quietly resolving tensions into a single
convergent story — and in many domains a contradiction *carries information* and
should be **preserved**, not flattened. Their proposed fix: typed edges in the
frontmatter, so a relationship can say *contradicts*, *supersedes*, *is an
alternative to* — not just *relates to*.

StrayMark already works that way. `supersedes` is a typed edge: this ADR
replaces that one, and the old one stays on disk, immutable, as preserved
history. `alternatives_documented` records the roads not taken. The entire
concept of a **Transversal Debt Entry** exists to write down an unresolved
tension and keep it visible instead of papering over it. Where the minimal
format chose untyped links and the gist's sharpest critic asked for typed ones,
StrayMark had already shipped typed, and built a graph engine that reasons over
the difference between *supersedes* and *relates-to*. Another reply on that same
gist — an implementation called *memwiki* — was built specifically to fight
*"Agent Amnesia,"* where agents forget architectural decisions. That is, word
for word, the problem StrayMark was founded to solve. The thread isn't a
competitor's roadmap. It's a list of the pattern's failure modes, several of
which we treated as requirements on day one.

## What we're going to do about it

Two things, neither of them defensive.

First, **interoperate.** Because OKF and StrayMark share the same primitives, the
distance from a StrayMark corpus to a conformant OKF bundle is small and mostly
mechanical — a type-name map, and rewriting our typed frontmatter references into
bundle-relative markdown links, which the reference-resolution work behind Loom
already does. A StrayMark project should be able to emit an OKF bundle of its
governance record, so anything that speaks OKF — including Google's own Knowledge
Catalog — can read it. And it should round-trip losslessly, because OKF promises
to preserve frontmatter keys it doesn't recognize, and our governance metadata is
exactly those keys.

Second, **Loom already renders this.** Loom — StrayMark's graph and architecture
visualizer — ingests markdown-with-frontmatter and builds a graph. An OKF bundle
is that same input. Loom's status overlays, graph analytics, and 3D architecture
view are a good deal richer than a static HTML graph. Pointing Loom at any OKF
bundle is a small adapter, not a rewrite.

## If you've read this far

The portable exercise this time is a question to ask about any "knowledge" your
team keeps for its AI agents. Two layers hide inside that word. There's the
*envelope* — the file format, the schema, how things link and ship — and OKF is
about to make that envelope a commodity, which is good for everyone. And there's
the *content* — what's actually worth writing down, in what types, under what
lifecycle, with what guarantees. A standard can give you the first layer. It
deliberately can't give you the second; the spec says so in a sentence. The
second layer is a position, taken on purpose, about what your project should
remember and be held to. That's not something to wait for a standards body to
hand you. It's the part that was always going to be left to the producer.

---

*Open Knowledge Format v0.1: [spec](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf) · [Google Cloud announcement](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing) · [the Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) it descends from. StrayMark's analysis lives in `experiment-okf/` in the repo.*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
