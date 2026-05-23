---
slug: what-the-binary-couldnt-hide
title: What the binary couldn't hide
authors:
  - jose
tags:
  - straymark
  - charters
  - governance
  - polish-charter
  - sentinel
  - anti-pattern
date: 2026-05-23T00:00:00.000Z
description: Ten latent gaps surfaced in a single polish Charter, and the anti-pattern that earned a name
---
*A polish Charter that was supposed to be cosmetic cleanup surfaced ten production gaps in six hours — including two features that had shipped to `main` and never functionally worked for ten days. The anti-pattern got named, the pattern got documented as `fw-4.18.0`, and the CLI helper got deferred on purpose.*

<!-- truncate -->

> *"The first thing the polish Charter tried — booting `./sentinel` from `main` to run the §5 smoke — failed with two distinct panics."*

That sentence opened Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199) on May 22, filed by the Sentinel adopter as an RFC. By the time the thread closed five comment-updates later, the count had grown from two to ten. The polish Charter — the closing Charter of Etapa 2, planned as a docket of WCAG audits and quickstart verification — had spent six hours doing something else: catching a class of latent regression that none of the per-Charter test suites of the eight previous Charters had caught, and that none of them *could* have caught given how they were written.

This post is the reconstruction of why that happened, what the recurring shape underneath it turned out to be, and the deliberate decision in PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) to **name the anti-pattern but not yet tool for it**. The cycle is familiar to readers who followed the previous posts on [emergent observation](emergent-observation-design) and [chain evolution](pattern-1-and-pattern-2-chain-evolution) — this is the third pattern in two weeks to crystallize via the same arc: surface in N=1, name the meta, defer the cross-project tooling until N=2 validates.

## The two cases that mattered

Of the ten gaps the polish session surfaced, eight were either dependency rot (a `huma` upgrade introduced a panic on `*string` parameters; Go 1.22 tightened `http.ServeMux` wildcard semantics) or runbook drift (env vars missing from §boot, smoke recipes that mixed mutually-exclusive modes, claims about a fake provider's stdout that the fake never wrote to). Each one merits a fix. None of them rewrites the playbook.

The two that did rewrite the playbook are different. They share a shape.

**US3 Preference Center, shipped May 12.** When recipients of a Sentinel-sent email click the unsubscribe-style footer link, they land on a JWT-in-path URL like `/preferences/<token>`. The handler is intentionally public-by-contract: the JWT *is* the auth; no `Authorization` header is expected. The handler's doc-comment said so. The integration test, written in `humatest`, mounted the handler directly through the testing adapter and confirmed: given a valid JWT in the path, the handler returns the right HTML. CI passed. The feature shipped to `main`.

What never happened: anything that exercised the production middleware chain in front of that handler. `internal/core/middleware/auth.go` has a `publicPrefixes` list of route prefixes that bypass the `Authorization` check. `/preferences/` was not in it. For ten days, every recipient who clicked an unsubscribe link got a `401 missing authorization header` from the middleware before the request ever reached the handler that knew not to expect one. The Preference Center was reachable in tests, unreachable in production.

**OTel observability instruments for the send pipeline, shipped the same week.** Eight metric instruments — counters, histograms, gauges — declared in `internal/core/metrics/commshub.go`, registered with the OpenTelemetry meter at boot. The Charter's `Constitution Check §IV` formally declared the FR-039..042 observability gate satisfied: the API layer was correct, the registrations went through, dashboards were planned. Seven of those eight instruments had **zero `.Add()` / `.Record()` call sites in the entire commshub module**. Declared, registered, never invoked. The dashboards that ops built on top of those series had been receiving zero data for ten days. The polish Charter's manual smoke — boot an OTel Collector, generate a few sends, grep collector output for the FR-039..042 names — surfaced this instantly.

Neither case is exotic. Neither case is a hard bug. They are both instances of the same mechanical mistake: an artifact got declared in one place, and the implementation that was supposed to wire it lived in another place, and nothing in the codebase or in CI correlated the two.

## The name the anti-pattern wanted

A name matters when the same shape shows up enough times to deserve one. By the third comment update on #199, the Sentinel author had counted four sub-classes of the same underlying mistake:

| Declaration site | Wiring site | Mechanical check |
|---|---|---|
| env var documented in the operator runbook | `os.Getenv(...)` (or stack equivalent) in code | each documented env var has at least one consumer |
| metric instrument declared in a metrics package | `.Record()` / `.Add()` call site in handler or worker code | each declared instrument is recorded at least once |
| URL referenced from rendered/embedded HTML (`<script src=...>`, `<link href=...>`) | route registered on the same API surface | each `src=`/`href=` in served HTML resolves to a registered route |
| route marked public-by-contract (doc-comment, dedicated marker) | entry in the auth middleware's public-prefix list | each public-by-contract handler has a matching prefix entry |

The unifier — the one-liner that the new governance doc opens with — is:

> *Every declared surface artifact has at least one wiring site reachable from a real request.*

The anti-pattern's name, which lands canonically in `dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md`, is **Surface declaration without wiring.** That is the deliverable. The polish Charter is the discovery vehicle, not the thesis.

## Why integration tests systematically miss this

This part is worth dwelling on because it shapes the rest of the post. The common failure mode across all four sub-classes is that the standard integration-test harness — `humatest.NewTestAdapter` in Go, the equivalent in TypeScript, Python, Rust — mounts handlers *directly* via the testing API. The handler under test is wired correctly by the fixture. The production composition step — where the route registration meets the middleware chain meets the env-var inventory meets the embedded asset table — is what's broken. CI's green light is genuine for what it claims: the handler returns the right response given the right request. It says nothing about whether the request can reach the handler in production, or whether the artifact the handler depends on was ever wired to the runtime.

There's a temptation to read this as a critique of `humatest`. It isn't. `humatest` (and its equivalents) is doing exactly what it was designed to do: let you test handler logic in isolation, fast, without standing up the whole composition tree. That isolation is a feature. The cost of the isolation is what we just named — and the cost only becomes visible when something outside the isolation surfaces a divergence. The polish Charter is the cheapest method to surface that divergence, because it does what no test fixture does: it boots the actual binary and runs the actual documented operator recipe against it.

This is the load-bearing claim of the new pattern doc. Not that polish Charters are valuable for cosmetic reasons. That they are **the only place** where the production composition gets exercised end-to-end against an externally-readable specification (the operator runbook). If you treat the polish Charter as cosmetic cleanup, you also treat that surfacing capacity as cosmetic. Sentinel's data argues the opposite.

## The decision: B′, not B

The RFC proposed three options. The decision in PR #200 was none of them as stated — call it B′. Worth saying why, because the structural choice matters more than it looks.

**The RFC's Option B** asked for a pattern doc under `docs/patterns/`. That directory does not exist in StrayMark today, and creating it would have split the project's documentation convention. The empirical patterns that already live in the canon — `FOLLOW-UPS-BACKLOG-PATTERN.md`, `CHARTER-CHAIN-EVOLUTION.md`, `EMERGENT-OBSERVATION-DESIGN.md`, `SPECKIT-CHARTER-BRIDGE.md` — all live in `dist/.straymark/00-governance/`. They share an i18n mirror infrastructure (every governance doc has English, Spanish, and Simplified Chinese siblings). Adding a third documentation surface would have multiplied the i18n overhead and pulled the project's patterns across two homes. Keeping the new doc inside `00-governance/` reused the existing infrastructure with zero marginal cost.

**The RFC's Option C** asked for a `straymark charter polish-checklist` or `straymark analyze declared-vs-wired` CLI helper. The Sentinel author had already prototyped one (CHARTERs 25/26/27, the three preparatory CI guards) and offered to seed it upstream. The decision here was conservative: defer. Not because the prototype isn't valuable — it is — but because Sentinel is N=1. The four sub-classes were the ones *one adopter, one stack* surfaced. The fifth, sixth, and seventh sub-classes that the framework would have to anticipate to ship a useful cross-project CLI don't exist yet, because no second adopter has pushed them. We've been down this road before. [`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) shipped as v0 in `fw-4.10.0` and has lived there for a month and a half, waiting for a second adopter to validate before graduating to a `straymark followups` subcommand. The same gate applies here. The new doc's `## Open questions` says so explicitly: "Crystallization as `straymark analyze declared-vs-wired` CLI subcommand … Gate: N=2 adopters."

**B′ is what landed.** A new governance pattern doc in the canonical location, in three languages. A seventh `Format conventions` bullet in the Charter template pointing authors at it when closing an Etapa or SpecKit `Polish` Phase. A row in the QUICK-REFERENCE `## Patterns` table. A `fw-4.18.0` bump with the footer cascade across thirty-some docs. No CLI subcommand. No new frontmatter field. No schema change. The anti-pattern has a name; the discovery ritual has a doc; the tooling waits.

## The arc that keeps repeating

Three weeks ago, the follow-ups backlog pattern crystallized via the same arc: an adopter surfaced a need, the pattern got named at v0, the CLI helper was deferred. Two weeks ago, the chain-evolution patterns crystallized: Pattern 1 (pre-declare SpecKit refresh) and Pattern 2 (post-close audit-driven Batch N.4), surfaced from the same Sentinel adopter, named in `fw-4.16.0`, with `straymark charter refresh-suggest` as a soft helper rather than a hard gate. One week ago, the meta — `EMERGENT-OBSERVATION-DESIGN.md` — codified what made any of those observations possible in the first place: formal cross-referencing plus cultural permission. This week, this pattern.

That's four crystallizations in a month, all from the same adopter, all following the same shape: **surface in N=1 → name the meta → tool only after N=2**. The temptation when a pattern surfaces vividly — and ten production gaps in six hours is vivid — is to skip the name and go straight to the tool. The discipline is the opposite. The name does most of the work. The tool, when it comes, is parameterized by what at least two adopters have surfaced; built on N=1, it's an extrapolation from one stack and one team's failure modes, and it tends to ossify those choices into framework defaults that don't generalize.

There is one piece of this iteration that is genuinely new and worth flagging. The new pattern doc names a falsifiable prediction the Sentinel author already committed to publishing: the next Etapa's polish Charter, executed against the three preparatory CI guards that just landed in Sentinel CHARTERs 25/26/27, should surface roughly 80% fewer gaps. If that prediction holds, the case for graduating Option C from "open question" to "CLI subcommand" gets quantitative backing. If it fails — if the next polish Charter still surfaces ten gaps but in a fifth, unanticipated sub-class — the failure itself is the next data point, and reshapes the spec before tooling is built. Either result is useful. Neither result has been observed yet. We wait.

## A note on what was deliberately not done

There is no `phase: polish` field in the Charter frontmatter. There is no `straymark charter polish-checklist <ID>`. There is no automated analyzer scanning Go (or any other language) code for declared-but-unwired symbols. The pattern doc lists each of those as a candidate evolution, gated explicitly on N=2 or on the post-Etapa-3 retrospective. None of them are present in `fw-4.18.0`.

This is intentional and worth saying out loud, because the natural reaction to a vivid finding is to over-build the response. Each one of those deferrals trades short-term thoroughness for long-term portability. A frontmatter field commits the framework to a vocabulary the next adopter may not share. A CLI helper commits the framework to a runtime that mirrors one specific stack's failure modes. An analyzer commits the framework to scanning a language whose conventions may diverge from the next adopter's. None of those commitments should be made on a single domain's signal, however clean the signal is. The pattern doc itself can be revised; a CLI subcommand cannot be unshipped without breaking adopters.

## What I'd suggest you look at, if you've read this far

If you operate a codebase with mock-adapter integration tests — which is most codebases — you can run a useful internal exercise without adopting anything from StrayMark at all. Pick the most recent feature your team shipped that has both (a) a docs-side declaration (env vars, OpenAPI specs, embedded HTML, metric instruments) and (b) a wiring site that lives in a different file. Boot the binary from a clean shell. Run the operator-facing recipe from the runbook end-to-end. If it works on the first try, your codebase doesn't yet have the latent debt this pattern catches — or it does and you got lucky on this one. If it doesn't, count the gaps. The number is the calibration you need for whether the polish-Charter-as-debt-detection ritual would be worth its overhead in your project.

If you've adopted StrayMark and you're closing an Etapa with handlers tested through `humatest`-style adapters, the new pattern doc has the four sub-class checks scoped for you. Budget the polish Charter as L, not XS or S. Expect emergent follow-on Charters, not residual cleanup scope creep. Read [`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md) before scoping the work, not after.

And if you're an adopter on a stack other than Go — TypeScript, Python, Rust, Elixir — the four sub-classes are language-agnostic, but the concrete check shape is not. Surfacing a fifth or sixth sub-class in your stack is exactly the second-adopter signal the pattern needs to graduate from v0. The path from "interesting observation in your project" to "named in the StrayMark canon" runs through the same channel #199 went through: open an issue. The cost of opening it is low; the cost of leaving the pattern under-validated is real.

---

*StrayMark `fw-4.18.0` — Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199) · PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) · tag [`fw-4.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.18.0). Sentinel anchors: [`AIDEC-2026-05-22-001`](https://github.com/StrangeDaysTech/sentinel/pull/93) · [CHARTERs 25/26/27 PR](https://github.com/StrangeDaysTech/sentinel/pull/94).*

*This document was produced with assistance from generative AI tools (Claude 4.7); all responsibility for the content rests with the human author.*
