<div align="center">

# DevTrail

**负责任软件开发的 AI 治理平台**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/devtrail-cli.svg)](https://crates.io/crates/devtrail-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](../../../dist/.devtrail/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[快速开始](#快速开始) •
[功能特性](#功能特性) •
[文档](#文档) •
[贡献](#贡献)

**语言**: [English](../../../README.md) | [Español](../es/README.md) | 简体中文

</div>

---

## 问题

随着 AI 日益成为软件开发的核心，组织面临三方面的压力：

- **法规合规**：EU AI Act 将于 2026 年 8 月起强制执行。ISO/IEC 42001 现已成为 AI 治理的国际标准。团队需要有据可查的证据。
- **治理缺口**：没有结构化的方法来证明 AI 决策受到治理、可审计且合规——每一个未记录的 AI 变更都是潜在风险。
- **运营风险**：谁做了这个变更？考虑了哪些替代方案？人工监督是否适当？没有答案，AI 辅助开发就是一个黑箱。

## 解决方案

DevTrail 是一个**符合 ISO 42001 的 AI 治理平台**，确保每一个有意义的变更——无论是人工还是 AI 做出的——都被记录、归属和可审计。

> **"没有记录的痕迹和治理证明，就不应有重大变更。"**

采用 DevTrail 的团队能产出兼容 **ISO/IEC 42001 认证**、**EU AI Act 合规**和 **NIST AI RMF** 风险管理的证据——同时提升开发质量和可追溯性。

---

## 功能特性

### 📋 结构化文档

十二种文档类型，覆盖完整的开发生命周期：

| 类型 | 用途 | 示例 |
|------|------|------|
| **REQ** | 需求 | 系统需求、用户故事 |
| **ADR** | 架构决策 | 技术选型、设计模式 |
| **TES** | 测试计划 | 测试策略、覆盖目标 |
| **INC** | 事件复盘 | 根因分析、经验教训 |
| **TDE** | 技术债务 | 已识别的债务、修复计划 |
| **AILOG** | AI 操作日志 | AI 助手做了什么以及为什么 |
| **AIDEC** | AI 决策 | AI 做出的选择及替代方案 |
| **ETH** | 伦理审查 | 隐私、偏见、负责任的 AI |
| **SEC** | 安全评估 | 威胁建模、漏洞分析 |
| **MCARD** | 模型/系统卡片 | AI 模型文档 |
| **SBOM** | 软件物料清单 | AI 组件清单 |
| **DPIA** | 数据保护影响评估 | 隐私影响分析 |

### 📐 标准对齐

| 标准 | DevTrail 集成 |
|------|---------------|
| **ISO/IEC 42001:2023** | 核心标准——AI 管理系统治理 |
| **EU AI Act** | 风险分类、事件报告、透明度 |
| **NIST AI RMF / 600-1** | ETH/AILOG 中的 12 个 GenAI 风险类别 |
| **ISO/IEC 25010:2023** | REQ/ADR 中的软件质量模型 |
| **ISO/IEC/IEEE 29148:2018** | REQ 中的需求工程 |
| **ISO/IEC/IEEE 29119-3:2021** | TES 中的测试文档 |
| **GDPR** | ETH/DPIA 中的数据保护 |
| **OpenTelemetry** | 可观测性（可选） |

### 🤖 AI Agent 支持

为主流 AI 编码助手预配置：

- **Claude Code** (Anthropic) → `CLAUDE.md`
- **Cursor** → `.cursorrules`
- **GitHub Copilot CLI** → `.github/copilot-instructions.md`
- **Gemini CLI** (Google) → `GEMINI.md`

每个配置指导 AI：
- 在每个文档中标识自身
- 声明置信度级别
- 在适当时请求人工审查
- 遵循命名规范
- **遵循 Git 分支策略**（不直接提交到 `main`）

### 👁️ 人工监督

内置安全机制确保人类保持控制：

- **自主权级别**：某些文档类型需要人工批准
- **审查触发**：低置信度或高风险 → 强制审查
- **伦理审查**：隐私和偏见问题标记为需人工决策

### ✅ 合规自动化

内置 CLI 治理工具：

- **`devtrail validate`** — 13 条验证规则，确保文档正确性
- **`devtrail compliance`** — 法规合规评分（EU AI Act、ISO 42001、NIST AI RMF）
- **`devtrail metrics`** — 治理 KPI、审查率、风险分布、趋势
- **`devtrail analyze`** — 代码复杂度分析（认知复杂度 + 圈复杂度），由 [arborist-metrics](https://github.com/StrangeDaysTech/arborist) 驱动——我们的开源 Rust 多语言代码度量库
- **`devtrail audit`** — 审计跟踪报告，含时间线、可追溯性映射和 HTML 导出
- **Pre-commit 钩子** + **GitHub Actions** 用于 CI/CD 验证

---

## 快速开始

### 选项 1：CLI（推荐）

**快速安装（预编译二进制文件）：**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

或从源码通过 Cargo 安装：

```bash
cargo install devtrail-cli
```

> **注意：** `devtrail update-cli` 会自动检测你的安装方式。预编译二进制安装会从 GitHub Releases 更新；Cargo 安装会通过 `cargo install` 更新。你可以使用 `--method=github` 或 `--method=cargo` 来覆盖默认行为。

然后在你的项目中初始化：

```bash
cd your-project
devtrail init .
```

CLI 会下载最新的 DevTrail 版本，设置框架，并自动配置你的 AI Agent 指令文件。

### 版本管理

DevTrail 为每个组件使用独立的版本标签：

| 组件 | 标签前缀 | 示例 | 包含内容 |
|------|----------|------|----------|
| Framework | `fw-` | `fw-4.4.1` | 模板（12 种类型）、治理文档、指令 |
| CLI | `cli-` | `cli-3.6.0` | `devtrail` 二进制文件 |

使用 `devtrail status` 或 `devtrail about` 查看已安装的版本。

### CLI 命令

| 命令 | 描述 |
|------|------|
| `devtrail init [path]` | 在项目中初始化 DevTrail |
| `devtrail update` | 更新框架和 CLI |
| `devtrail update-framework` | 仅更新框架 |
| `devtrail update-cli` | 更新 CLI 二进制文件 |
| `devtrail remove [--full]` | 从项目中移除 DevTrail |
| `devtrail status [path]` | 显示安装状态和文档统计 |
| `devtrail repair [path]` | 恢复缺失的目录和框架文件 |
| `devtrail validate [path]` | 验证文档的合规性和正确性（使用 `--include-charters` 同时验证 `docs/charters/`） |
| `devtrail charter <子命令>` | 管理章程：`new`、`list`、`status`（事前声明、事后审计的有界工作单元） |
| `devtrail compliance [path]` | 检查法规合规（EU AI Act、ISO 42001、NIST） |
| `devtrail metrics [path]` | 显示治理指标和文档统计 |
| `devtrail analyze [path]` | 分析代码复杂度（认知复杂度 + 圈复杂度指标） |
| `devtrail audit [path]` | 生成带时间线和可追溯性的审计跟踪报告 |
| `devtrail explore [path]` | 在终端中交互式浏览文档（TUI） |
| `devtrail about` | 显示版本和许可证信息 |

参见 [CLI 参考手册](adopters/CLI-REFERENCE.md) 了解详细用法。

### 选项 2：手动设置

```bash
# 从 GitHub 下载最新的框架发布 ZIP
# 前往 https://github.com/StrangeDaysTech/devtrail/releases
# 下载最新的 fw-* 发布（例如 fw-4.4.1）

# 解压并复制到你的项目
unzip devtrail-fw-*.zip -d your-project/
cd your-project

# 提交
git add .devtrail/ DEVTRAIL.md
git commit -m "chore: adopt DevTrail"
```

**参见 [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) 了解详细说明、迁移策略和团队推广计划。**

---

## 文档

DevTrail 文档按受众组织：

| 路径 | 适用对象 | 从这里开始 |
|------|----------|------------|
| [**采用者**](adopters/) | 在项目中采用 DevTrail 的团队 | [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) |
| [**贡献者**](../../../docs/contributors/) | 为 DevTrail 贡献代码的开发者 | [TRANSLATION-GUIDE.md](../../../docs/contributors/TRANSLATION-GUIDE.md) |

**采用者**：按照[采用指南](adopters/ADOPTION-GUIDE.md)获取分步说明，查看 [CLI 参考手册](adopters/CLI-REFERENCE.md)了解命令详情，阅读[工作流指南](adopters/WORKFLOWS.md)了解日常使用模式。

**贡献者**：参见 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发指南，以及[翻译指南](../../../docs/contributors/TRANSLATION-GUIDE.md)添加新语言。

### 关键参考

| 文档 | 描述 |
|------|------|
| [**快速参考**](../../../dist/.devtrail/QUICK-REFERENCE.md) | 文档类型和命名规范的单页概览 |
| [DEVTRAIL.md](../../../dist/DEVTRAIL.md) | 统一治理规则（唯一事实来源） |
| [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) | 新/现有项目的采用指南 |
| [CLI-REFERENCE.md](adopters/CLI-REFERENCE.md) | 完整的 CLI 命令参考 |
| [WORKFLOWS.md](adopters/WORKFLOWS.md) | 推荐的日常工作流和团队模式 |

### 内部结构

采用后，DevTrail 会在你的项目中创建一个 `.devtrail/` 目录用于开发治理：

```
.devtrail/
├── 00-governance/           # 策略和规则
├── 01-requirements/         # REQ 文档
├── 02-design/decisions/     # ADR 文档
├── 03-implementation/       # 实施指南（含 Git 策略）
├── 04-testing/              # TES 文档
├── 05-operations/incidents/ # INC 文档
├── 06-evolution/technical-debt/ # TDE 文档
├── 07-ai-audit/
│   ├── agent-logs/          # AILOG 文档
│   ├── decisions/           # AIDEC 文档
│   └── ethical-reviews/     # ETH、DPIA 文档
├── 08-security/             # SEC 文档
├── 09-ai-models/            # MCARD 文档
└── templates/               # 文档模板
```

### 命名规范

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

示例：`ADR-2025-01-27-001-use-postgresql-for-persistence.md`

---

## 工作原理

### 1. AI 进行变更

AI 助手在你的代码中工作时自动：

```yaml
# 创建：.devtrail/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implement-auth.md
---
id: AILOG-2025-01-27-001
title: Implement JWT authentication
agent: claude-code-v1.0
confidence: high
risk_level: high
review_required: true
---
```

### 2. 人工审查（需要时）

高风险或低置信度的变更会被标记：

```
📋 AILOG-2025-01-27-001-implement-auth.md
   Agent: claude-code-v1.0
   Confidence: high
   Risk Level: high ⚠️
   Review Required: YES
```

### 3. 决策被保留

在多个替代方案之间做出选择时，决策会被记录：

```yaml
# 创建：.devtrail/07-ai-audit/decisions/AIDEC-2025-01-27-001-auth-strategy.md
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

### 4. 伦理问题被标记

当 AI 遇到伦理考量时：

```yaml
# 创建：.devtrail/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-user-data.md
---
id: ETH-2025-01-27-001
title: User data collection scope
status: draft  # 需要人工批准
review_required: true
concerns:
  - GDPR compliance
  - Data minimization
---
```

---

## 验证

### Pre-commit 钩子

```bash
# 安装 pre-commit 钩子
echo 'devtrail validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### 手动验证

```bash
# 跨平台（任何安装了 devtrail 的操作系统）
devtrail validate
```

### GitHub Actions

包含的工作流（`.github/workflows/docs-validation.yml`）自动验证：
- 文件命名规范
- 必需的元数据字段
- 敏感信息检测
- Markdown 格式
- 内部链接完整性

---

## Skills

DevTrail 包含面向 AI Agent 的 Skills，支持**主动创建文档**。

> **双系统**：DevTrail 使用被动系统（Agent 通过上下文指令自动记录文档）和主动系统（用户调用 Skills 手动创建文档，或在 Agent 遗漏时补充）。

### 可用 Skills

| Skill | 用途 | Claude | Gemini |
|-------|------|--------|--------|
| `/devtrail-status` | 检查文档合规状态 | ✅ | ✅ |
| `/devtrail-new` | 创建任意类型的文档（统一入口） | ✅ | ✅ |
| `/devtrail-ailog` | 快速创建 AILOG | ✅ | ✅ |
| `/devtrail-aidec` | 快速创建 AIDEC | ✅ | ✅ |
| `/devtrail-adr` | 快速创建 ADR | ✅ | ✅ |
| `/devtrail-sec` | 创建安全评估 | ✅ | ✅ |
| `/devtrail-mcard` | 创建模型/系统卡片 | ✅ | ✅ |

### 使用示例

```bash
# 检查文档状态
/devtrail-status

# 创建文档（Agent 建议类型）
/devtrail-new

# 指定文档类型
/devtrail-new ailog

# 快捷方式
/devtrail-ailog
/devtrail-aidec
/devtrail-adr
```

### CLI 命令（手动使用）

对于偏好命令行或使用不支持 Skills 的 Agent 的用户：

```bash
# 交互式创建文档
devtrail new

# 直接创建指定类型
devtrail new --doc-type ailog

# 检查文档状态
devtrail status
```

### Agent 报告

AI Agent 在每个任务结束时报告文档状态：

| 状态 | 含义 |
|------|------|
| `DevTrail: Created AILOG-...` | 文档已创建 |
| `DevTrail: No documentation required` | 变更较小 |
| `DevTrail: Documentation pending` | 可能需要手动审查 |

### 多 Agent 架构

DevTrail 通过分层架构为多个 AI Agent 提供原生 Skill 支持：

```
your-project/
├── .agent/workflows/       # 🌐 通用（Antigravity，未来 Agent）
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

| 目录 | Agent | 产品 | 格式 |
|------|-------|------|------|
| `.agent/workflows/` | Antigravity, 通用 | VS Code/Cursor 扩展 | 带 YAML frontmatter 的 `skill-name.md` |
| `.gemini/skills/` | Gemini CLI | Google 终端 CLI | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Anthropic 编码 Agent | `skill-name/SKILL.md` |

> **注意**：`.agent/` 是**厂商中立**的标准。Agent 特定目录（`.gemini/`、`.claude/`）为这些平台提供兼容性，同时遵循其原生规范。

所有 Skill 实现**功能完全一致**——仅格式不同以匹配各 Agent 的要求。

---

## 支持的平台

### AI 编码助手

| 平台 | 配置文件 | 状态 |
|------|----------|------|
| Claude Code | `CLAUDE.md` | ✅ 完整支持 |
| Cursor | `.cursorrules` | ✅ 完整支持 |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | ✅ 完整支持 |
| Gemini CLI | `GEMINI.md` | ✅ 完整支持 |

### 操作系统

| 操作系统 | 验证方式 |
|----------|----------|
| Linux | `devtrail validate` |
| macOS | `devtrail validate` |
| Windows | `devtrail validate` |

### CI/CD 平台

| 平台 | 支持情况 |
|------|----------|
| GitHub Actions | ✅ 内置工作流 |
| GitLab CI | 🔧 可从 GitHub Actions 适配 |
| Azure DevOps | 🔧 可从 GitHub Actions 适配 |

---

---

## 贡献

欢迎贡献！参见 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

### 贡献方式

- 🐛 报告 Bug
- 💡 建议功能
- 📖 改进文档
- 🔧 提交 Pull Request
- 🌍 添加翻译

---

## 许可证

本项目使用 MIT 许可证——详情参见 [LICENSE](../../../LICENSE) 文件。

---

## 关于 Strange Days Tech, S.A.S.

<div align="center">

**[Strange Days Tech](https://strangedays.tech)** 构建负责任 AI 辅助软件开发的工具。

我们的开源生态系统：

| 项目 | 描述 |
|------|------|
| **[DevTrail](https://github.com/StrangeDaysTech/devtrail)** | 负责任软件开发的 AI 治理平台 |
| **[arborist-metrics](https://github.com/StrangeDaysTech/arborist)** | Rust 多语言代码复杂度分析库 — [crates.io](https://crates.io/crates/arborist-metrics) |

[网站](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**DevTrail** — AI 治理，有据可查。

[⬆ 回到顶部](#devtrail)

</div>
