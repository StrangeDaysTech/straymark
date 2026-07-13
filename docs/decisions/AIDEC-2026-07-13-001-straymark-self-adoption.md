---
id: AIDEC-2026-07-13-001
title: StrayMark adopts StrayMark — scoped, lagged self-adoption with a distribution-source guard
status: accepted
created: 2026-07-13
agent: claude-opus-4-8
confidence: high
review_required: true

# --- Approval workflow (fill at review time via `straymark approve`) ---
# reviewed_by: <reviewer-id>
# reviewed_at: YYYY-MM-DD
# review_outcome: approved
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
tags: [self-adoption, dogfooding, governance, control-center, ouroboros]
related: [ADR-2026-06-03-followups-first-class]
---

# AIDEC: StrayMark adopts StrayMark — scoped, lagged self-adoption

## Context

StrayMark is a framework for AI-assisted software governance (Charters, AILOGs, AIDECs,
telemetry, the follow-ups registry, the external-audit cycle, the architecture model/Loom).
StrayMark itself is developed by AI agents. The natural question — the "ouroboros" — is whether
StrayMark should adopt StrayMark for its own development, especially as the implementation load
grows (the continuity of the Baton/Loom experiments and a forthcoming "Control Center").

Two facts frame the decision:

1. **The ouroboros already exists, half of it.** The experiments already practice the *discipline*
   by hand: `experiment-baton/` has 3 Charters + 9 AILOGs, `experiment-loom/` has 2 Charters + 10
   AILOGs — authored spontaneously, without an operator instruction or a discussed decision. What is
   missing is the *tooling half*: an installed `.straymark/`, the CLI run against this repo, and the
   full governance loop. `CLAUDE.md` already mirrors `STRAYMARK.md`'s git rules.

2. **A latent catastrophic ambiguity is mechanical and exists today.** `resolve_project_root` picks
   the closest `.straymark/`, and `dist/.straymark/` (the shipped framework *source*) is a valid one.
   So running any mutating command with cwd or `--path` inside `dist/` treats the distribution
   template as an installed project — writing AILOGs/AIDECs into `dist/`, validating the source as a
   project, reading governance from the source instead of an install. (Not yet realized:
   `dist/.straymark/07-ai-audit/` currently holds only `.gitkeep`.)

## Problem

Should StrayMark self-adopt, and if so, how — without the catastrophic confusion between the
framework **distribution source** (`dist/.straymark/`) and an **installed framework** (`/.straymark/`),
and without the framework's in-development state breaking its own development loop?

## Alternatives Considered

### Alternative 1: Full, live ouroboros

**Description**: Install `.straymark/` at the repo root pointed at the *live* `dist/` framework, and
gate development on self-validation (CI fails on `straymark validate`, etc.). The snake eats today's tail.

**Pros**:
- Maximum dogfooding; every framework change is exercised on the framework immediately.
- Tightest possible feedback loop.

**Cons**:
- **Bootstrap paradox / version-skew (catastrophic):** editing `dist/.straymark/…AGENT-RULES.md` while
  a self-install validates against those half-built rules means a bad framework commit can *brick the
  maintainer's own development loop* — the tool can't develop itself when the in-progress version fails
  on itself.
- **Meta-noise / hall of mirrors:** AILOGs about editing AILOG templates, telemetry about the telemetry
  schema — the human loses orientation, precisely what StrayMark exists to prevent (and what the OKF
  analysis identified as StrayMark's core differentiator to protect).
- Double bookkeeping of framework files that duplicate the source.

### Alternative 2: Scoped, lagged self-adoption (with safeguards first)

**Description**: Install `.straymark/` at the root **pinned to the last released framework** (not live
`dist/`) — the snake eats *yesterday's* tail. Scope the loop to the high-benefit / low-coupling governance
layer (Charters + AILOGs + follow-ups registry + architecture model/Loom) applied to **new heavy work**
(the experiments' continuity and especially the Control Center). Keep validation **advisory, not a gate**.
Exclude framework-meta-work from the loop. Preserve Sentinel/lnxdrive as the N=2 stabilization gate.
Ship the distribution-source guard (and companions) **before** any `straymark init`.

**Pros**:
- Tight feedback loop *without* the bricking hazard — the last stable release governs while the next is
  developed.
- The Control Center is a genuine, non-trivial internal dogfood target for the Charter/Loom/Baton machinery.
- First-hand contact with adopter friction the project currently receives second-hand from external adopters.

**Cons**:
- Double maintenance (an installed `.straymark/` to keep valid on top of authoring the framework).
- Requires new safeguards (a distribution-source guard, provenance sentinel) before it is safe.
- A one-version lag between what governs and what ships.

### Alternative 3: No self-adoption

**Description**: Keep developing StrayMark without installing it on itself; rely solely on external
adopters (Sentinel, lnxdrive) for validation.

**Pros**:
- Zero bootstrap/ambiguity risk; no double bookkeeping.
- External adopters carry a blind-spot advantage — they surface what the maintainer cannot see from inside.

**Cons**:
- Loses the tight, first-hand feedback loop; every field report (e.g. #345/#346/#350) stays second-hand.
- Leaves the spontaneous, uncontrolled half-ouroboros (loose artifacts scattered in experiment dirs, agents
  reading rules from the *source*) unmanaged.

## Decision

**Chosen**: Alternative 2 — scoped, lagged self-adoption, safeguards first.

**Justification**: The ouroboros hazard is not "adopting"; it is "adopting *live*." Eating *yesterday's*
tail (pinning to the last release) closes the feedback loop while the framework can still evolve safely.
The mechanical ambiguity that makes even a lagged adoption dangerous (the `dist/` vs installed confusion)
is closable with a precise, cheap guard, which becomes the non-negotiable prerequisite. Self-adoption
**complements, never replaces**, the external N=2 adopter gate: the blind-spot advantage of Sentinel/lnxdrive
is preserved as additive, not substituted.

## Consequences

### Positive
- First-hand feedback on adopter friction, with full context, faster than second-hand field reports.
- The Control Center becomes a real dogfood of the architecture/Loom/Baton machinery on an internal target.
- The spontaneous discipline (already happening) gains a canonical home instead of scattering.

### Negative
- Double bookkeeping: an installed `.straymark/` to keep valid alongside authoring the framework.
- A one-version governance lag (governs with the last release while the next is in `dist/`).

### Risks
- **R1/R2 — operating on / writing artifacts into `dist/`** (catastrophic): mitigated by **S1** (the
  distribution-source guard) + **S6** (CI hygiene backstop). Landed in PR #358.
- **R3/R4 — reading context from both frameworks / invisible version-skew**: mitigated by **S3** (skew
  visibility in `status`) + **S4** (agent directive: `/.straymark/` = governance-in-force, `/dist/.straymark/`
  = product-under-edit).
- **R5 — a test fixture resolved as a project**: covered by **S2** (`role: test-fixture` sentinel).
- **R6 — divergent duplicate framework files in git**: **S5** (version artifacts; gitignore the pinned
  framework-file copies).
- **R7 — meta-noise / hall of mirrors**: bounded by scope (govern *product*, not framework-meta-work).

## Implementation

Sequenced so the guard exists before the install (full detail in the accompanying implementation plan):

1. **S1 — distribution-source guard** + **S6 — CI hygiene backstop**. *Done — PR #358 (`cli-3.34.0`).*
2. **S2 — provenance sentinel**: `straymark init` writes `role: installed-project`; the shipped `dist/`
   carries `role: distribution-source`; commands verify and refuse non-install roles, **tolerating an absent
   sentinel** (legacy adopters must not break).
3. **S3/S4/S5** — skew visibility, the agent directive, and the git strategy for the installed instance.
4. **Gate:** no `straymark init` at the repo root until S1 **and** S2 exist.
5. Then `straymark init` at the root, **pinned to the last released framework**, with the Control Center as
   the first pilot. Existing spontaneous artifacts stay as a pre-adoption historical record (not migrated).

## References

- Working analyses (local): `analisis-autoadopcion.md`, `spike-b-autoadopcion-riesgos.md`, `PLAN-centro-de-control.md`
- PR #358 — S1 distribution-source guard + S6 CI hygiene
- Related: verification-fidelity (#306), the close-time review checkpoint (#350), [ADR-2026-06-03-followups-first-class](ADR-2026-06-03-followups-first-class.md)
- Design principle #12 (N=2 stabilization gate)

---

<!-- Template: StrayMark | https://strangedays.tech -->
