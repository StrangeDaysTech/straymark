# Baton Phase 2 — dry-run router dogfood report (Sentinel)

> **Charter:** [CHARTER-03-dry-run-router.md](CHARTER-03-dry-run-router.md) · **AILOG:** AILOG-2026-06-26-002
> **Run:** `straymark-baton route <sentinel> --dry-run` and `… classify`, read-only, `git status` unchanged.
> **Costs are illustrative** (built-in defaults; no `baton:` block in Sentinel). The saving is **relative**, not a bill.

## 1. What ran

The dry-run router classified the **762 work units StrayMark already recorded** in
Sentinel — at the four existing granularities (no new vocabulary) — recommended a
tier for each, and computed the §4.2 verdict per granularity. No agent was
dispatched; no model was invoked.

| Granularity | Units |
|---|---|
| Charter | 45 |
| Batch | 82 |
| Follow-up | 135 |
| Task | 500 |
| **All** | **762** |

## 2. The headline economics (illustrative)

```
ALL (762 units) — routable
  tiers: economic 617 · frontier 14 · local 131
  cost: all-frontier 1293.60 → routed 93.68  (gross saving 1199.92 ≈ 93%)
  overhead 15.24 → net saving 1184.68
  caveats: 57% low-confidence · 57% of saving on low-confidence routing · 12% conflicted
  sensitivity: breakeven overhead/unit 1.575 · robust at 2× overhead: true
```

Every granularity is net-positive ("routable") and robust to 2× the illustrative
overhead. By the letter of the gate, routing pays at every granularity.

## 3. The honest reading — confidence, not granularity, is the constraint

The §4.2 verdict says "routable everywhere", but the honesty guards say the saving
is **fragile**:

| Granularity | conflict % | high+med confidence % |
|---|---|---|
| Charter | 6% | 46% |
| Batch | 8% | 39% |
| Follow-up | 5% | 37% |
| Task | 15% | 44% |

Two findings, both contrary to the original hypothesis:

1. **Heterogeneity is *not* the blocker.** The conjecture behind §10.4 was that a
   coarse unit (a Charter) bundles mixed work and so resists clean routing. The
   data inverts it: **Task** has the *highest* conflict (15%), Charter among the
   lowest (6%). The conflict metric is **confounded by title verbosity** — task
   titles are descriptive (more cues → more detected conflict), charter titles are
   terse (fewer cues → less). So conflict is a weak heterogeneity proxy, and there
   is no granularity that is meaningfully "more homogeneous" to prefer.

2. **Signal coverage is the binding constraint.** High+medium confidence sits at
   **37–46% across *all* granularities** — uniform. **57% of every granularity is
   Low-confidence**, dominated not by conflicts (5–15%) but by the *no-cue default*
   (45% of units surface no cue at all, B2). High confidence needs
   `effort_estimate`, which only Charters carry; most units classify on a terse
   title alone. **57% of the gross saving rests on those low-confidence guesses.**

**Empirical answer to §10.4:** *which granularity is routable?* — by net saving,
all of them; by trust, none is cleaner than another. **Granularity is not the
lever on this corpus. Signal coverage is.** The "instrument existing granularities"
decision stands; introducing a finer sub-unit (§4.3b) would not help — Task is
already the finest and is no more trustworthy.

## 4. Graduation-gate verdict

The charter gate: *net-positive relative saving after overhead at some granularity,
naming which; a net-negative result with evidence still graduates knowledge.*

**MET — and it graduates knowledge, which is the more valuable outcome here.**

- The dry-run establishes routing's **ceiling** (~93% illustrative saving) and that
  it survives 2× overhead — so the §4.2 economic principle is not violated by the
  classification overhead itself (overhead is 1.2% of gross saving).
- It establishes routing's current **trust floor**: only ~43% of units route at
  high+medium confidence, so ~57% of the saving is a guess.
- It **re-frames the next decision**: the path to a *trustworthy* saving is not a
  different routable unit, it is **wiring the deferred signals** (per-function
  complexity, architecture state, coherence findings) to lift confidence — exactly
  the calibration-gated refinements B2 deferred. That is now data-justified, not
  speculative.

## 5. Honest limitations & follow-ups

1. **Cheap signals under-cover.** 45% of units surface no cue; only Charters carry
   `effort_estimate`. **Follow-up:** wire the deferred signals (complexity needs
   `analyze` graduated from `cli` to `core`; arch_state from the Loom projection;
   coherence findings from Phase 1) and re-measure confidence. This is the lever.
2. **Costs are illustrative.** The 93% figure is shape, not money. Real
   provider-cost identity is deferred to Phase 3 (§10.8).
3. **Conflict is a weak heterogeneity proxy** (confounded by title length). A
   better within-unit heterogeneity measure would read the unit's *body/scope*, not
   just its title.
4. **Recommend-only by construction.** No model/network/agent dependency is linked;
   `route` requires `--dry-run`. Execution is Phase 3.

## 6. Status

Phase 2 (the dry-run router) is complete: unit inventory → cheap signals → cheap
classifier → tier policy + economic telemetry, validated read-only on a real
adopter corpus. `CHARTER-03-dry-run-router` closes with this batch. The economic
ceiling is real (~93%); the next move to make it *trustworthy* — wiring the heavier
signals — is now justified by data rather than asserted.
