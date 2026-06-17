//! `straymark loom` — launcher for Loom, the EXPERIMENTAL knowledge-graph
//! visualization server. The server binary is NOT bundled into the CLI: it is
//! downloaded on demand from the `loom-*` GitHub releases (the download gate
//! IS the opt-in boundary — see `experiment-loom/README.md`).

pub mod serve;
