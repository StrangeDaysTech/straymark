# Design — `straymark export --format okf`

*Prepared 2026-06-17. This is plan (c): a design sketch, not a committed feature.
It builds on the conformance + mapping work in [`03`](03-okf-technical-report.md).
EXPERIMENTAL if built (Loom-track v0 conventions apply).*

## 1. Goal

Emit a StrayMark project's governance corpus as a **conformant OKF v0.1 bundle**,
so anything that speaks OKF (Google Knowledge Catalog, OKF visualizers, other
agents) can consume it — and so a `StrayMark → OKF → StrayMark` round-trip is
lossless. Export-first; import is out of scope here (see `03` §5).

Non-goals: replacing StrayMark's native format, subsuming OKF, or shipping a
data-catalog feature. We expose our existing corpus *in* OKF; we don't become
OKF.

## 2. Why this is cheap

Most of the machinery exists in `straymark-core` (much of it built during the
Loom arc):

- `document` — `DocType`, `Frontmatter`, `parse_document`, `discover_documents`.
- `graph` — typed, bidirectional graph with **reference resolution** (exact id →
  basename → path suffix → `CHARTER-NN` → dated prefix) and per-node file paths.
  This is exactly what OKF bundle-relative links need.
- `entities` — discovers charters/plans/audits and their paths.
- `architecture` / `audit` — timelines we can render into `log.md`.

So the exporter is mostly **serialization + a type map**, not new subsystems.

## 3. CLI surface

```
straymark export [path] --format okf [--out DIR] [--include TYPES] [--with-log] [--zip]
```

- `--format okf` — the only format today; leaves room for others later.
- `--out DIR` — bundle output dir (default `.straymark-okf/` or `dist/okf/`).
- `--include TYPES` — comma list of DocTypes to include (default: all).
- `--with-log` — also render `log.md` files from the audit/AILOG timeline (off by
  default; it's optional in OKF).
- `--zip` — also emit a `.tar.gz`/`.zip` of the bundle (OKF distribution form).

Exit non-zero on a write error; print a summary (concepts written, links
resolved, dangling links preserved, index files generated). Mark output banner
**EXPERIMENTAL**.

## 4. Bundle layout

Group concepts by type for a clean, self-describing tree (see `03` §2):

```
<out>/
├── index.md                    # root listing; frontmatter: okf_version: "0.1"
├── adr/        ADR-YYYY-MM-DD-NNN.md
├── ailogs/     AILOG-….md
├── charters/   CHARTER-….md
├── tde/        TDE-….md
├── audits/     <charter>/review.md
└── (log.md per dir, if --with-log)
```

Concept ID = path minus `.md` (OKF §2). Keeping StrayMark's own IDs as filenames
makes the round-trip and human cross-checking trivial.

## 5. Per-document transform

For each discovered document:

1. **Frontmatter.**
   - Set `type` from a `DocType → string` map (e.g. `Adr → "Architecture Decision
     Record"`, `Ailog → "AI Work Log"`, `Charter → "Charter"`, `Tde →
     "Transversal Debt Entry"`). One small table; the only genuinely new mapping.
   - Carry `title`, `description`, `tags`, and `timestamp` (normalize date →
     ISO 8601) into the OKF-recommended fields.
   - **Preserve every other StrayMark key verbatim** (`status`, `related`,
     `supersedes`, `originating_ailogs`, `risk_level`, `eu_ai_act_risk`,
     `affects`, …). OKF §4.1 guarantees consumers keep unknown keys → this is
     what makes the round-trip lossless and our governance metadata survive.
   - Optionally set `resource` for docs that name a canonical asset/PR/URL;
     otherwise omit (OKF allows it).

2. **Body — link rewriting.** For each typed relationship the graph resolved
   (`related`, `supersedes`, `originating_ailogs`, …):
   - Ask `core::graph` for the target node's bundle path.
   - Emit a bundle-relative markdown link `/charters/CHARTER-02.md` in the body
     (e.g. under a generated `# Relationships` section), and/or inline where the
     prose already references it.
   - **Keep the typed edge in frontmatter too** (see `03` §3 fidelity note) so
     OKF graph viewers get a link *and* round-trip keeps the type. Unresolved
     references stay as written — OKF treats broken links as "not-yet-written
     knowledge," matching our `resolved:false` model.

3. **Citations.** External references already in StrayMark bodies map to a
   `# Citations` section (OKF §8) when present; otherwise skip.

## 6. Generated `index.md`

For each directory (and the bundle root), generate an `index.md` (no frontmatter,
except root which carries `okf_version: "0.1"`): one section per group, each
entry `* [Title](relative-url) - description` pulled from the concept's
frontmatter. Pure projection of data we already have. This is also what powers
progressive disclosure for consuming agents.

## 7. Optional `log.md` (`--with-log`)

Render the audit/AILOG timeline into per-scope `log.md` files: date-grouped
(`## YYYY-MM-DD`), newest first, entries prefixed `**Creation**` / `**Update**`
/ `**Deprecation**`. The `audit` engine already computes timelines; this is a
formatter.

## 8. Where it lives

- New module `straymark-core::okf` (pure: `Corpus → OkfBundle` value, zero I/O)
  so the CLI and a future Loom adapter share one emitter — the same
  pure-core/shared-projection discipline used for the architecture projection.
- CLI `commands/export.rs` does discovery + I/O + packaging around the pure core.
- Round-trip test: `export` a fixture corpus, assert §9 conformance (every
  non-reserved `.md` has frontmatter with non-empty `type`; reserved files
  well-formed), then re-parse and assert no frontmatter key was lost.

## 9. Loom-as-OKF-viewer (companion, separate effort)

Independently of export, Loom can *read* OKF: a small adapter that treats a
bundle dir as input, links as untyped `RELATED_TO` edges, and `type` as the node
type for coloring. Reuses Loom's existing graph builder; gives "Loom renders any
OKF bundle" with richer overlays/3D than OKF's static visualizer. Tracked
separately from the exporter; see `03` §6.

## 10. Phasing

| Phase | Scope | Effort |
|---|---|---|
| **E1** | `straymark-core::okf` pure emitter + `DocType→type` map + frontmatter passthrough | small |
| **E2** | link rewriting via `core::graph` resolution + generated `index.md` | small–medium |
| **E3** | CLI `export --format okf` + `--zip` + conformance/round-trip tests | small |
| **E4** (opt) | `--with-log` from audit timeline | small |
| **E5** (opt) | Loom OKF-bundle adapter | small |

E1–E3 deliver a conformant, round-trip-safe exporter. Everything reuses existing
core; the only net-new concept is the type map. Ship behind the EXPERIMENTAL
banner, gate hard stabilization on a real consumer (e.g. feeding a bundle to
Knowledge Catalog or an OKF viewer) — the same N≥2 discipline the project uses
elsewhere.
