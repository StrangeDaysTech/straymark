# StrayMark - Recommended Workflows

**Patterns and cadences for using StrayMark day to day.**


---

## Table of Contents

1. [After Initial Setup](#after-initial-setup)
2. [Daily Development](#daily-development)
3. [Keeping StrayMark Updated](#keeping-straymark-updated)
4. [Checking Project Health](#checking-project-health)
5. [Using Skills (Active Documentation)](#using-skills-active-documentation)
6. [Team Patterns](#team-patterns)
7. [Understanding Versions](#understanding-versions)

---

## After Initial Setup

You ran `straymark init .` and committed the result. Now what?

1. **Open your project** with your AI coding assistant (Claude Code, Cursor, Gemini CLI, etc.)
2. The assistant will **automatically read** the StrayMark directives (`CLAUDE.md`, `GEMINI.md`, etc.)
3. From this point on, the assistant **creates documentation** in `.straymark/` as part of its normal workflow
4. **No extra configuration needed** — StrayMark works passively through the directive files

---

## Daily Development

### The Passive Loop

1. Work normally with your AI assistant — write features, fix bugs, refactor
2. The AI creates documents in `.straymark/` according to the governance rules:
   - **AILOG** for significant implementations (>10 lines changed)
   - **AIDEC** when choosing between alternatives
   - **ADR** for architectural decisions
   - **ETH** when ethical concerns arise
3. Review documents flagged with `review_required: true`
4. Commit documentation together with the corresponding code changes

### When to Create Documents Manually

Use the active system (skills) when:

- The AI missed documenting a significant change
- You (a human) made a decision that should be recorded
- You want to create a REQ, TES, TDE, or INC document
- You want to check documentation compliance

---

## Keeping StrayMark Updated

### Recommended Cadence

- **Monthly** or when you see a new release on GitHub
- Check the [releases page](https://github.com/StrangeDaysTech/straymark/releases) for changelogs

### Update Commands

| Goal | Command |
|------|---------|
| Update both framework and CLI | `straymark update` |
| Update only templates and governance files | `straymark update-framework` |
| Update only the CLI binary | `straymark update-cli` |

Framework and CLI have **independent versions** — you can update one without the other. See [Understanding Versions](#understanding-versions).

### After Updating

1. Review changes to directive files and governance docs
2. Commit the updated files: `git add .straymark/ && git commit -m "chore: update StrayMark framework"`
3. If you customized any framework files, check for conflicts

---

## Checking Project Health

### CLI Status

```bash
straymark status
```

Shows: framework version, CLI version, directory structure integrity, and document statistics by type. Use it to verify that the installation is healthy.

### Documentation Compliance (Skill)

```bash
/straymark-status
```

The `/straymark-status` skill (available in Claude Code and Gemini CLI) analyzes:

- Which recent code changes lack corresponding documentation
- Document compliance against governance rules
- Overall documentation health

### Seeing the project (Loom, experimental)

For a *visual* answer to "where are we?", the **EXPERIMENTAL** Loom dashboard renders your project as a live document graph plus a "you are here" architecture map:

```bash
straymark status --where     # textual "you are here" (no server)
straymark loom serve         # the visual dashboard (downloads the loom-* binary on first use)
```

The architecture map is driven by a model you generate and then refine — see the **[Loom & the Architecture Map guide](./LOOM.md)** for the recommended `generate → refine → validate → sync → serve` workflow (the refinement step can be human or AI-assisted).

---

## Using Skills (Active Documentation)

StrayMark has two documentation systems:

| System | How it works | When to use |
|--------|-------------|-------------|
| **Passive** | AI auto-documents via directive files | Default — happens automatically |
| **Active** | User invokes skills to create docs | When passive missed something, or for human decisions |

### Available Skills

| Skill | Purpose |
|-------|---------|
| `/straymark-status` | Check documentation compliance |
| `/straymark-new` | Create any document type (suggests best fit) |
| `/straymark-ailog` | Quick AILOG creation |
| `/straymark-aidec` | Quick AIDEC creation |
| `/straymark-adr` | Quick ADR creation |
| `/straymark-audit-prompt CHARTER-XX` *(fw-4.8.0+, refactored in fw-4.9.0)* | Generate the unified audit prompt at the canonical path `.straymark/audits/<id>/audit-prompt.md`. Wraps `straymark charter audit --prepare`. Operator then opens N auditor CLIs and runs `/straymark-audit-execute` in each — no copy/paste. |
| `/straymark-audit-execute [CHARTER-XX]` *(fw-4.9.0+)* | **Run inside an auditor-side CLI** (gemini-cli, claude-cli, copilot-cli, codex-cli). Reads the prompt from disk, audits with tool use citing `path:line`, writes a report keyed on the auditor's model id. Argument optional — auto-discovers prompts pending from this model. |
| `/straymark-audit-review CHARTER-XX` *(fw-4.8.0+, expanded in fw-4.9.0)* | Counterpart to `audit-prompt`. Reads N reports, verifies findings against actual code, produces `review.md` consolidated 6-section analysis (Executive summary / Scope / Per-auditor evaluation / Remediation plan P0-P4 / Discarded / Auditor ratings), and merges `external_audit:` YAML into telemetry. |
| `/straymark-architecture` *(fw-4.29.0+, EXPERIMENTAL)* | Drives the `generate → refine → validate` arc of the architecture model in one guided pass: seeds it, reassigns components into real layers, wires dependency `links`, syncs the DrawIO so 2D shows arrows, and iterates `validate` to green. The agent-native counterpart to manual DrawIO refinement. |
| `/straymark-architecture-sync` *(fw-4.29.0+, EXPERIMENTAL)* | Wraps `straymark architecture sync` (append-only) to keep a curated model current as the code grows — dry-run, surface new dirs/components, confirm, apply, re-validate. Never re-refines from scratch. |
| `/straymark-loom` *(fw-4.29.0+, EXPERIMENTAL)* | Owns the Loom server lifecycle (up / down / status) from the agent window. Launches `straymark loom serve --no-open` in the background and hands the operator a link — the terminal-free path to the 2D/3D architecture views. |

For full skill details, see the [README](https://github.com/StrangeDaysTech/straymark/blob/main/README.md#skills).

### Charter audit checkpoint *(fw-4.8.0+)*

When co-implementing a Charter, the agent will proactively offer an external audit at one specific moment: when implementation is done, drift is clean, and `charter close` has not yet been invoked. The recommendation is YES/NO based on the Charter's risk surface and complexity (full heuristics in `.straymark/00-governance/AGENT-RULES.md` §12).

External audit is **fully optional** and **never enforced**. The Charter's declarative scope + drift check + AILOG discipline already provide rigorous closure without it. Audit adds cross-model signal where the Charter touched security surface, introduced new components, or has high-complexity functions in the diff. Decline freely if the cost (2-3 LLM auditors) does not match the value for your case.

---

## Team Patterns

### PR Reviews

- Check that significant code changes include corresponding `.straymark/` documents
- Review any documents with `review_required: true`
- Verify that AILOGs accurately describe what the AI did

### Onboarding New Team Members

1. Point them to `.straymark/QUICK-REFERENCE.md` for a quick overview
2. Have them read recent ADRs to understand architectural context
3. Show them AILOGs from recent features to see how documentation works in practice

### Sprint Retrospectives

- Review AILOGs and AIDECs from the sprint to understand AI contribution patterns
- Identify undocumented decisions that should have been recorded
- Check TDE documents for accumulating technical debt

### Shared AI Assistant Usage

When multiple team members use AI assistants on the same project:

- Each assistant session produces its own documents
- The `agent` field in metadata identifies which assistant created each document
- Review overlapping or contradictory AIDECs during PR review

---

## China Regulatory Workflow *(opt-in)*

If your project operates in mainland China or processes personal information of mainland China users, enable the China scope and follow this workflow.

### One-time Setup

1. Edit `.straymark/config.yml` and add `china` to `regional_scope`:
   ```yaml
   regional_scope:
     - global
     - eu      # if also EU-subject
     - china
   ```
2. Run `straymark compliance --region china` to see the baseline (all checks will fail until you create the supporting documents).
3. Read the guides installed under `.straymark/00-governance/`:
   - `CHINA-REGULATORY-FRAMEWORK.md` — overview and coverage matrix
   - `TC260-IMPLEMENTATION-GUIDE.md` — five-level risk grading
   - `PIPL-PIPIA-GUIDE.md` — when a PIPIA is required and what it must contain
   - `CAC-FILING-GUIDE.md` — single vs dual filing, status lifecycle
   - `GB-45438-LABELING-GUIDE.md` — explicit + implicit labeling design

### When You Add a Generative AI Model

Bundle of documents to create together (cross-linked via `related:`):

| Document | Purpose | Required when |
|----------|---------|---------------|
| `MCARD` | Model card with `cac_filing_required`, `gb45438_applicable`, `tc260_risk_level` | Always for in-scope models |
| `TC260RA` | Risk grading (scenario × intelligence × scale → 5 levels) | Always |
| `AILABEL` | Explicit + implicit labeling per GB 45438 | When the model generates content |
| `CACFILE` | Algorithm filing record | When `cac_filing_required: true` |
| `PIPIA` | Personal-info impact assessment (Art. 55-56) | When personal info is processed |
| `SBOM` | Training-data inventory + GB/T 45652 compliance | Always |

`straymark compliance --region china` confirms the bundle is complete.

### When an Incident Occurs

The `INC` template includes a *CSL 2026 Incident Reporting* section. Set:

```yaml
csl_severity_level: relatively_major   # or particularly_serious | major | general
csl_report_deadline_hours: 4           # 1 for particularly_serious, 4 for relatively_major
```

`straymark validate` enforces severity-to-deadline coherence (`CROSS-008`, `CROSS-009`). Major+ incidents must be closed (status `accepted`) within 30 days for the `CSL-003` check to pass.

### Cross-Border Data Transfer

When a process involves transferring personal information out of mainland China, set `pipl_cross_border_transfer: true` on the PIPIA and document the chosen mechanism (CAC security assessment / certification / standard contract) in the *Cross-Border Transfer Analysis* section. `CROSS-011` will warn if none is documented.

### Daily Compliance Check

```bash
# Before merging a feature branch that touches AI services
straymark validate                    # cross-rules including CROSS-004..011
straymark compliance --region china   # per-framework score
```

---

## Understanding Versions

StrayMark uses **independent versioning** for its two components:

| Component | Tag prefix | Contains | Updated via |
|-----------|-----------|----------|-------------|
| **Framework** | `fw-` | Templates, governance docs, directives, scripts | `straymark update-framework` |
| **CLI** | `cli-` | The `straymark` binary | `straymark update-cli` |

### Why Independent Versions?

- Framework changes (new templates, updated rules) are more frequent
- CLI changes (new commands, bug fixes) follow a different cadence
- You can update governance docs without needing a new CLI binary

### Checking Your Versions

```bash
straymark about     # Quick version check
straymark status    # Full health report including versions
```

For detailed CLI information, see the [CLI Reference](CLI-REFERENCE.md#versioning).
