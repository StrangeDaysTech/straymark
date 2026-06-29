# StrayMark — Loom & the Architecture Map (Experimental)

**See your project: the document web, and the "you are here" map of where the work is happening.**

> ⚠️ **EXPERIMENTAL — Loom v0 (N=1).** Loom is an opt-in, unstable experiment. Its API, CLI surface, on-disk model format, and very existence may change or be removed without a deprecation cycle until it graduates. It is **not** part of the supported Framework or CLI contract — don't build automation against it yet. The `architecture` model and `status --where` ship in the CLI but carry the same experimental caveat.

---

## What Loom is

Loom is StrayMark's third component (alongside the Framework and the CLI): a loopback-only, read-only **development dashboard** that makes a StrayMark project legible to the human eye. It has two surfaces over the same project:

1. **Knowledge Graph** — the web of your StrayMark *documents*, rendered as a live force-directed graph from their frontmatter links (`related`, `originating_ailogs`, …). Select a node and its whole thread of relationships lights up. This view is self-explanatory and needs no setup.
2. **Architecture Map** — the *system's* implementation map: components and layers (your real architecture), with a live **"you are here" status overlay** computed from governance state (active Charters, drift, closed work, open technical debt). It answers the daily *"where are we?"* visually. **This view is driven by a model you author + refine** — the rest of this guide is about that.

Loom never writes into your project — it only *visualizes* what the CLI computes. The CLI commands (`validate`, `audit`, `charter drift`) remain the source of truth and the gate.

---

## The mental model: one model, many views (BIM)

The Architecture Map borrows the **BIM "one model, many views"** idea. There is **one** model, expressed as two linked files under `.straymark/architecture/`:

| File | What it holds | Who owns it |
|---|---|---|
| `model.yml` | The **semantics** — components → file globs, layers, links | You (seeded by the CLI) |
| `plan.drawio` | The **layout** — the boxes-and-arrows diagram (DrawIO/mxGraph) | You (seeded by the CLI) |

From that one model, StrayMark renders **three projections**, all from the *same* status computation so they can't disagree:

- **Textual** — `straymark status --where` (the terminal "you are here").
- **2D plan** — Loom's Architecture tab (your `plan.drawio`, overlaid with live status).
- **3D axonometric** — Loom's `2D | 3D` toggle (an exploded, BIM-style view).

The **status overlay is computed live, never hand-maintained.** You author *structure* (which files belong to which component, how components relate); StrayMark colors it (`active` / `in-progress` / `implemented` / `has-debt` / `uncharted`) from governance signals every time you look.

---

## The recommended workflow

```
generate  →  refine (human or AI)  →  validate  →  sync (as code grows)  →  loom serve
   draft        the real model         CI gate       append-only            visualize
```

### 1. Generate — a first-draft seed

```bash
straymark architecture generate
```

Mines your codebase structure (one component per source directory) and enriches it from your ADRs (C4 diagrams + "Affected Components" tables improve labels and add links). It writes `model.yml` + `plan.drawio` with every component in a placeholder `unassigned` layer and the `.straymark` stages 00–09 as placeholder layers.

The seed is **language- and structure-aware** (it descends through `internal/`, `src/`, `pkg/`, … and skips Maven/Gradle scaffolding so a Java module isn't one `main` box). For non-default stacks you can extend it with an `architecture:` section in `.straymark/config.yml` — see [CLI-REFERENCE](./CLI-REFERENCE.md#straymark-architecture-generatesyncvalidate-cli-3250-experimental).

**The seed is a draft, not the answer.** It captures the shape; you make it meaningful in the next step.

### 2. Refine — the phase that matters (human *or* AI)

This is where a generated map becomes a real one. `model.yml` is plain YAML and `plan.drawio` is a standard DrawIO file, so refinement can be done **by a person, by an AI agent, or both**:

- **Rename the layers** from the placeholder stages to your real architecture (e.g. `entrypoints`, `domain`, `persistence`, `web`).
- **Reassign components** out of `unassigned` into those layers.
- **Fix labels** to human names (`internal-modules-commshub` → "CommsHub").
- **Tighten globs** so each component owns exactly its files.
- **Add `links`** between components to capture real dependencies.
- **Lay out the diagram** in DrawIO (open `plan.drawio`) so the 2D plan reads well.

> **AI-assisted refinement.** Because the model is just YAML + DrawIO XML, an AI coding agent can do this directly: point it at `model.yml` and your codebase and ask it to assign layers, fix labels, and add dependency links. It operates the same CLI (`generate` / `validate` / `sync`) and edits the same files you would. The refinement is a normal review-and-edit loop — keep what's right, correct what's wrong.
>
> **Skill shortcut** *(fw-4.29.0+)*: the `/straymark-architecture` skill drives this whole `generate → refine → validate` pass in one guided step (with the model's gotchas pre-encoded), `/straymark-architecture-sync` keeps a curated model current append-only, and `/straymark-loom` runs the server lifecycle (up/down/status) — the terminal-free path through this entire workflow.

### 3. Validate — catch model↔plan drift

```bash
straymark architecture validate            # text; exits 1 on any signal (CI-gateable)
straymark architecture validate --output json
```

Reports integrity signals: **undrawn** (a component with no diagram cell), **unmodeled** (a diagram cell missing from the model), **empty** (globs that match no files). Errors are now legible (they name the exact cause — e.g. a component id colliding with a layer id), so a malformed model tells you what to fix.

### 4. Sync — keep up as the code grows (append-only)

```bash
straymark architecture sync            # dry-run: shows what's new
straymark architecture sync --apply    # appends new dirs / ADR components
```

Detects new source directories and ADR components not yet in the model and **appends** them — it **never** clobbers your edits or your DrawIO geometry. Run it when you add a module; refine the new entries the same way.

### 5. Serve — see it live

```bash
straymark loom serve            # downloads the loom-* binary on first use, opens the browser
```

Opens both views on `localhost`. The Architecture tab shows your refined plan with the live overlay; the `2D | 3D` toggle gives the axonometric view. Edits to `.straymark/` (or the model) update the open browser in under a second. There's also a terminal-only answer that needs no server:

```bash
straymark status --where        # the textual "you are here"
```

---

## Honest limitations (it's a seed, not authoritative output)

- The **generated seed is a starting point.** It will not infer your intended layers or component boundaries — that's the refinement phase, by design.
- **Coverage depends on your stack.** The scan recognizes ~25 languages by default and handles common layouts (Go `internal/`, JS `src/`, multi-module Maven/Gradle). A non-default language or an unusual layout needs an `architecture:` config entry and more hand-refinement — but the seed never *misleads*; it just gives less of a head start.
- The **`has-debt` overlay is as precise as your debt records.** By default, debt is attributed to the components an open TDE's referenced AILOGs actually modified; coarse `files_modified` lists spread it wider than the debt itself. To scope it exactly, declare an **`affects`** field on the TDE (file globs, e.g. `affects: [internal/modules/audittrail/**]`) — when present it attributes the debt to those paths only, ignoring the AILOG footprint.
- **Loom is N=1.** It has been validated by dogfooding on this project and the Sentinel reference. Expect rough edges; report them.

---

## See also

- [CLI-REFERENCE](./CLI-REFERENCE.md) — `straymark architecture <generate|sync|validate>`, `status --where`, and `loom serve` flags + the `architecture:` config section.
- [WORKFLOWS](./WORKFLOWS.md) — how the architecture map fits the day-to-day StrayMark cadence.
- [ADOPTION-GUIDE](./ADOPTION-GUIDE.md) — getting StrayMark into a project in the first place.
