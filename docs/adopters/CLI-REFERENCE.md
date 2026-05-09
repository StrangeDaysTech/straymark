# StrayMark CLI Reference

**Complete reference for the `straymark` command-line tool.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**Languages**: English | [Español](../i18n/es/adopters/CLI-REFERENCE.md) | [简体中文](../i18n/zh-CN/adopters/CLI-REFERENCE.md)

---

## Table of Contents

1. [Installation](#installation)
2. [Versioning](#versioning)
3. [Commands](#commands) — init, update, remove, status, repair, validate, new, charter, compliance, metrics, analyze, audit, explore, about
4. [Environment Variables](#environment-variables)
5. [Exit Codes](#exit-codes)

---

## Installation

Install the StrayMark CLI using one of the methods below. For full setup instructions, see the [README](../../README.md#getting-started).

**Quick install (prebuilt binary):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

**From source:**

```bash
cargo install straymark-cli
```

---

## Versioning

StrayMark uses **independent version tags** for each component:

| Component | Tag prefix | Example | What it includes |
|-----------|-----------|---------|------------------|
| Framework | `fw-` | `fw-4.11.0` | Templates (12 types), governance docs, directives, Charter template + schema |
| CLI | `cli-` | `cli-3.11.0` | The `straymark` binary |

Framework and CLI are released independently. A framework update does not require a CLI update, and vice versa.

**Check installed versions:**

```bash
straymark about    # Shows CLI version + framework version (if installed)
straymark status   # Shows full installation health including versions
```

---

## Commands

### `straymark init [path] [--hooks]`

Initialize StrayMark in a project directory.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---|---|---|
| `path` | `.` (current directory) | Target project directory |
| `--hooks` *(cli-3.7.0+)* | off | After init, install the framework's pre-PR hook (`.straymark/hooks/pre-pr.sh`) as `.git/hooks/pre-push`. Runs `straymark charter drift` automatically before each push. Opt-in per principle #6 (cognitive discipline > raw productivity). Refuses to overwrite an existing `pre-push` hook; skips silently if not a git repo. |

**What it does:**

1. Downloads the latest framework release (`fw-*`) from GitHub
2. Creates the `.straymark/` directory structure
3. Creates `STRAYMARK.md` with governance rules
4. Configures AI agent directive files (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
5. Copies CI/CD workflows
6. *(`--hooks`)* installs the pre-PR hook

**Example:**

```bash
$ straymark init .
✔ Downloaded StrayMark fw-4.11.0
✔ Created .straymark/ directory structure
✔ Created STRAYMARK.md
✔ Configured AI agent directives

StrayMark initialized successfully!
Next: git add .straymark/ STRAYMARK.md && git commit -m "chore: adopt StrayMark"
```

---

### `straymark update`

Update **both** framework and CLI to their latest versions. Equivalent to running `update-framework` followed by `update-cli`.

If `.straymark/` does not exist in the current directory, the framework update is skipped with a warning.

**Example:**

```bash
$ straymark update
Updating framework...
✔ Framework updated to fw-4.11.0
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `straymark update-framework`

Update only the framework files. Looks for the latest `fw-*` release on GitHub.

**Conflict handling:** If you have modified framework files (e.g., governance docs or templates), the update preserves your changes and reports conflicts for manual resolution.

**Example:**

```bash
$ straymark update-framework
✔ Framework updated to fw-4.11.0
```

---

### `straymark update-cli`

Auto-update the `straymark` binary itself. Automatically detects the installation method and uses the appropriate update mechanism:

- **Prebuilt binary** (installed via `install.sh` / `install.ps1`): Downloads the latest binary from GitHub Releases
- **Cargo** (installed via `cargo install`): Runs `cargo install --force straymark-cli`

Use `--method` to override auto-detection: `--method=github` or `--method=cargo`.

**Example:**

```bash
$ straymark update-cli
✔ CLI updated to cli-3.5.2

$ straymark update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `straymark remove [--full]`

Remove StrayMark from the current project.

**Flags:**

| Flag | Description |
|------|-------------|
| `--full` | Remove everything, including user-created documents in `.straymark/`. Asks for confirmation. |

**Default behavior** (without `--full`): removes the framework structure but preserves documents you created inside `.straymark/`.

**Example:**

```bash
$ straymark remove
✔ StrayMark framework removed. User documents preserved in .straymark/.

$ straymark remove --full
⚠ This will delete all StrayMark files including your documents.
Continue? [y/N]: y
✔ StrayMark completely removed.
```

---

### `straymark status [path]`

Show installation health and documentation statistics.

**Arguments:**

| Argument | Default | Description |
|----------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |

**Output includes:**

- Project path
- Framework version
- CLI version
- Configured language
- Directory structure integrity
- Document statistics (count by type)

**Example:**

```
$ straymark status

  ╔════════════════════════════════════════════════╗
  ║ StrayMark Status                                ║
  ╚════════════════════════════════════════════════╝

  Project
  ┌───────────┬──────────────────────────┐
  │ Path      │ /home/user/my-project    │
  │ Framework │ fw-4.11.0                 │
  │ CLI       │ cli-3.5.2                │
  │ Language  │ en                       │
  └───────────┴──────────────────────────┘

  Structure
  ✓ All 15 items present
  ┌──────────────────────────────┬────────┐
  │ Directory / File             │ Status │
  ├──────────────────────────────┼────────┤
  │ 00-governance/               │ ✓ OK   │
  │ ...                          │ ...    │
  └──────────────────────────────┴────────┘

  Documentation
  ┌──────────────────────────────┬───────┐
  │ Type                         │ Count │
  ├──────────────────────────────┼───────┤
  │ AILOG AI Action Logs         │    12 │
  │ ADR   Architecture Decisions │     7 │
  │ ...                          │   ... │
  ├──────────────────────────────┼───────┤
  │ Total                        │    30 │
  └──────────────────────────────┴───────┘

  → Run straymark explore to browse documentation interactively
```

---

### `straymark repair [path]`

Repair a broken StrayMark installation by restoring missing directories and framework files.

**Arguments:**

| Argument | Default | Description |
|----------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |

**What it does:**

1. Checks for missing directories and restores them with `.gitkeep`
2. Downloads the framework release **once** if files need restoration (templates, governance docs, config)
3. Re-injects directives if `STRAYMARK.md` is missing
4. Recalculates checksums after repair
5. Never modifies or deletes user-generated documents

**Example:**

```bash
$ straymark repair
Repairing StrayMark in /home/user/my-project
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .straymark/templates/
→ Downloading framework to restore missing files...
  Using version: fw-4.11.0
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ StrayMark repaired successfully!
```

---

### `straymark validate [path] [--fix] [--staged] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Validate StrayMark documents for compliance and correctness.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--fix` | — | Automatically fix simple issues (e.g., missing `review_required: true` for high-risk docs) |
| `--staged` | — | Validate only staged (git-added) files. Ideal for pre-commit hooks. |
| `--include-charters` | — | Also validate Charters in `docs/charters/` against the Charter JSON Schema and referential integrity (originating AILOG IDs resolve, originating spec paths exist). Opt-in so projects that don't yet use the Charter pattern are unaffected. |
| `--check-pending-reviews` *(cli-3.7.0+)* | off | List documents with `review_required: true` and no `review_outcome` older than `--max-pending-days`. **Warn-only** — never fails the validate exit code; useful for CI dashboards of the approval backlog. |
| `--max-pending-days` *(cli-3.7.0+)* | `14` | Threshold in days for `--check-pending-reviews` |

**What it checks:**

- Naming conventions (`TYPE-YYYY-MM-DD-NNN-description.md`)
- Required metadata fields (id, title, status, created, agent, confidence, review_required, risk_level, tags, related)
- Cross-field consistency (e.g., high risk must have review_required)
- Type-specific fields (e.g., INC needs severity, SEC needs threat_model_methodology)
- Sensitive information detection (API keys, passwords)
- Related document existence

When `regional_scope` includes `china`, twelve additional rules activate (`CROSS-004` to `CROSS-011`, `TYPE-003` to `TYPE-006`) covering TC260 review escalation, PIPIA linkage from sensitive-data documents, CACFILE / AILABEL cross-references, CSL severity-to-deadline coherence, and PIPIA 3-year retention. Without `china` in scope, these rules are skipped — no false positives.

**Example:**

```bash
$ straymark validate
  StrayMark Validate
  All 15 document(s) passed validation
  0 error(s), 0 warning(s) in 15 document(s)

$ straymark validate --fix
  StrayMark Validate
  Auto-fixing 2 issue(s)...
  ✓ Fixed 2 issue(s)
```

---

### `straymark approve <doc-id> --outcome <outcome> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."] [--path <dir>]`

*Available since **cli-3.7.0** + **fw-4.6.0**. `--quiet` and high-risk warning added in cli-3.8.0.*

Record a formal human approval on a `review_required: true` document. Writes the three approval frontmatter fields (`reviewed_by`, `reviewed_at`, `review_outcome`) **and** appends the canonical `## Approval` body section in one atomic edit. Implements the closure signal canonized in `DOCUMENTATION-POLICY.md §3.5`.

| Argument/Flag | Default | Description |
|---|---|---|
| `<doc-id>` | — | Document ID. Accepts the bare prefix (`AIDEC-2026-05-02-001`) or full ID with slug (`AIDEC-2026-05-02-001-foo`). |
| `--outcome` | — | One of `approved`, `revisions_requested`, `rejected`. Prompts on TTY if absent. |
| `--reviewer` | — | Reviewer identity: email, GitHub handle, or DID. Prompts on TTY if absent. |
| `--at` | today | Approval date (`YYYY-MM-DD`) |
| `--notes` | — | Optional reviewer notes (appended in the body section) |
| `--path` | `.` | Target project directory |

**Behavior:**

- Warns (does not fail) if the document doesn't have `review_required: true` — retroactive sign-off is a real use case.
- **Frontmatter mutation** (latest-wins): replaces existing `reviewed_by/_at/outcome` if present; otherwise inserts after `review_required:`. This implements the multi-reviewer convention from §3.5: frontmatter holds the *latest* approval.
- **Body mutation** (chronological): appends a new `## Approval` block before any trailing template signature. Re-running `approve` preserves earlier blocks so the body shows the full review history.
- `review_required: true` is **not** toggled to `false` after approval — it remains as historical record of why review was needed.

**Examples:**

```bash
# Flag-driven (CI / scripts)
$ straymark approve AIDEC-2026-05-02-001 \
    --outcome approved \
    --reviewer pepe@example.com \
    --notes "Reviewed against ADR-007. LGTM."

  ✔ AIDEC-2026-05-02-001 marked as approved.
    Reviewer: pepe@example.com
    Date:     2026-05-02
    File:     .straymark/07-ai-audit/decisions/AIDEC-2026-05-02-001-foo.md

# Iterative review cycle: revisions_requested → re-approve
$ straymark approve AIDEC-... --outcome revisions_requested --reviewer reviewer@x.io
# (author iterates)
$ straymark approve AIDEC-... --outcome approved --reviewer reviewer@x.io
# Frontmatter shows the latest (approved); body shows BOTH blocks chronologically.

# Backlog visibility
$ straymark validate --check-pending-reviews --max-pending-days 14
```

> See `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` §3.5 "Recording Approval" for the canonical workflow definition (closure semantics, body format, multi-reviewer convention).

---

### `straymark new [path] [-t <type>] [--title <title>]`

Create a new StrayMark document from a template.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--doc-type`, `-t` | — | Document type. Core (12): `ailog`, `aidec`, `adr`, `eth`, `req`, `tes`, `inc`, `tde`, `sec`, `mcard`, `sbom`, `dpia`. China (4, opt-in): `pipia`, `cacfile`, `tc260ra`, `ailabel`. |
| `--title` | — | Title for the new document |

If `--doc-type` or `--title` are omitted, the command prompts interactively. China-only types are filtered out of the prompt (and rejected from `-t`) when `regional_scope` does not include `china`.

**Examples:**

```bash
# Interactive — prompts for type and title
$ straymark new

# Create an AILOG with a title (non-interactive)
$ straymark new -t ailog --title "Implement JWT authentication"

# Create an ADR
$ straymark new --doc-type adr --title "Use PostgreSQL for persistence"
```

**Example output:**

```
$ straymark new -t ailog --title "Implement JWT authentication"

  ✔ Created: .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md
```

---

### `straymark charter <subcommand>`

Manage **Charters**: bounded, auditable units of work declared ex-ante and validated ex-post. A Charter pairs declarative scope (files to touch, risks, executable verification) with ex-post audit anchoring (drift detection, multi-model audit). Charters live at `docs/charters/NN-slug.md` (project-root level, **not** under `.straymark/`).

> **Naming history.** In the Sentinel `/plan-audit` experiment that crystallized this pattern (2026-04, 6 cycles), Charters were called *Plans*. The StrayMark CLI uses **Charter** going forward to disambiguate from GitHub SpecKit's `plan.md`. Sentinel's historical files preserve "Plan" deliberately. The full conceptual scope and the rename rationale live in `Propuesta/que-es-un-charter.md`.

**Subcommands:**

- `straymark charter new` — scaffold a new Charter from the framework template
- `straymark charter list` — enumerate Charters with optional filters
- `straymark charter status` — show Charter detail, or the most recent 5 Charters
- `straymark charter close` — record post-execution telemetry and bump status to `closed` *(Phase 2, fw-4.6.0+)*
- `straymark charter drift` — detect file-vs-commit drift with AILOG-aware suppression *(Phase 2, fw-4.6.0+)*
- `straymark charter audit` — orchestrate a multi-model external review (3-step prepare/calibrate/finalize) *(Phase 3, fw-4.9.0+)*

#### `straymark charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

Scaffold a Charter from the framework template into `docs/charters/NN-slug.md`. Prompts for the title interactively if not passed. The two origin flags are mutually exclusive at the clap level.

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--type`, `-t` | `M` | Effort estimate. One of `XS`, `S`, `M`, `L`. |
| `--title` | — | Charter title. Used to build the slug and filename. Prompts if absent. |
| `--from-ailog` | — | Originating AILOG ID (e.g., `AILOG-2026-04-28-021`). Pre-populates `originating_ailogs` in frontmatter. **Mutually exclusive with `--from-spec`.** |
| `--from-spec` | — | Path to a SpecKit spec.md (e.g., `specs/001-feature/spec.md`). Pre-populates `originating_spec` in frontmatter. The path is verified at scaffold time. **Mutually exclusive with `--from-ailog`.** |

When neither origin flag is given, both `originating_ailogs` and `originating_spec` stay commented out in the generated frontmatter — the Charter is scaffolded "without explicit origin" and the user fills it in before status moves to `in-progress`.

**Examples:**

```bash
# Standalone (no origin) — interactive title prompt
$ straymark charter new --type M

# Maintenance / post-MVP mode — Charter rooted in an existing AILOG
$ straymark charter new -t S --from-ailog AILOG-2026-04-28-021 --title "per-service thresholds"

# Greenfield mode — Charter implementing a SpecKit spec
$ straymark charter new -t L --from-spec specs/001-payments/spec.md --title "wire payment provider"
```

**Example output:**

```
$ straymark charter new -t M --title "test charter"

  ✔ Created: docs/charters/01-test-charter.md

  Next steps:
    1. Edit the Charter to fill in Context, Scope, Files to modify, Verification, Risks, Tasks.
    2. Set the trigger field in frontmatter to a concrete observable signal.
    3. Set originating_ailogs or originating_spec in frontmatter (or leave both absent if standalone).
    4. When you start executing: change frontmatter status from `declared` to `in-progress`.
```

#### `straymark charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

Enumerate Charters as a table.

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` | Target project directory |
| `--status` | `all` | Filter by lifecycle status |
| `--origin` | `any` (no filter) | Filter by origin type: `ailog`, `spec`, or `any` |

Files that fail to parse are reported as warnings to stderr without failing the command — the table lists what it can.

**Example:**

```bash
$ straymark charter list
  NN  STATUS       EFFORT  ORIGIN                 TITLE
  01  declared     M       AILOG-2026-04-28-021   Per-service anomaly thresholds
  02  in-progress  XS      —                      Baseline recompute
  03  closed       L       specs/001/spec.md      Wire payment provider
```

#### `straymark charter status [CHARTER-ID] [--path <dir>]`

With an ID: print the full Charter detail (frontmatter, file location, body section list, Phase 2 placeholders). Without an ID: print the 5 most recent Charters by NN descending.

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `CHARTER-ID` | — | Charter identifier. Accepts the full `charter_id` (`CHARTER-01-test`), the `CHARTER-NN` prefix (`CHARTER-01`), or just the numeric NN (`01` or `1`). Numeric matching is permissive across zero-padding. |
| `--path` | `.` | Target project directory. Use a flag (rather than positional) so it cannot be confused with the optional `CHARTER-ID` positional. |

**Examples:**

```bash
# Most recent 5
$ straymark charter status

# Detail for a specific Charter (any of these resolves to CHARTER-02-baseline-recompute)
$ straymark charter status CHARTER-02-baseline-recompute
$ straymark charter status CHARTER-02
$ straymark charter status 2
```

#### `straymark charter close <CHARTER-ID> [--from-template] [--non-interactive] [--path <dir>]`

Record the post-execution telemetry and bump the Charter's status to `closed`. Telemetry is written to `.straymark/charters/CHARTER-NN.telemetry.yaml` (lateral file, **not** embedded in Charter frontmatter — frontmatter is declarative ex-ante; telemetry is voluminous ex-post). The shape is validated against `.straymark/schemas/charter-telemetry.schema.v0.json`.

Two modes:

| Mode | Flag combination | When to use |
|---|---|---|
| **Interactive** (default) | (none) | Walks the schema field by field with prompts. Target time: 5–10 min. |
| **From template** | `--from-template` | Copies the YAML skeleton next to the Charter for manual editing. Pre-fills `charter_id`, title, `closed_at`. |
| **From template, scripted** | `--from-template --non-interactive` | CI / batch use. Skips prompts entirely; idempotent on re-run. |

| Argument/Flag | Default | Description |
|---|---|---|
| `CHARTER-ID` | — | Same resolution rules as `charter status` |
| `--from-template` | false | Copy the template skeleton instead of running the interactive flow |
| `--non-interactive` | false | Skip all prompts. Requires `--from-template`. |
| `--path` | `.` | Target project directory |

**Example:**

```bash
$ straymark charter close CHARTER-01

  Closing CHARTER-01-test-charter
    Title: Test charter
  Press Enter to accept defaults; type to override.

  ── Trigger ──
  Declared trigger kind › event_trigger
  Declared trigger description › first false-positive ticket
  Fired at (YYYY-MM-DD) [2026-05-02]:
  ...

  ✔ Charter CHARTER-01 closed.
    Telemetry: .straymark/charters/CHARTER-01.telemetry.yaml
    Status updated: in-progress/declared → closed
```

#### `straymark charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--path <dir>]`

Detect file-vs-commit drift at Charter close. Wraps the framework's `.straymark/scripts/check-charter-drift.sh` (zero false positives validated empirically across PLAN-05 retrospective + PLAN-06 prospective in Sentinel). The CLI value-add over the raw script is **AILOG-awareness**: paths reported as "declared but not modified" are silenced when they appear in the `## Risk` / `## Riesgos` / `## 风险` section of any AILOG referenced by the Charter's `originating_ailogs`. Use `--no-ailog-suppress` to disable.

| Argument/Flag | Default | Description |
|---|---|---|
| `CHARTER-ID` | — | Same resolution rules as `charter status` |
| `--range` | `HEAD~1..HEAD` | Git revision range to check |
| `--no-ailog-suppress` *(cli-3.10.0+ always emits a confirming INFO line)* | false | Disable AILOG-aware suppression (show every declared-omitted path). When passed, the CLI always prints an `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` line — including when N=0 — so that the diagnostic mode is visible in output even on a clean run. |
| `--path` | `.` | Target project directory |

**Exit codes:** `0` if no drift (or only AILOG-suppressed); `1` if there's unaccounted drift; `2` for usage errors (Charter not found, bash missing, etc.).

**Example:**

```bash
$ straymark charter drift CHARTER-01 --range origin/main..HEAD
=== Charter drift check ===
  Charter: docs/charters/01-test.md
  Range:   origin/main..HEAD
  Declared: 5 files
  Modified: 3 files

WARNING: Declared in Charter but NOT modified (1 files):
  - src/services/policy/repository.go

AILOG-suppressed: 1 path(s)
  - src/services/policy/repository.go [documented in AILOG-2026-05-02-001]

OK all declared-omitted paths are documented in AILOGs — drift accepted.
```

> **Platform note.** The drift check delegates to `bash`. On Linux/macOS/WSL/Git Bash this works out of the box. On Windows native without WSL, install Git Bash; a pure-Rust fallback is on the roadmap but not in fw-4.6.x.

#### Wildcard support in declared paths *(fw-4.9.0+)*

The drift check resolves two forms of wildcard in `## Files to modify`:

| Form | Example | Use case |
|---|---|---|
| Ellipsis | `` `.straymark/07-ai-audit/agent-logs/AILOG-...md` `` | Any modified path with that prefix satisfies the wildcard. Used historically when an unknown number of AILOGs would be created during execution. |
| Glob | `` `AILOG-*.md` `` or `` `src/services/foo-*.rs` `` | Any modified path matching the glob (`*` → `.*`) satisfies the wildcard. Used for bulk Charter declarations where a parameterized set is touched. Added in fw-4.9.0 after the friction surfaced in Sentinel CHARTER-04 ([issue #81](https://github.com/StrangeDaysTech/straymark/issues/81)). |

Both forms are handled in both directions: a declared wildcard suppresses both "declared but not modified" warnings (when at least one matching file was modified) and "modified but not declared" warnings (when a modified path matches a declared wildcard).

#### Designed: governance paths are always in scope

Paths under `docs/charters/*` and `.straymark/07-ai-audit/*` are **never** reported as "modified but not declared". This is opinionated by design — those paths are always legitimate when the Charter itself or the AILOG of execution is touched. Empirically validated in Sentinel CHARTER-04: a stray `git add -A` staged unrelated user-untracked files (`.claude/skills/`, `cmd/sentinel/sentinel`); the rule correctly suppressed the governance noise without hiding the genuine project-file expansion ([issue #81 W2](https://github.com/StrangeDaysTech/straymark/issues/81#issuecomment-update)).

If you're running a Charter whose explicit scope is governance churn (e.g., a bulk approval Charter touching only `.straymark/07-ai-audit/`), the drift check will report 0 modified files and you'll need to verify scope by reading the AILOG. A `--strict-scope` flag that disables the always-in-scope rule is on the table for a future minor if a real adopter reports the asymmetry as a friction.

#### `straymark charter audit <CHARTER-ID> [--range <REV..REV>] [--prepare | --merge-reports] [--merge-into <PATH>] [--path <dir>]`

*Available since **cli-3.8.0** + **fw-4.7.0**. v1 unified flow shipped in **cli-3.10.0** + **fw-4.9.0** — replaces the v0 three-step (PREPARE/CALIBRATE/FINALIZE) with a two-step (PREPARE/MERGE-REPORTS), unifies the auditor template, and moves canonical paths to `.straymark/audits/`.*

Orchestrate a multi-model external review of a Charter's execution. **Orchestration-only** — the CLI prepares the unified audit prompt, validates auditor reports against the schema, and emits/merges the `external_audit` YAML block. **It does NOT invoke LLM APIs.** The operator runs N auditor-side CLIs (gemini-cli, claude-cli, copilot-cli, codex-cli — whatever they have) configured with read-only filesystem access; each invokes the `/straymark-audit-execute` skill to read the prompt, audit with tool use citing `path:line`, and write the report.

Two steps, each invokable independently:

| Step | Flag | What happens |
|---|---|---|
| 1. PREPARE | `--prepare` (default) | Resolves the unified audit prompt against the Charter + git diff + originating AILOGs. Writes it to `.straymark/audits/<CHARTER-ID>/audit-prompt.md`. |
| 2. MERGE-REPORTS | `--merge-reports` | Reads all `report-*.md` files in `.straymark/audits/<CHARTER-ID>/` (one per auditor that completed). Validates each against `audit-output.schema.v0.json`. Emits the YAML-formatted `external_audit` array — combine with `--merge-into <PATH>` to append it directly into the Charter's telemetry YAML. |

| Argument/Flag | Default | Description |
|---|---|---|
| `<CHARTER-ID>` | — | Same resolution rules as `charter status` |
| `--range` | `origin/main..HEAD` (with fallback to `origin/master..HEAD`, then `HEAD~1..HEAD` with warning) | Git revision range the auditors review. The default captures the full feature-branch implementation set; explicit `--range <REV..REV>` overrides without probing for upstream. |
| `--prepare` | off (default action when no other flag) | Run step 1. Mutually exclusive with `--merge-reports`. |
| `--merge-reports` | off | Run step 2. Mutually exclusive with `--prepare`. |
| `--merge-into <PATH>` | — | With `--merge-reports`: append the `external_audit:` array directly into the telemetry YAML at `<PATH>` instead of printing to stdout. The CLI rejects re-audit (telemetry already has the key) with a clear error. |
| `--path` | `.` | Project directory |

**Deprecated v0 flags (hidden in `--help`):**

- `--calibrate` — emits a warning and exits with error. The v0 calibrate step is replaced by the `/straymark-audit-review` skill, which reconciles N reports inline with filesystem access (no separate paste-based prompt).
- `--finalize` — deprecated alias for `--merge-reports` with backwards-compat behavior. Emits a warning and routes through the new path.

### Heterogeneity recommendation (not enforced in v0)

Per the design rationale (`straymark-cli-roadmap.md` §5.2), the auditor pair should be of **different model families**: one Anthropic + one Google + one OpenAI, in any combination, never two of the same family. Cross-family heterogeneity is what makes convergence on findings high-signal — same-family auditors share blind spots.

v1 supports **N≥2 auditors** (no longer fixed to 2). Operators may opt in to 3 or 4 auditors for high-risk Charters, including specialized models (security-focused, code-review-tuned, etc.). The skill `/straymark-audit-review` iterates over all `report-*.md` files in the audit dir.

The calibrator role moves from a paste-based template (v0) to the in-conversation main agent via the `/straymark-audit-review` skill — its task is definitional (reconcile already-produced verdicts), so heterogeneity from the implementer is NOT required.

### Canonical layout produced (v1)

```
.straymark/audits/CHARTER-NN/
├── audit-prompt.md                          # resolved by --prepare (single unified prompt)
├── report-claude-sonnet-4-6.md              # written by /straymark-audit-execute in claude-cli
├── report-gemini-2-5-pro.md                 # written by /straymark-audit-execute in gemini-cli
├── report-gpt-5-3-codex.md                  # optional 3rd auditor
├── review.md                                # written by /straymark-audit-review (consolidated 6-section analysis)
└── external-audit-pending.yaml              # written by /straymark-audit-review when telemetry doesn't yet exist (Branch B)
```

The directory is namespaced under `.straymark/` to avoid collisions with adopter-defined `audit/` folders. The `<UNIT-TYPE>-<UNIT-ID>` shape leaves room for future audit-unit categories beyond Charter (e.g. `MODULE-payments/`, `RELEASE-v2.0/`) without restructuring.

Adopters can `git add` the entire `.straymark/audits/` directory for a fully version-controlled audit trail, or `.gitignore` it if they prefer the cycle to be ephemeral.

**Example (v1, with the audit-skills wrappers — recommended for IDE-driven workflows):**

```bash
# In the main IDE (Claude Code, Gemini Code, Cursor, ...):
> /straymark-audit-prompt CHARTER-05
  → runs `straymark charter audit CHARTER-05 --prepare`
  → writes .straymark/audits/CHARTER-05/audit-prompt.md
  → instructs operator to open auditor CLIs

# In claude-cli (with read access to repo):
> /straymark-audit-execute CHARTER-05
  → reads .straymark/audits/CHARTER-05/audit-prompt.md
  → audits with tool use, citing path:line
  → writes .straymark/audits/CHARTER-05/report-claude-sonnet-4-6.md
  → reminds operator to wait for ALL audits before review

# In gemini-cli:
> /straymark-audit-execute CHARTER-05
  → writes .straymark/audits/CHARTER-05/report-gemini-2-5-pro.md

# Back in the main IDE, after ALL audits complete:
> /straymark-audit-review CHARTER-05
  → reads N reports, verifies each finding against code
  → writes .straymark/audits/CHARTER-05/review.md (6-section consolidated)
  → runs `straymark charter audit CHARTER-05 --merge-reports --merge-into <telemetry>`
  → external_audit YAML merged into Charter telemetry
```

**Example (CLI direct, without skills — for CI / batch / non-IDE use):**

```bash
$ straymark charter audit CHARTER-05 --prepare
  PREPARE audit prompt (CHARTER-05)
  ✔ Wrote .straymark/audits/CHARTER-05/audit-prompt.md

  Next:
    1. Open one or more auditor CLIs ... and invoke /straymark-audit-execute CHARTER-05.
    2. Each auditor reads the prompt above and writes report-<model-slug>.md.
    3. When ALL audits complete, run: /straymark-audit-review CHARTER-05

# (operator runs auditors in their CLIs of choice; reports land at canonical paths)

$ straymark charter audit CHARTER-05 --merge-reports \
    --merge-into .straymark/charters/CHARTER-05.telemetry.yaml
  MERGE-REPORTS audit cycle (CHARTER-05)
  ✔ Validated .straymark/audits/CHARTER-05/report-claude-sonnet-4-6.md (5 findings)
  ✔ Validated .straymark/audits/CHARTER-05/report-gemini-2-5-pro.md (4 findings)
  ✔ Validated .straymark/audits/CHARTER-05/report-gpt-5-3-codex.md (3 findings)

  Audit cycle merge complete.

  ✔ Merged external_audit array into .straymark/charters/CHARTER-05.telemetry.yaml

  Run `git diff` on the telemetry file to review the merge before commit.
```

> **Why orchestration-only?** Implementing 3 HTTP clients (OpenAI / Google / Anthropic) is 1-2 weeks + perpetual maintenance when APIs change. v1 audit-skills extend the orchestration-only stance to a second mode (CLI auditor-side with tool use enforcement) where the operator runs their own auditor CLIs and StrayMark's prompts enforce the discipline (`cite path:line of files actually opened`). StrayMark still doesn't manage API keys, doesn't invoke APIs, doesn't maintain HTTP clients.

> **Skill alternative *(fw-4.9.0+, expanded in fw-4.9.0)*.** Three skills wrap the CLI for IDE-driven workflows: `/straymark-audit-prompt CHARTER-ID` (calls `--prepare`), `/straymark-audit-execute CHARTER-ID` (runs in auditor CLIs to read the prompt and write a report), and `/straymark-audit-review CHARTER-ID` (consolidates N reports into `review.md` and merges YAML). With these skills the operator never copies/pastes prompts or reports — file exchange happens via the canonical filesystem paths under `.straymark/audits/`. See the [Skills](#skills) section below. The CLI remains the single source of truth — the skills only add UX-inline.

---

### `straymark compliance [path] [--standard <name>] [--region <name>] [--all] [--output <format>]`

Check regulatory compliance. By default, evaluates the standards whose region is in `regional_scope` from `.straymark/config.yml` (default `[global, eu]`). Six Chinese frameworks are available opt-in when `china` is added to `regional_scope`.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--standard` | — | Check a specific standard: `eu-ai-act`, `iso-42001`, `nist-ai-rmf`, `china-tc260`, `china-pipl`, `china-gb45438`, `china-cac`, `china-gb45652`, `china-csl` |
| `--region` | — | Run all standards in a region: `global`, `eu`, `china`, or `all` |
| `--all` | — | Check every standard, regardless of `regional_scope` |
| `--output` | `text` | Output format: `text`, `markdown`, or `json` |

Precedence: `--standard` > `--all` > `--region` > the project's `regional_scope`.

**What it checks:**

Global / EU (always available):

- **EU AI Act**: Risk classification, ethical review linkage, DPIA existence, incident reporting
- **ISO/IEC 42001**: Governance policy, risk planning (ETH), operations documentation (AILOG/AIDEC), Annex A coverage
- **NIST AI RMF**: MAP (AILOG), MEASURE (TES), MANAGE (ETH/INC), GOVERN (policy + ADR), GenAI risk coverage (12 NIST 600-1 categories)

China (opt-in via `regional_scope: china`):

- **TC260 v2.0**: TC260RA exists; high/very-high/extremely-severe levels require review; the three grading criteria (scenario × intelligence × scale) are populated
- **PIPL**: PIPIA exists when `pipl_applicable`; cross-border transfer documented; retention ≥ 3 years per Art. 56
- **GB 45438**: AILABEL exists for generative content; both explicit and implicit labeling tracks declared; mandatory metadata fields populated
- **CAC Algorithm Filing**: CACFILE exists when required; explicit `cac_filing_status`; `cac_filing_number` populated when status is `*_approved`
- **GB/T 45652**: SBOM and MCARD declare training-data security compliance
- **CSL 2026**: Every INC has `csl_severity_level`; deadline hours coherent with severity (1h ↔ particularly_serious, 4h ↔ relatively_major); 30-day post-mortem documented for major+ incidents

**Examples:**

```bash
# Default: runs only standards whose region is in regional_scope
$ straymark compliance
  StrayMark Compliance
  /home/user/my-project
  12 document(s) analyzed

  ■ EU AI Act 75%
    ✓ [EU-001] AI systems have EU AI Act risk classification
    ~ [EU-002] High-risk AI systems have ethical review (ETH) linked
    ✓ [EU-003] Data Protection Impact Assessment (DPIA) exists where required
    ✓ [EU-004] Incident reporting compliant with EU AI Act Art. 73

  ■ ISO/IEC 42001 100%
    ✓ [ISO-001] AI Governance Policy exists (Clauses 4-5)
    ✓ [ISO-002] Risk planning documented — ETH reviews exist (Clause 6)
    ✓ [ISO-003] AI lifecycle operations documented — AILOG + AIDEC (Clause 8)
    ✓ [ISO-004] Annex A control coverage (6/6 groups)

  ■ NIST AI RMF 60%
    ~ [NIST-GENAI-001] GenAI risk coverage — NIST AI 600-1 (4/12 categories)

  Overall compliance: 78%

# Run only the six Chinese frameworks (requires regional_scope: china)
$ straymark compliance --region china
  ■ China TC260 v2.0 67%
    ✓ [TC260-001] At least one TC260 Risk Assessment (TC260RA) is present
    ~ [TC260-002] High / very-high / extremely-severe TC260 levels mandate review
    ✗ [TC260-003] TC260RA documents specify scenario × intelligence × scale

  ■ China PIPL 100%
    ✓ [PIPL-001] PIPIA exists when pipl_applicable is true
    ✓ [PIPL-002] Documents handling sensitive personal info link to a PIPIA
    ✓ [PIPL-003] Cross-border personal info transfer is documented in a PIPIA
    ✓ [PIPL-004] PIPIA retention is ≥ 3 years per PIPL Art. 56

  ■ China GB 45438 ...
  ■ China CAC Algorithm Filing ...
  ■ China GB/T 45652 ...
  ■ China CSL 2026 ...

# A single Chinese framework
$ straymark compliance --standard china-pipl --output json
[{"standard":"ChinaPipl","standard_label":"China PIPL","checks":[...],"score":100.0}]

# Force every standard, ignoring regional_scope
$ straymark compliance --all
```

> **Activation note**: Chinese frameworks evaluate only when you opt in. Add to `.straymark/config.yml`:
>
> ```yaml
> regional_scope:
>   - global
>   - eu
>   - china
> ```
>
> Use `--standard china-*` or `--region china` to run them ad-hoc even when not in scope. See the `CHINA-REGULATORY-FRAMEWORK.md` guide installed under `.straymark/00-governance/`.

---

### `straymark metrics [path] [--period <period>] [--output <format>]`

Show governance metrics and documentation statistics.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--period` | `last-30-days` | Time period: `last-7-days`, `last-30-days`, `last-90-days`, or `all` |
| `--output` | `text` | Output format: `text`, `markdown`, or `json` |

**Metrics included:**

- Document count by type within the period
- Review compliance rate (% of review_required docs that reached accepted/superseded status)
- Risk distribution (low/medium/high/critical)
- Agent activity (documents per agent)
- Trends vs previous period (↑/↓/→)

**Example:**

```bash
$ straymark metrics --period last-30-days
  StrayMark Metrics
  /home/user/my-project
  Period: Last 30 days — 2026-02-25 to 2026-03-27

  Documents by Type
     AILOG   8 ████████
       ETH   3 ███
       ADR   2 ██
       INC   1 █

  Summary
    → Total documents: 14
    → Review compliance: 80% (4/5 reviewed)

  Risk Distribution
          low 8
       medium 4
         high 2

  Agent Activity
    claude-code 10
    gemini-cli 4

  Trends
    ↑ Total documents 14 (was 9)
    ↑ Reviews completed 4 (was 2)
    → High/critical risk 2 (was 2)
```

---

### `straymark analyze [path] [--threshold <N>] [--output <format>] [--top <N>]`

Analyze code complexity using cognitive and cyclomatic metrics powered by [arborist-metrics](https://crates.io/crates/arborist-metrics).

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target directory to analyze |
| `--threshold` | `8` (or from config) | Cognitive complexity threshold |
| `--output` | `text` | Output format: `text`, `json`, or `markdown` |
| `--top` | — | Show only top N most complex functions |

**Supported languages:** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**Threshold resolution:** CLI flag → `.straymark/config.yml` → default (8)

**Configuration** (optional, in `.straymark/config.yml`):

```yaml
complexity:
  threshold: 8
```

**Examples:**

```bash
# Analyze current directory
$ straymark analyze

# Custom threshold and top 10
$ straymark analyze --threshold 5 --top 10

# JSON output for CI integration
$ straymark analyze --output json

# Analyze a specific project
$ straymark analyze /path/to/project
```

**Example output:**

```
  StrayMark Analyze
  /home/user/project
  Threshold: cognitive complexity > 8

  Functions exceeding threshold (3 of 42 total)

    FILE                                     FUNCTION                  LINE  COGN  CYCL  SLOC
    src/parser.rs                            parse_expression            42    18    12    45
    src/compiler.rs                          Compiler::emit             128    15     9    38
    src/eval.rs                              evaluate                    67    12     8    29

  Summary
    → Files analyzed: 12
    → Total functions: 42
    → Above threshold: 3 (7.1%)
    → Max cognitive complexity: 18 (src/parser.rs:parse_expression)
    → Average cognitive complexity: 3.8
```

> **Note:** This command works without `straymark init`. It operates on source files, not StrayMark documents. The `analyze` feature can be disabled at compile time with `--no-default-features`.

> **Documentation trigger:** AI agents use `straymark analyze --output json` as the primary method to determine when to create AILOG documents. If `summary.above_threshold > 0` in the JSON output, the agent should create an AILOG. When the CLI is not available, agents fall back to the >20 lines of business logic heuristic.

---

### `straymark audit [path] [--from <date>] [--to <date>] [--system <name>] [--output <format>]`

Generate audit trail reports with timeline, traceability map, and compliance summary.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--from` | — | Start date for audit period (YYYY-MM-DD) |
| `--to` | — | End date for audit period (YYYY-MM-DD) |
| `--system` | — | Filter by system/component name (matches tags and title) |
| `--output` | `text` | Output format: `text`, `markdown`, `json`, or `html` |

**Report includes:**

- Chronological timeline of all documents with type, title, agent, and risk level
- Traceability map showing document relationship chains (e.g., REQ → ADR → AILOG → TES)
- Risk distribution (low/medium/high/critical)
- Compliance summary (EU AI Act, ISO 42001, NIST AI RMF scores)

**Output formats:**

| Format | Use case |
|--------|----------|
| `text` | Terminal review (colored, formatted) |
| `markdown` | Include in PRs, wikis, or reports |
| `json` | Integration with external tools |
| `html` | Standalone reports with styled tables and SVG risk chart |

**Examples:**

```bash
# Full audit report
$ straymark audit

# Audit for Q1 2026
$ straymark audit --from 2026-01-01 --to 2026-03-31

# Audit filtered by system
$ straymark audit --system auth-service

# Generate HTML report
$ straymark audit --from 2026-01-01 --to 2026-03-31 --output html > audit-q1.html

# Generate Markdown for a PR
$ straymark audit --output markdown
```

---

### `straymark explore [path]`

Browse and read StrayMark documentation interactively in a terminal UI.

**Arguments:**

| Argument | Default | Description |
|----------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--lang <code>` | resolved from project (see below) | Display language for the TUI shell and framework governance docs (`en`, `es`, `zh-CN`). Falls back silently to English when a translation is missing. |

**Language resolution order** (since cli-3.5.2):

1. `--lang <code>` flag, when provided
2. `language` field in `.straymark/config.yml`, when the file exists (an explicit value — even `language: en` — is treated as a deliberate user choice)
3. `$LC_ALL` / `$LANG` env vars, mapped to a supported locale (e.g., `zh_CN.UTF-8` → `zh-CN`, `es_MX.UTF-8` → `es`). Traditional Chinese (`zh_TW` / `zh_HK`) and other unsupported locales fall through.
4. `en`

**Features:**

- Two-panel layout: navigation tree + document viewer
- Metadata panel showing status, confidence, risk, tags, and related links
- Markdown rendering with colors, tables, code blocks, and heading indentation
- Navigate between related documents via hyperlinks
- Search by filename, title, tags, or date
- Fullscreen document mode, with `j` / `k` as alternate keys for `↓` / `↑`
- Localization-aware: framework docs (`QUICK-REFERENCE`, `AGENT-RULES`, China regulatory guides, etc.) are served in the language set by `language` in `.straymark/config.yml` or by `--lang`

**Key bindings:**

| Key | Action |
|-----|--------|
| `↑↓` / `j/k` | Navigate / Scroll |
| `Enter` | Expand group / Open document |
| `Tab` | Cycle panels: Navigation → Metadata → Document |
| `f` | Toggle fullscreen document |
| `/` | Search |
| `L` | Cycle display language (`en → es → zh-CN`) |
| `Esc` | Back / Collapse / Clear search |
| `?` | Help popup with all shortcuts |
| `q` | Quit |

**Examples:**

```bash
$ straymark explore                       # uses config.language (defaults to en)
$ straymark explore --lang zh-CN          # browse framework docs in Simplified Chinese
$ straymark explore --lang es             # session override to Spanish
```

> **Note:** The `explore` command requires the `tui` feature (enabled by default). To compile without it: `cargo build --no-default-features`.

---

### `straymark about`

Show version, authorship, and license information.

**Example:**

```bash
$ straymark about
StrayMark CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.11.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/straymark
  Website:           https://strangedays.tech
```

---

## Skills

StrayMark ships a set of skills (slash commands) for use inside an AI assistant (Claude Code, Gemini Code, Cursor, generic agent runtimes). Each skill is installed in 3 parallel forms during `straymark init`:

- `dist/.claude/skills/<skill>/SKILL.md` (Claude — frontmatter with `allowed-tools`)
- `dist/.gemini/skills/<skill>/SKILL.md` (Gemini — frontmatter without `allowed-tools`)
- `dist/.agent/workflows/<skill>.md` (generic agent — `description`-only frontmatter)

| Skill | Purpose | Files produced |
|---|---|---|
| `/straymark-status` | Check documentation compliance for recent changes. | none (read-only) |
| `/straymark-new` | Create any document type interactively. Suggests best fit from context. | `.straymark/<type-dir>/<TYPE>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | Quick AILOG creation shortcut. | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | Quick AIDEC creation shortcut. | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | Quick ADR creation shortcut. | `.straymark/04-architecture/decisions/ADR-*.md` |
| `/straymark-mcard` | Interactive Model Card creation flow. | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | Interactive SEC (security assessment) flow. | `.straymark/08-security/SEC-*.md` |
| `/straymark-audit-prompt CHARTER-ID` *(fw-4.9.0+, refactored in fw-4.9.0)* | Generate the unified audit prompt for a Charter at the canonical path. Wraps `straymark charter audit --prepare`. The operator then opens N auditor CLIs in the same repo and invokes `/straymark-audit-execute` in each — no copy/paste. | `.straymark/audits/<CHARTER-ID>/audit-prompt.md` |
| `/straymark-audit-execute [CHARTER-ID]` *(fw-4.9.0+)* | **Run inside an auditor-side CLI** (gemini-cli, claude-cli, copilot-cli, codex-cli, ...). Reads the prepared prompt from disk, audits with tool use citing `path:line`, writes a report keyed on the auditor's model id. CHARTER-ID argument is optional — auto-discovers prompts that don't yet have a report from this model. | `.straymark/audits/<CHARTER-ID>/report-<sluggified-model-id>.md` |
| `/straymark-audit-review CHARTER-ID` *(fw-4.9.0+, expanded in fw-4.9.0)* | Counterpart to `/straymark-audit-prompt`. Reads N reports under `.straymark/audits/<CHARTER-ID>/`, verifies each finding against actual code (Explore agents in parallel), produces a six-section consolidated `review.md` (Executive summary, Scope, Per-auditor evaluation, Remediation plan P0-P4, Discarded findings, Auditor ratings), and runs `straymark charter audit --merge-reports --merge-into` to append `external_audit:` into the Charter telemetry. If the telemetry doesn't yet exist (Charter not yet closed), writes `external-audit-pending.yaml` for later merge at close time. | `.straymark/audits/<CHARTER-ID>/review.md`, `external_audit:` array merged into telemetry (or pending YAML) |

### Skill vs CLI

The three audit skills are **wrappers** around the CLI commands and discipline. The canonical paths under `.straymark/audits/`, the unified prompt template, the schema validation, and the `external_audit` shape all live in the CLI + framework — the skills handle the UX-inline part: dispatching the operator across the audit cycle without manual file management. **Operators never copy/paste prompts or reports** — the skills exchange artifacts via the canonical filesystem paths.

Adopters using StrayMark without an AI assistant in the loop can drive the same workflow directly via `straymark charter audit` (`--prepare` / `--merge-reports [--merge-into <path>]`). The audit prompt at `.straymark/audits/<id>/audit-prompt.md` works equally well pasted into a chat-based LLM if no auditor-side CLI is available — the skill just automates the file exchange.

### Audit checkpoint *(fw-4.9.0+)*

`.straymark/00-governance/AGENT-RULES.md` §12 codifies a workflow checkpoint where the agent proactively offers the audit at one specific moment — when the Charter implementation is done, drift is clean, and `charter close` has not yet been invoked. The recommendation is YES/NO based on heuristics (security surface, new components, AILOG risks, complexity). External audit is **fully optional**; the checkpoint is **soft** — never blocks `charter close`, never enforced (permanent v0+v1 design decision).

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub personal access token for authenticated API requests. Useful to avoid rate limits when downloading releases. |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (details printed to stderr) |

---

<div align="center">

**StrayMark** — Because every change tells a story.

[Back to docs](../README.md) • [README](../../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
