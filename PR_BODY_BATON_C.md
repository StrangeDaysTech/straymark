## Baton Track C — adopter kit + Track A item A4 + fw 4.39.0

Track C (forward-validation with adopters) is now unblocked: its only dependency, **Track A** (`work_verb` schema graduation), shipped in **fw 4.38.0 / cli 3.40.0** (2026-07-29). This PR produces the adopter-facing handoff kit, closes the one documentation gap Track A left behind (item A4), and bumps the Framework to **fw 4.39.0**. Plan: [`experiment-baton/PLAN-avance-post-calibracion.md`](experiment-baton/PLAN-avance-post-calibracion.md) · step 3 of #332.

## The re-framing

The existing adopter test plan ([`05-adopter-test-plan.md`](experiment-baton/05-adopter-test-plan.md)) predates decision #332 (title-scan discontinued; declared `work_verb` as the sole classification signal). Two of its three experiments are now obsolete by design:

- **E2** (signal enrichment) — the signal to enrich *is* the declared verb itself.
- **E3** (real costs) — illustrative costs suffice for forward-validation.

And **E1 changes question**: with classification deterministic over the declared verb, the oracle question is no longer *"does the classifier predict right"* but ***"do authors declare the verb correctly in production?"*** — exactly the gap #332 step 3 assigned to StrayMark.

## Changes

### `07-track-c-adopter-kit.md` (new)

The adopter handoff: preconditions (fw ≥ 4.38.0), declaration placement table, vocabulary + the three determination rules (foundational-contract = `implement`; `implement`+`upstream` degrades to operator; non-work = `operate`), the **simplified E1 protocol** (sample 20–30 declared units, retrospective `true_verb`/`true_provenance` labeling, agreement ≥ 0.8 target, downward-error watch), the friction questions, the explicit *what we do NOT ask* list, the Track C done-criterion, and the read-only guarantees.

### `05-adopter-test-plan.md`

Supersession banner pointing to the kit; preserved as the historical record of the pre-graduation calibration.

### `PLAN-avance-post-calibracion.md`

Track C header records that Track A shipped and links the kit.

### `AILOG-2026-08-02-001`

Logs the kit work and dogfoods the graduated schema (`work_verb: operate`, `design_provenance: upstream`).

### Track A item A4 closed — CLI reference ×3 locales

The 4.38.0 graduation shipped the fields into templates, charter schema, and `straymark validate` — but the adopter reference never documented them. Two insertions per locale in `docs/adopters/CLI-REFERENCE.md` (EN/es/zh-CN): the advisory work-classification vocabulary check under `validate` (absent → silent, invalid → non-blocking warning), and the optional `work_verb`/`design_provenance` frontmatter fields under `charter new` with the determination rules (bounded foundational contract = `implement`; `implement` + `design_provenance: upstream` degrades to mechanical).

### Version bump fw 4.38.1 → 4.39.0

`dist/dist-manifest.yml`, version tables in README ×3 locales and CLI-REFERENCE ×3 locales. The CHANGELOG entry states plainly that this is a **documentation-only release** — no `dist/` content changed beyond the version itself (the repo policy from AILOG-2026-07-25-002, kept honest). Logged in `AILOG-2026-08-02-002`.

## Verification

Docs-only change; no code touched. Vocabulary/placement quoted from the ratified schema ([`06-work-verb-schema-ratification.md`](experiment-baton/06-work-verb-schema-ratification.md)), field names matched against the graduated dist templates, validate behaviour matched against `cli/src/validation.rs` (`check_charter_work_verb`, `check_followups_work_verb`), routing behaviour (declared → High, undeclared → frontier + nudge, `route` requires `--dry-run`) against `experiment-baton/src/classify.rs` and `main.rs`. Repo-wide grep: every live `fw-4.38.1` reference updated; CHANGELOG history and blog release tags untouched. `cargo test` unaffected.

## Deferred

- **The validation itself** — Track C now waits on 2–4 weeks of adopter production use with declared verbs before the simplified E1 runs.
- **Binary handoff** — packaging `straymark-baton` for adopters is a separate step.
