# straymark-cli

> **The cognitive discipline your AI-assisted projects need.**

[![Crates.io](https://img.shields.io/crates/v/straymark-cli.svg)](https://crates.io/crates/straymark-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/StrangeDaysTech/straymark/blob/main/LICENSE)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/QUICK-REFERENCE.md)

StrayMark externalizes the cognitive discipline of senior software engineering — explicit scope, declared decisions, named risks, recorded alternatives, audited trails — into versioned files that live alongside the code, and injects the same discipline into the AI agents that work in the repo. This crate provides the `straymark` CLI that manages the framework, validates documents, runs drift checks against Charter scope, and orchestrates external multi-CLI audits. As a side effect, the artifacts produced map cleanly onto **ISO/IEC 42001**, **EU AI Act**, **NIST AI RMF**, **GDPR** and (opt-in) the Chinese AI/data regulatory stack.

The goal is engineering quality first; compliance is what falls out when the discipline is real.

## Installation

### Via Cargo

```bash
cargo install straymark-cli
```

### Prebuilt binaries

Download the binary for your platform from the [latest release](https://github.com/StrangeDaysTech/straymark/releases/latest):

- **Linux (x86_64)** — `straymark-cli-v<version>-x86_64-unknown-linux-gnu.tar.gz`
- **macOS Intel** — `straymark-cli-v<version>-x86_64-apple-darwin.tar.gz`
- **macOS Apple Silicon** — `straymark-cli-v<version>-aarch64-apple-darwin.tar.gz`
- **Windows** — `straymark-cli-v<version>-x86_64-pc-windows-msvc.zip`

Extract and place the `straymark` binary somewhere on your `PATH`.

### Upgrade

If you already have an older version installed:

```bash
straymark update-cli           # self-update the binary
straymark update-framework     # update the framework in a project
straymark update               # both at once
```

## Quickstart

Inside a Git repository:

```bash
straymark init .               # install the framework into the project
straymark charter new          # scaffold a bounded unit of work (ex-ante scope)
straymark validate             # validate documents against schemas
straymark audit                # generate the project audit report
straymark explore              # interactive TUI to browse documentation
```

The full subcommand reference (`init`, `update`, `remove`, `status`, `repair`, `validate`, `new`, `compliance`, `metrics`, `analyze`, `audit`, `explore`, `charter`, `about`) lives in the [CLI Reference](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md).

## What StrayMark does

- **Versioned governance artifacts** — 12 base document types (`AILOG`, `AIDEC`, `ETH`, `ADR`, `REQ`, `TES`, `INC`, `TDE`, `SEC`, `MCARD`, `SBOM`, `DPIA`) + 4 China-scope types + the `Charter` unit of work, each with schemas and lifecycle.
- **Multi-CLI agent directives injected** — keeps `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules` and `.cursor/rules/straymark.md` in sync from a single `STRAYMARK.md` source of truth.
- **Charter drift check** — verifies declared files against `git diff` so the implementation cannot silently diverge from the ex-ante scope.
- **External multi-CLI audit orchestration** — generates unified audit prompts and consolidates reports from multiple auditor CLIs into Charter telemetry.
- **Multi-regulatory mapping** — output artifacts satisfy ISO/IEC 42001, EU AI Act, NIST AI RMF, ISO/IEC 25010, ISO/IEC/IEEE 29148, ISO/IEC/IEEE 29119-3, GDPR.
- **Optional China regulatory scope** — opt-in via `regional_scope: china` activates PIPIA, CACFILE, TC260RA and AILABEL document types plus PIPL / TC260 / GB 45438 / CAC / GB/T 45652 / CSL 2026 mapping.

## Multi-CLI agent support

The CLI reads (and writes injection markers into) the dominant standards for AI coding agents:

- **`AGENTS.md`** — the open standard donated to the Agentic AI Foundation (Linux Foundation, December 2025), read by Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others.
- **CLI-specific files** — `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules`, `.cursor/rules/straymark.md` for platform-specific identity strings.

All targets stay synchronized from a single `STRAYMARK.md` file maintained by `straymark update`.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tui`     | yes | Interactive `explore` TUI (ratatui + crossterm + pulldown-cmark) |
| `analyze` | yes | Code complexity analysis (cognitive + cyclomatic via arborist-metrics) |

Build without TUI:

```bash
cargo install straymark-cli --no-default-features --features analyze
```

## Documentation

- **Project README** — https://github.com/StrangeDaysTech/straymark
- **Handbook** — https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/QUICK-REFERENCE.md
- **CLI Reference** — https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md
- **Adoption Guide** — https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTION-GUIDE.md
- **Changelog** — https://github.com/StrangeDaysTech/straymark/blob/main/CHANGELOG.md
- **Español** — https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/README.md
- **简体中文** — https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/zh-CN/README.md

## License

MIT — © Strange Days Tech, S.A.S.

## See also

The framework that ships alongside this CLI lives in the same repository: [`StrangeDaysTech/straymark`](https://github.com/StrangeDaysTech/straymark). Framework releases are tagged `fw-X.Y.Z`; CLI releases are tagged `cli-X.Y.Z` and published here on crates.io.

---

*[Strange Days Tech](https://strangedays.tech) — Because every change tells a story.*
