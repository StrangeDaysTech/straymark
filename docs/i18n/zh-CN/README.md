<div align="center">

<img src="../../../assets/straymark-symbol.svg" alt="StrayMark — Strange Days Tech 出品" width="240" />

# StrayMark

**你的 AI 辅助项目所需的认知纪律**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/straymark-cli.svg)](https://crates.io/crates/straymark-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![CLA assistant](https://cla-assistant.io/readme/badge/StrangeDaysTech/straymark)](https://cla-assistant.io/StrangeDaysTech/straymark)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](../../../dist/.straymark/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[快速开始](#快速开始) •
[为什么叫 StrayMark？](#为什么叫-straymark) •
[适用人群](#straymark-的适用人群) •
[设计原则](#设计原则) •
[功能特性](#功能特性) •
[合规性](#合规性) •
[文档](#文档)

**语言**: [English](../../../README.md) | [Español](../es/README.md) | 简体中文

</div>

---

## 问题

AI Agent 写代码很快。但写出的代码并不连贯。经过足够多的轮次后，Agent 会失去主线：重新引入团队已经否决的模式、积累隐藏的技术债务、产出可以编译但与系统肌理不匹配的工作。Agent 越快，这些债务越难被察觉——直到一次回归、一次事故或一次重构把它们暴露出来。

编排这些 Agent 的资深工程师并不需要*更多*的 Agent 自主权。他们需要的恰恰相反：一种以 Agent 可被约束的节奏将范围、决策和风险显性外化的方式——让 Agent 在约束之下执行，而不是自己发明约束。

## 解决方案

StrayMark 是一个**框架 + CLI**，将资深软件工程工作的认知纪律——明确的范围、声明的决策、命名的风险、记录的备选方案、可审计的轨迹——外化为与代码并存的版本化文件。

> **"没有记录痕迹的重大变更不应发生——并且 Agent 的决策空间应受约束。"**

作为副作用，这一纪律会产出与 **ISO/IEC 42001**、**EU AI Act**、**NIST AI RMF** 以及（按选项启用的）中国 AI/数据法规栈兼容的证据。但目标首先是工程质量；当纪律真正落地时，合规便是自然而然的副产品。

---

## 为什么叫 StrayMark？

代码只是一场心智之战留下的化石痕迹。真正的工程发生在决策的混乱中、在权衡过的风险里、在那些你选择不去走的路上。传统上，所有这些人类留下的痕迹都被当作项目历史中的 stray marks（意外涂痕）而被丢弃。在 Strange Days Tech，我们相信这些痕迹才是信号，而不是噪音。当企业级合规软件迫使你事后回溯、填写空洞的表格时，StrayMark 让你在当下就划下你的领地。

对我们而言，监管合规（ISO 42001、EU AI Act、NIST）不是终极目的，而是经线。我们把这些标准当作结构性的织物，用以构建多维度的技术记忆。这不是为了向审计员打勾；而是用这些线编织出一个能够支撑设计与实现过程中复杂认知活动的脚手架。StrayMark 把官僚程序转化为一张战术网格，帮助你以代码本身无法企及的分辨率去思考、导航与构建。

在这个自主 Agent 的时代，StrayMark 不是用来强化「替代人类操作者」的机制。恰恰相反：它是一项工具，用以人化 AI Agent 所运作的高速、极度复杂的流程。当机器以淹没直觉的节奏运行时，StrayMark 充当人类意图的锚点，迫使 Agent 在一个受限、可读、可审计的决策空间中运作。我们不追求把工程自动化到湮灭的程度，而是为 AI 的速度赋予一种深刻人性的意识与结构。

**捕捉噪音。编织信号。人化机器。**

---

## StrayMark 的适用人群

StrayMark 的主要用户是**在非平凡系统上编排 AI Agent 的资深工程师**——具备扎实技术判断力、借助 Agent 完成自己单独无法实际完成的工作、并需要外化的认知纪律以防止 Agent 引入系统性混乱的人。

如果你符合这个画像，StrayMark 的流程、默认值和语言都是为你量身打造的。

StrayMark 还服务于三类次要受众，建立在主用户基础之上——绝不以牺牲主用户为代价：

- **技术负责人和架构师**：标准化团队与 AI 助手的协作方式。
- **合规官和审计师**：需要受治理 AI 开发的证据（ISO 42001、EU AI Act、NIST AI RMF、PIPL、TC260……）。
- **受监管环境的采用者**（金融、医疗、公共部门、中国）：将可追溯性内建到工作流，而非事后补建。

StrayMark **并不**试图成为：LLM 网关、模型评估器、"代码 10 倍快"的生产力外壳，或工程判断的替代品。详见下文 [诚实的边界](#诚实的边界)。

---

## 设计原则

StrayMark 的产品决策基于十二条明确的原则。它们按层级排序：当两条原则发生冲突时，靠前的胜出。

1. **工具服务于手艺，而非产品自身。** 衡量标准是工程师是否能产出令自己自豪的工作——而不是采用率、留存或营收。
2. **主要用户是编排 Agent 的资深工程师。** 不是 VP，不是 CISO，也不是合规官。
3. **核心严格开源，毫无附加条件。** Framework、CLI 和 TUI 均为 MIT 许可，没有为推动付费而被阉割的功能。
4. **法规合规是副作用，而非产品目标。** ISO 42001、EU AI Act、NIST AI RMF 是有用的框架；它们不是目标。
5. **Schema 驱动优先于功能驱动。** 核心实体（Stage Closure Bundle、Charter、Document）首先以版本化 schema 定义，然后再构建功能。
6. **认知纪律优先于原始生产力。** StrayMark 对抗的是 AI 快速产出代码在严肃项目中带来的混乱——而不是速度本身。
7. **Local-first，Cloud 作为放大器。** CLI 完全离线可用。Cloud 可以增加价值（跨仓聚合、签名证据），但绝不会成为核心的门槛。
8. **项目记忆存活于仓库中，而非外部数据库。** AILOG、ADR、AIDEC、Charter 和 Bundle 是与代码并存的版本化文件，使用 markdown + JSON Schema。
9. **简洁优先于能力。** 当两种设计满足同一目标时，更简单的胜出。模式在真实项目中得到验证后才结晶化，不在之前。
10. **诚实地说明工具不做什么。** 不评估模型、不做 LLM 网关、不自动认证合规、不替代工程判断。
11. **社区维护工具，而非反过来。** 贡献和反馈被认真对待，但不会变成民主决策。
12. **产品的速度等于学习的速度。** 不过早结晶化；schema 标记为 `v0`，直到在第二个领域得到验证。

完整文档（包含来自验证周期的实证注解）见 [`docs/i18n/zh-CN/contributors/DESIGN-PRINCIPLES.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/zh-CN/contributors/DESIGN-PRINCIPLES.md)。

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

### 🤖 AI Agent 支持

为主流 AI 编码助手预配置：

- **通用（AGENTS.md 标准）** → `AGENTS.md` — 被 Claude Code、OpenAI Codex CLI、Cursor、Aider、Devin、Sourcegraph Amp、Google Jules、Zed AI、Continue、Roo Code、Factory Droids、GitHub Copilot、Gemini CLI、Windsurf、Amazon Q 等读取
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

### ✅ CLI 工具集

将纪律转化为可执行反馈的内置命令：

- **`straymark charter <new|list|status|close|drift|batch-complete|audit|refresh-suggest|amend>`** — 事前声明、事后审计的有界工作单元。`close` 记录执行后遥测；`drift` 以 AILOG-aware 抑制方式检测文件-与-commit 的偏差，并（cli-3.13.0+）对 AILOG `## Batch Ledger` 中的 `### Batch N (pending)` 条目进行门控；`batch-complete`（cli-3.13.0+）将批次标记为已在多批次章程（3+ 批次或 >1 天）的台账中完成；`audit` 编排多模型外部审查（三步骤 prepare/calibrate/finalize，仅编排——不调用 LLM API）；`refresh-suggest`（cli-3.14.0+）当多 Charter 模块的 `r_n_plus_one_emergent_count` 滚动均值超过阈值时，输出预声明 SpecKit 刷新的启发式建议；`amend`（cli-3.14.0+）为关闭后审计驱动的 Batch N.4 修订做脚手架，在原 execute 分支上落地而不开启新 Charter。对于 IDE 驱动的工作流，内联 skills `/straymark-audit-prompt` 和 `/straymark-audit-review` 封装 CLI，在对话中展示 prompts 并将 findings 合并到遥测中。
- **`straymark followups <list|status|drift|promote>`** *(cli-3.19.0+)* — 一等的后续事项积压注册表（`.straymark/follow-ups-backlog.md`，schema v1 实验性）：`drift --apply` 按 AILOG 从 AILOG 中提取 `§Follow-ups` / `R<N> (new)` 条目（带闭合标记的 bullet 落为 `suspected-closed`），计数器由 CLI 拥有并在每次写入时重新计算，`promote` 将条目提升为带完整溯源的 TDE 文档。详见 `STRAYMARK.md §16` 与 `FOLLOW-UPS-BACKLOG-PATTERN.md`。
- **`straymark approve <doc-id>`** — 记录一次正式的人工审批（一次性写入 `reviewed_by` / `reviewed_at` / `review_outcome` 与 `## Approval` body 章节；闭合 DOCUMENTATION-POLICY §3.5 中规范化的缺口）
- **`straymark validate`** — 25+ 条文档正确性验证规则（其中 12 条针对中国法规、按 scope 启用）；`--include-charters` 可同时检查 `.straymark/charters/`；`--check-pending-reviews` 列出审批积压（仅警告）
- **`straymark metrics`** — 治理 KPI、审查率、风险分布、趋势
- **`straymark analyze`** — 代码复杂度分析（认知复杂度 + 圈复杂度），由 [arborist-metrics](https://github.com/StrangeDaysTech/arborist-metrics/) 驱动——我们开源的 Rust 多语言代码度量库，同样由 StrangeDaysTech S.A.S. de C.V. 开发
- **`straymark audit`** — 审计跟踪报告，含时间线、可追溯性映射和 HTML 导出
- **`straymark compliance`** — 作为已记录工作的副作用产出法规合规评分（EU AI Act、ISO 42001、NIST AI RMF；六项中国框架通过 `--region china` 按选项启用）
- **`straymark explore`** — 用于浏览项目文档图谱的交互式 TUI，包含章程视图（生命周期状态、来源 AILOG/spec、文件位置）
- **Pre-commit 钩子** + **GitHub Actions** 用于 CI/CD 验证

---

## 诚实的边界

StrayMark **不会**：

- 评估、对比或排名 LLM；
- 充当 LLM 网关或路由层；
- 防止幻觉或保证 Agent 正确性；
- 自动颁发法规合规认证——它产出证据，而不是认证；
- 替代资深工程师的判断。

如果你的问题属于以上任一类，StrayMark 不是合适的工具。

---

## 合规性

StrayMark 外化的纪律——明确的范围、声明的决策、命名的风险、记录的备选方案——作为副作用，会产出与主流 AI 治理框架清晰映射的证据。因此，合规性被定位为*认真做工程工作的结果*，而非产品本身（原则 #4）。

### 标准对齐

| 标准 | StrayMark 集成 |
|------|---------------|
| **ISO/IEC 42001:2023** | 核心标准——AI 管理系统治理 |
| **EU AI Act** | 风险分类、事件报告、透明度 |
| **NIST AI RMF / 600-1** | ETH/AILOG 中的 12 个 GenAI 风险类别 |
| **ISO/IEC 25010:2023** | REQ/ADR 中的软件质量模型 |
| **ISO/IEC/IEEE 29148:2018** | REQ 中的需求工程 |
| **ISO/IEC/IEEE 29119-3:2021** | TES 中的测试文档 |
| **GDPR** | ETH/DPIA 中的数据保护 |
| **OpenTelemetry** | 可观测性（可选） |

### 中国法规支持

StrayMark 现在以 **opt-in**（自愿启用）的方式覆盖六项中国 AI / 数据法规：**TC260《人工智能安全治理框架 v2.0》**（五级风险分级）、**《个人信息保护法》(PIPL)** 及其配套的 **PIPIA**（个人信息保护影响评估，留存 ≥ 3 年）、**强制性国家标准 GB 45438-2025**《网络安全技术 人工智能生成合成内容标识方法》（显式 + 隐式标识）、**CAC 算法备案**（包括省级 + 国家级双重备案）、**GB/T 45652-2025** 预训练与微调数据安全，以及自 2026-01-01 生效的 **《网络安全法》修订** 与《国家网络安全事件报告管理办法》（1 小时 / 4 小时 + 72 小时评估 + 30 天事后审查的报告窗口）。

#### 启用方式

在 `.straymark/config.yml` 中添加：

```yaml
regional_scope:
  - global   # NIST + ISO 42001（始终可用）
  - eu       # EU AI Act + GDPR
  - china    # 启用上述六项中国法规
```

#### 启用后获得

- **4 个中国专属文档类型**：`PIPIA`、`CACFILE`、`TC260RA`、`AILABEL`（均经 `straymark new` 生成，模板已翻译为中文，位于 `.straymark/templates/i18n/zh-CN/`）。
- **6 个合规检查器**：通过 `straymark compliance --region china` 一次性运行，或单独运行 `--standard china-tc260 | china-pipl | china-gb45438 | china-cac | china-gb45652 | china-csl`。
- **12 条新的验证规则**（`CROSS-004…011`、`TYPE-003…006`）：自动校验跨文档引用一致性，例如：`cac_filing_required: true` 必须关联 CACFILE；`csl_severity_level: particularly_serious` 必须配合 `csl_report_deadline_hours: 1`；PIPIA 的 `pipl_retention_until` 必须至少为 `created` + 3 年。
- **5 份中文治理指南**，位于 `.straymark/00-governance/i18n/zh-CN/`：`CHINA-REGULATORY-FRAMEWORK.md`、`TC260-IMPLEMENTATION-GUIDE.md`、`PIPL-PIPIA-GUIDE.md`、`CAC-FILING-GUIDE.md`、`GB-45438-LABELING-GUIDE.md`。

#### 适用人群

- 在中国大陆运营 AI 服务的团队，需办理 CAC 算法备案或对外提供生成式 AI。
- 处理中国大陆个人信息（尤其是敏感个人信息）、需进行 PIPIA 的处理者。
- 涉及跨境数据传输，须依据 PIPL 第 38-40 条选择安全评估、认证或标准合同机制的组织。
- 采用 ISO/IEC 42001 全球治理框架并希望在中国境内补充本地合规证据的企业。

不在 `regional_scope` 中包含 `china` 的项目完全不受影响——这是完全向后兼容的扩展。

---

## 快速开始

### 选项 1：CLI（推荐）

**快速安装（预编译二进制文件）：**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

或从源码通过 Cargo 安装：

```bash
cargo install straymark-cli
```

> **注意：** `straymark update-cli` 会自动检测你的安装方式。预编译二进制安装会从 GitHub Releases 更新；Cargo 安装会通过 `cargo install` 更新。你可以使用 `--method=github` 或 `--method=cargo` 来覆盖默认行为。

然后在你的项目中初始化：

```bash
cd your-project
straymark init .
```

CLI 会下载最新的 StrayMark 版本，设置框架，并自动配置你的 AI Agent 指令文件。

### 版本管理

StrayMark 为每个组件使用独立的版本标签：

| 组件 | 标签前缀 | 示例 | 包含内容 |
|------|----------|------|----------|
| Framework | `fw-` | `fw-4.22.0` | 模板（12 种类型）、治理文档、指令、Charter 模板 + schema |
| CLI | `cli-` | `cli-3.19.1` | `straymark` 二进制文件 |

使用 `straymark status` 或 `straymark about` 查看已安装的版本。

### CLI 命令

| 命令 | 描述 |
|------|------|
| `straymark init [path]` | 在项目中初始化 StrayMark |
| `straymark update` | 更新框架和 CLI |
| `straymark update-framework` | 仅更新框架 |
| `straymark update-cli` | 更新 CLI 二进制文件 |
| `straymark remove [--full]` | 从项目中移除 StrayMark |
| `straymark status [path]` | 显示安装状态和文档统计 |
| `straymark repair [path]` | 恢复缺失的目录和框架文件 |
| `straymark validate [path]` | 验证文档的合规性和正确性（`--include-charters` 同时校验 Charter；`--check-pending-reviews` 列出审批积压） |
| `straymark charter <子命令>` | 管理章程：`new`、`list`、`status`、`close`（记录遥测）、`drift`（带 AILOG-awareness 的偏差检测） |
| `straymark followups <子命令>` *(cli-3.19.0+)* | 管理后续事项积压注册表：`list`（可过滤的条目）、`status`（带 CLI 拥有、即时重新计算的计数器的概览）、`drift`（检测/提取未处理的 AILOG —— v0 bash 脚本的原生替代，带抗噪声的 `suspected-closed` 提取）、`promote`（FU → TDE，带 `promoted_from_followup` 溯源） |
| `straymark approve <doc-id>` | 在 `review_required: true` 的文档上记录一次正式的人工审批（frontmatter + 规范的 body 章节） |
| `straymark compliance [path]` | 检查法规合规（EU AI Act、ISO 42001、NIST） |
| `straymark metrics [path]` | 显示治理指标和文档统计 |
| `straymark analyze [path]` | 分析代码复杂度（认知复杂度 + 圈复杂度指标） |
| `straymark audit [path]` | 生成带时间线和可追溯性的审计跟踪报告 |
| `straymark explore [path]` | 在终端中交互式浏览文档（TUI） |
| `straymark about` | 显示版本和许可证信息 |

参见 [CLI 参考手册](adopters/CLI-REFERENCE.md) 了解详细用法。

### 选项 2：手动设置

```bash
# 从 GitHub 下载最新的框架发布 ZIP
# 前往 https://github.com/StrangeDaysTech/straymark/releases
# 下载最新的 fw-* 发布（例如 fw-4.19.0）

# 解压并复制到你的项目
unzip straymark-fw-*.zip -d your-project/
cd your-project

# 提交
git add .straymark/ STRAYMARK.md
git commit -m "chore: adopt StrayMark"
```

**参见 [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) 了解详细说明、迁移策略和团队推广计划。**

---

## 文档

StrayMark 文档按受众组织：

| 路径 | 适用对象 | 从这里开始 |
|------|----------|------------|
| [**采用者**](adopters/) | 在项目中采用 StrayMark 的团队 | [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) |
| [**贡献者**](../../../docs/contributors/) | 为 StrayMark 贡献代码的开发者 | [TRANSLATION-GUIDE.md](../../../docs/contributors/TRANSLATION-GUIDE.md) |

**采用者**：按照[采用指南](adopters/ADOPTION-GUIDE.md)获取分步说明，查看 [CLI 参考手册](adopters/CLI-REFERENCE.md)了解命令详情，阅读[工作流指南](adopters/WORKFLOWS.md)了解日常使用模式。

**贡献者**：参见 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发指南，以及[翻译指南](../../../docs/contributors/TRANSLATION-GUIDE.md)添加新语言。

### 关键参考

| 文档 | 描述 |
|------|------|
| [**快速参考**](../../../dist/.straymark/QUICK-REFERENCE.md) | 文档类型和命名规范的单页概览 |
| [STRAYMARK.md](../../../dist/STRAYMARK.md) | 统一治理规则（唯一事实来源） |
| [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) | 新/现有项目的采用指南 |
| [CLI-REFERENCE.md](adopters/CLI-REFERENCE.md) | 完整的 CLI 命令参考 |
| [WORKFLOWS.md](adopters/WORKFLOWS.md) | 推荐的日常工作流和团队模式 |

### 内部结构

采用后，StrayMark 会在你的项目中创建一个 `.straymark/` 目录用于开发治理：

```
.straymark/
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
# 创建：.straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implement-auth.md
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
# 创建：.straymark/07-ai-audit/decisions/AIDEC-2025-01-27-001-auth-strategy.md
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
# 创建：.straymark/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-user-data.md
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
echo 'straymark validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### 手动验证

```bash
# 跨平台（任何安装了 straymark 的操作系统）
straymark validate
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

StrayMark 包含面向 AI Agent 的 Skills，支持**主动创建文档**。

> **双系统**：StrayMark 使用被动系统（Agent 通过上下文指令自动记录文档）和主动系统（用户调用 Skills 手动创建文档，或在 Agent 遗漏时补充）。

### 可用 Skills

| Skill | 用途 | Claude | Gemini | Codex |
|-------|------|--------|--------|-------|
| `/straymark-status` | 检查文档合规状态 | ✅ | ✅ | ✅ |
| `/straymark-new` | 创建任意类型的文档（统一入口） | ✅ | ✅ | ✅ |
| `/straymark-ailog` | 快速创建 AILOG | ✅ | ✅ | ✅ |
| `/straymark-aidec` | 快速创建 AIDEC | ✅ | ✅ | ✅ |
| `/straymark-adr` | 快速创建 ADR | ✅ | ✅ | ✅ |
| `/straymark-sec` | 创建安全评估 | ✅ | ✅ | ✅ |
| `/straymark-mcard` | 创建模型/系统卡片 | ✅ | ✅ | ✅ |

> **Codex CLI 用户** *(fw-4.19.0+)*：Codex 从 `~/.codex/skills/`（用户级）读取 skills，而不是从项目树读取。`straymark init` 之后（以及后续每次 `straymark update` 之后），运行一次 `straymark install-skills --agent codex` 从项目的 `.codex/skills/` 填充该目录。

### 使用示例

```bash
# 检查文档状态
/straymark-status

# 创建文档（Agent 建议类型）
/straymark-new

# 指定文档类型
/straymark-new ailog

# 快捷方式
/straymark-ailog
/straymark-aidec
/straymark-adr
```

### CLI 命令（手动使用）

对于偏好命令行或使用不支持 Skills 的 Agent 的用户：

```bash
# 交互式创建文档
straymark new

# 直接创建指定类型
straymark new --doc-type ailog

# 检查文档状态
straymark status
```

### Agent 报告

AI Agent 在每个任务结束时报告文档状态：

| 状态 | 含义 |
|------|------|
| `StrayMark: Created AILOG-...` | 文档已创建 |
| `StrayMark: No documentation required` | 变更较小 |
| `StrayMark: Documentation pending` | 可能需要手动审查 |

### 多 Agent 架构

StrayMark 通过分层架构为多个 AI Agent 提供原生 Skill 支持：

```
your-project/
├── .agent/workflows/       # 🌐 通用（Antigravity，未来 Agent）
│   ├── straymark-new.md
│   ├── straymark-status.md
│   └── ...
├── .gemini/skills/         # 🔵 Gemini CLI (Google)
│   ├── straymark-new/SKILL.md
│   └── ...
├── .claude/skills/         # 🟣 Claude Code (Anthropic)
│   ├── straymark-new/SKILL.md
│   └── ...
└── .codex/skills/          # 🟢 Codex CLI (OpenAI) — 安装到 ~/.codex/skills/
    ├── straymark-new/SKILL.md
    └── ...
```

| 目录 | Agent | 产品 | 格式 |
|------|-------|------|------|
| `.agent/workflows/` | Antigravity, 通用 | VS Code/Cursor 扩展 | 带 YAML frontmatter 的 `skill-name.md` |
| `.gemini/skills/` | Gemini CLI | Google 终端 CLI | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Anthropic 编码 Agent | `skill-name/SKILL.md` |
| `.codex/skills/` *(fw-4.19.0+)* | Codex CLI | OpenAI 编码 Agent | `skill-name/SKILL.md`（最小 frontmatter）— 通过 `straymark install-skills --agent codex` 安装到 `~/.codex/skills/` |

> **注意**：`.agent/` 是**厂商中立**的标准。Agent 特定目录（`.gemini/`、`.claude/`）为这些平台提供兼容性，同时遵循其原生规范。

所有 Skill 实现**功能完全一致**——仅格式不同以匹配各 Agent 的要求。

---

## 支持的平台

### AI 编码助手

| 平台 | 配置文件 | 状态 |
|------|----------|------|
| 通用（AGENTS.md 标准） | `AGENTS.md` | ✅ 完整支持 |
| Claude Code | `CLAUDE.md` | ✅ 完整支持 |
| Cursor | `.cursorrules` | ✅ 完整支持 |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | ✅ 完整支持 |
| Gemini CLI | `GEMINI.md` | ✅ 完整支持 |
| Codex CLI (OpenAI) *(fw-4.19.0+)* | `AGENTS.md` + `~/.codex/skills/` | ✅ 完整支持（运行 `straymark install-skills --agent codex`） |

### 操作系统

| 操作系统 | 验证方式 |
|----------|----------|
| Linux | `straymark validate` |
| macOS | `straymark validate` |
| Windows | `straymark validate` |

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
| **[StrayMark](https://github.com/StrangeDaysTech/straymark)** | 你的 AI 辅助项目所需的认知纪律 |
| **[arborist-metrics](https://github.com/StrangeDaysTech/arborist-metrics/)** | Rust 多语言代码复杂度分析库 — [crates.io](https://crates.io/crates/arborist-metrics) |

[网站](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**StrayMark** — 工程纪律，外化于代码。合规，作为副作用。

[⬆ 回到顶部](#straymark)

</div>
