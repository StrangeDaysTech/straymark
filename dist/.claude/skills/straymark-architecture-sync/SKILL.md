---
name: straymark-architecture-sync
description: Keep the architecture model alive as code grows — wrap `straymark architecture sync` (append-only) to detect new source dirs / ADR components, surface them, reconcile against human edits, apply, and re-validate. Never re-refines from scratch. EXPERIMENTAL (Loom A1.3).
allowed-tools: Read, Bash(straymark architecture sync *, straymark architecture validate *, git diff *, git status *, ls *)
---

# StrayMark Architecture Sync Skill

Keep an already-refined architecture model current as the codebase grows. `straymark architecture sync` is **append-only**: it detects new top-level source directories and ADR components not yet in the model and appends them — it **never** clobbers your edits or your DrawIO geometry. This skill runs it as a guided dry-run → confirm → apply → re-validate loop.

> ⚠️ **EXPERIMENTAL.** The `architecture` model and Loom are an opt-in Loom A1.x experiment — not part of the supported Framework/CLI contract. The model lives at `.straymark/architecture/{model.yml,plan.drawio}`. See `docs/adopters/LOOM.md`.

## When to use this skill

Trigger on any of:

- A new module / source directory was added and the architecture model should reflect it.
- A new ADR introduced components not yet in the model.
- The operator asks to "update" or "refresh" the architecture model without re-refining it.

If `.straymark/architecture/model.yml` does **not** exist yet, or is still a raw seed (every component in `unassigned`), use `/straymark-architecture` instead — sync extends a curated model, it does not create or refine one.

## Instructions

### 1. Dry-run — see what's new

```bash
straymark architecture sync            # dry-run (default): lists components that would be added
```

Surface the proposed additions to the operator verbatim — each new component shows its id, globs, and any inferred links:

```
2 new components would be added (dry-run — pass --apply to write):
  + internal-modules-billing (globs: internal/modules/billing/**) → links: core
  + internal-modules-reports (globs: internal/modules/reports/**)
```

If it reports nothing new, stop and report **`Model is up to date — nothing to append.`**

### 2. Reconcile against human edits

Before applying, check the proposed additions against the curated model:

- New components land in the placeholder `unassigned` layer — note that they will need reassigning to a real layer afterward (refine them the same way `/straymark-architecture` does).
- Confirm the new ids don't collide with an existing `layer.id` (a `component.id` must never equal a `layer.id`).
- **Confirm with the operator before writing** — sync is append-only but it still mutates `model.yml`.

### 3. Apply

```bash
straymark architecture sync --apply    # appends the new components to model.yml (+ plan.drawio cells if it exists)
```

It appends with a `# Added by 'straymark architecture sync'` marker and, when `plan.drawio` is a recognized DrawIO document, appends matching cells. Existing geometry and edits are untouched.

### 4. Re-validate

```bash
straymark architecture validate        # exits 1 on any signal
```

Resolve any new `undrawn` / `unmodeled` / `empty` signals, then remind the operator to **refine the appended components** — reassign them out of `unassigned` into real layers and wire their `links` (that's a `/straymark-architecture` refinement on just the new entries).

## Report result

Surface the CLI output verbatim and name the follow-up. Example:

```
✓ Appended 2 components to model.yml (billing, reports) — now in `unassigned`.
  Next: reassign them to real layers + add links via /straymark-architecture, then /straymark-loom to view.
```

## What this skill does NOT do

- **It does not re-refine the model.** It only appends what's new; the existing curated layers, labels, links, and geometry are left exactly as they are.
- **It does not assign new components to real layers.** Appended components land in `unassigned`; refining them is a `/straymark-architecture` step.
- **It does not run when no curated model exists.** Use `/straymark-architecture` to generate and refine the first model.

> **Terminal compatibility**: If the terminal does not support box-drawing characters (Unicode), use plain-text formatting with dashes and pipes instead.
