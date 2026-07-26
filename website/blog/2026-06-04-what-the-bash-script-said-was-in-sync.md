---
slug: what-the-bash-script-said-was-in-sync
title: What the bash script said was in sync
authors:
  - jose
tags:
  - straymark
  - follow-ups
  - governance
  - adopters
  - sentinel
  - lnxdrive
  - cli
date: 2026-06-04T00:00:00.000Z
description: Follow-ups became StrayMark's second first-class entity — and within a day, two external migrations stress-tested the design. The reference adopter's deprecated bash script had been reporting "in sync" while 29 entries sat invisible to it.
---
*The day after follow-ups became a first-class entity — schema, CLI namespace, shipped agent directives, an invocable skill — two adopters ran the migration end to end and filed their reports. One found the loop we hadn't closed. The other found something better: the deprecated bash script it was replacing had been silently blind for who knows how long, reporting "in sync" while 8 AILOGs and 29 entries sat unextracted. Both reports turned into releases the same day.*

<!-- truncate -->

> *"…despite the legacy script reporting **'in sync' as recently as the day before**, including `--scan-all` runs."*

That line is from Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225), the [Sentinel](https://github.com/StrangeDaysTech/sentinel) update report, and it's the sentence this post is named after. To explain why it matters — and why it's the strongest argument for the release lane it closed — we have to start with what a follow-up *is*.

## The pending-work problem

Every AILOG in StrayMark can end with a `## Follow-ups` section: the things this change deferred. Risks that surfaced mid-Charter get the same treatment — `R4 (new, not in Charter): …`. It's a write-time convention, and at write time it works perfectly: the implementer knows exactly what they're deferring, and they write it down next to the work that deferred it.

The failure mode is read time. *"What's pending across the project?"* is an operator question, and the per-AILOG convention answers it with a multi-file scan that gets worse with every Charter. The reference adopter hit the wall first: by Sentinel's CHARTER-12 the answer was 47 follow-ups deep and scattered across dozens of AILOGs, and the framework's response — [`fw-4.10.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.10.0), back in May — was a *convention*: one central registry file, `.straymark/follow-ups-backlog.md`, five buckets keyed on **when each entry becomes actionable**, and an adopter-side bash script (~296 lines of POSIX awk and grep) to detect AILOGs whose follow-ups hadn't been extracted yet.

That v0 lane worked — and then it accumulated exactly the kind of debt you'd expect from a convention maintained by hand at N=91. Issue [#214](https://github.com/StrangeDaysTech/straymark/issues/214) documented the signals from the operator's seat. The frontmatter counters, maintained manually, had drifted to fiction: declared `total_open: 47` against 65 real, four weeks after the last reconciliation. And every extraction batch carried 20–75% noise — entries that were *already resolved* when the script appended them, because the bullet text said so (`closed in-Charter`, `fixed in batch 3`) and the script couldn't read.

## First-class, with a gate

The response was [ADR-2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-03-001-followups-first-class.md): the follow-ups backlog stops being a convention and becomes StrayMark's second first-class artifact, the same lane Charter took — its own canonical path, its own schema, its own CLI namespace, its own group in the `explore` TUI.

[`fw-4.21.0` / `cli-3.19.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.19.0) shipped the substance:

- **A JSON schema (v1, experimental)** for the registry frontmatter, plus the v1 entry dimensions the operator had been improvising in prose: `Severity: blocking` (canonicalizing a `PROD-BLOCKER` notes convention), `Origin-class` (planning artifact vs execution reality), free `Labels`, a `Destination` vocabulary.
- **`straymark followups list / status / drift / promote`** — native, replacing the bash script. `drift --apply` extracts unextracted AILOGs per-AILOG (the granularity that produced 0 false positives across 76 AILOGs in the reference adopter), `promote` automates the FU → TDE elevation with traceability, `status` is the registry pulse.
- **CLI-owned counters.** The `total_*` fields are recomputed from actual entry statuses on every write. The silent counter-drift failure mode is not "discouraged" — it is mechanically dead.
- **Anti-noise extraction.** Bullets whose source text carries a closure marker land as `suspected-closed` instead of polluting the ready bucket as TBD noise. The operator confirms or reopens at triage.
- **Agent directives that travel.** `AGENT-RULES.md §13` ships with the framework: glance at the registry at session start, `drift --apply` in the same commit as the AILOG, triage at Charter close. Adopters stopped copying a block into their own agent configs.

And one deliberate brake, in the ADR itself: the schema ships as **experimental v1**. Hard stabilization is gated on a second adopter — the same N=2 discipline that [the last post](what-the-feature-flag-compiled-away) spent its middle section defending. One adopter's workflow, however well documented, is one stack's shape.

[`fw-4.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.22.0) completed the surface with the part that had to wait for a framework release: the `/straymark-followups` skill, in all four agent surfaces, wrapping the §13 directives. A thin wrapper by design — its `allowed-tools` deliberately omits `Write`, because every registry mutation goes through the CLI and the counters stay CLI-owned. The skill drives discipline; it owns no logic.

## Then both adopters showed up at once

What happened next is the part worth writing down, because it compressed the feedback loop the gate exists to wait for into a single day.

**[LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) adopted the registry cold** — first external adoption, 74 docs, Issue [#222](https://github.com/StrangeDaysTech/straymark/issues/222). The extraction itself was clean (cross-checked against a manual grep: zero missed AILOGs). Both findings were at the edges:

- **The loop we hadn't closed.** The lifecycle explicitly sanctions *manual* triage — the operator flips statuses by hand. But the only commands that recomputed the CLI-owned counters were `drift --apply` and `promote`, and both are no-ops when there's nothing to extract or promote. After a pure-triage session the registry sat with knowingly stale counters and no compliant way to fix them: hand-editing violates §13, and the `status` warning's promise ("the next write recomputes") was only true if a future write happened to occur. The sanctioned workflow and the invariant didn't meet.
- **A closure idiom we didn't speak.** Both extracted entries were born resolved — their source text said `Charter row updated atomically in this PR`. The anti-noise vocabulary knew `closed in-Charter`, `fixed in batch N`, and commit hashes; it didn't know this phrasing, so both entries landed as `open` TBD noise. The exact regression the refinement existed to kill, through an unrecognized idiom.

[`fw-4.23.0` / `cli-3.20.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.20.0) shipped the same day: a dedicated **`followups recount`** verb (recompute counters, touch nothing else, idempotent), `drift --apply` reconciling counters even with zero extractions, the **born-resolved idiom family** in the closure vocabulary — and, more durably, the canonical idiom table documented in the pattern doc, so AILOG authors converge on recognizable phrasings at write time instead of discovering unrecognized ones at extraction.

**Sentinel ran the official migration** — Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225): delete the script, `drift --scan-all --apply`, repoint the pre-commit hook. And the native parser immediately extracted 29 entries from 8 AILOGs that the bash script had reported as fully in sync — including in `--scan-all` runs — the day before.

The root cause, verified against the deleted script in git history, is the kind of thing that makes "deprecated" feel insufficient as a word. The awk extractor required both a `## Risk` section heading *and* the exact `- **R<N> (new` bullet shape. Those 8 AILOGs wrote their risks as bare paragraphs — `R4 (new, not in Charter): …` at column 0, no bullet, no heading. Format-sensitive matching, zero matches, no error. The AILOGs never even registered as *having* follow-up content. The script wasn't missing features. It was producing **silent false negatives on drift detection itself — the one thing it existed to prevent.**

The lenient parser caught all of them because leniency was a design decision, not an accident. `cli-3.19.1` had already widened status parsing for the same reason: operators annotate in place (`open — **OVERDUE** (…)`) and an exact-match parser would have demoted 4 of Sentinel's 65 entries to `unknown` and written wrong counters on the very first migration. Parse what humans actually write, then let the schema validation advise. Every formatting variant the strict tools dropped on the floor turned out to be load-bearing.

[`fw-4.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.23.1) closed the report: the migration paragraph in the pattern doc now mandates `--scan-all` on the first post-migration sweep *even if the legacy script said "in sync"* — citing the 8/29 data point as the reason. (Sentinel's other finding, the v0 → v1 upgrade not firing on a no-op `--apply`, had been fixed hours earlier by the LNXDrive lane — the two reports resolved each other's edges without knowing it.)

There was also a quiet confirmation buried in the Sentinel report, and it deserves the spotlight for a sentence. Mid-triage, with 29 entries being reclassified by hand, `status` printed *"frontmatter says total_open: 91 but the real count is 77 — run `straymark followups recount`"* — at exactly the right moment — and one command reconciled everything. The counter-drift failure mode that opened issue #214 is dead, and the adopter who reported it watched it die.

## What we still deliberately didn't do

The restraint section, because every post in this series has one and the discipline only counts if it survives good news.

Sentinel's report offered two more closure idioms (`Validated: … No vulnerabilities found`, `Bundled into CHARTER-19`) — one occurrence each. They are *noted*, not shipped. The pattern doc's new vocabulary section states the rule we're holding ourselves to: propose an idiom upstream when it **recurs**. A vocabulary that absorbs every one-off phrasing stops being a vocabulary.

The schema is still **experimental v1**. Two production migrations with zero schema-level findings is exactly the favorable signal the gate was built to measure — both reports were ergonomics and vocabulary, nothing structural — but the hard-stabilization call is a deliberate decision to take with a clear head, not a reflex to ship in the same week as the feature. And the `charter close` soft-integration (auto-running `drift --apply` at Charter close) stays gated behind a friction signal no adopter has sent.

## If you've read this far

The portable exercise this time is almost embarrassingly small. Pick the file — or the issue label, or the TODO convention — that answers *"what's pending?"* in your project. Now check two things. First: who maintains its summary numbers, a human or a tool? If a human, they're wrong; the only question is by how much (we measured: 18, after four weeks). Second: if your extraction tooling reported "nothing to do" today, would you know whether that's *true* or whether your last three deferred items just didn't match its regex? The difference between those two states is invisible by construction — that's what *silent* means — and the only way out is tooling that's lenient about what it reads and exclusive about what it owns.

And if you maintain a deferred-work convention we haven't seen — a different bucket vocabulary, a different closure idiom, a different answer to who owns the counters — the channel is the same one #214, #222 and #225 went through: [open an issue](https://github.com/StrangeDaysTech/straymark/issues). The v1 schema is experimental precisely because your registry is the one it hasn't met yet.

---

*StrayMark `fw-4.21.0` → `fw-4.23.1`, `cli-3.19.0` → `cli-3.20.0` — ADR [2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-03-001-followups-first-class.md) · Issues [#214](https://github.com/StrangeDaysTech/straymark/issues/214) · [#220](https://github.com/StrangeDaysTech/straymark/issues/220) · [#222](https://github.com/StrangeDaysTech/straymark/issues/222) · [#225](https://github.com/StrangeDaysTech/straymark/issues/225) · PRs [#217](https://github.com/StrangeDaysTech/straymark/pull/217) · [#218](https://github.com/StrangeDaysTech/straymark/pull/218) · [#221](https://github.com/StrangeDaysTech/straymark/pull/221) · [#223](https://github.com/StrangeDaysTech/straymark/pull/223) · [#227](https://github.com/StrangeDaysTech/straymark/pull/227). Pattern: [`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) (v1, experimental). Predecessors: [What the binary couldn't hide](what-the-binary-couldnt-hide) · [What the feature flag compiled away](what-the-feature-flag-compiled-away).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
