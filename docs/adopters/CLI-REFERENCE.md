# DevTrail CLI Reference

**Complete reference for the `devtrail` command-line tool.**

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

Install the DevTrail CLI using one of the methods below. For full setup instructions, see the [README](../../README.md#getting-started).

**Quick install (prebuilt binary):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

**From source:**

```bash
cargo install devtrail-cli
```

---

## Versioning

DevTrail uses **independent version tags** for each component:

| Component | Tag prefix | Example | What it includes |
|-----------|-----------|---------|------------------|
| Framework | `fw-` | `fw-4.7.1` | Templates (12 types), governance docs, directives, Charter template + schema |
| CLI | `cli-` | `cli-3.8.1` | The `devtrail` binary |

Framework and CLI are released independently. A framework update does not require a CLI update, and vice versa.

**Check installed versions:**

```bash
devtrail about    # Shows CLI version + framework version (if installed)
devtrail status   # Shows full installation health including versions
```

---

## Commands

### `devtrail init [path] [--hooks]`

Initialize DevTrail in a project directory.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---|---|---|
| `path` | `.` (current directory) | Target project directory |
| `--hooks` *(cli-3.7.0+)* | off | After init, install the framework's pre-PR hook (`.devtrail/hooks/pre-pr.sh`) as `.git/hooks/pre-push`. Runs `devtrail charter drift` automatically before each push. Opt-in per principle #6 (cognitive discipline > raw productivity). Refuses to overwrite an existing `pre-push` hook; skips silently if not a git repo. |

**What it does:**

1. Downloads the latest framework release (`fw-*`) from GitHub
2. Creates the `.devtrail/` directory structure
3. Creates `DEVTRAIL.md` with governance rules
4. Configures AI agent directive files (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
5. Copies CI/CD workflows
6. *(`--hooks`)* installs the pre-PR hook

**Example:**

```bash
$ devtrail init .
✔ Downloaded DevTrail fw-4.7.1
✔ Created .devtrail/ directory structure
✔ Created DEVTRAIL.md
✔ Configured AI agent directives

DevTrail initialized successfully!
Next: git add .devtrail/ DEVTRAIL.md && git commit -m "chore: adopt DevTrail"
```

---

### `devtrail update`

Update **both** framework and CLI to their latest versions. Equivalent to running `update-framework` followed by `update-cli`.

If `.devtrail/` does not exist in the current directory, the framework update is skipped with a warning.

**Example:**

```bash
$ devtrail update
Updating framework...
✔ Framework updated to fw-4.7.1
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail update-framework`

Update only the framework files. Looks for the latest `fw-*` release on GitHub.

**Conflict handling:** If you have modified framework files (e.g., governance docs or templates), the update preserves your changes and reports conflicts for manual resolution.

**Example:**

```bash
$ devtrail update-framework
✔ Framework updated to fw-4.7.1
```

---

### `devtrail update-cli`

Auto-update the `devtrail` binary itself. Automatically detects the installation method and uses the appropriate update mechanism:

- **Prebuilt binary** (installed via `install.sh` / `install.ps1`): Downloads the latest binary from GitHub Releases
- **Cargo** (installed via `cargo install`): Runs `cargo install --force devtrail-cli`

Use `--method` to override auto-detection: `--method=github` or `--method=cargo`.

**Example:**

```bash
$ devtrail update-cli
✔ CLI updated to cli-3.5.2

$ devtrail update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail remove [--full]`

Remove DevTrail from the current project.

**Flags:**

| Flag | Description |
|------|-------------|
| `--full` | Remove everything, including user-created documents in `.devtrail/`. Asks for confirmation. |

**Default behavior** (without `--full`): removes the framework structure but preserves documents you created inside `.devtrail/`.

**Example:**

```bash
$ devtrail remove
✔ DevTrail framework removed. User documents preserved in .devtrail/.

$ devtrail remove --full
⚠ This will delete all DevTrail files including your documents.
Continue? [y/N]: y
✔ DevTrail completely removed.
```

---

### `devtrail status [path]`

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
$ devtrail status

  ╔════════════════════════════════════════════════╗
  ║ DevTrail Status                                ║
  ╚════════════════════════════════════════════════╝

  Project
  ┌───────────┬──────────────────────────┐
  │ Path      │ /home/user/my-project    │
  │ Framework │ fw-4.7.1                 │
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

  → Run devtrail explore to browse documentation interactively
```

---

### `devtrail repair [path]`

Repair a broken DevTrail installation by restoring missing directories and framework files.

**Arguments:**

| Argument | Default | Description |
|----------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |

**What it does:**

1. Checks for missing directories and restores them with `.gitkeep`
2. Downloads the framework release **once** if files need restoration (templates, governance docs, config)
3. Re-injects directives if `DEVTRAIL.md` is missing
4. Recalculates checksums after repair
5. Never modifies or deletes user-generated documents

**Example:**

```bash
$ devtrail repair
Repairing DevTrail in /home/user/my-project
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .devtrail/templates/
→ Downloading framework to restore missing files...
  Using version: fw-4.7.1
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ DevTrail repaired successfully!
```

---

### `devtrail validate [path] [--fix] [--staged] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Validate DevTrail documents for compliance and correctness.

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
$ devtrail validate
  DevTrail Validate
  All 15 document(s) passed validation
  0 error(s), 0 warning(s) in 15 document(s)

$ devtrail validate --fix
  DevTrail Validate
  Auto-fixing 2 issue(s)...
  ✓ Fixed 2 issue(s)
```

---

### `devtrail approve <doc-id> --outcome <outcome> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."] [--path <dir>]`

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
$ devtrail approve AIDEC-2026-05-02-001 \
    --outcome approved \
    --reviewer pepe@example.com \
    --notes "Reviewed against ADR-007. LGTM."

  ✔ AIDEC-2026-05-02-001 marked as approved.
    Reviewer: pepe@example.com
    Date:     2026-05-02
    File:     .devtrail/07-ai-audit/decisions/AIDEC-2026-05-02-001-foo.md

# Iterative review cycle: revisions_requested → re-approve
$ devtrail approve AIDEC-... --outcome revisions_requested --reviewer reviewer@x.io
# (author iterates)
$ devtrail approve AIDEC-... --outcome approved --reviewer reviewer@x.io
# Frontmatter shows the latest (approved); body shows BOTH blocks chronologically.

# Backlog visibility
$ devtrail validate --check-pending-reviews --max-pending-days 14
```

> See `dist/.devtrail/00-governance/DOCUMENTATION-POLICY.md` §3.5 "Recording Approval" for the canonical workflow definition (closure semantics, body format, multi-reviewer convention).

---

### `devtrail new [path] [-t <type>] [--title <title>]`

Create a new DevTrail document from a template.

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
$ devtrail new

# Create an AILOG with a title (non-interactive)
$ devtrail new -t ailog --title "Implement JWT authentication"

# Create an ADR
$ devtrail new --doc-type adr --title "Use PostgreSQL for persistence"
```

**Example output:**

```
$ devtrail new -t ailog --title "Implement JWT authentication"

  ✔ Created: .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md
```

---

### `devtrail charter <subcommand>`

Manage **Charters**: bounded, auditable units of work declared ex-ante and validated ex-post. A Charter pairs declarative scope (files to touch, risks, executable verification) with ex-post audit anchoring (drift detection, multi-model audit). Charters live at `docs/charters/NN-slug.md` (project-root level, **not** under `.devtrail/`).

> **Naming history.** In the Sentinel `/plan-audit` experiment that crystallized this pattern (2026-04, 6 cycles), Charters were called *Plans*. The DevTrail CLI uses **Charter** going forward to disambiguate from GitHub SpecKit's `plan.md`. Sentinel's historical files preserve "Plan" deliberately. The full conceptual scope and the rename rationale live in `Propuesta/que-es-un-charter.md`.

**Subcommands:**

- `devtrail charter new` — scaffold a new Charter from the framework template
- `devtrail charter list` — enumerate Charters with optional filters
- `devtrail charter status` — show Charter detail, or the most recent 5 Charters
- `devtrail charter close` — record post-execution telemetry and bump status to `closed` *(Phase 2, fw-4.6.0+)*
- `devtrail charter drift` — detect file-vs-commit drift with AILOG-aware suppression *(Phase 2, fw-4.6.0+)*
- `devtrail charter audit` — orchestrate a multi-model external review (3-step prepare/calibrate/finalize) *(Phase 3, fw-4.7.1+)*

#### `devtrail charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

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
$ devtrail charter new --type M

# Maintenance / post-MVP mode — Charter rooted in an existing AILOG
$ devtrail charter new -t S --from-ailog AILOG-2026-04-28-021 --title "per-service thresholds"

# Greenfield mode — Charter implementing a SpecKit spec
$ devtrail charter new -t L --from-spec specs/001-payments/spec.md --title "wire payment provider"
```

**Example output:**

```
$ devtrail charter new -t M --title "test charter"

  ✔ Created: docs/charters/01-test-charter.md

  Next steps:
    1. Edit the Charter to fill in Context, Scope, Files to modify, Verification, Risks, Tasks.
    2. Set the trigger field in frontmatter to a concrete observable signal.
    3. Set originating_ailogs or originating_spec in frontmatter (or leave both absent if standalone).
    4. When you start executing: change frontmatter status from `declared` to `in-progress`.
```

#### `devtrail charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

Enumerate Charters as a table.

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` | Target project directory |
| `--status` | `all` | Filter by lifecycle status |
| `--origin` | `any` (no filter) | Filter by origin type: `ailog`, `spec`, or `any` |

Files that fail to parse are reported as warnings to stderr without failing the command — the table lists what it can.

**Example:**

```bash
$ devtrail charter list
  NN  STATUS       EFFORT  ORIGIN                 TITLE
  01  declared     M       AILOG-2026-04-28-021   Per-service anomaly thresholds
  02  in-progress  XS      —                      Baseline recompute
  03  closed       L       specs/001/spec.md      Wire payment provider
```

#### `devtrail charter status [CHARTER-ID] [--path <dir>]`

With an ID: print the full Charter detail (frontmatter, file location, body section list, Phase 2 placeholders). Without an ID: print the 5 most recent Charters by NN descending.

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `CHARTER-ID` | — | Charter identifier. Accepts the full `charter_id` (`CHARTER-01-test`), the `CHARTER-NN` prefix (`CHARTER-01`), or just the numeric NN (`01` or `1`). Numeric matching is permissive across zero-padding. |
| `--path` | `.` | Target project directory. Use a flag (rather than positional) so it cannot be confused with the optional `CHARTER-ID` positional. |

**Examples:**

```bash
# Most recent 5
$ devtrail charter status

# Detail for a specific Charter (any of these resolves to CHARTER-02-baseline-recompute)
$ devtrail charter status CHARTER-02-baseline-recompute
$ devtrail charter status CHARTER-02
$ devtrail charter status 2
```

#### `devtrail charter close <CHARTER-ID> [--from-template] [--non-interactive] [--path <dir>]`

Record the post-execution telemetry and bump the Charter's status to `closed`. Telemetry is written to `.devtrail/charters/CHARTER-NN.telemetry.yaml` (lateral file, **not** embedded in Charter frontmatter — frontmatter is declarative ex-ante; telemetry is voluminous ex-post). The shape is validated against `.devtrail/schemas/charter-telemetry.schema.v0.json`.

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
$ devtrail charter close CHARTER-01

  Closing CHARTER-01-test-charter
    Title: Test charter
  Press Enter to accept defaults; type to override.

  ── Trigger ──
  Declared trigger kind › event_trigger
  Declared trigger description › first false-positive ticket
  Fired at (YYYY-MM-DD) [2026-05-02]:
  ...

  ✔ Charter CHARTER-01 closed.
    Telemetry: .devtrail/charters/CHARTER-01.telemetry.yaml
    Status updated: in-progress/declared → closed
```

#### `devtrail charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--path <dir>]`

Detect file-vs-commit drift at Charter close. Wraps the framework's `.devtrail/scripts/check-charter-drift.sh` (zero false positives validated empirically across PLAN-05 retrospective + PLAN-06 prospective in Sentinel). The CLI value-add over the raw script is **AILOG-awareness**: paths reported as "declared but not modified" are silenced when they appear in the `## Risk` / `## Riesgos` / `## 风险` section of any AILOG referenced by the Charter's `originating_ailogs`. Use `--no-ailog-suppress` to disable.

| Argument/Flag | Default | Description |
|---|---|---|
| `CHARTER-ID` | — | Same resolution rules as `charter status` |
| `--range` | `HEAD~1..HEAD` | Git revision range to check |
| `--no-ailog-suppress` *(cli-3.8.1+ always emits a confirming INFO line)* | false | Disable AILOG-aware suppression (show every declared-omitted path). When passed, the CLI always prints an `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` line — including when N=0 — so that the diagnostic mode is visible in output even on a clean run. |
| `--path` | `.` | Target project directory |

**Exit codes:** `0` if no drift (or only AILOG-suppressed); `1` if there's unaccounted drift; `2` for usage errors (Charter not found, bash missing, etc.).

**Example:**

```bash
$ devtrail charter drift CHARTER-01 --range origin/main..HEAD
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

#### Wildcard support in declared paths *(fw-4.7.1+)*

The drift check resolves two forms of wildcard in `## Files to modify`:

| Form | Example | Use case |
|---|---|---|
| Ellipsis | `` `.devtrail/07-ai-audit/agent-logs/AILOG-...md` `` | Any modified path with that prefix satisfies the wildcard. Used historically when an unknown number of AILOGs would be created during execution. |
| Glob | `` `AILOG-*.md` `` or `` `src/services/foo-*.rs` `` | Any modified path matching the glob (`*` → `.*`) satisfies the wildcard. Used for bulk Charter declarations where a parameterized set is touched. Added in fw-4.7.1 after the friction surfaced in Sentinel CHARTER-04 ([issue #81](https://github.com/StrangeDaysTech/devtrail/issues/81)). |

Both forms are handled in both directions: a declared wildcard suppresses both "declared but not modified" warnings (when at least one matching file was modified) and "modified but not declared" warnings (when a modified path matches a declared wildcard).

#### Designed: governance paths are always in scope

Paths under `docs/charters/*` and `.devtrail/07-ai-audit/*` are **never** reported as "modified but not declared". This is opinionated by design — those paths are always legitimate when the Charter itself or the AILOG of execution is touched. Empirically validated in Sentinel CHARTER-04: a stray `git add -A` staged unrelated user-untracked files (`.claude/skills/`, `cmd/sentinel/sentinel`); the rule correctly suppressed the governance noise without hiding the genuine project-file expansion ([issue #81 W2](https://github.com/StrangeDaysTech/devtrail/issues/81#issuecomment-update)).

If you're running a Charter whose explicit scope is governance churn (e.g., a bulk approval Charter touching only `.devtrail/07-ai-audit/`), the drift check will report 0 modified files and you'll need to verify scope by reading the AILOG. A `--strict-scope` flag that disables the always-in-scope rule is on the table for a future minor if a real adopter reports the asymmetry as a friction.

#### `devtrail charter audit <CHARTER-ID> [--range <REV..REV>] [--calibrate | --finalize] [--path <dir>]`

*Available since **cli-3.8.0** + **fw-4.7.0** (Phase 3 v0).*

Orchestrate a multi-model external review of a Charter's execution. **Orchestration-only** — the CLI prepares prompts, validates outputs against the schema, and prints findings ready to paste into Charter telemetry. **It does NOT invoke LLM APIs.** The operator runs the prompts in their auditor of choice (Copilot, Gemini, Claude, etc.) and saves responses to canonical paths.

Three steps, each invokable independently:

| Step | Flag | What happens |
|---|---|---|
| 1. PREPARE | (default) | Resolves `auditor-primary` and `auditor-secondary` prompts against the Charter + git diff + originating AILOGs. Writes them under `audit/charters/<CHARTER-ID>/prompts/`. |
| 2. CALIBRATE | `--calibrate` | Reads `auditor-primary.md` and `auditor-secondary.md` (operator must save these between steps 1 and 2). Validates them against `audit-output.schema.v0.json`. Resolves the calibrator prompt with both responses embedded. |
| 3. FINALIZE | `--finalize` | Reads the calibrator response. Validates all 3 outputs. Prints a YAML-formatted `external_audit` array block ready to paste into the Charter telemetry. |

| Argument/Flag | Default | Description |
|---|---|---|
| `<CHARTER-ID>` | — | Same resolution rules as `charter status` |
| `--range` | `HEAD~1..HEAD` | Git revision range the auditors will review |
| `--calibrate` | off | Run step 2. Mutually exclusive with `--finalize`. |
| `--finalize` | off | Run step 3. Mutually exclusive with `--calibrate`. |
| `--path` | `.` | Project directory |

### Heterogeneity recommendation (not enforced in v0)

Per the design rationale (`devtrail-cli-roadmap.md` §5.2), the auditor pair should be of **different model families**: one Anthropic + one Google + one OpenAI, in any combination, never two of the same family. Cross-family heterogeneity is what makes convergence on findings high-signal — same-family auditors share blind spots.

The calibrator-reconciler MAY be of any family (including the implementer's family) because its task is definitional (apply the schema to already-produced verdicts), not discovery. Heterogeneity matters for the auditor pair, not the calibrator.

v0 documents this recommendation but does not auto-detect or enforce it. A `--implementer-family X` flag with rejection of monochromatic configurations is a v1 candidate when an adopter reports a real case.

### Layout produced

```
audit/charters/CHARTER-NN/
├── prompts/
│   ├── auditor-primary.prompt.md      # resolved by step 1, what was sent
│   ├── auditor-secondary.prompt.md    # resolved by step 1
│   └── calibrator-reconciler.prompt.md  # resolved by step 2
├── auditor-primary.md                 # operator pastes auditor 1 response
├── auditor-secondary.md               # operator pastes auditor 2 response
└── calibrator-reconciler.md           # operator pastes calibrator response
```

The `prompts/` subdirectory persists what was sent to each auditor *before* the API call (closes [RFC #82](https://github.com/StrangeDaysTech/devtrail/issues/82) on audit visibility). Adopters can `git add` the entire `audit/` directory for a fully version-controlled audit trail, or `.gitignore` it if they prefer the cycle to be ephemeral.

**Example:**

```bash
$ devtrail charter audit CHARTER-05
  Step 1/3: PREPARE (CHARTER-05)
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-primary.prompt.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-secondary.prompt.md

  Next:
    1. Paste each prompt into your auditor of choice (use a model
       of a different family per auditor — see CLI-REFERENCE).
    2. Save the auditor responses to:
         audit/charters/CHARTER-05/auditor-primary.md
         audit/charters/CHARTER-05/auditor-secondary.md
    3. Run: devtrail charter audit CHARTER-05 --calibrate

# (operator runs auditor 1 in Copilot, saves response. Runs auditor 2
# in Gemini, saves response.)

$ devtrail charter audit CHARTER-05 --calibrate
  Step 2/3: CALIBRATE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/calibrator-reconciler.prompt.md

  Next:
    1. Run the calibrator prompt in a model of your choice (calibrator
       may be of any family).
    2. Save the response to: audit/charters/CHARTER-05/calibrator-reconciler.md
    3. Run: devtrail charter audit CHARTER-05 --finalize

# (operator runs calibrator in Claude, saves response.)

$ devtrail charter audit CHARTER-05 --finalize
  Step 3/3: FINALIZE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md (5 findings, prompt: prompts/auditor-primary.prompt.md)
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md (4 findings, prompt: prompts/auditor-secondary.prompt.md)
  ✔ Validated audit/charters/CHARTER-05/calibrator-reconciler.md

  Charter audit complete.

  external_audit YAML — paste into telemetry:
    - auditor: "copilot-v1.0.37"
      findings_total: 5
      findings_by_category:
        hallucination: 0
        implementation_gap: 2
        real_debt: 2
        false_positive: 1
      audit_quality: "high"
      audit_notes: "see audit/charters/<charter-id>/auditor-primary.md"
    - auditor: "gemini-cli-v1.5"
      findings_total: 4
      findings_by_category: ...

  Calibrator summary (copy to outcome.scope_change_notes if relevant):
    audit/charters/CHARTER-05/calibrator-reconciler.md
```

> **Why orchestration-only?** Implementing 3 HTTP clients (OpenAI / Google / Anthropic) is 1-2 weeks + perpetual maintenance when APIs change. Phase 3 v0 is experimental — the CLI's value is the canon (prompt shape + output schema + telemetry integration), not the API call. v1 may add HTTP clients when an adopter reports a real need; until then the human-in-the-loop shape matches Sentinel's empirical `/plan-audit` pattern that motivated Phase 3 in the first place.

---

### `devtrail compliance [path] [--standard <name>] [--region <name>] [--all] [--output <format>]`

Check regulatory compliance. By default, evaluates the standards whose region is in `regional_scope` from `.devtrail/config.yml` (default `[global, eu]`). Six Chinese frameworks are available opt-in when `china` is added to `regional_scope`.

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
$ devtrail compliance
  DevTrail Compliance
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
$ devtrail compliance --region china
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
$ devtrail compliance --standard china-pipl --output json
[{"standard":"ChinaPipl","standard_label":"China PIPL","checks":[...],"score":100.0}]

# Force every standard, ignoring regional_scope
$ devtrail compliance --all
```

> **Activation note**: Chinese frameworks evaluate only when you opt in. Add to `.devtrail/config.yml`:
>
> ```yaml
> regional_scope:
>   - global
>   - eu
>   - china
> ```
>
> Use `--standard china-*` or `--region china` to run them ad-hoc even when not in scope. See the `CHINA-REGULATORY-FRAMEWORK.md` guide installed under `.devtrail/00-governance/`.

---

### `devtrail metrics [path] [--period <period>] [--output <format>]`

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
$ devtrail metrics --period last-30-days
  DevTrail Metrics
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

### `devtrail analyze [path] [--threshold <N>] [--output <format>] [--top <N>]`

Analyze code complexity using cognitive and cyclomatic metrics powered by [arborist-metrics](https://crates.io/crates/arborist-metrics).

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target directory to analyze |
| `--threshold` | `8` (or from config) | Cognitive complexity threshold |
| `--output` | `text` | Output format: `text`, `json`, or `markdown` |
| `--top` | — | Show only top N most complex functions |

**Supported languages:** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**Threshold resolution:** CLI flag → `.devtrail/config.yml` → default (8)

**Configuration** (optional, in `.devtrail/config.yml`):

```yaml
complexity:
  threshold: 8
```

**Examples:**

```bash
# Analyze current directory
$ devtrail analyze

# Custom threshold and top 10
$ devtrail analyze --threshold 5 --top 10

# JSON output for CI integration
$ devtrail analyze --output json

# Analyze a specific project
$ devtrail analyze /path/to/project
```

**Example output:**

```
  DevTrail Analyze
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

> **Note:** This command works without `devtrail init`. It operates on source files, not DevTrail documents. The `analyze` feature can be disabled at compile time with `--no-default-features`.

> **Documentation trigger:** AI agents use `devtrail analyze --output json` as the primary method to determine when to create AILOG documents. If `summary.above_threshold > 0` in the JSON output, the agent should create an AILOG. When the CLI is not available, agents fall back to the >20 lines of business logic heuristic.

---

### `devtrail audit [path] [--from <date>] [--to <date>] [--system <name>] [--output <format>]`

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
$ devtrail audit

# Audit for Q1 2026
$ devtrail audit --from 2026-01-01 --to 2026-03-31

# Audit filtered by system
$ devtrail audit --system auth-service

# Generate HTML report
$ devtrail audit --from 2026-01-01 --to 2026-03-31 --output html > audit-q1.html

# Generate Markdown for a PR
$ devtrail audit --output markdown
```

---

### `devtrail explore [path]`

Browse and read DevTrail documentation interactively in a terminal UI.

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
2. `language` field in `.devtrail/config.yml`, when the file exists (an explicit value — even `language: en` — is treated as a deliberate user choice)
3. `$LC_ALL` / `$LANG` env vars, mapped to a supported locale (e.g., `zh_CN.UTF-8` → `zh-CN`, `es_MX.UTF-8` → `es`). Traditional Chinese (`zh_TW` / `zh_HK`) and other unsupported locales fall through.
4. `en`

**Features:**

- Two-panel layout: navigation tree + document viewer
- Metadata panel showing status, confidence, risk, tags, and related links
- Markdown rendering with colors, tables, code blocks, and heading indentation
- Navigate between related documents via hyperlinks
- Search by filename, title, tags, or date
- Fullscreen document mode, with `j` / `k` as alternate keys for `↓` / `↑`
- Localization-aware: framework docs (`QUICK-REFERENCE`, `AGENT-RULES`, China regulatory guides, etc.) are served in the language set by `language` in `.devtrail/config.yml` or by `--lang`

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
$ devtrail explore                       # uses config.language (defaults to en)
$ devtrail explore --lang zh-CN          # browse framework docs in Simplified Chinese
$ devtrail explore --lang es             # session override to Spanish
```

> **Note:** The `explore` command requires the `tui` feature (enabled by default). To compile without it: `cargo build --no-default-features`.

---

### `devtrail about`

Show version, authorship, and license information.

**Example:**

```bash
$ devtrail about
DevTrail CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.7.1
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/devtrail
  Website:           https://strangedays.tech
```

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

**DevTrail** — Because every change tells a story.

[Back to docs](../README.md) • [README](../../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
