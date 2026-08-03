## Harden audit prompt from a 4-model cycle that missed an unreachable feature

Implements the three audit prompt rules proposed in #382, plus the verification-quality guidance from the issue's second-order finding. Plan: [`experiment-baton/PLAN-382-audit-prompt-hardening.md`](experiment-baton/PLAN-382-audit-prompt-hardening.md).

## The case

A 4-model external audit of CHARTER-29 (break-glass elevation, private repo) missed that the feature was completely non-functional. The Charter added `ResolverAsync` — the only method consulting elevation state — but all 8 consumers still called the synchronous `Resolver`. Three of four auditors reported "no critical or high findings". The defect was mechanically detectable with a single grep; one auditor ran that exact grep and read the result backwards.

## Changes

### Audit prompt v1.2 (EN + ES)

**New mandatory Step 3 — Enumerate callers of new public entry points.** For each public method, endpoint, or component the Charter adds, run a call-site search across production code and state the count explicitly. Zero production callers = High finding, no judgement required. Non-zero: verify the callers are the intended ones — an existing overload or legacy path may still be winning. Steps 3–5 renumbered to 4–6.

**Enhanced Step 2.6 — Consolidated test seam check.** When a test is documented as "consolidated" into another, verify the replacement exercises the same seam, not merely the same unit. The Charter's own closing notes are a claim by the audited party — treat as hypothesis, not evidence.

**Enhanced Step 4 — Red gate enumeration.** When a verification gate is red, enumerate what only that gate could have caught. A broken guard test reported as "config defect" without asking what it was protecting is a missed finding.

### AILOG + Charter templates (EN/ES/zh-CN)

"Tests pass" checkbox now requires declaring the exact command run. A verification that cannot produce a negative result is not verification — summing pass counts without checking failure output is the canonical anti-pattern. The Charter template's Local checks section carries the same warning.

## What surprised us

The second-order finding is arguably the more useful half of the case: the implementing agent built a verification method that *could not produce a red result* (summed pass counts, never checked failures), then trusted it seven times in a row. Self-verification never could have caught this — the audit did. Both sides (auditor + implementer) get guidance in this PR.

## Deferred

- **Blog post** about the case (requested in #382) — separate content task.
- **Version bump** — after this PR merges, as planned.

## Verification

No code changes — all changes are markdown templates. `cargo test` unaffected. `#382` references verified in all 7 modified files.
