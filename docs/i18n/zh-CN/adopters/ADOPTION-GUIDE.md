# StrayMark - 采用指南

**在新项目或现有项目中采用 StrayMark 的完整指南。**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**语言**: [English](../../../adopters/ADOPTION-GUIDE.md) | [Español](../../es/adopters/ADOPTION-GUIDE.md) | 简体中文

---

## 目录

1. [什么是 StrayMark？](#什么是-straymark)
2. [适用对象](#适用对象)
3. [收益](#收益)
4. [标准合规](#标准合规)
5. [采用路径 A：新项目](#采用路径-a新项目)
6. [采用路径 B：现有项目](#采用路径-b现有项目)
7. [配置](#配置)
8. [验证](#验证)
9. [常见问题](#常见问题)

---

## 什么是 StrayMark？

StrayMark 是一个**框架 + CLI**，将资深软件工程工作的认知纪律——明确的范围、声明的决策、命名的风险、记录的备选方案、可审计的轨迹——外化为与代码并存的版本化文件。其意图在于约束 Agent 的决策空间，使 AI 辅助的工作在多轮交互中保持连贯，而不是漂移成隐藏的技术债务。

它提供：

- **12 种结构化文档类型**，覆盖完整的开发和 AI 生命周期
- **Charter** 作为 Agent 执行的单位——事前声明、事后审计的有界工作
- **AI Agent 问责制**，通过强制身份标识、置信度跟踪和自主权限制
- **人工监督**，通过对关键和高风险变更的必需审查工作流
- **可追溯性**，连接需求 → 设计 → 实现 → 测试 → 事件
- **法规证据** — 作为副作用，所产出的文物清晰映射到 EU AI Act、ISO 42001、NIST AI RMF 以及（按选项启用的）PIPL/TC260/GB-45438/CAC

### 核心原则

> **"没有记录痕迹的重大变更不应发生——并且 Agent 的决策空间应受约束。"**

StrayMark 确保每一个有意义的变更——无论是人工还是 AI 做出的——都被记录、归属和可审计。这一纪律产出兼容 **ISO/IEC 42001**、**EU AI Act** 和 **NIST AI RMF** 的证据——但目标首先是工程质量；当纪律真正落地时，合规便是自然而然的副产品（参见 [`Propuesta/straymark-design-principles.md`](https://github.com/StrangeDaysTech/straymark/blob/main/Propuesta/straymark-design-principles.md) 中的原则 #4）。

### 为什么是现在？

AI Agent 写代码很快。但写出的代码并不连贯。经过足够多的轮次后，Agent 会失去主线：重新引入团队已经否决的模式、积累隐藏的技术债务、产出可以编译但与系统肌理不匹配的工作。Agent 越快，这些债务越难被察觉——直到一次回归、一次事故或一次重构把它们暴露出来。

与此并行，法规底线也在抬升——**EU AI Act 将于 2026 年 8 月起强制执行**，**ISO/IEC 42001** 现已成为 AI 管理系统的国际标准。采用 StrayMark 首先解决的是工程问题；法规证据作为这种纪律的副产品被产出，在审计师询问时即可呈现。

### StrayMark 不是什么

- 它不是文档生成器——它提供结构、模板和治理规则
- 它不是代码注释或 API 文档的替代品
- 它不是项目管理工具或版本控制系统
- 它不是 LLM 网关、模型评估器或幻觉过滤器
- 它不是完整的 ISO 42001 认证——它在其范围内产出兼容的证据，而不是认证本身
- 它不是资深工程师判断的替代品

---

## 适用对象

### 主要用户

StrayMark 的主要用户是**在非平凡系统上编排 AI Agent 的资深工程师**——具备扎实技术判断力、借助 Agent 完成自己单独无法实际完成的工作、并需要外化的认知纪律以防止 Agent 引入系统性混乱的人。框架的流程、默认值和语言都是为这个角色量身打造的；次要受众建立在主用户基础之上，绝不以牺牲主用户为代价。

### 目标用户

| 用户类型 | 采用驱动因素 |
|----------|--------------|
| **编排 AI Agent 的资深工程师** | 外化范围/决策/风险，使 Agent 在多轮交互中保持在轨 |
| **技术负责人和架构师** | 标准化团队与 AI 助手的协作；保留决策的组织记忆 |
| **从事严肃项目的独立开发者** | 以结构化方式跟踪决策和 AI 辅助变更，不必每次都重新发明结构 |
| **开源维护者** | 以透明、可审查的方式记录贡献决策 |
| **受监管环境的团队**（金融、医疗、欧盟、在中国运营） | 将可追溯性内建到工作流，而非为审计而事后重建 |
| **追求 ISO 42001 / EU AI Act 就绪的组织** | 作为正常工程工作的副作用产出可认证证据 |

### 兼容的开发环境

StrayMark 提供以下平台的配置文件：

| 平台 | 配置文件 | 状态 |
|------|----------|------|
| **Claude Code** (Anthropic) | `CLAUDE.md` | ✅ 已支持 |
| **Cursor** | `.cursorrules` | ✅ 已支持 |
| **GitHub Copilot CLI** | `.github/copilot-instructions.md` | ✅ 已支持 |
| **Gemini CLI** (Google) | `GEMINI.md` | ✅ 已支持 |
| **其他 AI 工具** | 从任意配置文件复制规则 | ✅ 可适配 |

### 兼容的方法论

StrayMark 适用于任何开发方法论：

| 方法论 | StrayMark 如何融入 |
|--------|-------------------|
| **Agile/Scrum** | REQ 文档映射到用户故事；ADR 记录 Sprint 决策 |
| **瀑布模型** | 从需求到实现的完整可追溯性 |
| **DevOps/SRE** | INC 文档用于事后复盘；TDE 用于技术债务跟踪 |
| **Domain-Driven Design** | ADR 记录限界上下文决策 |
| **Test-Driven Development** | TES 文档记录测试策略 |

---

## 收益

### 工程纪律（主要意图）

| 收益 | 描述 |
|------|------|
| **约束 Agent 的决策空间** | Charter 在事前声明范围/决策/风险，使 Agent 在约束之下执行而非自行发明 |
| **跨多轮交互保持连贯** | 项目记忆存活于版本化文件中；Agent（以及下一位维护者）无须依赖单次聊天会话即可重建上下文 |
| **组织记忆** | 决策在团队成员变动、Agent 轮换、工具栈更迭中依然存续 |
| **加速新人上手** | 新成员（无论是人还是 Agent）通过 ADR 和 AIDEC 理解*为什么*，而不只是通过代码看到*是什么* |
| **减少返工** | 当范围在数月后再次打开时，保留的上下文防止重复犯错 |
| **清晰的问责** | 知道谁（或哪个 Agent）以何种置信度做了每个变更 |

### AI 辅助开发

| 收益 | 描述 |
|------|------|
| **AI 透明度** | 每个 AI 操作都带有置信度级别记录 |
| **人工监督** | 关键决策需要人工批准 |
| **伦理保障** | ETH 和 DPIA 文档将隐患标记给人类做决定 |
| **治理指标** | `straymark metrics` 跟踪审查率、风险分布和趋势 |

### 法规合规（作为副作用）

| 收益 | 描述 |
|------|------|
| **EU AI Act 就绪** | 内置风险分类、事件报告和透明度模板 |
| **ISO 42001 兼容** | 文档结构符合认证审计要求 |
| **NIST AI RMF 映射** | 明确覆盖 12 个 GenAI 风险类别和治理功能 |
| **完整的审计跟踪** | `straymark audit` 生成可导出的时间线和可追溯性报告 |
| **合规评分** | `straymark compliance` 提供基于百分比的法规差距分析 |

---

## 标准合规

StrayMark 外化的纪律——明确的范围、声明的决策、命名的风险、记录的备选方案——作为副作用，会产出与主流工程及 AI 治理框架清晰映射的证据。下表描述了 StrayMark 的文物*如何*与各项标准对齐；目标是工程工作，对齐则是其副产品：

### 软件工程标准

| 标准 | StrayMark 如何帮助 |
|------|-------------------|
| **ISO/IEC/IEEE 29148:2018** | REQ 文档遵循结构化需求格式，包含外部接口和可追溯性 |
| **ISO/IEC 25010:2023** | 在 ADR 和 REQ 非功能性需求中评估 9 个质量特性 |
| **ISO/IEC/IEEE 29119-3:2021** | TES 文档遵循测试文档层次结构（策略 → 策略 → 计划） |
| **ISO/IEC 12207** | 生命周期文档覆盖 |

### AI 管理与治理

| 标准 | StrayMark 如何帮助 |
|------|-------------------|
| **ISO/IEC 42001:2023** | 核心标准 — AI-GOVERNANCE-POLICY.md 将所有附录 A 控制项映射到 StrayMark 文档 |
| **EU AI Act** | ETH 中的风险分类、INC 中的事件报告时间线、AILOG 中的法规字段 |
| **NIST AI RMF 1.0 / 600-1** | ETH/AILOG 中的 12 个 GenAI 风险类别，MAP/MEASURE/MANAGE/GOVERN 覆盖 |
| **ISO/IEC 23894:2023** | AI 风险管理与 ETH 和 AI-RISK-CATALOG 对齐 |
| **GDPR** | ETH 文档含 GDPR 法律依据部分，DPIA 用于隐私影响评估 |

### 中国法规覆盖 *(通过 `regional_scope: china` opt-in)*

| 标准 | StrayMark 如何帮助 |
|------|-------------------|
| **TC260 人工智能安全治理框架 v2.0** | 五级风险分级(TC260RA),ETH/MCARD/AILOG/SEC 上的 `tc260_*` 字段 |
| **PIPL — 个人信息保护法** | PIPIA 模板含第55-56条节,`pipl_*` 字段,留存 ≥ 3 年(`TYPE-003`) |
| **GB 45438-2025** *(强制)* | AILABEL 模板覆盖生成式内容的显式 + 隐式标识 |
| **CAC 算法备案** | CACFILE 模板跟踪单一/双重备案;通过 `cac_filing_required` 从 MCARD 交叉检查 |
| **GB/T 45652-2025** | SBOM 与 MCARD 的训练数据安全部分;`gb45652_training_data_compliance` 字段 |
| **CSL 2026** | INC 扩展含 `csl_severity_level`、时限一致性规则(1h / 4h+72h+30d 窗口) |

> 在 `.straymark/config.yml` 的 `regional_scope` 中加入 `china` 即可启用。详见下方 *配置* 章节及安装在 `.straymark/00-governance/` 下的 `CHINA-REGULATORY-FRAMEWORK.md` 指南。

### 架构文档

| 标准 | StrayMark 如何帮助 |
|------|-------------------|
| **ADR (Architecture Decision Records)** | 原生 ADR 支持，含扩展元数据和不可变规则 |
| **arc42** | ADR 补充 arc42 决策文档 |
| **C4 Model** | ADR 在 C4 每个层级记录决策 |

### 合规与治理

| 法规 | StrayMark 如何帮助 |
|------|-------------------|
| **GDPR** | ETH 文档用于隐私评估，DPIA 用于数据保护影响 |
| **SOC 2** | 通过 AILOG 进行变更文档和访问记录 |
| **ISO 27001** | 通过 SEC 评估进行安全决策文档 |
| **HIPAA** | 医疗应用的审计跟踪 |

### 可观测性（可选）

| 标准 | StrayMark 如何帮助 |
|------|-------------------|
| **OpenTelemetry** | REQ、TES、INC 中的可选可观测性部分；`observabilidad` 标签用于仪表化变更 |

---

## 采用路径 A：新项目

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

然后初始化并提交：

```bash
cd your-project
straymark init .

git add .straymark/ STRAYMARK.md
git commit -m "chore: adopt StrayMark"
```

CLI 自动完成：
- 从 GitHub 下载最新的 StrayMark 版本
- 设置 `.straymark/` 目录结构
- 创建包含治理规则的 `STRAYMARK.md`
- 配置 AI Agent 指令（`CLAUDE.md`、`GEMINI.md`、`.cursorrules` 等）
- 复制 CI/CD 工作流

### 选项 2：手动设置

1. **下载最新版本**

   前往 [GitHub Releases](https://github.com/StrangeDaysTech/straymark/releases)，下载最新的 `fw-*` 版本 ZIP（例如 `fw-4.10.0`）。

2. **解压到你的项目**
   ```bash
   unzip straymark-fw-*.zip -d your-project/
   ```

3. **提交结构**
   ```bash
   git add .straymark/ STRAYMARK.md
   git commit -m "chore: adopt StrayMark for documentation governance"
   ```

---

## 采用路径 B：现有项目

### 阶段 1：评估（第 1 天）

1. **评估当前文档**

   回答以下问题：
   - 你有现有的 ADR 吗？它们在哪里？
   - 你有 `docs/` 文件夹吗？里面有什么？
   - 是否已有命名规范？
   - 你使用 AI 编码助手吗？

2. **规划迁移**

   | 当前状态 | 建议操作 |
   |----------|----------|
   | 没有文档 | 从零开始使用 StrayMark |
   | `docs/` 文件夹中有文档 | 保留 `docs/` 存放面向用户的文档，添加 `.straymark/` 存放开发文档 |
   | 已有 ADR | 迁移到 `.straymark/02-design/decisions/`，使用新命名 |
   | 混合文档 | 分类并逐步迁移 |

### 阶段 2：安装（第 1-2 天）

1. **添加 StrayMark 结构**
   ```bash
   # 使用 CLI（推荐）
   straymark init .

   # 或手动：从 GitHub Releases 下载最新的 fw-* 版本
   # https://github.com/StrangeDaysTech/straymark/releases
   ```

2. **解决与现有 `docs/` 的冲突**

   StrayMark 专门使用 `.straymark/` 以避免冲突：

   ```
   your-project/
   ├── docs/                    ← 保留用于 API 文档、用户指南等
   │   ├── api/
   │   └── user-guide/
   ├── .straymark/              ← 添加用于开发文档
   │   ├── 00-governance/
   │   ├── 01-requirements/
   │   └── ...
   └── src/
   ```

### 阶段 3：迁移（第 1-2 周）

1. **迁移现有 ADR**

   对于每个现有 ADR：
   ```bash
   # 旧：docs/adr/001-use-postgresql.md
   # 新：.straymark/02-design/decisions/ADR-2024-01-15-001-use-postgresql.md
   ```

   在 front-matter 中添加 StrayMark 元数据：
   ```yaml
   ---
   id: ADR-2024-01-15-001
   title: Use PostgreSQL for primary database
   status: accepted
   created: 2024-01-15
   agent: human
   confidence: high
   review_required: false
   risk_level: high
   # 保留原始元数据
   original_id: "001"
   migrated_from: "docs/adr/001-use-postgresql.md"
   ---
   ```

2. **记录迁移**

   创建 AILOG 记录迁移过程：
   ```
   .straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-straymark-adoption.md
   ```

### 阶段 4：团队采用（第 2-4 周）

1. **更新贡献指南**

   在你的 `CONTRIBUTING.md` 中添加：
   ```markdown
   ## Documentation

   This project uses [StrayMark](https://github.com/StrangeDaysTech/straymark) for documentation governance.

   - All significant changes must be documented in `.straymark/`
   - AI-assisted changes require AILOG entries
   - Architectural decisions require ADR documents

   See `.straymark/QUICK-REFERENCE.md` for document types and naming.
   ```

2. **启用 pre-commit 钩子（可选）**
   ```bash
   # 安装 pre-commit 钩子
   echo 'straymark validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit

   # 或使用 Husky
   npx husky add .husky/pre-commit "straymark validate --staged"
   ```

3. **启用 GitHub Actions（可选）**

   `.github/workflows/docs-validation.yml` 工作流将自动在 PR 中验证文档。

### 阶段 5：渐进推广

| 周次 | 重点 |
|------|------|
| 第 1 周 | 核心团队开始为新决策使用 StrayMark |
| 第 2 周 | 迁移关键的现有 ADR |
| 第 3 周 | 在 CI/CD 中启用验证 |
| 第 4 周 | 全团队采用；记录现有技术债务 |

---

## 配置

### 区域法规范围(Regional Regulatory Scope)

`.straymark/config.yml` 控制 `straymark compliance` 评估哪些合规框架,以及 `straymark new` 暴露哪些文档类型:

```yaml
regional_scope:
  - global   # NIST AI RMF + ISO/IEC 42001(始终建议)
  - eu       # EU AI Act + GDPR
  - china    # TC260 v2.0、PIPL/PIPIA、GB 45438、CAC、GB/T 45652、CSL 2026
```

**省略时的默认值**:`[global, eu]` — 保持 `fw-4.3.0` 之前所有 StrayMark 版本的行为。

将 `china` 加入列表时:

- 通过 `straymark new` 可使用 4 种中国专属文档类型:`PIPIA`、`CACFILE`、`TC260RA`、`AILABEL`。
- `straymark compliance` 运行 6 个新的合规检查器(也可通过 `--region china` / `--standard china-*`)。
- 启用 12 条范围感知的验证规则(`CROSS-004` 至 `CROSS-011`、`TYPE-003` 至 `TYPE-006`)。
- 在 `.straymark/00-governance/` 下提供 5 份新的治理指南(`CHINA-REGULATORY-FRAMEWORK.md`,以及 TC260、PIPL/PIPIA、CAC 备案、GB 45438 标识的逐项指南)— 全部提供 EN、ES、zh-CN 三种语言。

`regional_scope` 中不含 `china` 的项目不受任何影响:无新增文件、无新提示、无新规则。后续添加 `china` 完全可逆。

### 自定义 Agent 标识符

每个 AI 平台都有自己的配置文件，用于：

1. 标识 Agent（例如 `claude-code-v1.0`）
2. 定义何时需要文档（>10 行、安全变更等）
3. 设置自主权限制
4. 指定模板位置
5. 要求文档报告
6. **强制 Git 工作流**（分支命名、约定式提交、不直接提交到 `main`）

更新 Agent 标识符以匹配你的版本：

```yaml
# 在任意 Agent 配置文件中
agent: claude-code-v1.0      # 默认
agent: claude-code-v2.1      # 你的自定义版本
agent: acme-corp-claude-v1   # 组织特定
```

### 自定义文档类型

要添加新的文档类型：

1. **创建模板**
   ```
   .straymark/templates/TEMPLATE-NEWTYPE.md
   ```

2. **更新治理文档**

   在以下文件中添加新类型：
   - `.straymark/00-governance/DOCUMENTATION-POLICY.md`
   - `.straymark/00-governance/AGENT-RULES.md`
   - `.straymark/QUICK-REFERENCE.md`

3. **更新 Agent 配置**

   在所有 Agent 配置文件中添加新类型。

4. **更新验证**

   在以下位置添加新类型：
   - CLI 验证逻辑（`straymark validate`）
   - `.github/workflows/docs-validation.yml`

### 自定义文件夹结构

编号的文件夹结构（`00-governance`、`01-requirements` 等）设计目的是：
- 在文件浏览器中逻辑排序
- 清晰的关注点分离
- 便于导航

你可以重命名文件夹，但需更新以下位置的所有引用：
- Agent 配置文件
- 治理文档

---

## 验证

### 使用 Skills 验证（Claude Code）

如果使用 Claude Code，可以用内置 Skill 验证文档合规性：

```bash
/straymark-status
```

此 Skill 显示：
- 最近创建了哪些 StrayMark 文档
- 哪些修改过的文件可能需要文档
- 整体文档合规状态

### 手动验证

采用后，验证你的设置：

```bash
# 运行验证（跨平台）
straymark validate
```

### 检查清单

- [ ] `.straymark/` 文件夹结构存在
- [ ] 至少有一个 Agent 配置文件（`CLAUDE.md`、`GEMINI.md` 等）
- [ ] 治理文档存在于 `.straymark/00-governance/`
- [ ] 模板存在于 `.straymark/templates/`
- [ ] Git 分支策略记录在 `.straymark/03-implementation/`
- [ ] `QUICK-REFERENCE.md` 可访问
- [ ] `straymark validate` 无错误运行
- [ ] （可选）Pre-commit 钩子已安装
- [ ] （可选）GitHub Actions 工作流已启用

---

## 外部审计（可选）

自 `fw-4.9.0` 起，当你与 AI 助手在循环中协作实现 Charter 时（Claude Code、Gemini Code、Cursor），你可以在 Charter 关闭时可选地运行外部多模型审计。两个 skills 封装底层 CLI 编排：

- **`/straymark-audit-prompt CHARTER-XX`** — 在规范路径 `.straymark/audits/<id>/audit-prompt.md` 处写入统一审计 prompt。操作员打开 N 个审计员 CLI 并在每个中运行 `/straymark-audit-execute`。无需复制/粘贴。
- **`/straymark-audit-execute [CHARTER-XX]`** *(fw-4.9.0+)* — 在审计员 CLI 中运行（gemini-cli、claude-cli、copilot-cli、codex-cli）。读取 prompt，使用 tool use 进行审计并引用 `path:line`，写入以审计员模型 ID 为键的 report。
- **`/straymark-audit-review CHARTER-XX`** — 将 N 个 reports 合并为六节 `review.md`（执行摘要 / 范围 / 按审计员评估 / 修复计划 P0-P4 / 丢弃 / 审计员评分）并将 `external_audit:` YAML 合并到 Charter 遥测。

Agent 会在工作流的特定时刻**主动提议**审计 — 当实现完成、drift check 干净，且 `charter close` 尚未调用时。推荐基于 Charter 的风险面和复杂度给出 是/否（启发式见 `.straymark/00-governance/AGENT-RULES.md` §12）。

**外部审计完全可选**且**永不强制**。Charter 的声明性范围 + drift check + AILOG 纪律已为关闭提供严格支撑。审计在 Charter 触及安全面、引入新组件或 diff 中存在高复杂度函数时增加跨模型信号。如果你的情况下成本（2-3 个 LLM 审计员）与价值不匹配，可以自由拒绝。

底层 CLI 命令（PREPARE / CALIBRATE / FINALIZE / `--merge-into`）见 [`CLI-REFERENCE.md` § straymark charter audit](./CLI-REFERENCE.md#straymark-charter-audit-charter-id-range-revrev-calibrate--finalize-path-dir)。

---

## 常见问题

### 通用问题

**问：StrayMark 会替代我现有的文档吗？**

答：不会。StrayMark 用于*开发过程文档*（决策、变更、审查）。保留你现有的 `docs/` 文件夹存放面向用户的文档、API 参考和指南。

**问：我不使用 AI 编码助手也能从 StrayMark 受益吗？**

答：可以。StrayMark 同样适用于纯人工团队。AI 审计功能（AILOG、AIDEC、ETH）在使用 AI 助手时特别有价值，但 ADR、REQ、TDE 和其他文档类型对任何团队都有用。

**问：StrayMark 增加多少额外开销？**

答：StrayMark 遵循"最小可行文档"原则。只有重大变更需要文档。微小变更（错别字、格式）被明确排除在外。

### 技术问题

**问：为什么使用 `.straymark/` 而不是 `docs/`？**

答：`docs/` 文件夹通常用于面向用户的文档、GitHub Pages 或生成的 API 文档。使用 `.straymark/` 避免冲突，并清晰地将开发文档与用户文档分开。

**问：可以在 Monorepo 中使用 StrayMark 吗？**

答：可以。你可以：
- 在根目录放一个 `.straymark/`，覆盖整个 Monorepo
- 在每个包/服务中放单独的 `.straymark/` 文件夹
- 使用混合方案，在根目录共享治理

**问：如何处理敏感信息？**

答：StrayMark 明确禁止记录凭据、令牌或密钥。验证脚本会检测常见的敏感模式并发出警告。对于确实敏感的决策，记录决策的*存在*而不包含敏感细节。

### 采用问题

**问：我的团队抗拒更多文档，怎么说服他们？**

答：从小处开始：
1. 先只用 ADR 记录架构决策
2. 通过加速新成员上手展示价值
3. 展示回顾旧决策时节省的时间
4. 逐步扩展到其他文档类型

**问：如何处理采用 StrayMark 之前创建的文档？**

答：有三种选择：
1. **迁移**：将旧文档转换为 StrayMark 格式（推荐用于重要文档）
2. **引用**：保留旧文档，从 StrayMark 文档中引用它们
3. **归档**：将旧文档移至归档文件夹，使用 StrayMark 重新开始

**问：如果我的 AI 助手不遵守规则怎么办？**

答：StrayMark 规则是指令，而非强制执行。如果 AI 助手创建了不合规的文档：
1. Pre-commit 钩子（`straymark validate --staged`）会捕获验证错误
2. CI/CD 会在 PR 中标记问题
3. 你可以手动修正并在下一次提示中指导 AI

---

## 获取帮助

- **CLI 参考手册**: [CLI-REFERENCE.md](CLI-REFERENCE.md) — 详细的命令参考
- **工作流**: [WORKFLOWS.md](WORKFLOWS.md) — 推荐的日常使用模式
- **Issues**: [GitHub Issues](https://github.com/StrangeDaysTech/straymark/issues)
- **讨论**: [GitHub Discussions](https://github.com/StrangeDaysTech/straymark/discussions)
- **贡献**: 参见 [CONTRIBUTING.md](../CONTRIBUTING.md)

---

---

<div align="center">

**StrayMark** — 因为每一次变更都值得被记录。

[返回 README](../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
