# straymark-core

Shared document model and knowledge graph for [StrayMark](https://github.com/StrangeDaysTech/straymark).

This crate is the single source of truth for:

- **`document`** — the StrayMark document model: `DocType`, `Frontmatter`,
  `StrayMarkDocument`, `parse_document`, `discover_documents`, `detect_doc_type`.
- **`graph`** — the typed, bidirectional, orphan-preserving knowledge graph built
  from document frontmatter cross-links (`related`, `supersedes`,
  `alternatives_documented`, `api_changes`, `originating_ailogs`).

It exists so the StrayMark CLI (`straymark-cli`) and the experimental Loom
visualization server (`straymark-loom`) parse documents with **exactly the same
code** — making model drift between the two structurally impossible.

Extracted from the CLI in Loom milestone M0 (`CHARTER-01-loom-server`,
`ADR-2026-06-02-001`).

## License

MIT — see the repository's `LICENSE`.
