---
slug: what-the-feature-flag-compiled-away
title: What the feature flag compiled away
authors:
  - jose
tags:
  - straymark
  - charters
  - governance
  - polish-charter
  - anti-pattern
  - adopters
  - lnxdrive
date: 2026-05-31T00:00:00.000Z
description: A second adopter, in a different language, validated the "surface declaration without wiring" anti-pattern — and the CLI helper we deferred on purpose finally shipped
---
*A week after we named an anti-pattern and deliberately declined to tool for it, a second project — in a different language, on a different stack — surfaced it again, in a sharper form: not a fresh gap, but a regression of an already-shipped security mitigation, hidden in dead code behind a feature flag that was never defined. That was the N=2 signal the framework said it was waiting for. The deferred helper shipped.*

<!-- truncate -->

> *"It compiled clean — the GOA module was dead code, and zbus proxies are validated at runtime, not compile time, so nothing flagged it. Activating the feature for the first time even surfaced a latent type error that had never compiled."*

That sentence comes from Issue [#209](https://github.com/StrangeDaysTech/straymark/issues/209), filed on May 31 by [LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) — a Linux cloud-sync daemon and GTK desktop client written in Rust, FUSE and D-Bus and systemd all the way down. It is the second project in the [StrayMark adopters registry](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md), and the first in a domain genuinely unlike Sentinel's Go backend. That difference is the whole point of this post.

Eight days earlier, in [*What the binary couldn't hide*](what-the-binary-couldnt-hide), we documented an anti-pattern — **surface declaration without wiring** — that a polish Charter in Sentinel surfaced ten times in six hours. We named it, shipped the pattern doc as `fw-4.18.0`, and **deferred the CLI helper on purpose**, with the reason stated plainly: Sentinel is N=1. The four sub-classes were the ones one adopter, one stack had surfaced. Tooling built on one domain's failure modes tends to ossify those choices into framework defaults that don't generalize. The pattern doc's `## Open questions` set the gate in one line: *"Crystallization as `straymark analyze declared-vs-wired` CLI subcommand … Gate: N=2 adopters."*

This post is what happened when N=2 arrived — and why the occurrence was a stronger data point than the dozen that named the pattern.

## The regression that compiled itself out

LNXDrive's Charter `CHARTER-01-road-to-v0-1-0-alpha-1` had a Fase 1 that closed a security risk, `RISK-002`. The shape of the fix is worth stating precisely, because the regression is its mirror image.

The authentication flow originally had the daemon expose a D-Bus method, `Auth.CompleteAuthWithTokens`, that carried OAuth tokens across the bus from the client. Tokens on the bus is the risk. Fase 1 closed it the right way: it **removed** that method and shipped `Auth.CompleteAuthViaGOA(account_path)` instead — the daemon now fetches the tokens itself from GNOME Online Accounts, daemon-side, and they never cross the bus. The mitigation shipped. The daemon's tests passed. `RISK-002` was closed.

Then Fase 3 audited the GTK4 preferences panel — a separate crate, compiled through Meson rather than the daemon's Cargo build. The panel **still called `complete_auth_with_tokens`**, and still fetched tokens client-side. The consumer had never been updated when the producer changed its contract. The mitigation had regressed across a component boundary, in the half of the system that the Fase 1 work never touched.

Here is why it stayed invisible for the gap between Fase 1 and Fase 3 — and why no test, no compiler, and no reviewer caught it:

- **The dead call was behind a feature flag that did not exist.** The client's GOA code path sat under `#[cfg(feature = "goa")]`. `Cargo.toml` never defined a `goa` feature. So the entire module compiled *out* — it was dead code. The crate built green because the broken path was never built at all. CI doesn't exercise an undefined feature; neither does code review, which reads the line and moves on. When the panel work in Fase 3 finally activated the feature for the first time, it surfaced a latent **type error that had never compiled** — the concrete, unfalsifiable proof that the path had never once been wired.
- **The boundary was triple.** Producer and consumer live in different crates, built by different toolchains (Cargo for the daemon, Meson for the panel), joined only at runtime over D-Bus. And zbus proxies are validated at *runtime*, not at compile time — so even with the feature on, the proxy declaring `complete_auth_with_tokens` would have compiled against a daemon interface that no longer offered it, and only failed when a real call hit a real bus.

Strip away the Rust and the D-Bus specifics and you have the same mechanical mistake the Sentinel post named: an artifact declared in one place (the client proxy method), the implementation that should back it living in another place (the daemon interface), and nothing — not the compiler, not CI, not review — correlating the two. The declaration site and the wiring site live far apart, and the gap between them is where the regression hid.

## Why this was sharper than the ten gaps that named the pattern

The Sentinel cases were *fresh* gaps: a Preference Center that 401-looped for ten days, seven OTel instruments declared and never recorded. Features that shipped and never functionally worked. Bad enough.

The LNXDrive case is qualitatively worse in a way that matters for the pattern's canon. It is a **regression of an already-shipped, already-audited mitigation.** `RISK-002` was closed correctly. The fix was real. And it silently drifted back out of compliance — not because anyone re-introduced the risky method, but because the *consumer* of the API was never updated when the *producer* changed it. The thing that made the regression legible wasn't a test. It was the audit's **ex-ante contract check**: a diff of the proxy methods the client declares against the interface the daemon actually implements.

This earns sub-class 5 in the pattern doc, and a name of its own: **shipped-mitigation regression via an un-updated downstream consumer.** The four original sub-classes were variations on "a declared surface was never wired." The fifth adds a sharper edge: "a declared surface *was* wired, the wiring was removed as a fix, and a different component kept calling the old contract."

A word on counting, because StrayMark crystallizes patterns by validation count and it's worth being honest about the arithmetic. There are two axes, and the pattern doc now reports them separately so they aren't conflated:

- **Independent domains: 2.** Sentinel (Go backend) and LNXDrive (Rust desktop). A Rust desktop app validating a pattern first seen in a Go backend is the strong cross-domain signal — far stronger than another Go backend would have been. This is the axis that gates CLI automation.
- **Occurrences: 3.** Sentinel's original surfacing, plus an earlier Fase 1 documentation drift inside LNXDrive, plus this cross-component regression.

The gate is the domain axis, and it is now crossed.

## The gate we said we were waiting for

In the Sentinel post I wrote that the natural reaction to a vivid finding is to over-build the response, and that the discipline is the opposite: name the meta, defer the tool until at least two adopters have surfaced its shape. I also flagged the specific risk — *"A CLI helper commits the framework to a runtime that mirrors one specific stack's failure modes."*

That risk is exactly what the second domain lets us route around. The helper that shipped — `straymark analyze declared-vs-wired`, in [`cli-3.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.18.0) — does not encode Go, or Rust, or D-Bus, or HTTP. It is a **config-driven set difference.** You give it a *declared* side and a *wired* side, each as a `(glob, regex)` pair, and the capture group of each regex is the symbol name. It reports the symbols that appear on the declared side and not on the wired side:

```bash
straymark analyze declared-vs-wired \
  --declared-glob "client/**/*.rs"     --declared-pattern 'fn (\w+)' \
  --wired-glob    "daemon/**/*.rs"      --wired-pattern   'fn (\w+)'
```

Run against the LNXDrive layout, that flags `complete_auth_with_tokens` — declared in the client proxy, absent from the daemon interface — and exits non-zero. The stack-specific knowledge lives entirely in the adopter's two regexes, committed once as a named profile in `.straymark/config.yml`. The framework supplies the set-difference machinery; the adopter supplies the meaning of "declared" and "wired" for their stack. That is the design that N=1 couldn't have produced: with one domain, you can't tell which parts of your tool are essential and which are accidental to that stack. The second domain is what separates them.

This is the whole argument for the gate, observed in the wild rather than asserted. Had we shipped the helper at N=1 — say, an analyzer that walked Go ASTs for declared-but-unrecorded OTel instruments — it would have been useless to a Rust project talking over D-Bus, and we'd have spent a release cycle generalizing a thing we built too early. Waiting cost us eight days and bought us a tool whose v0 surface is honestly cross-stack.

## The companion finding, and the cheaper backstop

LNXDrive filed a second issue the same day, [#210](https://github.com/StrangeDaysTech/straymark/issues/210), and it's the quieter but more broadly useful of the two. It observed that the Charter's `## Files to modify` section had been written against *assumed* code, not *read* code — and was wrong repeatedly. `RISK-002` declared a new `dbus_iface.rs` and an opaque `SessionHandle` type; neither existed, and the fix shipped in the already-present `service.rs`. `ISSUE-002` named `lnxdrive-config/src/parser.rs`; there is no such crate, and the real parser is `lnxdrive-core/src/config.rs`. A CI-hardening item targeted an engine whose workflow lived at a subdirectory path GitHub Actions silently ignores — it had never run. Each row became a documented "premise correction" during execution.

That's the framework working as a backstop: the ex-ante declaration is what made the later drift *legible*. But the root cause is upstream of drift — it's an authoring error, a path that never existed, not an implementation that diverged. Those are different failures and conflating them adds noise. So `cli-3.17.0` added a validate rule, `CHARTER-FILES-EXIST`: when a `## Files to modify` row names a path that isn't on disk and isn't tagged as newly created, `straymark validate --include-charters` warns. It's warn-only, pure-Rust (so it works without the bash the drift check needs), and — this is the point #210 asked for — it lives in a *different command* from `charter drift`. A path that never existed is a Charter mis-declaration to fix in place; a path declared and not modified is implementation drift to reconcile. Two failures, two rule codes, two commands. The Charter template now also tells authors, in the comment above `## Files to modify`, to read each path before declaring it — and, when a Charter touches a cross-component API, to list *all* the consumers, not just the producer. That last line is the discipline that would have caught the GOA regression at authoring time, before any code was written.

## What we still deliberately didn't do

The same restraint as last time, in a different place. `analyze declared-vs-wired` ships **only** sub-class 5 — the IPC proxy-vs-interface check, the one LNXDrive actually surfaced and the one that is mechanically tractable across stacks with nothing but two regexes. The AST-based variants of sub-classes 1 through 4 — walking code for env-var consumers, metric record sites, HTML route resolution, public-route prefixes — are still deferred, and the pattern doc still lists them under `## Open questions`. They need per-stack parsers, and a set-difference over regex captures is not that. The dynamic checks — booting the binary, resolving routes at runtime — remain inherently project-local. We crossed the gate that the config-driven check needed; we did not use the crossing as license to build everything the pattern could eventually justify.

There's a smaller deferral worth naming too. We considered a hard `recon: confirmed` frontmatter field on Charters — a box the author ticks to assert they read the tree before declaring it. We didn't ship it. A required field breaks non-interactive and skill-driven Charter creation, and it creates a lie surface: agents will tick it reflexively. The soft nudge in `charter new`'s output plus the mechanical `CHARTER-FILES-EXIST` backstop does the same work without the failure mode. If the soft version proves insufficient across more adopters, that's the signal to revisit — the same gate logic, one level down.

## If you've read this far

The portable exercise from the last post still stands, and the second domain extends it. If you run a system split across a producer and a consumer joined at runtime — a daemon and a client, a service and an SDK, a server and a generated stub — write down the methods the consumer *declares* it will call, and the methods the producer *actually implements*, and diff the two sets. You don't need StrayMark to do it; `grep` and `comm` will get you most of the way. The set difference in one direction is the regression this post is about. The set difference in the other direction is dead surface area you can delete. Either way the number is information you didn't have.

And if you operate on a stack we haven't seen yet — and at N=2, that's still most stacks — the path from "interesting observation in my project" to "named in the StrayMark canon" runs through the same channel #199 and #209 both went through: open an issue. Sub-class 6 is out there in some codebase right now, declared in one file and wired in another, waiting for the adopter who'll be the one to surface it.

---

*StrayMark `fw-4.20.0` + `cli-3.17.0` (Release A) and `cli-3.18.0` (Release B) — Issues [#209](https://github.com/StrangeDaysTech/straymark/issues/209) · [#210](https://github.com/StrangeDaysTech/straymark/issues/210) · PRs [#211](https://github.com/StrangeDaysTech/straymark/pull/211) · [#212](https://github.com/StrangeDaysTech/straymark/pull/212). Pattern: [`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md) (v1, N=2). Predecessor: [*What the binary couldn't hide*](what-the-binary-couldnt-hide).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
