# Polish Charter Pattern — StrayMark

> The closing Charter of an Etapa (or SpecKit `Polish` Phase) is the load-bearing gate for detecting a recurring anti-pattern — **"Surface declaration without wiring"** — that user-story Charters' test suites systematically cannot catch.

**Languages**: English | [Español](i18n/es/POLISH-CHARTER-PATTERN.md) | [简体中文](i18n/zh-CN/POLISH-CHARTER-PATTERN.md)

---

## Status

**v1 — validated in N=2 independent domains.** Two axes, deliberately reported
separately so they are not conflated:

- **Independent domains: 2.** `StrangeDaysTech/sentinel` (Go backend, CHARTER-19 → CHARTER-27, 2026-05-22) and `StrangeDaysTech/lnxdrive` (Rust Linux cloud-sync daemon + GTK desktop, 2026-05, [finding #209](https://github.com/StrangeDaysTech/straymark/issues/209)). A Rust desktop app validating a pattern first seen in a Go backend is the strong cross-domain signal the [N-status gate](../../../ADOPTERS.md) requires.
- **Occurrences: 3.** Sentinel surfaced the original sub-classes (1–4); LNXDrive surfaced a qualitatively new occurrence — a *cross-component regression of a shipped mitigation* (sub-class 5 below).

The N=2 gate for CLI crystallization (mirrored from [`FOLLOW-UPS-BACKLOG-PATTERN.md`](FOLLOW-UPS-BACKLOG-PATTERN.md)) is **now crossed**. The convention + named anti-pattern remains the portable core; the mechanical check graduates to a `straymark analyze declared-vs-wired` subcommand (config-driven set-difference, v0 scope ships in cli-3.17.x+ — see [Open questions](#open-questions)). Adopters can still reproduce the discovery locally with a dedicated polish Charter and (optionally) project-local CI guards.

---

## When this pattern applies

The closing Charter of an Etapa (in SpecKit vocabulary: the `Polish` Phase that follows Foundation + N user-story Phases — see [`SPECKIT-CHARTER-BRIDGE.md`](SPECKIT-CHARTER-BRIDGE.md)) is often treated as cosmetic cleanup: WCAG audits, copy fixes, deferred-but-not-blocking residue. The empirical signal from the reference implementation is stronger — when the polish Charter is the **first place** the end-to-end binary is exercised against the documented operator runbook, it surfaces a class of latent regressions that integration-test harnesses with mock adapters (e.g. `humatest`, `gomock`, in-memory event buses) systematically bypass.

Adopt this pattern (treat the polish Charter as a load-bearing debt-detection gate, not as cosmetic cleanup) when **any** of these conditions hold:

- The Etapa shipped ≥3 handlers whose integration tests use a mock adapter that bypasses the composed-app boot path (the real router, the real middleware chain, the real env-var inventory).
- The Etapa introduced ≥1 surface artifact whose declaration site and wiring site live in **different files or different modules** (e.g. a metric instrument declared in `metrics/` and recorded in `handlers/`; an env var documented in a quickstart and read in a fake adapter; an HTML embed that references a route registered elsewhere).
- The operator runbook (`quickstart.md` §boot, §smoke, §verification) documents commands that have never been executed end-to-end against the binary built from the Etapa's HEAD.
- The Etapa closes a SpecKit `Polish` Phase that earlier Phases deferred work into via annotations like `(T103 polish)`.

Below these thresholds (Etapa with no mock-adapter integration tests, no cross-module declaration/wiring split, runbook continuously exercised in CI), the per-Charter test suites alone are usually sufficient — adopting this pattern earlier than needed adds polish-Charter overhead without payoff.

---

## Shape

### The named anti-pattern: Surface declaration without wiring

The pattern's core deliverable is a **named anti-pattern** that polish Charters consistently surface and that is reducible to a mechanical check:

> **Surface declaration without wiring** — when one part of a feature contract (docs, public API, metric registration, HTML template body, public-route marker) advertises a behavior that another part (env-var consumer, handler invocation, instrument record call, route registrar, prefix list) was supposed to implement but didn't. The declaration site and the wiring site live far apart in the codebase. Neither tooling nor review process correlates them. CI tests each side in isolation and passes green.

The polish Charter is the **discovery vehicle**: it is the cheapest method to surface this class of regression because it exercises the documented runbook end-to-end against the binary, not against a test harness that has been wired to the declaration site directly.

### Four generalized sub-classes

The anti-pattern presents in at least four sub-classes. Each maps to a mechanical check that can be encoded in CI (per-project today; a candidate for cross-project CLI tooling in a future v1 — see [Open questions](#open-questions)). The list is intentionally agnostic of language and runtime; concrete instantiations vary by stack:

| # | Declaration site | Wiring site | Mechanical check |
|---|---|---|---|
| 1 | Env var documented in operator runbook (`quickstart.md`, `deploy/README.md`) | Env-var consumer in application code (`os.Getenv`, `process.env`, `ENV[]`) | Each documented env var has at least one consumer site |
| 2 | Metric instrument / observability symbol declared in a metrics package | Record / increment call site in handler or worker code | Each declared instrument has at least one record-call site |
| 3 | URL referenced from rendered HTML or embedded template (`<script src="/...">`, `<link href="/...">`) | Route registered with the same API surface | Each `src=`/`href=` in served HTML resolves to a registered route |
| 4 | Route marked public-by-contract (handler doc-comment, dedicated marker) | Entry in the auth middleware's public-prefix / public-paths list | Each public-by-contract handler has a matching prefix entry |
| 5 | Client-side IPC/RPC proxy method declared (D-Bus proxy, gRPC stub, REST client) — **especially one re-introduced after a shipped mitigation removed the server method** | Server / daemon interface that actually implements the method | Each declared proxy method resolves to an implemented interface method; a cross-component API change must update **all** consumers |

The unifying one-liner across the sub-classes is:

> **Every declared surface artifact has at least one wiring site reachable from a real request.**

Adopters extending the list (new declaration↔wiring pairs the reference implementation has not yet surfaced) are welcome to contribute additional sub-classes via issue or PR.

### Sub-class 5 named: shipped-mitigation regression via an un-updated downstream consumer

LNXDrive surfaced sub-class 5 as a *regression of an already-shipped mitigation, across a component boundary* — a sharper data point than a fresh gap. The producer (a D-Bus daemon) had closed a security risk by removing a token-bearing method and shipping a token-safe replacement. A separate component (a GTK preferences client, compiled via a different build system) **kept calling the removed method** and fetching tokens client-side — the exact behavior the mitigation had eliminated.

Two compounding factors made it invisible to every existing backstop:

- **Cross-boundary blindness.** Producer and consumer live in different crates, built by different toolchains (Cargo vs Meson), joined only at runtime over the bus. zbus/D-Bus proxies are validated at *runtime*, not compile time — so the daemon's own tests passed, the client compiled clean, and no single test suite spanned the contract.
- **Feature-gated dead code.** The stale call sat behind a `#[cfg(feature = "goa")]` whose feature `Cargo.toml` never defined. It compiled *out* entirely — dead code that defeats both CI and code review, since neither exercises an undefined feature. Activating the feature for the first time even surfaced a latent type error that had never compiled: concrete proof the path was never wired.

The legible signal that caught it was the polish/audit's **ex-ante contract check** — a diff of the client's declared proxy methods against the daemon's implemented interface. This generalizes the mechanical check from "every declared surface artifact has a wiring site" to its cross-component corollary: **a producer-side API change must update, or at least account for, every declared consumer of that API.** The Charter discipline that operationalizes this lives in [the template guidance](#related) (#209.c): a mitigation touching a cross-component API lists *all* consumers in `## Files to modify`, so a producer change can't silently orphan a consumer.

### Why integration tests miss these

The common cause across the four sub-classes is that the standard integration-test harness mounts handlers directly via the testing API (`humatest.NewTestAdapter`, equivalent in other stacks), bypassing the composition step where the declaration and wiring are joined. The handler under test is wired correctly *by the test fixture*; the production composition is what is broken. CI's green signal reflects "the handler behaves correctly given a request" — not "the request can reach the handler" nor "the declared artifact is reachable from production".

The polish Charter's manual smoke (`./binary && curl <documented-recipe>`) re-introduces the composition step, and surfaces the gap at the first sub-class instance it touches.

---

## Adoption walkthrough

For an adopter closing an Etapa for the first time using this pattern:

1. **Declare a polish Charter** that is scoped explicitly to (a) executing the documented operator runbook end-to-end against `./binary` built from the Etapa's HEAD, and (b) verifying each of the four sub-classes above against the artifacts the Etapa introduced. Budget the Charter as **L** (not XS/S/M) — empirical evidence from the reference implementation is ~10 gaps surfaced per first-time polish session.
2. **Expect emergent follow-on Charters**, not residual scope creep. Each gap the polish Charter surfaces gets a dedicated follow-on Charter (e.g. server-boot fix, auth-middleware fix, fake-provider implementation, instrument record-call wiring). The polish Charter does not absorb them — it triages them.
3. **Update the operator runbook atomically** with documentation gaps (env vars missing from §boot, smoke shapes that don't match the implementation, claimed-but-absent behaviors in fake adapters). The runbook is the test specification; if it is wrong, both the binary and the docs lose alignment.
4. **At Etapa close, file a retrospective** ([`AIDEC`](../../../docs/contributors/WHAT-IS-A-CHARTER.md) or equivalent) that classifies the surfaced gaps by root cause: ambient dependency rot, documentation drift, or "surface declaration without wiring". The cleaner cut is load-bearing for predicting which CI guards (if any) would have caught each class at PR time.
5. **Optionally land CI guards** for the sub-classes most prevalent in the Etapa. The reference implementation landed three: a full-chain boot test (catches sub-classes 3+4 of the runtime variety), a declared-vs-wired analyzer (sub-classes 1+2 statically; 3+4 dynamically), and an operator-runbook smoke test (catches runbook drift). The analyzer is the most portable; the boot test is project-specific in shape.

For an adopter on subsequent Etapas: same flow, with the prediction that the gap count per polish Charter drops as the project-local CI guards mature and as engineers internalize the four sub-classes.

---

## Reference implementation

`StrangeDaysTech/sentinel` CHARTER-19 (polish Charter, May 2026) → CHARTER-27 (post-AIDEC CI guards):

- The polish Charter session surfaced **10 distinct latent gaps** in ~6 hours, spawning 5 follow-on Charters (CHARTERs 20/21/22/23/24) plus 3 deferred follow-ups. Two of the gaps were features that had shipped to production and never functionally worked (US3 Preference Center 401-looping for 10 days; 7 OTel instruments declared and never recorded for 10 days).
- The root-cause retrospective is [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93) ("adopt polish-Charter-as-debt-detection pattern + 3 preparatory CI guards for Etapa 3"). It classifies the 10 gaps by category (ambient rot, documentation drift, surface declaration without wiring) and commits Sentinel to landing three CI guards before opening the next Etapa.
- The three CI guards landed as [sentinel#94](https://github.com/StrangeDaysTech/sentinel/pull/94) (CHARTERs 25/26/27): full-chain boot test, declared-vs-wired multipass analyzer (sub-class 2 fully wired; sub-classes 1+3+4 stubbed for follow-up), operator-runbook smoke test.

The reference implementation includes a falsifiable prediction: the next-Etapa polish Charter will surface ~80% fewer gaps. Validation of that prediction (or the new gap categories surfaced if the prediction fails) is the natural empirical trigger to revisit this pattern's `v0 → v1` graduation.

The originating RFC discussion is [straymark#199](https://github.com/StrangeDaysTech/straymark/issues/199), preserving the empirical chain across five comment updates as the polish Charter session unfolded.

---

## Open questions

These are not resolved in v0. Future revisions of this pattern, or a CLI helper, may address them:

- **Crystallization as `straymark analyze declared-vs-wired` CLI subcommand** — *N=2 gate crossed; v0 scope resolved.* With LNXDrive validating the pattern in a second domain, the framework ships a **config-driven set-difference** v0: the operator supplies a declared-side glob+regex and a wired-side glob+regex (the regex capture group is the symbol name), and the command reports symbols declared-but-not-wired (`D \ W`). This is mechanically tractable on *any* stack precisely because the stack-specific knowledge lives in the adopter's regexes, not in the CLI — and it directly catches sub-class 5 (D-Bus proxy method names client-side vs interface method names server-side). **Deferred to a later revision:** AST-based variants of sub-classes 1–4 (env-var docs, metric instruments, HTML embeds, public-route markers), which need per-stack parsers; and the runtime/dynamic checks (full-chain boot, route resolution), which are inherently project-local.
- **Sub-class enumeration completeness**. The four sub-classes were the ones the reference implementation surfaced. Additional candidates: database column declared in a migration but never read/written by application code; feature flag declared but never checked; protobuf field defined but never serialized. Each additional sub-class needs at least one adopter empirical surfacing to enter the canon.
- **Integration with `straymark charter close --polish-checklist`**. A polish-specific subcommand could surface the canonical checklist (run runbook end-to-end; verify each declared artifact has a wiring site; verify env-var inventory matches the binary's actual requirements; verify CLI tooling referenced in the runbook exists). Gate: after the `declared-vs-wired` CLI subcommand lands, since the checklist's last item would invoke it.
- **Per-stack instantiation guides**. The four sub-classes are language-agnostic; the concrete check shape (Go `analysis.Pass`, TypeScript AST walker, Python `ast` module, etc.) is not. A future pattern revision may host per-stack reference implementations as sibling docs.
- **Effort budget calibration**. The reference implementation observed ~10 gaps per first-time polish Charter. The prediction is that this drops sharply as project-local guards mature. A v1 of this pattern may publish budget guidance derived from N≥2 datapoints (XS/S/M/L per Etapa maturity).

---

## Credits

Originated via [issue #199](https://github.com/StrangeDaysTech/straymark/issues/199) by the Sentinel adopter (N=1). Empirical foundation: CHARTER-19 → CHARTER-27 chain in `StrangeDaysTech/sentinel`, retrospective [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93).

Crystallized to **v1 (N=2)** via [finding #209](https://github.com/StrangeDaysTech/straymark/issues/209) by the LNXDrive adopter (Rust desktop, second independent domain), which contributed sub-class 5 (shipped-mitigation regression via an un-updated downstream consumer) and triggered the `analyze declared-vs-wired` subcommand. The companion [finding #210](https://github.com/StrangeDaysTech/straymark/issues/210) added the `charter new` reconnaissance discipline and the `CHARTER-FILES-EXIST` validate rule. Author: José Villaseñor Montfort.

*This document was produced with assistance from generative AI tools (Claude 4.7); all responsibility for the content rests with the human author.*

---

## Related

- [SPECKIT-CHARTER-BRIDGE.md](SPECKIT-CHARTER-BRIDGE.md) — defines the SpecKit `Polish` Phase that this pattern attaches load-bearing semantics to.
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — sibling v0 pattern proven in the same adopter; shares the N=1 → N=2 graduation gate for CLI crystallization.
- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) — meta-pattern that the polish Charter's debt-detection role instantiates at the Etapa-close surface.
- [AGENT-RULES.md](AGENT-RULES.md) — agent-side directives that govern how follow-up surfaces (`R<N> (new, not in Charter)`, TDE promotion) flow from polish Charter findings into the broader governance backlog.

---

*StrayMark fw-4.20.0 | [Strange Days Tech](https://strangedays.tech)*
