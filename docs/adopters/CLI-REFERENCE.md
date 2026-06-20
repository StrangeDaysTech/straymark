# StrayMark CLI Reference

**Complete reference for the `straymark` command-line tool.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)


---

## Table of Contents

1. [Installation](#installation)
2. [Versioning](#versioning)
3. [Commands](#commands) — init, update, remove, status, repair, validate, new, charter, followups, compliance, metrics, analyze, audit, explore, about
4. [Environment Variables](#environment-variables)
5. [Exit Codes](#exit-codes)

---

## Installation

Install the StrayMark CLI using one of the methods below. For full setup instructions, see the [README](https://github.com/StrangeDaysTech/straymark/blob/main/README.md#getting-started).

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
| Framework | `fw-` | `fw-4.30.0` | Templates (12 types), governance docs, directives, Charter template + schema |
| CLI | `cli-` | `cli-3.29.0` | The `straymark` binary |
| Loom (EXPERIMENTAL) | `loom-` | `loom-0.4.2` | The `straymark-loom` visualization server, downloaded on demand by `straymark loom serve` |

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
4. Configures AI agent directive files (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
5. Copies CI/CD workflows
6. *(`--hooks`)* installs the pre-PR hook

**Example:**

```bash
$ straymark init .
✔ Downloaded StrayMark fw-4.15.0
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
✔ Framework updated to fw-4.15.0
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
✔ Framework updated to fw-4.15.0
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
  │ Framework │ fw-4.15.0                 │
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

#### `straymark status --where [path] [--out <dir>]` *(cli-3.25.0+, EXPERIMENTAL)*

The textual **"you are here"** companion to Loom's Architecture Plan view (the terminal half of the dashboard). Loads `architecture/model.yml` and projects each component's state from your live governance signals, so you can answer *"where are we?"* without opening a browser.

> ⚠️ **EXPERIMENTAL (Loom v0).** Requires an `architecture/model.yml` (see `straymark architecture generate`). The output shape may change without a deprecation cycle.

| Flag | Default | Description |
|---|---|---|
| `--where` | off | Switch `status` into the architecture projection view |
| `--out <dir>` | `.straymark/architecture` | Directory holding `model.yml` (point it elsewhere to project a model kept outside `.straymark/`) |

Each component is tagged with one or more states, derived purely from governance (no new frontmatter):

- **`active`** — an `in-progress` Charter declares files matching the component (← *you are here*);
- **`in-progress`** — within an active component, declared files already touched (declared ∩ git-modified);
- **`implemented`** — a closed Charter (and its AILOGs) touched the component;
- **`has-debt`** — an open `TDE` relates to the component;
- **`uncharted`** — files on disk that no Charter or document references.

The view ends with a **"Where are we" summary**: the active Charter, declared-vs-modified progress, recent AILOGs, and open debt. The `active`/`implemented` flags line up with `straymark charter list --status in-progress`/`--status closed` + `charter drift` by construction (one shared projection).

```
$ straymark status --where

  Where are we

  Tooling (Rust workspace)
    ▸ straymark-cli   [active] [in-progress]  ← you are here
    · straymark-core  [implemented]

  Visualization
    · Loom            [has-debt]

  Summary
    Active Charter: A1.4 — status --where
    Progress: 1/2 declared files touched (50%)
    Debt: 1 component with open debt, 0 uncharted
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
  Using version: fw-4.15.0
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ StrayMark repaired successfully!
```

---

### `straymark install-skills --agent <codex|claude|gemini> [--path .] [--dry-run] [--symlink]` *(cli-3.16.0+)*

Install StrayMark skills into an AI agent's **user-level** skills directory. Currently only `--agent codex` is supported: it copies each `straymark-*` skill from `<path>/.codex/skills/` (materialized by `straymark init` or `straymark update`) into `$CODEX_HOME/skills/` (or `$HOME/.codex/skills/` if `CODEX_HOME` is unset).

For `--agent claude` and `--agent gemini`, the command exits with an explanatory error: those agents read skills directly from the project tree (`.claude/skills/`, `.gemini/skills/`), so no user-level install is needed.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---|---|---|
| `--agent` | required | One of `codex`, `claude`, `gemini`. Only `codex` performs work; the others exit with guidance. |
| `--path` | `.` | Project directory whose `.codex/skills/` is the source. |
| `--dry-run` | off | Print what would be installed without writing anything. |
| `--symlink` | off | Symlink each skill instead of copying it (Unix-only; convenient for framework developers iterating on skill content). |

**Example:**

```bash
$ straymark install-skills --agent codex
  StrayMark Install Skills
  agent: codex
  from:  /work/project/.codex/skills
  to:    /home/me/.codex/skills

✔ straymark-ailog
✔ straymark-aidec
✔ straymark-status
…
  ✓ 11 skill(s) installed in /home/me/.codex/skills (0 replaced).
  → Codex will discover them on next session.
```

Re-running the command replaces any existing `straymark-*` directories under the target; non-`straymark-*` skills (for example Codex's `.system/` bundle) are left untouched.

---

### `straymark validate [path] [--fix] [--staged] [--agent <codex>] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Validate StrayMark documents for compliance and correctness.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` (current directory) | Target project directory |
| `--fix` | — | Automatically fix simple issues (e.g., missing `review_required: true` for high-risk docs) |
| `--staged` | — | Validate only staged (git-added) files. Ideal for pre-commit hooks. |
| `--agent` *(cli-3.16.0+)* | — | Switch to agent-targeted mode and inspect a user-level skills installation instead of project documents. Currently only `codex` — verifies `~/.codex/skills/straymark-*` for presence, parseable YAML frontmatter, required `name`/`description`, and absence of Claude-only keys like `allowed-tools` (whose presence indicates someone copied skills from `.claude/` by mistake). |
| `--include-charters` | — | Also validate Charters in `.straymark/charters/` against the Charter JSON Schema and referential integrity (originating AILOG IDs resolve, originating spec paths exist). Includes **`CHARTER-FILES-EXIST`** *(cli-3.17.0+)*: warns when a `## Files to modify` row names a path that does not exist on disk and is not tagged "New" — catching Charters authored against assumed, un-read code (finding #210). Warn-only; distinct from `charter drift` (which compares declared vs git-modified files). Opt-in so projects that don't yet use the Charter pattern are unaffected. |
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

Manage **Charters**: bounded, auditable units of work declared ex-ante and validated ex-post. A Charter pairs declarative scope (files to touch, risks, executable verification) with ex-post audit anchoring (drift detection, multi-model audit). Charters live at `.straymark/charters/NN-slug.md` (project-root level, **not** under `.straymark/`).

> **Naming history.** In the Sentinel `/plan-audit` experiment that crystallized this pattern (2026-04, 6 cycles), Charters were called *Plans*. The StrayMark CLI uses **Charter** going forward to disambiguate from GitHub SpecKit's `plan.md`. Sentinel's historical files preserve "Plan" deliberately. The full conceptual scope and the rename rationale live in `Propuesta/que-es-un-charter.md`.

**Subcommands:**

- `straymark charter new` — scaffold a new Charter from the framework template
- `straymark charter list` — enumerate Charters with optional filters
- `straymark charter status` — show Charter detail, or the most recent 5 Charters
- `straymark charter close` — record post-execution telemetry and bump status to `closed` *(Phase 2, fw-4.6.0+)*
- `straymark charter drift` — detect file-vs-commit drift with AILOG-aware suppression and Batch Ledger gate *(Phase 2, fw-4.6.0+; Batch Ledger gate cli-3.13.0+)*
- `straymark charter batch-complete` — mark a Charter batch as complete in the AILOG `## Batch Ledger` *(cli-3.13.0+, fw-4.14.0+)*
- `straymark charter audit` — orchestrate a multi-model external review (3-step prepare/calibrate/finalize) *(Phase 3, fw-4.9.0+)*
- `straymark charter refresh-suggest` — heuristic recommendation for a pre-declare SpecKit refresh across a multi-Charter module *(cli-3.14.0+, fw-4.16.0+)*
- `straymark charter amend` — scaffold a post-close Batch N.4 amendment (audit-driven remediation on the same execute branch) *(cli-3.14.0+, fw-4.16.0+)*

#### `straymark charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

Scaffold a Charter from the framework template into `.straymark/charters/NN-slug.md`. Prompts for the title interactively if not passed. The two origin flags are mutually exclusive at the clap level.

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

  ✔ Created: .straymark/charters/01-test-charter.md

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

  ── Follow-ups ──
  ✔ Extracted 2 follow-ups from 1 AILOG(s) into the registry (`## Bucket: ready`).
    Review against the TDE-promotion criteria (AGENT-RULES.md §3): ...
  Promote FU-078 — wire the retry budget into the sync loop to a TDE? [y/N]
```

**Follow-ups reconciliation** *(cli-3.22.0+, RFC #135 Tier 3)*. After an **interactive** close, the command runs the default `followups drift` scan (committed git range ∪ working tree) over the recently-written AILOGs, extracts any `§Follow-ups` / `R<N> (new)` content **not yet in the registry** into `## Bucket: ready`, then offers **per-entry TDE promotion** against the four §3 criteria (prior-Charter heritage, multi-module/Charter span, dedicated Charter, human prioritization). Declining a prompt leaves the follow-up extracted (captured in the registry, just not promoted); accepting runs the `followups promote` flow (creates the TDE with `promoted_from_followup` traceability). It is a **no-op** when the project has no follow-ups registry or when nothing is unextracted, and is **skipped on the `--from-template` paths** (no interactive prompt context) — run `straymark followups drift --apply` there.

#### `straymark charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--no-batch-ledger-check] [--path <dir>]`

Detect file-vs-commit drift at Charter close. **Native Rust since cli-3.23.0** (#237) — no longer delegates to the (now-deprecated) `.straymark/scripts/check-charter-drift.sh`, so it runs on Windows-native (no WSL, no Git Bash). The zero-false-positives property (PLAN-05 retrospective + PLAN-06 prospective in Sentinel) is preserved by the script-equivalence test suite. The CLI value-add over the raw script is **AILOG-awareness**: paths reported as "declared but not modified" are silenced when they appear in the `## Risk` / `## Riesgos` / `## 风险` section of any AILOG referenced by the Charter's `originating_ailogs`. Use `--no-ailog-suppress` to disable.

**Batch Ledger gate** *(cli-3.13.0+)*. When the Charter status is `in-progress` or `closed`, the command also checks each originating AILOG for `## Batch Ledger` entries left as `(pending)` and fails with a clear diagnostic listing the missing batches. AILOGs without a ledger contribute nothing — the section is opt-in. Use `--no-batch-ledger-check` to bypass (intended for adopters consolidating the ledger post-close).

| Argument/Flag | Default | Description |
|---|---|---|
| `CHARTER-ID` | — | Same resolution rules as `charter status` |
| `--range` | `HEAD~1..HEAD` | Git revision range to check |
| `--no-ailog-suppress` *(cli-3.10.0+ always emits a confirming INFO line)* | false | Disable AILOG-aware suppression (show every declared-omitted path). When passed, the CLI always prints an `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` line — including when N=0 — so that the diagnostic mode is visible in output even on a clean run. |
| `--no-batch-ledger-check` *(cli-3.13.0+)* | false | Disable the Batch Ledger gate. Use when the Charter's AILOG opted out of the ledger pattern at close time. |
| `--path` | `.` | Target project directory |

**Exit codes:** `0` if no drift (or only AILOG-suppressed) and no pending batches; `1` if there's unaccounted drift or any `### Batch N` is still `(pending)`; `2` for usage errors (Charter not found, bash missing, etc.).

**Example:**

```bash
$ straymark charter drift CHARTER-01 --range origin/main..HEAD
=== Charter drift check ===
  Charter: .straymark/charters/01-test.md
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

Paths under `.straymark/charters/*` and `.straymark/07-ai-audit/*` are **never** reported as "modified but not declared". This is opinionated by design — those paths are always legitimate when the Charter itself or the AILOG of execution is touched. Empirically validated in Sentinel CHARTER-04: a stray `git add -A` staged unrelated user-untracked files (`.claude/skills/`, `cmd/sentinel/sentinel`); the rule correctly suppressed the governance noise without hiding the genuine project-file expansion ([issue #81 W2](https://github.com/StrangeDaysTech/straymark/issues/81#issuecomment-update)).

If you're running a Charter whose explicit scope is governance churn (e.g., a bulk approval Charter touching only `.straymark/07-ai-audit/`), the drift check will report 0 modified files and you'll need to verify scope by reading the AILOG. A `--strict-scope` flag that disables the always-in-scope rule is on the table for a future minor if a real adopter reports the asymmetry as a friction.

#### `straymark charter batch-complete <CHARTER-ID> <N> [--note <body>] [--non-interactive] [--path <dir>]`

*Available since **cli-3.13.0** + **fw-4.14.0**.*

Mark a Charter batch as complete in the originating AILOG's `## Batch Ledger`. The command substitutes the `(pending)` placeholder under `### Batch <N>` with the batch notes captured interactively (default) or via `--note` (one-shot / scripted). The drift gate at close time (`straymark charter drift`) rejects any `### Batch N` left as `(pending)`, making the per-batch update load-bearing rather than a discipline reminder.

**When to use it.** Only for multi-batch Charters that span 3+ batches or >1 day of execution. Single-batch AILOGs document everything in `## Actions Performed` and skip the ledger entirely. The Charter template's §Tasks reminds the author to maintain the ledger when applicable.

| Argument/Flag | Default | Description |
|---|---|---|
| `CHARTER-ID` | — | Same resolution rules as `charter status` |
| `N` | — | Batch number (1-based) — matches the `### Batch <N>` heading in the AILOG `## Batch Ledger` |
| `--note <body>` | — | Pre-filled batch note body. With this flag, the command writes the note non-interactively and skips prompts. Designed for agents and scripts. |
| `--non-interactive` | false | Disable prompts. Requires `--note` (a missing `--note` aborts instead of hanging). |
| `--path` | `.` | Target project directory |

The command reads `originating_ailogs[0]` from the Charter frontmatter to locate the target AILOG. It rejects with a clear error when:

- the Charter has no `originating_ailogs` entry (cannot resolve a target file);
- the AILOG file is missing from `.straymark/07-ai-audit/agent-logs/`;
- the AILOG has no `## Batch Ledger` section (add the section first, or skip the command if the Charter is single-batch);
- no `### Batch <N>` heading exists under `## Batch Ledger`;
- the target batch is already completed (refuses to overwrite — edit the AILOG manually for corrections).

**Interactive flow.** Three prompts: files touched (one-liner), tests added or status (one-liner), and a multiline design note (terminate with `.` on a line by itself or Ctrl-D). Empty fields are skipped in the output. The resulting body is written under the `### Batch <N>` heading; the `(pending)` placeholder is replaced.

**Examples:**

```bash
# Interactive — humans pasting batch notes
$ straymark charter batch-complete CHARTER-17 5
✔ AILOG: .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md
✔ ### Batch 5 — Migration 022 + handlers
Files touched: migrations/022_dedup.sql, services/handler_x.go
Tests added or status: handler_x_test.go +12 -0, passing
note (multiline; finish with `.` on a line):
> Used existing processed_events table; saw R8 (CHECK constraint missing ARCHIVED).
> .

OK `### Batch 5` written.
Reminder: `git add .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md` before pushing.

# One-shot — agents / scripts
$ straymark charter batch-complete CHARTER-17 5 \
    --note "Migration 022 + handlers. Files: migrations/022_dedup.sql, services/handler_x.go. Tests passing. R8 surfaced (CHECK constraint missing ARCHIVED), fixed atomically."
OK `### Batch 5` written.
```

**Workflow integration.** Per `Charter §Tasks` template guidance, run `batch-complete` *immediately after* the batch commit lands but *before* pushing. The AILOG update and the work it documents then ride the same push. The drift gate at close (`straymark charter drift CHARTER-NN`) rejects any pending batch and prints the list — making "forgot to update the ledger" a hard failure rather than silent erosion of the audit trail.

#### `straymark charter audit <CHARTER-ID> [--range <REV..REV>] [--prepare | --merge-reports] [--merge-into <PATH>] [--path <dir>]` {#straymark-charter-audit}

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
| `--range` | `origin/main..HEAD` (with fallback to `origin/master..HEAD`, then `HEAD~1..HEAD` with warning) | Git revision range the auditors review. The default captures the full feature-branch implementation set; explicit `--range <REV..REV>` overrides without probing for upstream. **Multi-batch pitfall:** for a phase-scoped audit of a Charter whose earlier phases already merged to the base branch, the default `origin/main..HEAD` *excludes* the merged commits and silently under-covers the phase — pass an explicit `--range <charter-first-commit>..HEAD` to span it. `--prepare` prints a warning when it detects completed batches and no explicit range. |
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

> **Placeholder-aware merge *(fw-4.16.0+, Issue #156).*** From `fw-4.16.0` `straymark charter close` writes an `external_audit: []` placeholder under `charter_telemetry:`. `--merge-into` now REPLACES that placeholder in place; only a non-empty `external_audit:` array still triggers the re-audit guard (anti-duplicate). Round-trip with the post-close audit cycle (`/straymark-audit-review`) requires no manual YAML editing.

#### `straymark charter refresh-suggest <module> [--threshold N] [--path <dir>]`

*Available since **cli-3.14.0** + **fw-4.16.0**. Operationalizes Pattern 1 of `.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` (Issue [#156](https://github.com/StrangeDaysTech/straymark/issues/156)).*

Read-only heuristic recommending a pre-declare SpecKit refresh PR before the next Charter declare for a multi-Charter module. Reads the `.telemetry.yaml` files of closed Charters whose `charter_id` contains the module name (case-insensitive substring), picks the 3 most recent by `closed_at`, computes the rolling mean of `agent_quality.r_n_plus_one_emergent_count`, and compares against a threshold (default 6).

| Argument/Flag | Default | Description |
|---|---|---|
| `<module>` | — | Module name to match (e.g. `commshub` matches `CHARTER-18-commshub-us5-...`). Required. |
| `--threshold N` | `6` | Override the rolling-mean threshold used to trigger the recommendation. **The flag is the only override path in v0.2** — there is no `config.yml` field for this threshold yet (the heuristic is Sentinel-derived; tuning lives at the operator-invocation level until a second adopter validates a project-wide default). |
| `--path <dir>` | `.` | Project directory |

**Exit code is always 0** — this is informational, never a CI gate. The output is a table of matched Charters (with the 3 in the rolling window highlighted) plus a recommendation: refresh-now, refresh-not-needed, or insufficient-data.

**Example:**

```bash
$ straymark charter refresh-suggest commshub

  Closed Charters matched (window: top 3 by closed_at)

    charter_id                              closed_at      R<N+1>  telemetry
    ----------                              ----------     ------  ---------
  ● CHARTER-18-commshub-us5-cloud-tasks…    2026-05-15          9  18-commshub-us5-….telemetry.yaml
  ● CHARTER-17-commshub-us4-templating      2026-05-10          8  17-commshub-us4-….telemetry.yaml
  ● CHARTER-16-commshub-us3-routing         2026-05-04          7  16-commshub-us3-….telemetry.yaml
    CHARTER-13-commshub-us1-foundation      2026-04-18          5  13-commshub-us1-….telemetry.yaml

  Heuristic
    Chain length (closed Charters in window): 3
    Threshold for rolling mean:               > 6
    Rolling mean of agent_quality.r_n_plus_one_emergent_count: 8.00

  ▶ Recommend a pre-declare SpecKit refresh for `commshub` before the next Charter.
    See .straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md Pattern 1 for mechanics.
    Telemetry slot: `charter_telemetry.pre_declare_refresh` in the next Charter.
```

#### `straymark charter amend <CHARTER-ID> --trigger <kind> --ailog-title <title> [--findings-closed N] [--merge-into <PATH>] [--path <dir>]`

*Available since **cli-3.14.0** + **fw-4.16.0**. Operationalizes Pattern 2 of `.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` (Issue [#156](https://github.com/StrangeDaysTech/straymark/issues/156)).*

Scaffold a post-close Batch N.4 amendment for an already-closed Charter when external audit findings, a production incident, or a deferred-implementation gap warrant a bounded code-level fix on the same execute branch (no new Charter, no branch off `main`).

The command does **NOT** touch git. It prepares three artifacts so the operator can commit at their leisure:

1. **A new AILOG stub** under `.straymark/07-ai-audit/agent-logs/AILOG-YYYY-MM-DD-NNN-<slug>.md` with `risk_level: high`, `review_required: true`, and an `amends:` field linking back to the most-recent prior AILOG that mentions the Charter.
2. **A `## Historical correction (YYYY-MM-DD)` subsection** appended to the located prior AILOG, pointing forward to the new id and recording the trigger.
3. **A rendered `post_close_amendment:` YAML block**, printed to stdout for manual paste or merged directly into the Charter's telemetry YAML via `--merge-into`.

| Argument/Flag | Default | Description |
|---|---|---|
| `<CHARTER-ID>` | — | Same resolution rules as `charter status`. The Charter MUST be `status: closed`. |
| `--trigger <kind>` | — | Required. One of `external_audit`, `production_incident`, `deferred_implementation`. |
| `--ailog-title <title>` | — | Required. Drives the slug of the new AILOG filename and its in-body heading. |
| `--findings-closed N` | `0` | Number of audit findings closed by the amendment. Populates the `post_close_amendment.findings_closed` field. |
| `--merge-into <PATH>` | — | Optional. Write the `post_close_amendment:` block directly into the telemetry YAML at `<PATH>` instead of printing it. Refuses to overwrite an already-populated block. |
| `--path <dir>` | `.` | Project directory |

**Bounded-use guard:** the amendment pattern applies only when (a) the fix surface fits in **one cohesive PR** (~< 25 files, no architectural reopen), (b) the Charter's closure criterion is materially unmet by the un-remediated trigger, and (c) the findings are Critical or High (for `external_audit`). Larger fixes warrant a new Charter — the CLI does not enforce these gates, but the doc at `.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` Pattern 2 makes them explicit.

**Example:**

```bash
$ straymark charter amend CHARTER-18 \
    --trigger external_audit \
    --findings-closed 5 \
    --ailog-title "post-close DI wiring + retry header" \
    --merge-into .straymark/charters/18-commshub-us5.telemetry.yaml

  ✔ Appended `## Historical correction (2026-05-15)` to .straymark/07-ai-audit/agent-logs/AILOG-2026-05-14-049-original.md
  ✔ Created new AILOG at .straymark/07-ai-audit/agent-logs/AILOG-2026-05-15-050-post-close-di-wiring-retry-header.md
  ✔ Merged `post_close_amendment:` block into .straymark/charters/18-commshub-us5.telemetry.yaml

  Next: review the new AILOG, edit it with concrete details, then commit
  on the original execute branch (do NOT branch off main).
```

---

### `straymark followups <subcommand>` *(cli-3.19.0+)*

Manage the **follow-ups backlog registry** (`.straymark/follow-ups-backlog.md`) — the first-class artifact that aggregates `§Follow-ups` and `R<N> (new, not in Charter)` entries across AILOGs. Schema: `.straymark/schemas/follow-ups-backlog.schema.v1.json` (experimental v1). Convention: `FOLLOW-UPS-BACKLOG-PATTERN.md` and `STRAYMARK.md §16`; shipped agent directives in `AGENT-RULES.md §13`.

Parsing is **lenient**: v0 registries (pre-fw-4.21.0) are read without errors; the first write command (`drift --apply` — even with nothing to extract, cli-3.20.0+ —, `recount`, or `promote`) upgrades them to v1 in place, non-destructively. Frontmatter `total_*` counters are **CLI-owned** — recomputed on every write; never edit them by hand.

- `straymark followups list` — enumerate entries *(cli-3.19.0+)*
- `straymark followups status` — registry pulse / entry detail *(cli-3.19.0+)*
- `straymark followups drift` — sync the registry with AILOGs (native replacement for the deprecated adopter-side `check-followups-drift.sh`) *(cli-3.19.0+)*
- `straymark followups recount` — recompute the CLI-owned counters after a manual-triage session *(cli-3.20.0+)*
- `straymark followups promote` — elevate an entry to a TDE document *(cli-3.19.0+)*

#### `straymark followups list [--bucket <name>] [--status <s>] [--severity <s>] [--label <tag>] [path]`

Table of entries: FU id, status, severity, bucket, destination, description. Parse warnings (malformed `### FU-` headings) go to stderr without failing the command.

| Flag | Default | Description |
|------|---------|-------------|
| `--bucket <name>` | — | Filter by bucket: `ready`, `time-triggered`, `charter-triggered`, `phase-blocked`, `operational` |
| `--status <s>` | — | Filter by status: `open`, `in-progress`, `suspected-closed`, `closed`, `superseded`, `promoted` |
| `--severity <s>` | — | Filter by severity: `normal`, `blocking` |
| `--label <tag>` | — | Filter by label (case-insensitive exact match on one tag) |

```bash
$ straymark followups list --severity blocking
  FU      STATUS  SEV       BUCKET  DEST          DESCRIPTION
  FU-010  open    blocking  ready   mini-charter  Harden staging probe
```

#### `straymark followups status [FU-NNN] [--path <dir>]`

Without an id: the registry pulse — counters **recomputed on the fly** from actual entry statuses (trustworthy even when the file's frontmatter is stale; divergence is flagged), per-bucket open breakdown, blocking and suspected-closed alerts, advisory schema validation. With an id: the entry's full field detail.

#### `straymark followups drift [--apply] [--scan-all] [--range <REV..REV>] [--path <dir>]`

Detect AILOGs whose follow-up content is not yet extracted into the registry. Granularity is **per-AILOG** (the `fully_extracted_ailogs` frontmatter list) — the empirically validated design choice (0 false positives across 76 AILOGs in the reference adopter).

| Flag | Default | Description |
|------|---------|-------------|
| *(default)* | — | Scan AILOGs changed in `origin/main..HEAD` (fallback `origin/master..HEAD`, then `HEAD~1..HEAD` with a warning). Warn + **exit 1** on drift. |
| `--apply` | off | Extract the missing entries into `## Bucket: ready` with auto-numbered `FU-NNN` ids, append the AILOG ids to `fully_extracted_ailogs`, **recompute the counters**, and upgrade v0 registries to v1 in place. Seeds the registry from the framework template when absent. Since cli-3.20.0 the counters are recomputed **even when there is nothing to extract** (#222 Finding 1). |
| `--scan-all` | off | Sweep every AILOG in the project instead of the git range. |
| `--range <REV..REV>` | — | Explicit git range for the default scan. |

**Anti-noise refinement** (issue #214 Signal 1): bullets whose AILOG text carries an explicit closure marker — `closed in-Charter`, `fixed in batch N`, a backtick-wrapped commit hash, or *(cli-3.20.0+, #222 Finding 2)* a born-resolved idiom (a closure verb `updated`/`corrected`/`remediated`/`resolved`/`fixed`/`closed` followed by `in this PR` / `in this commit`, e.g. `updated atomically in this PR`) — are extracted as **`suspected-closed`** instead of `open`, so already-resolved work stops polluting the `ready` bucket as TBD noise. The operator confirms (→ `closed`) or reopens at the next triage. The canonical idiom vocabulary is documented in `FOLLOW-UPS-BACKLOG-PATTERN.md`.

```bash
$ straymark followups drift --scan-all --apply
✓ Extracted 4 entries from 1 AILOG(s) into `## Bucket: ready`.
  ! 1 extracted as suspected-closed (closure marker in source AILOG) — confirm at the next triage.
  Counters recomputed: 3 open / 1 suspected-closed / 0 promoted (total 4).
```

#### `straymark followups recount [--path <dir>]` *(cli-3.20.0+)*

Recompute the CLI-owned `total_*` counters from actual entry statuses and rewrite the frontmatter — without scanning AILOGs, extracting, or touching entries. The §13-compliant way to reconcile counters after a **manual-triage session** (statuses flipped by hand per the sanctioned Triage/Consumption lifecycle, nothing left to extract or promote — #222 Finding 1, first external adopter). Idempotent: a second run reports the counters as already in sync. Upgrades v0 registries to v1 in place, like every other write command.

```bash
$ straymark followups recount
✓ Counters recomputed: 0 open / 0 suspected-closed / 2 promoted (total 2).
```

#### `straymark followups promote <FU-NNN> [--title <title>] [--path <dir>]`

Automate the FU → TDE elevation (`FOLLOW-UPS-BACKLOG-PATTERN.md` §Promotion to TDE): creates the TDE document from the framework template with `promoted_from_followup: FU-NNN` traceability, flips the entry to `Status: promoted` with `Destination`/`Promoted to` pointing at the TDE id, and recomputes the counters. Non-interactive (agent-friendly); the FU description becomes the TDE title unless `--title` overrides it. Prioritization and assignment stay human (`AGENT-RULES.md §3`).

```bash
$ straymark followups promote FU-010
✓ FU-010 promoted → TDE-2026-06-04-001
  TDE created: .straymark/06-evolution/technical-debt/TDE-2026-06-04-001-harden-staging-probe.md
```

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

> **Powered by arborist-metrics:** the cognitive and cyclomatic complexity factors are computed by [`arborist-metrics`](https://github.com/StrangeDaysTech/arborist-metrics/) — our open-source Rust library for multi-language code metrics, developed by StrangeDaysTech S.A.S. de C.V. Also available standalone on [crates.io](https://crates.io/crates/arborist-metrics).

---

### `straymark analyze declared-vs-wired [path] [--profile <name> | --declared-glob … --wired-glob … --declared-pattern … --wired-pattern …] [--show-orphans] [--output <format>]` *(cli-3.18.0+)*

Flag declared symbols that have **no wiring counterpart** on the implementation side — the *"surface declaration without wiring"* anti-pattern, sub-class 5 (client-side IPC/RPC proxy method vs server interface). Crystallized from the LNXDrive N=2 validation (findings [#209](https://github.com/StrangeDaysTech/straymark/issues/209)/[#210](https://github.com/StrangeDaysTech/straymark/issues/210)); see `.straymark/00-governance/POLISH-CHARTER-PATTERN.md`.

It is a **config-driven set difference**, language/IPC-agnostic by construction: you supply a *declared* side and a *wired* side as `(glob, regex)` pairs; the **capture group 1** of each regex is the symbol name. The command reports **D \ W** (declared but not wired) and, with `--show-orphans`, **W \ D** (wired but never declared).

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---------------|---------|-------------|
| `path` | `.` | Target directory |
| `--profile` | — | Named profile from `.straymark/config.yml` (`declared_vs_wired.profiles`). Alternative to the four inline flags. |
| `--declared-glob` | — | Glob (relative to `path`) for files holding declarations (proxy/stub/client). |
| `--wired-glob` | — | Glob (relative to `path`) for files holding implementations (daemon/server interface). |
| `--declared-pattern` | — | Regex over declared files; **capture group 1** = symbol name. |
| `--wired-pattern` | — | Regex over wired files; **capture group 1** = symbol name. |
| `--show-orphans` | off | Also report symbols wired but never declared (`W \ D`). |
| `--output` | `text` | `text`, `json`, or `markdown`. |

You must pass **either** `--profile` **or** all four inline globs/patterns.

**Exit codes:** `0` clean (every declared symbol is wired); `1` at least one declared symbol has no wiring counterpart (a finding) — suitable as a CI gate.

**Configuration** (optional, in `.straymark/config.yml`):

```yaml
declared_vs_wired:
  profiles:
    - name: dbus
      declared_glob: "client/**/*.rs"     # the GTK client's D-Bus proxy
      declared_pattern: "fn (\\w+)"
      wired_glob: "daemon/src/interface.rs" # the daemon's implemented interface
      wired_pattern: "fn (\\w+)"
```

**Examples:**

```bash
# Inline (one-off)
$ straymark analyze declared-vs-wired \
    --declared-glob "client/**/*.rs" --declared-pattern 'fn (\w+)' \
    --wired-glob "daemon/**/*.rs"    --wired-pattern 'fn (\w+)'

# Named profile (committed once), JSON for CI
$ straymark analyze declared-vs-wired --profile dbus --output json
```

**Example output:**

```
  StrayMark Analyze — declared-vs-wired
  /home/user/project
  Profile: dbus   Symbols: 12 declared / 11 wired

  ✗ 1 declared symbol(s) with NO wiring counterpart:
    - complete_auth_with_tokens  (client/src/proxy.rs)

  → a declared surface with no implementation is the "surface declaration
    without wiring" anti-pattern (POLISH-CHARTER-PATTERN.md sub-class 5).
```

> **v0 scope:** this is the mechanically-tractable cross-stack check (sub-class 5). The AST-based variants of sub-classes 1–4 (env-var docs, metric instruments, HTML embeds, public-route markers) and the dynamic runtime checks remain project-local — see the pattern doc's Open questions.

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
  Framework version: fw-4.15.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/straymark
  Website:           https://strangedays.tech
```

---

### `straymark architecture <generate|sync|validate>` *(cli-3.25.0+, EXPERIMENTAL)*

Author and maintain the **architecture model** that powers Loom's Architecture Plan view (Spec 002): a semantic `model.yml` (components → file globs + layers + links) paired with a `plan.drawio` blueprint, linked by a stable `component_id` (the BIM "model vs drawing" split). The textual projection over this model is `straymark status --where`; the visual overlay is Loom (A2).

> ⚠️ **EXPERIMENTAL (Loom v0).** The model schema, DrawIO conventions, and command surface may change without a deprecation cycle. All three subcommands operate on `.straymark/architecture/` by default; `--out <dir>` overrides it.

**`straymark architecture generate [path] [--force] [--out <dir>]`** — write a first-draft `model.yml` + `plan.drawio` by mining your codebase structure (one component per source directory, glob `dir/**`) enriched with ADR signal (C4 Mermaid diagrams + "Affected Components" tables improve labels and add links). The miner descends through *container* directories (`internal/`, `src/`, `pkg/`, `lib/`, `app/`, `modules/`, …) so a tree like `internal/modules/<x>` breaks out into real components instead of one blob, and it skips build *scaffolding* (`src/main/java`, `src/main/kotlin`, …) so a Maven/Gradle module is attributed to its module directory rather than collapsing to a single `main` box. Components land in a placeholder `unassigned` layer and the `.straymark` stages 00–09 seed the layer list — you then refine by hand (reassign/rename layers, tighten globs, add links). `--force` overwrites existing artifacts.

> **Tuning the scan for your stack (`config.yml`).** The seed is language- and structure-aware out of the box, but you can extend it for non-default ecosystems via an `architecture:` section in `.straymark/config.yml`. Lists are **additive** — they extend the built-in defaults, never replace them:
>
> ```yaml
> architecture:
>   source_extensions:   [rb, ex, exs, gleam]   # add languages the seed should count
>   container_dirs:      [services, domains]     # extra dirs to descend through
>   scaffolding_prefixes: [src/main/java]        # extra build scaffolding to skip (Maven/Gradle defaults included)
>   excluded_dirs:       [generated]             # extra dirs to skip
> ```
>
> The status overlay itself (glob → file matching) is already language-agnostic; this only shapes the generated seed.

**`straymark architecture sync [path] [--out <dir>] [--apply]`** — **append-only** reconciliation: detect new top-level source directories / ADR components not yet covered by the model and append them to `model.yml` + `plan.drawio`, **never** clobbering human edits or DrawIO geometry. Dry-run by default; `--apply` writes.

**`straymark architecture validate [path] [--out <dir>] [--output <text|json|markdown>]`** — report model↔plan integrity signals: **undrawn** (component with no DrawIO cell), **unmodeled** (a DrawIO cell absent from the model), **empty** (globs match no files on disk). **Exits 1** when any signal is found (CI-gateable). Degrades to glob-coverage-only when `plan.drawio` is absent.

| Subcommand | Key flags | Writes? |
|---|---|---|
| `generate` | `--force`, `--out` | Yes (refuses to overwrite without `--force`) |
| `sync` | `--apply`, `--out` | Only with `--apply` (append-only) |
| `validate` | `--output`, `--out` | No (read-only; exit 1 on any signal) |

**Example:**

```bash
$ straymark architecture generate
✓ Wrote .straymark/architecture/model.yml (4 components, 9 layers)
✓ Wrote .straymark/architecture/plan.drawio
→ Mined 3 ADRs: 2 labels improved, 1 link added.
→ Refine by hand: reassign components from `unassigned` to real layers, then open the plan in DrawIO.

$ straymark architecture validate
✓ Architecture model is consistent (4 components).
```

---

### `straymark loom serve [path] [--port <port>] [--no-open]` *(cli-3.24.0+, EXPERIMENTAL)*

Launch **Loom**, the EXPERIMENTAL knowledge-graph visualization server: a loopback-only, read-only web dashboard that renders the project's StrayMark documents as a live force-directed graph (nodes colored by Louvain community and sized by connectivity; node and corpus-stat panels; server-side metadata/date filters; selecting a node lights up its whole thread of relationships; edits to watched `.md` files update the open browser in under a second).

> ⚠️ **Loom is EXPERIMENTAL (v0).** Its API, CLI surface, and very existence may change or be removed without a deprecation cycle. The `straymark-loom` binary is **not** bundled into the CLI — it is downloaded on demand from the `loom-*` GitHub releases on first use and cached under `~/.straymark/bin/`. The download gate *is* the opt-in boundary.

**Arguments and flags:**

| Argument/Flag | Default | Description |
|---|---|---|
| `path` | `.` (current directory) | Project directory. Loom watches its `.straymark/` subdirectory if present, otherwise the directory itself |
| `--port` | `7700` | Port on `127.0.0.1` to serve on |
| `--no-open` | off | Do not open the browser automatically |

**Security posture:** binds `127.0.0.1` exclusively (refuses to start otherwise), rejects non-loopback `Host` headers (anti DNS-rebinding), and never writes to the watched directory.

**Example:**

```bash
$ straymark loom serve

  ⚠  LOOM IS EXPERIMENTAL (v0)
     Unstable: API, CLI surface, and on-disk layout may change or be
     removed without a deprecation cycle. Loopback-only. Read-only.

ℹ Downloading Loom 0.4.2 (x86_64-unknown-linux-gnu) — first use is opt-in by download
✔ Loom 0.4.2 cached at ~/.straymark/bin/straymark-loom
loom: watching /project/.straymark (142 docs, 318 links)
loom: serving http://127.0.0.1:7700
```

---

## Skills

StrayMark ships a set of skills (slash commands) for use inside an AI assistant (Claude Code, Gemini Code, Codex CLI, Cursor, generic agent runtimes). Each skill is installed in 4 parallel forms during `straymark init`:

- `.claude/skills/<skill>/SKILL.md` (Claude — frontmatter with `allowed-tools`)
- `.gemini/skills/<skill>/SKILL.md` (Gemini — frontmatter without `allowed-tools`)
- `.codex/skills/<skill>/SKILL.md` *(fw-4.19.0+)* (Codex — minimal frontmatter, only `name`+`description`; generated from the Claude variant)
- `.agent/workflows/<skill>.md` (generic agent — `description`-only frontmatter)

Claude and Gemini discover skills directly from the project tree. **Codex reads skills from `~/.codex/skills/` (user-level), not from the project tree** — run `straymark install-skills --agent codex` once after `straymark init` (or after every framework update) to populate that directory from `.codex/skills/`.

| Skill | Purpose | Files produced |
|---|---|---|
| `/straymark-status` | Check documentation compliance for recent changes. | none (read-only) |
| `/straymark-new` | Create any document type interactively. Suggests best fit from context. | `.straymark/<type-dir>/<TYPE>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | Quick AILOG creation shortcut. | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | Quick AIDEC creation shortcut. | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | Quick ADR creation shortcut. | `.straymark/02-design/decisions/ADR-*.md` |
| `/straymark-mcard` | Interactive Model Card creation flow. | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | Interactive SEC (security assessment) flow. | `.straymark/08-security/SEC-*.md` |
| `/straymark-charter-new` *(fw-4.12.0+)* | Scaffold a Charter — declarative ex-ante work unit. Wraps `straymark charter new` (slug derivation, sequential numbering, template substitution); the skill drives origin/effort selection and the reconnaissance-before-declaration discipline. | `.straymark/charters/NN-slug.md` |
| `/straymark-followups` *(fw-4.22.0+)* | Maintain the follow-ups backlog registry (`AGENT-RULES.md §13`): session-start "what's pending?" answered from the canonical registry, pre-commit `followups drift --apply` riding the same commit as the AILOG, post-Charter-close triage and operator-gated `promote`. Thin wrapper over `straymark followups` — never edits CLI-owned counters. | none directly (writes go through `straymark followups drift --apply` / `promote` → `.straymark/follow-ups-backlog.md`, TDE on promote) |
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
