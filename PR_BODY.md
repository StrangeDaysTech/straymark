## Baton: graduate work_verb to framework + harden coherence engine

Post-calibration work following the Sentinel E1/E2/E3 adopter feedback (#331). The title-scan classifier proved unviable (0.57 high+medium precision, 4 errors *downward*); `work_verb`/`design_provenance` declared at authoring is the ratified replacement (#332, schema ratified in `experiment-baton/06-work-verb-schema-ratification.md`).

This PR does two things: **graduates the ratified schema to the framework** (Track A) and **applies three hardening fixes to the Baton experiment** (Track B). Both tracks come from the [post-calibration plan](experiment-baton/PLAN-avance-post-calibracion.md).

## Track A — work_verb graduation

The schema was ratified but only existed in the Baton prototype. For adopters to declare verbs, the fields need to live in the framework templates and the validator needs to check them.

**Templates** — `work_verb`/`design_provenance` added to:
- AILOG template (EN/ES/zh-CN) — frontmatter comments, matching the Charter template which already had them
- Follow-ups backlog template — `**Work verb**:` / `**Design provenance**:` in the entry shape

**Validation** — `straymark validate` now checks follow-ups backlog entries for out-of-vocabulary `work_verb`/`design_provenance` values. Advisory only, same anti-noise posture as the existing charter check: absent fields emit nothing, present-but-invalid values emit a Warning. This closes the gap where only charters were checked but follow-ups (the other declaration surface) were not.

**Docs** — QUICK-REFERENCE.md gains the verbo→tier table and determination rules. CHANGELOG.md has the unreleased entry.

## Track B — Baton hardening

### #319 — Producer-side route keying (huma Go)

Sentinel's Go backend registers all routes in one huma block; the response struct is defined ~75 lines below. Nearest-anchor keying bound it to whatever route was registered last — not the correct one. Net effect: producer=None for that contract, so C2/C3 (field/enum mismatch) could never fire on an un-remediated repo — exactly the #304 scenario the engine exists to catch.

Fix: parse `huma.Get(api, "/path", h.handlerMethod)` registrations into a handler→route map, then key `<handler>Output` structs via the huma naming convention. Conservative: ambiguous bindings are dropped, and the nearest-anchor fallback is preserved for non-huma code.

### #315 — EPIPE/SIGPIPE handling

CLI crashed when output was piped to `head` or `less`. Standard fix: reset SIGPIPE to SIG_DFL on Unix.

### #314 — Component→path mapping for C1

C1 (intended-not-implemented) was Info/Low confidence because it mined component names from `.specify/memory/` filenames and slug-matched them against file paths — producing false positives on architectural concepts that were never modules.

Fix: memory files can now declare explicit `paths:` globs in frontmatter. When present, C1 uses glob matching (proper recursive `**` matcher) instead of the slug heuristic, and the finding is promoted to Warning/High confidence.

## What surprised us

The glob matcher needed a full rewrite mid-PR. The initial implementation only handled a single `**` and treated the rest as a literal suffix — `src/**/policyengine/**` failed against `src/internal/policyengine/handler.go` because the trailing `/**` was compared literally. The recursive segment-based algorithm handles `**` as zero-or-more whole segments, which is the standard glob semantics.

## Deferred

- **#321** (language-agnostic codescan): blocked on N=2 — a second adopter with a non-Go⇄TS stack. The `LanguageAdapter` seam is documented in the issue but deliberately not built from a single example.
- **#335** (declared-signal principle for codescan): design note already covered by #332 and the ratification doc.
- **Framework version bump**: deferred to release time per the standard workflow.

## Verification

- `cargo test -p straymark-baton` — 51 passed, 0 failed (includes new huma keying and glob matching tests)
- `cargo check -p straymark-cli` — clean
- `cargo test -p straymark-cli -- validate` — green

## Release bookkeeping

No version bump (framework fields are optional/additive, Baton is experimental). CHANGELOG entry under "Unreleased".
