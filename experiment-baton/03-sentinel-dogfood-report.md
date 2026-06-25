# Baton Phase 1 — Sentinel dogfood report (graduation gate)

> **Version:** 1.0 · **Date:** 25 June 2026 · **Batch:** B5 of [CHARTER-01-coherence-bridge](CHARTER-01-coherence-bridge.md)
> **Target:** Sentinel (`/home/montfort/StrangeDaysTech/sentinel`), HEAD `24d5a66`, read-only.
> **Issue:** #304. **Scope rule:** no models, no mutation.

## 1. What was run

The full Coherence Bridge, **read-only**, against the real Sentinel repo:

```
straymark-baton coherence /…/sentinel      # findings C1–C4
straymark-baton overlay   /…/sentinel      # intent vs emergent vs code
```

## 2. Read-only verified (NFR1)

`git status --porcelain` in Sentinel was **empty before and after** both runs;
HEAD unchanged (`24d5a66`). The bridge mutates nothing in the target repo.

## 3. Result — the #304 class is caught, cleanly

`coherence` emits **6 findings (0 blocking)** — down from **90** before the B5
calibration (see §4). The headline is one **precise** C4:

```
[C4] spec '005-frontend-dashboard' consumes contract 'services.public-visibility'
     but never references its defining decision(s): PM-001, AILOG-2026-04-21-002
```

This is exactly the issue #304 pattern on real data: a consumer spec depends on a
contract a decision in **another** spec defined, without acknowledging it — the
cross-spec decision-propagation drift that nothing previously surfaced. It is a
real finding human review had not flagged, caught structurally and read-only.

The other 5 findings are **info-grade C1 hints** (memory-derived, low confidence
by design — R1):

| Component | Reality |
|---|---|
| DevPortal | designed, **no module** — a real gap |
| UsageGuard | designed, **no module** — a real gap |
| SentinelAgent | memory says `SentinelAgent`, the module dir is `agents/` — a **naming drift** |
| Integración LLM | an architectural concept, not a discrete module — noise |
| Monolito Modular | the whole-app pattern, not a module — noise |

`overlay` renders the legible three-plane view, e.g.:

```
✓ Policy Engine — intended & implemented      ← the past gap is now closed
✓ Status Center / Identity / CommsHub / …     ← intended & implemented
! DevPortal / UsageGuard / …                  ← intended, NOT implemented
? Agents (AIOps) / Core / Database / …        ← implemented, NOT intended
```

Notably **PolicyEngine now shows `intended & implemented`** — the module the team
once forgot and dispersed has since been built, and the tool correctly does *not*
flag it. The bridge reflects current reality, not a stale anecdote.

## 4. Calibration applied in B5 (90 → 6 findings)

The first raw run produced 90 mostly-noise findings. Targeted fixes, each a real
precision bug surfaced only by real data:

1. **Precise decision→contract linkage** — a backlog decision now defines only the
   contracts its *own section* names (via the endpoint it cites), not every
   endpoint its spec mentions. Collapsed C4 from **84 → 1**.
2. **Test files are not producers** — excluded `*_test.go` / `*.test.*` / `*.spec.*`
   / `*.d.ts`. Removed a bogus C2 whose "producer" was a `mockService` in a test.
3. **C1 → info, low confidence, first-word matching** — `Identity Module` now
   matches `internal/modules/identity/` (was a false positive); memory-derived
   findings are clearly low-confidence hints, not blockers.
4. **C4 aggregation** — one finding per (spec, contract) listing all unreferenced
   decisions, instead of one per decision.

## 5. Graduation gate — MET

> *Phase 1 succeeds if the diagnostic, run read-only against Sentinel, catches at
> least one real drift (#304 and/or PolicyEngine) that human review let through.*

Met: the C4 above is a real, previously-unflagged #304-class cross-spec
decision-propagation drift, caught read-only on real data, with legible signal
(6 findings, not 90). The intent overlay additionally surfaces real
intended-not-implemented gaps (DevPortal, UsageGuard) and a naming drift
(SentinelAgent vs `agents/`).

## 6. Honest limitations & follow-ups

Phase 1 is a calibrated first cut, not a finished product. Known gaps:

1. **Contract keying on generated type files.** Sentinel's `web/src/api/types.gen.ts`
   holds *all* API types in one file with sparse endpoint anchors, so many
   interfaces collapse onto a few `ContractId`s. The #304 *field/enum-level* health
   mismatch (C2/C3) therefore isn't isolated on Sentinel the way it is on the
   fixture — it merges into a coarse `services` contract and is currently dropped
   (no non-test producer). **Follow-up:** map each generated type to its endpoint
   (response-type ↔ route), e.g. via an OpenAPI/codegen manifest, so C2/C3 fire per
   real contract. *This is why the Sentinel run has 0 blocking findings.*
2. **C1 is inherently fuzzy.** Free-form `.specify/memory` naming conflates real
   gaps with architectural concepts; kept info-grade. **Follow-up:** optional
   explicit component→path mapping (in memory or `model.yml`) to promote C1 to a
   trustworthy signal.
3. **EPIPE panic.** Piping `coherence` to `head` panics on broken pipe (cosmetic;
   CI redirects to a file). **Follow-up:** handle `SIGPIPE`/`EPIPE` gracefully.
4. **Activation seam.** Phase 1 is read-only detection. The next Charter is the
   *activation* seam — a SpecKit extension hooking `before_implement` so the
   coherence check runs at authoring time, not just on demand.

## 7. Status

Phase 1 (the read-side Coherence Bridge) is complete: SpecKit adapter → IntentModel
+ provenance → coherence engine (C1–C4) → Loom-consumable intent overlay, validated
read-only on a real adopter repo. `CHARTER-01-coherence-bridge` closes with this
batch.
