<div align="center">

<img src="assets/straymark-symbol.svg" alt="StrayMark — by Strange Days Tech" width="240" />

# StrayMark

**The cognitive discipline your AI-assisted projects need**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/StrangeDaysTech/straymark/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/straymark-cli.svg)](https://crates.io/crates/straymark-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/StrangeDaysTech/straymark/blob/main/CONTRIBUTING.md)
[![CLA assistant](https://cla-assistant.io/readme/badge/StrangeDaysTech/straymark)](https://cla-assistant.io/StrangeDaysTech/straymark)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[Getting Started](#getting-started) •
[Why StrayMark?](#why-straymark) •
[Who is it for](#who-is-straymark-for) •
[Design Principles](#design-principles) •
[Features](#features) •
[Compliance](#compliance) •
[Documentation](#documentation)

**Languages**: English | [Español](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/README.md) | [简体中文](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/zh-CN/README.md)

</div>

---

## The Problem

AI agents make code fast. They don't make code coherent. After enough turns, an agent loses the thread: it re-introduces patterns the team rejected, accumulates hidden technical debt, and produces work that compiles but doesn't fit the system's grain. The faster the agent, the harder this debt is to see — until a regression, an incident, or a refactor surfaces it.

The senior engineers orchestrating these agents don't need *more* agent autonomy. They need the opposite: a way to externalize scope, decisions and risks at a cadence the agent can be held to — so that the agent executes against constraints instead of inventing its own.

## The Solution

StrayMark is a **framework + CLI** that externalizes the cognitive discipline of senior software engineering work — explicit scope, declared decisions, named risks, recorded alternatives, audited trails — into versioned files that live alongside the code.

> **"No significant change without a documented trace — and a constrained decision space for the agent."**

The discipline produces, as a side effect, evidence compatible with **ISO/IEC 42001**, **EU AI Act**, **NIST AI RMF**, and (opt-in) the Chinese AI/data regulatory stack. But the goal is engineering quality first; compliance is what falls out when the discipline is real.

---

## Why StrayMark?

Code is just the fossil trace of a mental battle. Real engineering happens in the chaos of decisions, calculated risks, and the paths you chose not to take. Traditionally, all that human trail is discarded as stray marks (accidental smudges) in a project's history. At Strange Days Tech, we believe those marks are the signal, not the noise. While corporate compliance software forces you to lie in retrospect to fill empty forms, StrayMark lets you mark your territory in real time.

For us, regulatory compliance (ISO 42001, EU AI Act, NIST) is not the end goal — it is the warp. We use these standards as structural fabric to build a multidimensional technical memory. It is not about checking boxes for an auditor; it is about using those threads to weave a scaffold that supports complex cognitive processes during design and implementation. StrayMark turns bureaucracy into a tactical mesh that helps you think, navigate, and build at a resolution that code alone cannot reach.

In this era of autonomous agents, StrayMark is not a reinforcement mechanism for replacing the human operator. Quite the opposite: it is a tool to humanize the high-velocity, vastly complex processes in which AI agents operate. While the machine moves at rhythms that overflow intuition, StrayMark acts as the anchor of human intent, forcing agents to operate within a constrained, legible, and auditable decision space. We do not seek to automate engineering into oblivion, but to give AI's velocity a deeply human consciousness and structure.

**Capture the noise. Weave the signal. Humanize the machine.**

---

## Who is StrayMark for

StrayMark's primary user is the **senior engineer orchestrating AI agents on a non-trivial system** — someone with strong technical judgment who uses agents to take on work they couldn't realistically do alone, and who needs externalized cognitive discipline so the agent doesn't introduce systemic chaos.

If that describes you, StrayMark's flows, defaults, and language are tuned for you.

StrayMark also serves three secondary audiences, on top of that base — never at its expense:

- **Tech leads and architects** standardizing how their team works with AI assistants.
- **Compliance officers and auditors** who need evidence of governed AI development (ISO 42001, EU AI Act, NIST AI RMF, PIPL, TC260, …).
- **Adopters in regulated environments** (finance, health, public sector, China) who need traceability built into the workflow rather than reconstructed after the fact.

StrayMark is **not** trying to be: an LLM gateway, a model evaluator, a "10× your code" productivity skin, or a substitute for engineering judgment. See [Honest Limits](#honest-limits) below.

---

## Design Principles

StrayMark's product decisions are anchored in twelve explicit principles. They are ordered by hierarchy: when two conflict, the earlier one wins.

1. **The tool serves the craft, not the product.** The metric is whether the engineer produces work they're proud of — not adoption, retention, or revenue.
2. **The primary user is the senior engineer orchestrating agents.** Not the VP, not the CISO, not the compliance officer.
3. **Strict open source, no asterisks in the core.** Framework, CLI and TUI are MIT, with no capped features pushing toward a paid tier.
4. **Regulatory compliance is a side effect, not the product.** ISO 42001, EU AI Act, NIST AI RMF are useful frames; they are not the goal.
5. **Schema-driven before feature-driven.** Central entities (Stage Closure Bundle, Charter, Document) are versioned schemas first, features second.
6. **Cognitive discipline over raw productivity.** StrayMark competes against the chaos that fast AI code generates in serious projects — not against the speed itself.
7. **Local-first, Cloud as amplifier.** The CLI works fully offline. Cloud may add value (cross-repo aggregation, signed evidence) but never gates the core.
8. **Project memory lives in the repo, not in an external database.** AILOGs, ADRs, AIDECs, Charters and bundles are versioned files alongside the code, in markdown + JSON Schema.
9. **Simplicity over capability.** When two designs meet the goal, the simpler one wins. Patterns crystallize after they're validated in real projects, not before.
10. **Honesty about what the tool does not do.** No model evaluation, no LLM gateway, no automatic compliance certification, no replacement for engineering judgment.
11. **The community takes care of the tool, not the other way around.** Contributions and feedback are taken seriously without becoming a democracy.
12. **The product's velocity is the velocity of learning.** No premature crystallization; schemas marked `v0` until validated against a second domain.

The full document, with empirical annotations from validation cycles, lives in [`docs/contributors/DESIGN-PRINCIPLES.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/contributors/DESIGN-PRINCIPLES.md).

---

## Features

### 📋 Structured Documentation

Sixteen document types covering the full development lifecycle (twelve core + four China-specific opt-in):

| Type | Purpose | Example |
| --- | --- | --- |
| **REQ** | Requirements | System requirements, user stories |
| **ADR** | Architecture Decisions | Technology choices, design patterns |
| **TES** | Test Plans | Test strategies, coverage goals |
| **INC** | Incident Post-mortems | Root cause analysis, lessons learned |
| **TDE** | Technical Debt | Identified debt, remediation plans |
| **AILOG** | AI Action Logs | What AI assistants did and why |
| **AIDEC** | AI Decisions | Choices made by AI with alternatives |
| **ETH** | Ethical Reviews | Privacy, bias, responsible AI |
| **SEC** | Security Assessments | Threat modeling, vulnerability analysis |
| **MCARD** | Model/System Cards | AI model documentation |
| **SBOM** | Software Bill of Materials | AI component inventory |
| **DPIA** | Data Protection Impact Assessment | Privacy impact analysis |
| **PIPIA** ⚪ | Personal Information Protection Impact Assessment (China — PIPL Art. 55-56) | Sensitive data, cross-border transfer |
| **CACFILE** ⚪ | CAC Algorithm Filing (China) | Algorithm registration, dual filing |
| **TC260RA** ⚪ | TC260 Risk Assessment (China) | Five-level risk grading per AI Safety Framework v2.0 |
| **AILABEL** ⚪ | GB 45438 Content Labeling Plan (China) | Explicit + implicit labeling for generative AI |

⚪ Available only when `regional_scope: china` is enabled in `.straymark/config.yml` — see [Compliance](#compliance) below.

### 🤖 AI Agent Support

Pre-configured for popular AI coding assistants:

- **Universal (AGENTS.md standard)** → `AGENTS.md` — read by Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others
- **Claude Code** (Anthropic) → `CLAUDE.md`
- **Cursor** → `.cursorrules`
- **GitHub Copilot CLI** → `.github/copilot-instructions.md`
- **Gemini CLI** (Google) → `GEMINI.md`

Each configuration instructs the AI to:
- Identify itself in every document
- Declare confidence levels
- Request human review when appropriate
- Follow naming conventions
- **Follow Git branching strategy** (never commit to `main` directly)

### 👁️ Human Oversight

Built-in safeguards ensure humans stay in control:

- **Autonomy levels**: Some document types require human approval
- **Review triggers**: Low confidence or high risk → mandatory review
- **Ethical reviews**: Privacy and bias concerns flagged for human decision

### ✅ CLI Tooling

Built-in commands that turn the discipline into actionable feedback:

- **`straymark charter <new|list|status|close|drift|batch-complete|audit|refresh-suggest|amend>`** — Bounded units of work declared ex-ante, audited ex-post. `close` records post-execution telemetry; `drift` detects file-vs-commit drift with AILOG-aware suppression and (cli-3.13.0+) gates on `### Batch N (pending)` entries in the AILOG `## Batch Ledger`; `batch-complete` (cli-3.13.0+) marks a batch as complete in the ledger for multi-batch Charters (3+ batches or >1 day); `audit` orchestrates a multi-model external review (3-step prepare/calibrate/finalize, orchestration-only — no LLM API calls); `refresh-suggest` (cli-3.14.0+) prints a heuristic recommendation for a pre-declare SpecKit refresh when a multi-Charter module's rolling `r_n_plus_one_emergent_count` mean exceeds a threshold; `amend` (cli-3.14.0+) scaffolds a post-close Batch N.4 amendment (audit-driven remediation) on the same execute branch without opening a new Charter. For IDE-driven workflows, the inline skills `/straymark-audit-prompt` and `/straymark-audit-review` wrap the CLI to surface prompts in the conversation and merge findings into telemetry.
- **`straymark followups <list|status|drift|recount|promote>`** *(cli-3.19.0+; `recount` cli-3.20.0+)* — First-class follow-ups backlog registry (`.straymark/follow-ups-backlog.md`, schema v1 experimental): `drift --apply` extracts `§Follow-ups` / `R<N> (new)` entries from AILOGs per-AILOG (closure-marked bullets land as `suspected-closed`), counters are CLI-owned and recomputed on every write, `recount` reconciles them after a manual-triage session, and `promote` elevates entries to TDE documents with full traceability. See `STRAYMARK.md §16` and `FOLLOW-UPS-BACKLOG-PATTERN.md`.
- **`straymark approve <doc-id>`** — Record a formal human approval (writes `reviewed_by` / `reviewed_at` / `review_outcome` and the `## Approval` body section in one edit; closes the gap canonized in DOCUMENTATION-POLICY §3.5)
- **`straymark validate`** — 25+ validation rules for document correctness (12 China-specific are scope-aware); `--include-charters` extends to `.straymark/charters/`; `--check-pending-reviews` lists approval backlog (warn-only)
- **`straymark metrics`** — Governance KPIs, review rates, risk distribution, trends
- **`straymark analyze`** — Code complexity analysis (cognitive + cyclomatic) powered by [arborist-metrics](https://github.com/StrangeDaysTech/arborist-metrics/), our open-source Rust library for multi-language code metrics, also developed by StrangeDaysTech S.A.S. de C.V.
- **`straymark audit`** — Audit trail reports with timeline, traceability maps, and HTML export
- **`straymark compliance`** — Regulatory compliance scoring as a side effect of the documented work (EU AI Act, ISO 42001, NIST AI RMF; six Chinese frameworks opt-in via `--region china`)
- **`straymark explore`** — Interactive TUI for navigating the project's documentation graph, including a Charter view (lifecycle status, origin AILOG/spec, file location)
- **Pre-commit hooks** + **GitHub Actions** for CI/CD validation

---

## Honest Limits

StrayMark does **not**:

- evaluate, benchmark, or rank LLMs;
- act as an LLM gateway or routing layer;
- prevent hallucinations or guarantee agent correctness;
- automatically certify regulatory compliance — it produces evidence, not certifications;
- replace the judgment of a senior engineer.

If your problem is one of those, StrayMark is not the tool.

---

## Compliance

The discipline StrayMark externalizes — explicit scope, declared decisions, named risks, recorded alternatives — produces, as a side effect, evidence that maps cleanly onto the major AI governance frameworks. Compliance is therefore positioned as a *consequence of doing the engineering work well*, not as the product itself (Principle #4).

### Standards Alignment

| Standard | StrayMark Integration |
| --- | --- |
| **ISO/IEC 42001:2023** | Vertebral standard — AI Management System governance |
| **EU AI Act** | Risk classification, incident reporting, transparency |
| **NIST AI RMF / 600-1** | 12 GenAI risk categories in ETH/AILOG |
| **ISO/IEC 25010:2023** | Software quality model in REQ/ADR |
| **ISO/IEC/IEEE 29148:2018** | Requirements engineering in REQ |
| **ISO/IEC/IEEE 29119-3:2021** | Test documentation in TES |
| **GDPR** | Data protection in ETH/DPIA |
| **OpenTelemetry** | Observability (optional) |

### China Regulatory Coverage — opt-in via `regional_scope: china`

| Standard | StrayMark Integration |
| --- | --- |
| **TC260 AI Safety Governance Framework v2.0** | Five-level risk grading (TC260RA) |
| **PIPL — Personal Information Protection Law** | Personal Information Protection Impact Assessment (PIPIA), retention ≥ 3 years |
| **GB 45438-2025** *(mandatory)* | AI-generated content labeling — explicit + implicit (AILABEL) |
| **CAC Algorithm Filing** | Algorithm registration, dual filing process (CACFILE) |
| **GB/T 45652-2025** | Pre-training & fine-tuning data security (SBOM/MCARD) |
| **CSL 2026** | Cybersecurity incident reporting (1h / 4h+72h+30d windows) on INC |

StrayMark covers six Chinese AI / data regulations as an **opt-in** regional scope. Activate by adding `regional_scope: china` to `.straymark/config.yml`; projects without it are unaffected. When enabled, four China-specific document types (PIPIA, CACFILE, TC260RA, AILABEL) become available, twelve validation rules begin to enforce the new cross-references, and `straymark compliance --region china` produces a per-framework score. Detailed guides live under `.straymark/00-governance/` (`CHINA-REGULATORY-FRAMEWORK.md`, `TC260-IMPLEMENTATION-GUIDE.md`, `PIPL-PIPIA-GUIDE.md`, `CAC-FILING-GUIDE.md`, `GB-45438-LABELING-GUIDE.md`).

### 中国法规支持

StrayMark 现在以 **opt-in**(自愿启用)的方式覆盖六项中国 AI / 数据法规:**TC260《人工智能安全治理框架 v2.0》**(五级风险分级)、**《个人信息保护法》(PIPL)** 及其配套的 **PIPIA**(个人信息保护影响评估,留存 ≥ 3 年)、**强制性国家标准 GB 45438-2025**《网络安全技术 人工智能生成合成内容标识方法》(显式 + 隐式标识)、**CAC 算法备案**(包括省级 + 国家级双重备案)、**GB/T 45652-2025** 预训练与微调数据安全,以及自 2026-01-01 生效的 **《网络安全法》修订** 与《国家网络安全事件报告管理办法》(1 小时 / 4 小时 + 72 小时评估 + 30 天事后审查的报告窗口)。

#### 启用方式

在 `.straymark/config.yml` 中添加:

```yaml
regional_scope:
  - global   # NIST + ISO 42001(始终可用)
  - eu       # EU AI Act + GDPR
  - china    # 启用上述六项中国法规
```

#### 启用后获得

- **4 个中国专属文档类型**:`PIPIA`、`CACFILE`、`TC260RA`、`AILABEL`(均经 `straymark new` 生成,模板已翻译为中文,位于 `.straymark/templates/i18n/zh-CN/`)。
- **6 个合规检查器**:通过 `straymark compliance --region china` 一次性运行,或单独运行 `--standard china-tc260 | china-pipl | china-gb45438 | china-cac | china-gb45652 | china-csl`。
- **12 条新的验证规则**(`CROSS-004…011`、`TYPE-003…006`):自动校验跨文档引用一致性,例如:`cac_filing_required: true` 必须关联 CACFILE;`csl_severity_level: particularly_serious` 必须配合 `csl_report_deadline_hours: 1`;PIPIA 的 `pipl_retention_until` 必须至少为 `created` + 3 年。
- **5 份中文治理指南**,位于 `.straymark/00-governance/i18n/zh-CN/`:`CHINA-REGULATORY-FRAMEWORK.md`、`TC260-IMPLEMENTATION-GUIDE.md`、`PIPL-PIPIA-GUIDE.md`、`CAC-FILING-GUIDE.md`、`GB-45438-LABELING-GUIDE.md`。

#### 适用人群

- 在中国大陆运营 AI 服务的团队,需办理 CAC 算法备案或对外提供生成式 AI。
- 处理中国大陆个人信息(尤其是敏感个人信息)、需进行 PIPIA 的处理者。
- 涉及跨境数据传输,须依据 PIPL 第 38-40 条选择安全评估、认证或标准合同机制的组织。
- 采用 ISO/IEC 42001 全球治理框架并希望在中国境内补充本地合规证据的企业。

不在 `regional_scope` 中包含 `china` 的项目完全不受影响 — 这是完全向后兼容的扩展。

---

## Getting Started

### Option 1: CLI (Recommended)

**Quick install (prebuilt binary):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

Or install from source with Cargo:

```bash
cargo install straymark-cli
```

> **Note:** `straymark update-cli` automatically detects how you installed the CLI. Prebuilt binary installs update from GitHub Releases; cargo installs update via `cargo install`. You can override with `--method=github` or `--method=cargo`.

Then initialize in your project:

```bash
cd your-project
straymark init .
```

The CLI downloads the latest StrayMark release, sets up the framework, and configures your AI agent directive files automatically.

### Versioning

StrayMark uses independent version tags for each component:

| Component | Tag prefix | Example | Includes |
| --- | --- | --- | --- |
| Framework | `fw-` | `fw-4.35.0` | Templates (12 types), governance, directives, Charter template + schema |
| CLI | `cli-` | `cli-3.33.0` | The `straymark` binary |
| Loom (EXPERIMENTAL) | `loom-` | `loom-0.4.2` | The `straymark-loom` visualization server, downloaded on demand by `straymark loom serve` |

Check installed versions with `straymark status` or `straymark about`.

### CLI Commands

| Command | Description |
| --- | --- |
| `straymark init [path]` | Initialize StrayMark in a project |
| `straymark update` | Update both framework and CLI |
| `straymark update-framework` | Update only the framework |
| `straymark update-cli` | Update the CLI binary |
| `straymark remove [--full]` | Remove StrayMark from project |
| `straymark status [path]` | Show installation health and doc stats |
| `straymark repair [path]` | Restore missing directories and framework files |
| `straymark validate [path]` | Validate documents for compliance and correctness (use `--include-charters` for Charters, `--check-pending-reviews` for approval backlog) |
| `straymark charter <subcommand>` | Manage Charters: `new`, `list`, `status`, `close` (record telemetry), `drift` (file-vs-commit drift with AILOG-awareness + Batch Ledger gate), `batch-complete` (mark a batch as complete in the AILOG `## Batch Ledger` for multi-batch Charters), `audit` (multi-model external review, orchestration-only), `refresh-suggest` (pre-declare SpecKit refresh heuristic for multi-Charter modules, fw-4.16.0+), `amend` (scaffold a post-close Batch N.4 amendment, fw-4.16.0+) |
| `straymark followups <subcommand>` *(cli-3.19.0+)* | Manage the follow-ups backlog registry: `list` (filterable entries), `status` (pulse with CLI-owned counters recomputed on the fly), `drift` (detect/extract unprocessed AILOGs — native replacement for the v0 bash script, with anti-noise `suspected-closed` extraction), `promote` (FU → TDE with `promoted_from_followup` traceability) |
| `straymark approve <doc-id>` | Record a formal human approval on a `review_required: true` document (frontmatter + canonical body section) |
| `straymark compliance [path]` | Check regulatory compliance (EU AI Act, ISO 42001, NIST) |
| `straymark metrics [path]` | Show governance metrics and documentation statistics |
| `straymark analyze [path]` | Analyze code complexity (cognitive + cyclomatic metrics) |
| `straymark audit [path]` | Generate audit trail reports with timeline and traceability |
| `straymark explore [path]` | Browse documentation interactively in a TUI |
| `straymark about` | Show version and license info |

See [CLI Reference](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md) for detailed usage.

### Option 2: Manual Setup

```bash
# Download the latest framework release ZIP from GitHub
# Go to https://github.com/StrangeDaysTech/straymark/releases
# and download the latest fw-* release (e.g., fw-4.28.0)

# Extract and copy to your project
unzip straymark-fw-*.zip -d your-project/
cd your-project

# Commit
git add .straymark/ STRAYMARK.md
git commit -m "chore: adopt StrayMark"
```

📖 **See [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTION-GUIDE.md) for detailed instructions, migration strategies, and team rollout plans.**

---

## Documentation

StrayMark documentation is organized by audience:

| Track | For | Start here |
| --- | --- | --- |
| [**Adopters**](https://github.com/StrangeDaysTech/straymark/tree/main/docs/adopters) | Teams adopting StrayMark in their projects | [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTION-GUIDE.md) |
| [**Contributors**](https://github.com/StrangeDaysTech/straymark/tree/main/docs/contributors) | Developers contributing to StrayMark | [TRANSLATION-GUIDE.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/contributors/TRANSLATION-GUIDE.md) |

**Adopters**: Follow the [Adoption Guide](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTION-GUIDE.md) for step-by-step instructions, the [CLI Reference](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md) for command details, and the [Workflows Guide](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/WORKFLOWS.md) for daily usage patterns.

**Contributors**: See [CONTRIBUTING.md](https://github.com/StrangeDaysTech/straymark/blob/main/CONTRIBUTING.md) for development guidelines, and the [Translation Guide](https://github.com/StrangeDaysTech/straymark/blob/main/docs/contributors/TRANSLATION-GUIDE.md) for adding new languages.

### Key References

| Document | Description |
| --- | --- |
| [**Quick Reference**](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/QUICK-REFERENCE.md) | One-page overview of document types and naming |
| [STRAYMARK.md](https://github.com/StrangeDaysTech/straymark/blob/main/dist/STRAYMARK.md) | Unified governance rules (source of truth) |
| [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTION-GUIDE.md) | Adoption guide for new/existing projects |
| [CLI-REFERENCE.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md) | Complete CLI command reference |
| [WORKFLOWS.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/WORKFLOWS.md) | Recommended daily workflows and team patterns |

### Internal Structure

Once adopted, StrayMark creates a `.straymark/` directory in your project for development governance:

```
.straymark/
├── 00-governance/           # Policies and rules (China guides ⚪)
├── 01-requirements/         # REQ documents
├── 02-design/decisions/     # ADR documents
├── 03-implementation/       # Implementation guides (incl. Git strategy)
├── 04-testing/              # TES documents
├── 05-operations/incidents/ # INC documents
├── 06-evolution/technical-debt/ # TDE documents
├── 07-ai-audit/
│   ├── agent-logs/          # AILOG documents
│   ├── decisions/           # AIDEC documents
│   ├── ethical-reviews/     # ETH, DPIA, PIPIA ⚪
│   ├── regulatory-filings/  # CACFILE ⚪
│   └── risk-assessments/    # TC260RA ⚪
├── 08-security/             # SEC documents
├── 09-ai-models/            # MCARD documents
│   └── labeling/            # AILABEL ⚪
└── templates/               # Document templates

⚪ Created only when regional_scope: china is enabled in .straymark/config.yml
```

### Naming Convention

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

Example: `ADR-2025-01-27-001-use-postgresql-for-persistence.md`

---

## How It Works

### 1. AI Makes a Change

An AI assistant working on your code automatically:

```yaml
# Creates: .straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implement-auth.md
---
id: AILOG-2025-01-27-001
title: Implement JWT authentication
agent: claude-code-v1.0
confidence: high
risk_level: high
review_required: true
---
```

### 2. Human Reviews (When Needed)

High-risk or low-confidence changes are flagged:

```
📋 AILOG-2025-01-27-001-implement-auth.md
   Agent: claude-code-v1.0
   Confidence: high
   Risk Level: high ⚠️
   Review Required: YES
```

### 3. Decisions Are Preserved

When choosing between alternatives, decisions are documented:

```yaml
# Creates: .straymark/07-ai-audit/decisions/AIDEC-2025-01-27-001-auth-strategy.md
---
id: AIDEC-2025-01-27-001
title: Choose JWT over session-based auth
alternatives_considered:
  - JWT tokens (chosen)
  - Session cookies
  - OAuth only
justification: "Stateless architecture requirement..."
---
```

### 4. Ethical Concerns Are Flagged

When AI encounters ethical considerations:

```yaml
# Creates: .straymark/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-user-data.md
---
id: ETH-2025-01-27-001
title: User data collection scope
status: draft  # Requires human approval
review_required: true
concerns:
  - GDPR compliance
  - Data minimization
---
```

---

## Validation

### Pre-commit Hook

```bash
# Install the pre-commit hook
echo 'straymark validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Manual Validation

```bash
# Cross-platform (any OS with straymark installed)
straymark validate
```

### GitHub Actions

The included workflow (`.github/workflows/docs-validation.yml`) automatically validates:
- File naming conventions
- Required metadata fields
- Sensitive information detection
- Markdown formatting
- Internal link integrity

---

## Skills

StrayMark includes skills for AI agents that enable **active documentation creation**.

> **Binary System**: StrayMark uses a passive system (agents auto-document via context instructions) and an active system (users invoke skills to create documentation manually or when the agent missed something).

### Available Skills

| Skill | Purpose | Claude | Gemini | Codex |
| --- | --- | --- | --- | --- |
| `/straymark-status` | Check documentation compliance | ✅ | ✅ | ✅ |
| `/straymark-new` | Create any document type (unified) | ✅ | ✅ | ✅ |
| `/straymark-ailog` | Quick AILOG creation | ✅ | ✅ | ✅ |
| `/straymark-aidec` | Quick AIDEC creation | ✅ | ✅ | ✅ |
| `/straymark-adr` | Quick ADR creation | ✅ | ✅ | ✅ |
| `/straymark-sec` | Security Assessment creation | ✅ | ✅ | ✅ |
| `/straymark-mcard` | Model/System Card creation | ✅ | ✅ | ✅ |
| `/straymark-architecture` *(fw-4.29.0+, EXPERIMENTAL)* | Generate + agent-refine the architecture model (reassign layers, wire links, sync DrawIO, validate) | ✅ | ✅ | ✅ |
| `/straymark-architecture-sync` *(fw-4.29.0+, EXPERIMENTAL)* | Keep the architecture model current as code grows (append-only) | ✅ | ✅ | ✅ |
| `/straymark-loom` *(fw-4.29.0+, EXPERIMENTAL)* | Drive the Loom visualization server lifecycle (up / down / status), terminal-free | ✅ | ✅ | ✅ |

> **Codex CLI users** *(fw-4.19.0+)*: Codex reads skills from `~/.codex/skills/` (user-level), not from the project tree. After `straymark init` (or any subsequent `straymark update`), run `straymark install-skills --agent codex` once to populate it from the project's `.codex/skills/`.

### Usage Examples

```bash
# Check documentation status
/straymark-status

# Create documentation (agent suggests type)
/straymark-new

# Force specific document type
/straymark-new ailog

# Direct shortcuts
/straymark-ailog
/straymark-aidec
/straymark-adr
```

### CLI Commands (Manual Use)

For users who prefer the command line or use agents without skill support:

```bash
# Interactive document creation
straymark new

# Create specific type directly
straymark new --doc-type ailog

# Check documentation status
straymark status
```

### Agent Reporting

AI agents report documentation status at the end of each task:

| Status | Meaning |
| --- | --- |
| `StrayMark: Created AILOG-...` | Documentation was created |
| `StrayMark: No documentation required` | Change was minor |
| `StrayMark: Documentation pending` | May need manual review |

### Multi-Agent Architecture

StrayMark provides native skill support for multiple AI agents through a layered architecture:

```
your-project/
├── .agent/workflows/       # 🌐 Agnostic (Antigravity, future agents)
│   ├── straymark-new.md
│   ├── straymark-status.md
│   └── ...
├── .gemini/skills/         # 🔵 Gemini CLI (Google)
│   ├── straymark-new/SKILL.md
│   └── ...
├── .claude/skills/         # 🟣 Claude Code (Anthropic)
│   ├── straymark-new/SKILL.md
│   └── ...
└── .codex/skills/          # 🟢 Codex CLI (OpenAI) — installed to ~/.codex/skills/
    ├── straymark-new/SKILL.md
    └── ...
```

| Directory | Agent | Product | Format |
| --- | --- | --- | --- |
| `.agent/workflows/` | Antigravity, generic | VS Code/Cursor extensions | `skill-name.md` with YAML frontmatter |
| `.gemini/skills/` | Gemini CLI | Google's terminal CLI | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Anthropic's coding agent | `skill-name/SKILL.md` |
| `.codex/skills/` *(fw-4.19.0+)* | Codex CLI | OpenAI's coding agent | `skill-name/SKILL.md` (minimal frontmatter) — installed to `~/.codex/skills/` via `straymark install-skills --agent codex` |

> **Note**: `.agent/` is the **vendor-agnostic** standard. Agent-specific directories (`.gemini/`, `.claude/`) provide compatibility for those platforms while following their native conventions.

All skill implementations are **functionally identical**—only the format differs to match each agent's requirements.

---

## Supported Platforms

### AI Coding Assistants

| Platform | Config File | Status |
| --- | --- | --- |
| Universal (AGENTS.md standard) | `AGENTS.md` | ✅ Full support |
| Claude Code | `CLAUDE.md` | ✅ Full support |
| Cursor | `.cursorrules` | ✅ Full support |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | ✅ Full support |
| Gemini CLI | `GEMINI.md` | ✅ Full support |
| Codex CLI (OpenAI) *(fw-4.19.0+)* | `AGENTS.md` + `~/.codex/skills/` | ✅ Full support (run `straymark install-skills --agent codex`) |

### Operating Systems

| OS | Validation |
| --- | --- |
| Linux | `straymark validate` |
| macOS | `straymark validate` |
| Windows | `straymark validate` |

### CI/CD Platforms

| Platform | Support |
| --- | --- |
| GitHub Actions | ✅ Included workflow |
| GitLab CI | 🔧 Adaptable from GitHub Actions |
| Azure DevOps | 🔧 Adaptable from GitHub Actions |

---

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](https://github.com/StrangeDaysTech/straymark/blob/main/CONTRIBUTING.md) for guidelines.

### Ways to Contribute

- 🐛 Report bugs
- 💡 Suggest features
- 📖 Improve documentation
- 🔧 Submit pull requests
- 🌍 Add translations

---

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/StrangeDaysTech/straymark/blob/main/LICENSE) file for details.

---

## About Strange Days Tech, S.A.S.

<div align="center">

[**Strange Days Tech**](https://strangedays.tech) builds tools for responsible AI-assisted software development.

Our open-source ecosystem:

| Project | Description |
| --- | --- |
| [**StrayMark**](https://github.com/StrangeDaysTech/straymark) | The cognitive discipline your AI-assisted projects need |
| [**arborist-metrics**](https://github.com/StrangeDaysTech/arborist-metrics/) | Multi-language code complexity analysis library for Rust — [crates.io](https://crates.io/crates/arborist-metrics) |

[Website](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**StrayMark** — Engineering discipline, externalized. Compliance, as a side effect.

[⬆ Back to top](#straymark)

</div>
