# Implementation plan — StrayMark self-adoption (AIDEC-2026-07-13-001)

> Companion to [`AIDEC-2026-07-13-001-straymark-self-adoption.md`](AIDEC-2026-07-13-001-straymark-self-adoption.md).
> Living checklist of the safeguard sequence and its status, kept **versioned** so we can retrace
> our steps (and roll back) if self-adoption misbehaves. Last updated: 2026-07-13.

## Gate (non-negotiable)

**No `straymark init` at the repo root until S1 AND S2 exist.** The distribution-source guard is the
mechanical prerequisite; the provenance sentinel is its defense-in-depth + the pinned-version record.

## Safeguard sequence

| ID | What | Surface | Status |
|----|------|---------|--------|
| **S1** | Distribution-source guard: `resolve_project_root` skips a `.straymark/` whose dir holds a `dist-manifest.yml` | CLI `utils` | ✅ **Done** — PR #358, `cli-3.34.0`, merged |
| **S6** | CI hygiene: fail if dated artifacts appear under `dist/.straymark/` | `.github/workflows/dist-hygiene.yml` | ✅ **Done** — PR #358 |
| **S2** | Provenance sentinel: `init` writes `.straymark/.provenance.yml { role, framework_version, installed_at, source_release }`; commands refuse non-install roles, tolerate absent (legacy) | CLI `init` + `utils` | 🔵 **In progress** — this PR, `cli-3.35.0` |
| **S3** | Skew visibility: `straymark status` prints a `skew:` line (installed fw-X pinned vs `dist/` in-dev fw-Y) when a sibling `dist/dist-manifest.yml` exists | CLI `status` (`load_dist_source_version`) | ✅ **Done** — `cli-3.36.0` |
| **S4** | Agent directive: `dist/.straymark/` = product-under-edit; `/.straymark/` = governance-in-force (pinned) | `CLAUDE.md` § Framework path discipline | ✅ **Done** — `cli-3.36.0` |
| **S5** | Git strategy for the install (see spec below) | at `init` time (Fase 2) | 📋 **Spec ready** (applied at init) |

## Design decisions taken (S2)

- **Option A (chosen):** only `init` stamps installs (`role: installed-project`); the distribution `dist/`
  is caught by S1's `dist-manifest.yml` detection. **No `role: distribution-source` marker is shipped in
  the distribution** — no adopter-facing change.
- **Own file (chosen):** `.straymark/.provenance.yml`, written by `init`; the shipped `config.yml` is not touched.
- **Legacy tolerance (invariant):** an **absent** `.provenance.yml` = legacy install = operable. Existing
  adopters (Sentinel, lnxdrive) have none and must not break. Only an explicit non-install `role:` is refused.
- **Primary value:** the `framework_version` field records the pinned "yesterday's tail" release — the input
  S3 needs to surface the skew, and the mechanism that makes the lagged self-adoption concrete.
- **Known follow-up (deferred with S3):** `straymark update-framework` should refresh `framework_version` /
  `source_release` in `.provenance.yml` (preserving `role` + original `installed_at`, adding an `updated_at`).
  Deferred to keep S2 focused; harmless until S3 reads the field. Tracked here so a stale version after an
  update is a known, not silent, gap. `straymark remove` already cleans `.provenance.yml`.

## S5 — git strategy spec (apply at `init`, Fase 2)

Two coherent options; **decide at init**. S3 (skew line) + S4 (directive) already make the pin/skew
visible, so the git layout no longer has to carry that job.

- **Option 1 — version everything (recommended).** Commit the whole `/.straymark/` (framework files
  *and* artifacts). Pros: self-contained, reproducible checkout; each `update-framework` is a visible,
  reviewable pin bump; git history records exactly which framework version governed us at each point.
  Con: the pinned framework files are duplicated in git vs `dist/` — but that duplication is *intended*
  and benign (they are one release apart on purpose; R6 "divergence" is expected, not a bug).
- **Option 2 — gitignore the regenerable framework files.** Version only the produced artifacts
  (`/.straymark/07-ai-audit/`, `/.straymark/charters/`, follow-ups registry, architecture model);
  gitignore `/.straymark/{00-governance,templates,schemas,audit-prompts}/`, `config.yml`,
  `.checksums.json`, `dist-manifest.yml`, `.provenance.yml`. Pro: no duplication. Cons: fragile
  ignore rules; a fresh checkout needs `straymark init`/`update-framework` to be usable; artifacts
  live inside otherwise-ignored trees.

**Recommendation: Option 1.** Simpler, self-contained, and the skew is already surfaced by S3/S4 —
gitignore gymnastics buy little. The `.provenance.yml` + `.checksums.json` committed also give a clean
audit trail of the pin over time.

## Phased rollout

- **Fase 0 — Mechanical safeguards.** S1+S6 ✅ · S2 🔵 (this PR). *Gate opens when S2 merges.*
- **Fase 1 — Context safeguards.** ✅ S3 (status skew) · ✅ S4 (agent directive) · 📋 S5 (git strategy spec ready, applied at init). `cli-3.36.0`.
- **Fase 2 — The `init`.** `straymark init` at the root, **pinned to the last release**, with S5's git strategy.
  Existing spontaneous experiment artifacts stay as pre-adoption historical record (not migrated).
  First governed work: the Control Center.
- **Fase 3 — Close the loop.** `straymark approve AIDEC-2026-07-13-001` on the real install (dogfoods the
  #350 review checkpoint).

## Rollback notes (if self-adoption misbehaves)

- **Before the root `init`:** nothing to roll back — S1/S2/S6 are inert guards + a CI check; they only ever
  *refuse* or *stamp*, never mutate an adopter's tree.
- **After the root `init`:** `straymark remove` (or delete the root `.straymark/` + revert the S5 gitignore
  entries). The pinned framework files are regenerable from the release; only the produced artifacts
  (charters/AILOGs/telemetry) are hand-authored and versioned — back them up before removing.
- **If a command ever resolves the wrong framework:** the `note:` printed on a skipped source, plus
  `straymark status`' skew line (S3), are the first diagnostics. The CI hygiene check (S6) catches
  `dist/` pollution post-hoc.

## References

- Decision: [`AIDEC-2026-07-13-001-straymark-self-adoption.md`](AIDEC-2026-07-13-001-straymark-self-adoption.md)
- Working analyses (local, non-versioned): `analisis-autoadopcion.md`, `spike-b-autoadopcion-riesgos.md`, `PLAN-centro-de-control.md`
- S1+S6: PR #358 (`cli-3.34.0`)
