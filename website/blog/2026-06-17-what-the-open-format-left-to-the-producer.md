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
date: 2026-06-17T00:00:00.000Z
description: Google published the Open Knowledge Format — markdown, YAML frontmatter, a graph of links, an agent that maintains them. StrayMark reached the same primitives independently. The convergence is real; so is the divergence — OKF builds cognition for the AI, StrayMark builds it for the AI and the human engineer together.
---
*On 2026-06-12 Google Cloud published the Open Knowledge Format: a directory of markdown files with YAML frontmatter, cross-linked into a graph, written and maintained by AI agents, with a static visualizer and a one-page spec. If that description gives you déjà vu reading this blog, it should — it's the shape StrayMark has shipped for months. We arrived there from our own ideas; OKF arrived crediting the Karpathy gist the whole "LLM-wiki pattern" traces back to. Two roads, same primitives. The honest reaction isn't defensiveness — someone with Google's reach just validated the bet. But validation of the shape is not agreement about its purpose, and that's where the only interesting question lives — the one their spec answers in a single sentence.*

<!-- truncate -->

> *"OKF requires exactly one thing of every concept: a type field. Everything else is left to the producer."* — the OKF v0.1 spec

That sentence is the whole post. Because **everything else — the part the spec hands back to you — is where the two projects stop being the same thing.**

## The convergence

Strip OKF and StrayMark to their mechanics and the parts list is identical. A
unit of knowledge is a markdown file with a YAML frontmatter block. The
frontmatter carries one required field — OKF calls it `type`, we call it the
document type. Documents link to each other, and those links form a graph that a
tool renders. The corpus is written and kept current by AI agents. Bundles are
just files: diffable, git-hostable, readable without tooling. There's a generator
that seeds the corpus by walking an existing system. There's a graph visualizer.

We didn't copy OKF and OKF didn't copy us — and, to be precise, we didn't get
here from Karpathy either. OKF credits his gist (itself a callback to Vannevar
Bush's 1945 Memex) as its headwater; StrayMark reached the same primitives from
its own ideas, without that lineage. That's exactly what makes the overlap worth
noting: when two projects starting from different places land on markdown +
frontmatter + graph + agent, the primitives aren't a fashion. They're the right
substrate.

So we're not going to argue with OKF's mechanics. They're ours too. We're going
to point at the place where identical parts get assembled into two machines that
do opposite jobs.

## Two readers, two paradigms

This is the heart of it, and it isn't a feature gap.

Ask the simplest question about any knowledge corpus: *who is it for?* OKF
answers, cleanly, *the AI.* Its job is to make an organization's data legible to
agents — to lift the cognitive burden of "knowing the system" off people and
encode it into a graph the model can instrument for itself. That's why the
announcement reaches for BigQuery to make the graph queryable: the implied
consumer doing the querying is the agent. And it fits the dominant current in the
industry right now — long-running autonomous agents, the ones that don't sleep,
that take a one-paragraph prompt and go work for days. In that world, the *less* a
human needs to be in the loop, the better the system is judged to be performing.
OKF is good infrastructure for that world.

StrayMark is built for a different bet about where this is going. Not the human
removed from the loop — the human given a defensible place *in* it. The corpus
exists so an engineer can know what was decided and why, what's underway, what's
still owed, and where the whole thing is heading — without having to hold the
entire codebase in their head at every step, and without being reduced to
rubber-stamping output they can't actually evaluate. It's the position Scott
Hanselman and others have been arguing under the name **AI-augmented
engineering**: the human isn't displaced, they're *repositioned* into the part
that is actually engineering — judgment, direction, accountability — and
amplified there by the AI. The opposite pole from vibe coding, and from the
fully-autonomous driver-agent that decides on its own what to do, where to go,
and when to stop.

So the two projects build cognition for different readers. **OKF builds cognition
for the agent. StrayMark builds cognition for the pair** — the agent and the human
engineer at once — and works to keep both oriented to the same map. Loom, our
(experimental) graph and architecture visualizer, is the clearest tell: its
status overlays and its 3D architecture view are *didactic aids for a person's
spatio-temporal location* in a project — you are here, this is active, this
carries debt, this is where the work is heading. You don't build that for an
agent. You build it for the human in the duo.

Read this carefully: none of it is about slowing the AI down. A map doesn't slow
you down — it's how you move fast without getting lost. The point is to make the
AI useful in a setting where the human keeps control and keeps *knowing*: the
what, the why, the what's-next, the what's-still-open.

## What got left to the producer

Now the spec's sentence reads differently. OKF is, by intent, *minimally
opinionated*. It standardizes the envelope — how a concept is shaped, how links
work, how a bundle ships — and stops. *Everything else is left to the producer.*
For OKF's job — data legible to agents — that's exactly the right call; the
envelope is the contribution.

But "everything else" isn't a grab-bag of optional extras. It's the entire
**human-facing layer**: which document types are worth keeping (an ADR — a
decision and why; an AILOG — what the agent actually did; a Charter — what was
planned; a TDE — debt knowingly taken on), under what lifecycle (charters drift
against the code, follow-ups get captured and promoted, audits run across
independent models, compliance maps to the EU AI Act and ISO 42001), with what
guarantees. That layer isn't data for an agent to consume. It's **knowledge — not
just information — put in front of a person so they can make decisions.**
StrayMark is maximally opinionated about exactly that. It *is* the "everything
else," on purpose.

The cleanest way to hold the two in your head:

> **OKF describes the system so an AI can understand it. StrayMark describes how
> the system was built so a human can trust it — and stay in command of it.**

Could you extend OKF outward until it did StrayMark's job — bolt on types, a
lifecycle, a human-facing visualizer? Yes. But you'd be rebuilding, externally,
something that already exists and already does it well. The honest relationship
isn't competition or replacement. It's that the two face opposite directions: one
toward the data and the agent that reads it, the other toward the process and the
human accountable for it. A team can run both. They don't overlap — and where they
appear to, StrayMark already covers the ground.

## The part the spec didn't standardize, and we did

There's a sharper, more concrete version of the same divergence, hiding in OKF's
link model.

In OKF, links are **untyped**. The spec is explicit: *"the specific kind of
relationship … is conveyed by the surrounding prose, not by the link itself."* A
link from A to B means *related, somehow* — read the paragraph. Reasonable for a
minimal format whose reader is a model that can read the paragraph.

But read the comment thread under Karpathy's original gist — the lineage OKF
descends from — and you find the pattern's own critics naming its failure modes.
One reply (pursultani's) makes the point: the LLM-wiki pattern defaults to
*reconciling* contradictions, quietly resolving tensions into a single convergent
story — and in many domains a contradiction *carries information* and should be
**preserved**, not flattened. The proposed fix: typed edges in the frontmatter,
so a relationship can say *contradicts*, *supersedes*, *is an alternative to* —
not just *relates to*.

Preserving a contradiction instead of flattening it is, precisely, a
human-in-the-loop move: a tension kept visible is a tension a person can be asked
to *decide*. StrayMark already works this way. `supersedes` is a typed edge: this
ADR replaces that one, and the old one stays on disk, immutable, as preserved
history. `alternatives_documented` records the roads not taken. The whole concept
of a **Transversal Debt Entry** exists to write down an unresolved tension and
keep it in front of someone, instead of papering over it. Another reply on that
gist — an implementation called *memwiki* — was built specifically to fight
*"Agent Amnesia,"* agents forgetting architectural decisions. That is, word for
word, the problem StrayMark was founded to solve — which we came to on our own,
and which the thread happens to corroborate. The thread isn't a competitor's
roadmap. It's a catalogue of the pattern's failure modes, and several of them are
things we treated as requirements on day one — because we were building for a
reader who has to live with the consequences: a person.

## What we're going to do about it

Two things, neither defensive.

First, **interoperate.** Because the primitives are shared, the distance from a
StrayMark corpus to a conformant OKF bundle is small and mostly mechanical — a
type-name map, and rewriting our typed frontmatter references into bundle-relative
markdown links, which the reference-resolution work behind Loom already does. A
StrayMark project should be able to emit an OKF bundle of its governance record,
so anything that speaks OKF — including Google's own Knowledge Catalog — can read
it. And it should round-trip losslessly: OKF promises to preserve frontmatter keys
it doesn't recognize, and our governance metadata is exactly those keys. We meet
OKF on its own envelope and lose nothing of ours.

Second, **Loom already renders this.** Loom ingests markdown-with-frontmatter and
builds a graph; an OKF bundle is that same input. The difference is what Loom does
with it — status overlays, graph analytics, a 3D architecture view, all of it
pointed at orienting a human. Pointing Loom at any OKF bundle is a small adapter,
not a rewrite. It's also the most direct demonstration of the thesis: take a
corpus designed to be read by an agent, and render it so a person can stand inside
it.

## If you've read this far

The portable exercise this time isn't about file formats. It's one question to
ask about any "knowledge" your team keeps for its AI: **who is it for?** If the
answer is *the agent alone* — so it can work longer without you — then a minimal,
standard envelope like OKF is most of what you need, and that envelope is about to
become a commodity, which is good for everyone. If the answer is *you and the
agent together* — so a person stays oriented, in command, and able to make the
calls that are theirs to make — then the envelope was never the hard part. The
hard part is deciding what your project should remember, in what types, under what
lifecycle, and who all of it is meant to keep in the loop. A standards body can
hand you the envelope. It deliberately can't hand you a position on the human's
place in the work. *That* was always going to be left to the producer.

---

*Open Knowledge Format v0.1: [spec](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf) · [Google Cloud announcement](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing) · [the Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) OKF descends from. StrayMark's analysis lives in `experiment-okf/` in the repo.*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
