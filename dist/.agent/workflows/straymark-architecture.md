---
description: Generate the architecture model and refine it in one guided pass — seed with `straymark architecture generate`, then reassign components into real layers, wire dependency links, sync the DrawIO, and validate to green. The agent-native counterpart to the manual DrawIO refinement. EXPERIMENTAL (Loom A1.x).
---

# StrayMark Architecture Skill

Drive the `generate → refine → validate` arc of the StrayMark architecture model from the agent window. `straymark architecture generate` mines *structure* (top-level source dirs → components); the model encodes *intent* (real layers, dependency links, human labels) that the filesystem does not contain. This skill performs the refinement a human would otherwise do in DrawIO, using the agent's knowledge of the repo, and iterates `validate` to green in a single pass.

> ⚠️ **EXPERIMENTAL.** The `architecture` model, its on-disk format, and Loom are an opt-in Loom A1.x experiment — not part of the supported Framework/CLI contract. See `docs/adopters/LOOM.md`. The model lives at `.straymark/architecture/{model.yml,plan.drawio}`.

## When to use this skill

Trigger on any of:

- The operator asks to "map the architecture", "set up the architecture model", or "refine the architecture seed".
- A `model.yml` was just generated and every component still sits in the placeholder `unassigned` layer.
- The operator wants the 2D/3D Loom views to show real layers and dependency arrows.

If `.straymark/architecture/model.yml` already exists **and is already refined** (components assigned to real layers, links present), prefer `/straymark-architecture-sync` to extend it append-only instead of re-refining from scratch.

## Instructions

### 1. Generate the seed

```bash
straymark architecture generate            # writes model.yml + plan.drawio
straymark architecture generate --force    # only if a seed already exists and the operator wants a fresh one
```

`generate` enriches from ADRs (C4 diagrams + "Affected Components" tables) **only if they exist**; with none, the seed is structure-only. It seeds every component into a placeholder `unassigned` layer and the `.straymark` stages 00–09 as placeholder layers. **The seed is a draft, not the answer** — the next step is where it becomes meaningful.

### 2. Refine the model (the phase that matters)

Read `model.yml` and the codebase, then edit `model.yml` to encode real intent:

- **Replace the placeholder doc-stage layers with real architecture layers**, inferred from directory conventions and ADRs (e.g. `entrypoints`, `domain`, `persistence`, `web`). **Ask the operator when the layering is ambiguous** — do not guess at the system's intended shape.
- **Reassign every component out of `unassigned`** into a real layer.
- **Fix labels** to human names (`internal-modules-commshub` → "CommsHub").
- **Tighten globs** so each component owns exactly its files.
- **Infer `links`** between components from the import graph / directory structure / ADR "Affected Components" tables.

The model schema (`.straymark/architecture/model.yml`):

```yaml
version: 0
layers:
  - { id: "domain", label: "Domain", order: 0 }      # id is the join key from component.layer; order is render order (low first)
components:
  - id: "commshub"          # stable join key to the DrawIO cell + status overlay
    label: "CommsHub"        # human label
    layer: "domain"          # MUST name an existing layer id
    globs: ["internal/modules/commshub/**"]   # the join to governance state
    links: ["audittrail"]    # list of target component ids (strings)
    docs: []                 # optional explicit doc ids; normally inferred via globs
    external: false          # true only for third-party / external systems
```

**Gotchas — each one costs a debugging cycle; pre-empt them:**

- **A `component.id` must not equal any `layer.id`.** For a single-component layer use a suffixed id (`core` layer → `core-infra` component). fw-4.27.0 emits a clear error for this, but write the model so it never trips.
- **`links` is a list of target component ids (strings)** — `["audittrail", "core"]`, **not** objects like `[{to: …}]` (which fails to parse).
- **Never delete a layer that a component still points at** via `component.layer` — that yields `references unknown layer`. Reassign the components first, then drop the empty placeholder layer.
- The placeholder `unassigned` layer is not required by the schema once empty — but every `component.layer` must still name a layer that exists.

### 3. Sync the DrawIO so 2D shows arrows

**3D renders edges from `model.yml` `links`; 2D renders them from `plan.drawio` edges.** Write **both**, or arrows show in only one view. After editing `model.yml`, update `plan.drawio` so each component has a vertex and each `link` has an edge between the matching vertices (the DrawIO cell id joins on the component `id`). Keep human-authored geometry where it exists.

### 4. Validate — iterate to green

```bash
straymark architecture validate            # text; exits 1 on any signal
straymark architecture validate --output json
```

Resolve every signal, then re-run until it exits 0:

- **`undrawn`** — a component with no cell in `plan.drawio` → add the vertex.
- **`unmodeled`** — a DrawIO cell with no component in `model.yml` → add the component or remove the stale cell.
- **`empty`** — a component whose globs match no files on disk → fix the globs.

### 5. Report result

Summarize what changed (layers created, components reassigned, links added) and surface the final `validate` output verbatim. Point the operator at the live views:

```
✓ Architecture model refined: 4 layers, 13 components, 10 links — validate is green.
  Next: `/straymark-loom` up to see the 2D/3D overlay, or `straymark status --where` for the terminal view.
```

## What this skill does NOT do

- **It does not invent the intended architecture.** When layering or boundaries are ambiguous, it asks the operator — the model encodes human intent, not a filesystem heuristic.
- **It does not maintain the status overlay.** `active` / `in-progress` / `implemented` / `has-debt` / `uncharted` are computed live from governance signals every time you look; this skill authors *structure* only.
- **It does not re-refine an already-curated model.** Use `/straymark-architecture-sync` (append-only) once real layers and links exist.

> **Terminal compatibility**: If the terminal does not support box-drawing characters (Unicode), use plain-text formatting with dashes and pipes instead.
