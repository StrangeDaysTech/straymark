<div align="center">

# DevTrail

**AI Governance Platform for Responsible Software Development**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/StrangeDaysTech/devtrail/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/devtrail-cli.svg)](https://crates.io/crates/devtrail-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/StrangeDaysTech/devtrail/blob/main/CONTRIBUTING.md)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](https://github.com/StrangeDaysTech/devtrail/blob/main/dist/.devtrail/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[Getting Started](#getting-started) •
[Features](#features) •
[China Compliance 中国合规](#china-regulatory-compliance--中国合规) •
[Documentation](#documentation) •
[Contributing](#contributing)

**Languages**: English | [Español](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/i18n/es/README.md) | [简体中文](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/i18n/zh-CN/README.md)

</div>

---

## The Problem

As AI becomes integral to software development, organizations face three converging pressures:

- **Regulatory compliance**: The EU AI Act becomes mandatory in August 2026. ISO/IEC 42001 is now the international standard for AI governance. Teams need documented evidence.
- **Governance gap**: No structured way to prove that AI decisions are governed, auditable, and compliant — every undocumented AI change is a liability.
- **Operational risk**: Who made this change? What alternatives were considered? Was human oversight appropriate? Without answers, AI-assisted development is a black box.

## The Solution

DevTrail is an **ISO 42001-aligned AI governance platform** that ensures every meaningful change — whether by human or AI — is documented, attributed, and auditable.

> **"No significant change without a documented trace — and proof of governance."**

Teams that adopt DevTrail produce evidence compatible with **ISO/IEC 42001 certification**, **EU AI Act compliance**, and **NIST AI RMF** risk management — while improving development quality and traceability.

---

## Features

### 📋 Structured Documentation

Sixteen document types covering the full development lifecycle (twelve core + four China-specific opt-in):

| Type | Purpose | Example |
|------|---------|---------|
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

⚪ Available only when `regional_scope: china` is enabled in `.devtrail/config.yml` — see [China Regulatory Compliance](#china-regulatory-compliance--中国合规) below.

### 📐 Standards Alignment

| Standard | DevTrail Integration |
|----------|---------------------|
| **ISO/IEC 42001:2023** | Vertebral standard — AI Management System governance |
| **EU AI Act** | Risk classification, incident reporting, transparency |
| **NIST AI RMF / 600-1** | 12 GenAI risk categories in ETH/AILOG |
| **ISO/IEC 25010:2023** | Software quality model in REQ/ADR |
| **ISO/IEC/IEEE 29148:2018** | Requirements engineering in REQ |
| **ISO/IEC/IEEE 29119-3:2021** | Test documentation in TES |
| **GDPR** | Data protection in ETH/DPIA |
| **OpenTelemetry** | Observability (optional) |

#### China Regulatory Coverage — opt-in via `regional_scope: china`

| Standard | DevTrail Integration |
|----------|---------------------|
| **TC260 AI Safety Governance Framework v2.0** | Five-level risk grading (TC260RA) |
| **PIPL — Personal Information Protection Law** | Personal Information Protection Impact Assessment (PIPIA), retention ≥ 3 years |
| **GB 45438-2025** *(mandatory)* | AI-generated content labeling — explicit + implicit (AILABEL) |
| **CAC Algorithm Filing** | Algorithm registration, dual filing process (CACFILE) |
| **GB/T 45652-2025** | Pre-training & fine-tuning data security (SBOM/MCARD) |
| **CSL 2026** | Cybersecurity incident reporting (1h / 4h+72h+30d windows) on INC |

### 🤖 AI Agent Support

Pre-configured for popular AI coding assistants:

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

### ✅ Compliance Automation

Built-in CLI tools for governance:

- **`devtrail validate`** — 25+ validation rules for document correctness (12 China-specific are scope-aware)
- **`devtrail compliance`** — Regulatory compliance scoring (EU AI Act, ISO 42001, NIST AI RMF; six Chinese frameworks opt-in via `--region china`)
- **`devtrail metrics`** — Governance KPIs, review rates, risk distribution, trends
- **`devtrail analyze`** — Code complexity analysis (cognitive + cyclomatic) powered by [arborist-metrics](https://github.com/StrangeDaysTech/arborist), our open-source Rust library for multi-language code metrics
- **`devtrail audit`** — Audit trail reports with timeline, traceability maps, and HTML export
- **Pre-commit hooks** + **GitHub Actions** for CI/CD validation

---

## China Regulatory Compliance — 中国合规

DevTrail covers six Chinese AI / data regulations as an **opt-in** regional scope: **TC260 AI Safety Governance Framework v2.0**, **PIPL** (Personal Information Protection Law), **GB 45438-2025** (mandatory AI content labeling), **CAC Algorithm Filing**, **GB/T 45652-2025**, and the **CSL 2026** incident-reporting amendments. Activate by adding `regional_scope: china` to `.devtrail/config.yml`; projects without it are unaffected.

When enabled, four China-specific document types (PIPIA, CACFILE, TC260RA, AILABEL) become available, twelve validation rules begin to enforce the new cross-references, and `devtrail compliance --region china` produces a per-framework score. Detailed guides live under `.devtrail/00-governance/` (`CHINA-REGULATORY-FRAMEWORK.md`, `TC260-IMPLEMENTATION-GUIDE.md`, `PIPL-PIPIA-GUIDE.md`, `CAC-FILING-GUIDE.md`, `GB-45438-LABELING-GUIDE.md`).

### 中国法规支持

DevTrail 现在以 **opt-in**(自愿启用)的方式覆盖六项中国 AI / 数据法规:**TC260《人工智能安全治理框架 v2.0》**(五级风险分级)、**《个人信息保护法》(PIPL)** 及其配套的 **PIPIA**(个人信息保护影响评估,留存 ≥ 3 年)、**强制性国家标准 GB 45438-2025**《网络安全技术 人工智能生成合成内容标识方法》(显式 + 隐式标识)、**CAC 算法备案**(包括省级 + 国家级双重备案)、**GB/T 45652-2025** 预训练与微调数据安全,以及自 2026-01-01 生效的 **《网络安全法》修订** 与《国家网络安全事件报告管理办法》(1 小时 / 4 小时 + 72 小时评估 + 30 天事后审查的报告窗口)。

#### 启用方式

在 `.devtrail/config.yml` 中添加:

```yaml
regional_scope:
  - global   # NIST + ISO 42001(始终可用)
  - eu       # EU AI Act + GDPR
  - china    # 启用上述六项中国法规
```

#### 启用后获得

- **4 个中国专属文档类型**:`PIPIA`、`CACFILE`、`TC260RA`、`AILABEL`(均经 `devtrail new` 生成,模板已翻译为中文,位于 `.devtrail/templates/i18n/zh-CN/`)。
- **6 个合规检查器**:通过 `devtrail compliance --region china` 一次性运行,或单独运行 `--standard china-tc260 | china-pipl | china-gb45438 | china-cac | china-gb45652 | china-csl`。
- **12 条新的验证规则**(`CROSS-004…011`、`TYPE-003…006`):自动校验跨文档引用一致性,例如:`cac_filing_required: true` 必须关联 CACFILE;`csl_severity_level: particularly_serious` 必须配合 `csl_report_deadline_hours: 1`;PIPIA 的 `pipl_retention_until` 必须至少为 `created` + 3 年。
- **5 份中文治理指南**,位于 `.devtrail/00-governance/i18n/zh-CN/`:`CHINA-REGULATORY-FRAMEWORK.md`、`TC260-IMPLEMENTATION-GUIDE.md`、`PIPL-PIPIA-GUIDE.md`、`CAC-FILING-GUIDE.md`、`GB-45438-LABELING-GUIDE.md`。

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
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

Or install from source with Cargo:

```bash
cargo install devtrail-cli
```

> **Note:** `devtrail update-cli` automatically detects how you installed the CLI. Prebuilt binary installs update from GitHub Releases; cargo installs update via `cargo install`. You can override with `--method=github` or `--method=cargo`.

Then initialize in your project:

```bash
cd your-project
devtrail init .
```

The CLI downloads the latest DevTrail release, sets up the framework, and configures your AI agent directive files automatically.

### Versioning

DevTrail uses independent version tags for each component:

| Component | Tag prefix | Example | Includes |
|-----------|-----------|---------|----------|
| Framework | `fw-` | `fw-4.4.2` | Templates (12 types), governance, directives, Charter template + schema |
| CLI | `cli-` | `cli-3.6.0` | The `devtrail` binary |

Check installed versions with `devtrail status` or `devtrail about`.

### CLI Commands

| Command | Description |
|---------|-------------|
| `devtrail init [path]` | Initialize DevTrail in a project |
| `devtrail update` | Update both framework and CLI |
| `devtrail update-framework` | Update only the framework |
| `devtrail update-cli` | Update the CLI binary |
| `devtrail remove [--full]` | Remove DevTrail from project |
| `devtrail status [path]` | Show installation health and doc stats |
| `devtrail repair [path]` | Restore missing directories and framework files |
| `devtrail validate [path]` | Validate documents for compliance and correctness (use `--include-charters` to also check `docs/charters/`) |
| `devtrail charter <subcommand>` | Manage Charters: `new`, `list`, `status` (bounded units of work declared ex-ante, audited ex-post) |
| `devtrail compliance [path]` | Check regulatory compliance (EU AI Act, ISO 42001, NIST) |
| `devtrail metrics [path]` | Show governance metrics and documentation statistics |
| `devtrail analyze [path]` | Analyze code complexity (cognitive + cyclomatic metrics) |
| `devtrail audit [path]` | Generate audit trail reports with timeline and traceability |
| `devtrail explore [path]` | Browse documentation interactively in a TUI |
| `devtrail about` | Show version and license info |

See [CLI Reference](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/CLI-REFERENCE.md) for detailed usage.

### Option 2: Manual Setup

```bash
# Download the latest framework release ZIP from GitHub
# Go to https://github.com/StrangeDaysTech/devtrail/releases
# and download the latest fw-* release (e.g., fw-4.4.2)

# Extract and copy to your project
unzip devtrail-fw-*.zip -d your-project/
cd your-project

# Commit
git add .devtrail/ DEVTRAIL.md
git commit -m "chore: adopt DevTrail"
```

📖 **See [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/ADOPTION-GUIDE.md) for detailed instructions, migration strategies, and team rollout plans.**

---

## Documentation

DevTrail documentation is organized by audience:

| Track | For | Start here |
|-------|-----|------------|
| [**Adopters**](https://github.com/StrangeDaysTech/devtrail/tree/main/docs/adopters) | Teams adopting DevTrail in their projects | [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/ADOPTION-GUIDE.md) |
| [**Contributors**](https://github.com/StrangeDaysTech/devtrail/tree/main/docs/contributors) | Developers contributing to DevTrail | [TRANSLATION-GUIDE.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/contributors/TRANSLATION-GUIDE.md) |

**Adopters**: Follow the [Adoption Guide](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/ADOPTION-GUIDE.md) for step-by-step instructions, the [CLI Reference](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/CLI-REFERENCE.md) for command details, and the [Workflows Guide](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/WORKFLOWS.md) for daily usage patterns.

**Contributors**: See [CONTRIBUTING.md](https://github.com/StrangeDaysTech/devtrail/blob/main/CONTRIBUTING.md) for development guidelines, and the [Translation Guide](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/contributors/TRANSLATION-GUIDE.md) for adding new languages.

### Key References

| Document | Description |
|----------|-------------|
| [**Quick Reference**](https://github.com/StrangeDaysTech/devtrail/blob/main/dist/.devtrail/QUICK-REFERENCE.md) | One-page overview of document types and naming |
| [DEVTRAIL.md](https://github.com/StrangeDaysTech/devtrail/blob/main/dist/DEVTRAIL.md) | Unified governance rules (source of truth) |
| [ADOPTION-GUIDE.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/ADOPTION-GUIDE.md) | Adoption guide for new/existing projects |
| [CLI-REFERENCE.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/CLI-REFERENCE.md) | Complete CLI command reference |
| [WORKFLOWS.md](https://github.com/StrangeDaysTech/devtrail/blob/main/docs/adopters/WORKFLOWS.md) | Recommended daily workflows and team patterns |

### Internal Structure

Once adopted, DevTrail creates a `.devtrail/` directory in your project for development governance:

```
.devtrail/
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

⚪ Created only when regional_scope: china is enabled in .devtrail/config.yml
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
# Creates: .devtrail/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implement-auth.md
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
# Creates: .devtrail/07-ai-audit/decisions/AIDEC-2025-01-27-001-auth-strategy.md
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
# Creates: .devtrail/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-user-data.md
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
echo 'devtrail validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Manual Validation

```bash
# Cross-platform (any OS with devtrail installed)
devtrail validate
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

DevTrail includes skills for AI agents that enable **active documentation creation**.

> **Binary System**: DevTrail uses a passive system (agents auto-document via context instructions) and an active system (users invoke skills to create documentation manually or when the agent missed something).

### Available Skills

| Skill | Purpose | Claude | Gemini |
|-------|---------|--------|--------|
| `/devtrail-status` | Check documentation compliance | ✅ | ✅ |
| `/devtrail-new` | Create any document type (unified) | ✅ | ✅ |
| `/devtrail-ailog` | Quick AILOG creation | ✅ | ✅ |
| `/devtrail-aidec` | Quick AIDEC creation | ✅ | ✅ |
| `/devtrail-adr` | Quick ADR creation | ✅ | ✅ |
| `/devtrail-sec` | Security Assessment creation | ✅ | ✅ |
| `/devtrail-mcard` | Model/System Card creation | ✅ | ✅ |

### Usage Examples

```bash
# Check documentation status
/devtrail-status

# Create documentation (agent suggests type)
/devtrail-new

# Force specific document type
/devtrail-new ailog

# Direct shortcuts
/devtrail-ailog
/devtrail-aidec
/devtrail-adr
```

### CLI Commands (Manual Use)

For users who prefer the command line or use agents without skill support:

```bash
# Interactive document creation
devtrail new

# Create specific type directly
devtrail new --doc-type ailog

# Check documentation status
devtrail status
```

### Agent Reporting

AI agents report documentation status at the end of each task:

| Status | Meaning |
|--------|---------|
| `DevTrail: Created AILOG-...` | Documentation was created |
| `DevTrail: No documentation required` | Change was minor |
| `DevTrail: Documentation pending` | May need manual review |

### Multi-Agent Architecture

DevTrail provides native skill support for multiple AI agents through a layered architecture:

```
your-project/
├── .agent/workflows/       # 🌐 Agnostic (Antigravity, future agents)
│   ├── devtrail-new.md
│   ├── devtrail-status.md
│   └── ...
├── .gemini/skills/         # 🔵 Gemini CLI (Google)
│   ├── devtrail-new/SKILL.md
│   └── ...
└── .claude/skills/         # 🟣 Claude Code (Anthropic)
    ├── devtrail-new/SKILL.md
    └── ...
```

| Directory | Agent | Product | Format |
|-----------|-------|---------|--------|
| `.agent/workflows/` | Antigravity, generic | VS Code/Cursor extensions | `skill-name.md` with YAML frontmatter |
| `.gemini/skills/` | Gemini CLI | Google's terminal CLI | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Anthropic's coding agent | `skill-name/SKILL.md` |

> **Note**: `.agent/` is the **vendor-agnostic** standard. Agent-specific directories (`.gemini/`, `.claude/`) provide compatibility for those platforms while following their native conventions.

All skill implementations are **functionally identical**—only the format differs to match each agent's requirements.

---

## Supported Platforms

### AI Coding Assistants

| Platform | Config File | Status |
|----------|-------------|--------|
| Claude Code | `CLAUDE.md` | ✅ Full support |
| Cursor | `.cursorrules` | ✅ Full support |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | ✅ Full support |
| Gemini CLI | `GEMINI.md` | ✅ Full support |

### Operating Systems

| OS | Validation |
|----|------------|
| Linux | `devtrail validate` |
| macOS | `devtrail validate` |
| Windows | `devtrail validate` |

### CI/CD Platforms

| Platform | Support |
|----------|---------|
| GitHub Actions | ✅ Included workflow |
| GitLab CI | 🔧 Adaptable from GitHub Actions |
| Azure DevOps | 🔧 Adaptable from GitHub Actions |

---

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](https://github.com/StrangeDaysTech/devtrail/blob/main/CONTRIBUTING.md) for guidelines.

### Ways to Contribute

- 🐛 Report bugs
- 💡 Suggest features
- 📖 Improve documentation
- 🔧 Submit pull requests
- 🌍 Add translations

---

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/StrangeDaysTech/devtrail/blob/main/LICENSE) file for details.

---

## About Strange Days Tech, S.A.S.

<div align="center">

**[Strange Days Tech](https://strangedays.tech)** builds tools for responsible AI-assisted software development.

Our open-source ecosystem:

| Project | Description |
|---------|-------------|
| **[DevTrail](https://github.com/StrangeDaysTech/devtrail)** | AI governance platform for responsible software development |
| **[arborist-metrics](https://github.com/StrangeDaysTech/arborist)** | Multi-language code complexity analysis library for Rust — [crates.io](https://crates.io/crates/arborist-metrics) |

[Website](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**DevTrail** — AI governance, documented.

[⬆ Back to top](#devtrail)

</div>
