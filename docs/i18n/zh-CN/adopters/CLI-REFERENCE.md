# DevTrail CLI 参考手册

**`devtrail` 命令行工具的完整参考。**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**语言**: [English](../../../adopters/CLI-REFERENCE.md) | [Español](../../es/adopters/CLI-REFERENCE.md) | 简体中文

---

## 目录

1. [安装](#安装)
2. [版本管理](#版本管理)
3. [命令](#命令) — init, update, remove, status, repair, validate, new, charter, compliance, metrics, analyze, audit, explore, about
4. [环境变量](#环境变量)
5. [退出码](#退出码)

---

## 安装

使用以下方法之一安装 DevTrail CLI。完整安装说明参见 [README](../README.md#快速开始)。

**快速安装（预编译二进制文件）：**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

**从源码安装：**

```bash
cargo install devtrail-cli
```

---

## 版本管理

DevTrail 为每个组件使用**独立的版本标签**：

| 组件 | 标签前缀 | 示例 | 包含内容 |
|------|----------|------|----------|
| Framework | `fw-` | `fw-4.6.0` | 模板（12 种类型）、治理文档、指令 |
| CLI | `cli-` | `cli-3.7.0` | `devtrail` 二进制文件 |

Framework 和 CLI 独立发布。Framework 更新不需要 CLI 更新，反之亦然。

**检查已安装的版本：**

```bash
devtrail about    # 显示 CLI 版本 + Framework 版本（如已安装）
devtrail status   # 显示完整的安装状态，包括版本
```

---

## 命令

### `devtrail init [path]`

在项目目录中初始化 DevTrail。

**参数：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |

**功能：**

1. 从 GitHub 下载最新的 Framework 版本（`fw-*`）
2. 创建 `.devtrail/` 目录结构
3. 创建包含治理规则的 `DEVTRAIL.md`
4. 配置 AI Agent 指令文件（`CLAUDE.md`、`GEMINI.md`、`.cursorrules` 等）
5. 复制 CI/CD 工作流

**示例：**

```bash
$ devtrail init .
✔ Downloaded DevTrail fw-4.6.0
✔ Created .devtrail/ directory structure
✔ Created DEVTRAIL.md
✔ Configured AI agent directives

DevTrail initialized successfully!
Next: git add .devtrail/ DEVTRAIL.md && git commit -m "chore: adopt DevTrail"
```

---

### `devtrail update`

将 Framework 和 CLI **同时**更新到最新版本。等同于依次运行 `update-framework` 和 `update-cli`。

如果当前目录中不存在 `.devtrail/`，Framework 更新将被跳过并显示警告。

**示例：**

```bash
$ devtrail update
Updating framework...
✔ Framework updated to fw-4.6.0
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail update-framework`

仅更新 Framework 文件。在 GitHub 上查找最新的 `fw-*` 版本。

**冲突处理：** 如果你修改了 Framework 文件（例如治理文档或模板），更新会保留你的修改并报告冲突以便手动解决。

**示例：**

```bash
$ devtrail update-framework
✔ Framework updated to fw-4.6.0
```

---

### `devtrail update-cli`

自动更新 `devtrail` 二进制文件。自动检测安装方式并使用相应的更新机制：

- **预编译二进制文件**（通过 `install.sh` / `install.ps1` 安装）：从 GitHub Releases 下载最新二进制文件
- **Cargo**（通过 `cargo install` 安装）：运行 `cargo install --force devtrail-cli`

使用 `--method` 覆盖自动检测：`--method=github` 或 `--method=cargo`。

**示例：**

```bash
$ devtrail update-cli
✔ CLI updated to cli-3.5.2

$ devtrail update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail remove [--full]`

从当前项目中移除 DevTrail。

**标志：**

| 标志 | 描述 |
|------|------|
| `--full` | 移除所有内容，包括你在 `.devtrail/` 中创建的文档。会请求确认。 |

**默认行为**（不带 `--full`）：移除 Framework 结构但保留你在 `.devtrail/` 中创建的文档。

**示例：**

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

显示安装状态和文档统计。

**参数：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |

**输出包含：**

- 项目路径
- Framework 版本
- CLI 版本
- 配置的语言
- 目录结构完整性
- 文档统计（按类型计数）

**示例：**

```
$ devtrail status

  ╔════════════════════════════════════════════════╗
  ║ DevTrail Status                                ║
  ╚════════════════════════════════════════════════╝

  Project
  ┌───────────┬──────────────────────────┐
  │ Path      │ /home/user/my-project    │
  │ Framework │ fw-4.6.0                 │
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

通过恢复缺失的目录和 Framework 文件来修复损坏的 DevTrail 安装。

**参数：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |

**功能：**

1. 检查缺失的目录并使用 `.gitkeep` 恢复
2. 如果需要恢复文件（模板、治理文档、配置），**一次性**下载 Framework 版本
3. 如果 `DEVTRAIL.md` 缺失，重新注入指令
4. 修复后重新计算校验和
5. 不会修改或删除用户生成的文档

**示例：**

```bash
$ devtrail repair
Repairing DevTrail in /home/user/my-project
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .devtrail/templates/
→ Downloading framework to restore missing files...
  Using version: fw-4.6.0
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ DevTrail repaired successfully!
```

---

### `devtrail validate [path] [--fix] [--staged] [--include-charters]`

验证 DevTrail 文档的合规性和正确性。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--fix` | — | 自动修复简单问题（例如为高风险文档添加缺失的 `review_required: true`） |
| `--staged` | — | 仅验证已暂存（git add）的文件。适合 pre-commit 钩子。 |
| `--include-charters` | — | 同时根据章程 JSON Schema 和引用完整性（`originating_ailogs` 中的 ID 解析；`originating_spec` 路径存在）验证 `docs/charters/` 中的章程。Opt-in，默认 `false`，确保未使用章程模式的项目不受影响。目前仅在非 `--staged` 模式下生效；staged 模式的章程验证将在 cli-3.7.0 中加入。 |

**检查项目：**

- 命名规范（`TYPE-YYYY-MM-DD-NNN-description.md`）
- 必需的元数据字段（id、title、status、created、agent、confidence、review_required、risk_level、tags、related）
- 跨字段一致性（例如高风险必须有 review_required）
- 类型特定字段（例如 INC 需要 severity，SEC 需要 threat_model_methodology）
- 敏感信息检测（API 密钥、密码）
- 关联文档存在性

当 `regional_scope` 包含 `china` 时,启用十二条额外规则(`CROSS-004` 至 `CROSS-011`、`TYPE-003` 至 `TYPE-006`),涵盖 TC260 审核升级、敏感数据文档的 PIPIA 关联、CACFILE / AILABEL 交叉引用、CSL 严重程度-时限一致性、PIPIA 三年留存。未启用 `china` 时,这些规则被跳过 — 不会产生误报。

**示例：**

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

### `devtrail new [path] [-t <type>] [--title <title>]`

从模板创建新的 DevTrail 文档。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--doc-type`, `-t` | — | 文档类型。核心(12 种):`ailog`、`aidec`、`adr`、`eth`、`req`、`tes`、`inc`、`tde`、`sec`、`mcard`、`sbom`、`dpia`。中国(4 种,opt-in):`pipia`、`cacfile`、`tc260ra`、`ailabel`。 |
| `--title` | — | 新文档的标题 |

如果未指定 `--doc-type` 或 `--title`,将以交互方式提示。当 `regional_scope` 不包含 `china` 时,中国专属类型从提示中过滤(`-t` 也会拒绝)。

**示例：**

```bash
# 交互式 — 提示输入类型和标题
$ devtrail new

# 创建带标题的 AILOG（非交互式）
$ devtrail new -t ailog --title "Implement JWT authentication"

# 创建 ADR
$ devtrail new --doc-type adr --title "Use PostgreSQL for persistence"
```

**输出示例：**

```
$ devtrail new -t ailog --title "Implement JWT authentication"

  ✔ Created: .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md
```

---

### `devtrail charter <子命令>`

管理**章程（Charter）**：事前声明、事后审计的有界工作单元。一个章程将声明性范围（要修改的文件、风险、可执行的验证命令）与事后审计锚点（漂移检测、多模型审计）配对。章程位于 `docs/charters/NN-slug.md`（项目根目录级别，**不在** `.devtrail/` 之下）。

> **命名历史。**在使该模式定型的 Sentinel `/plan-audit` 实验中（2026 年 4 月，6 个周期），章程被称为 *Plans*。DevTrail CLI 从此版本开始使用 **Charter** 以避免与 GitHub SpecKit 的 `plan.md` 命名冲突。Sentinel 的历史文件刻意保留 "Plan" 命名。完整的概念范围与重命名理由见 `Propuesta/que-es-un-charter.md`。

**子命令：**

- `devtrail charter new` — 从框架模板创建新的章程
- `devtrail charter list` — 用可选过滤器枚举章程
- `devtrail charter status` — 显示章程详情，或最近的 5 个章程

CLI 路线图的第 2 阶段将增加 `charter close`（交互式遥测）和 `charter drift`（文件与提交的漂移检查）。第 3 阶段将增加 `charter audit`（多模型外部审计）。

#### `devtrail charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

从框架模板将章程创建到 `docs/charters/NN-slug.md`。如果未传入 `--title`，会以交互方式提示。两个来源标志在 clap 级别互斥。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--type`, `-t` | `M` | 工作量估计。`XS`、`S`、`M`、`L` 之一。 |
| `--title` | — | 章程标题。用于构造 slug 和文件名。缺失时提示。 |
| `--from-ailog` | — | 来源 AILOG ID（如 `AILOG-2026-04-28-021`）。在前置元数据中预填充 `originating_ailogs`。**与 `--from-spec` 互斥。** |
| `--from-spec` | — | SpecKit 规范文件路径（如 `specs/001-feature/spec.md`）。在前置元数据中预填充 `originating_spec`。创建时会校验路径存在性。**与 `--from-ailog` 互斥。** |

未传入任何来源标志时，`originating_ailogs` 和 `originating_spec` 在生成的前置元数据中均保持注释状态——章程"无显式来源"地创建，由用户在状态变为 `in-progress` 之前手动填写。

#### `devtrail charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

以表格形式枚举章程。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.` | 目标项目目录 |
| `--status` | `all` | 按生命周期状态过滤 |
| `--origin` | `any`（无过滤） | 按来源类型过滤：`ailog`、`spec` 或 `any` |

无法解析的文件作为警告输出到 stderr，不会中断命令——表格显示能列出的内容。

#### `devtrail charter status [CHARTER-ID] [--path <dir>]`

带 ID：打印完整的章程详情（前置元数据、文件位置、正文章节列表、第 2 阶段功能占位符）。无 ID：按 NN 降序打印最近的 5 个章程。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `CHARTER-ID` | — | 章程标识符。接受完整的 `charter_id`（`CHARTER-01-test`）、`CHARTER-NN` 前缀（`CHARTER-01`），或仅数字 NN（`01` 或 `1`）。数字匹配对零填充宽容。 |
| `--path` | `.` | 目标项目目录。使用标志（而非位置参数）以避免与可选位置参数 `CHARTER-ID` 混淆。 |

---

### `devtrail compliance [path] [--standard <name>] [--region <name>] [--all] [--output <format>]`

检查法规合规状态。默认评估 `.devtrail/config.yml` 中 `regional_scope` 所列区域的标准(默认 `[global, eu]`)。在 `regional_scope` 中加入 `china` 后,六个中国法规框架可用。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`(当前目录) | 目标项目目录 |
| `--standard` | — | 检查特定标准:`eu-ai-act`、`iso-42001`、`nist-ai-rmf`、`china-tc260`、`china-pipl`、`china-gb45438`、`china-cac`、`china-gb45652`、`china-csl` |
| `--region` | — | 运行某区域的全部标准:`global`、`eu`、`china` 或 `all` |
| `--all` | — | 运行全部标准(忽略 `regional_scope`) |
| `--output` | `text` | 输出格式:`text`、`markdown` 或 `json` |

优先级:`--standard` > `--all` > `--region` > 项目的 `regional_scope`。

**中国标准(通过 `regional_scope: china` 启用):**

- **TC260 v2.0**:存在 TC260RA;高/很高/极重等级要求人工审核;三项分级标准(场景 × 智能 × 规模)已填充
- **PIPL**:`pipl_applicable: true` 时存在 PIPIA;跨境传输已记录;留存 ≥ 3 年(第56条)
- **GB 45438**:生成式内容存在 AILABEL;声明显式 + 隐式标识策略;必填元数据字段已填充
- **CAC**:必要时存在 CACFILE;`cac_filing_status` 已显式设置;状态为 `*_approved` 时填写 `cac_filing_number`
- **GB/T 45652**:SBOM 与 MCARD 声明训练数据安全合规
- **CSL 2026**:每个 INC 有 `csl_severity_level`;时限与严重程度一致(1h ↔ particularly_serious、4h ↔ relatively_major);major+ 事件 30 天内提交事后审查

**示例:**

```bash
# 默认:仅运行 regional_scope 中包含的区域的标准
$ devtrail compliance

# 六个中国框架(需要 regional_scope: china)
$ devtrail compliance --region china

# 单一中国框架
$ devtrail compliance --standard china-pipl --output json

# 强制运行全部标准,忽略 regional_scope
$ devtrail compliance --all
```

> **启用方式**:在 `.devtrail/config.yml` 中添加:
>
> ```yaml
> regional_scope:
>   - global
>   - eu
>   - china
> ```

**检查内容：**

- **EU AI Act**：风险分类、伦理审查关联、DPIA 存在性、事件报告
- **ISO/IEC 42001**：治理策略、风险规划（ETH）、运营文档（AILOG/AIDEC）、附录 A 覆盖
- **NIST AI RMF**：MAP（AILOG）、MEASURE（TES）、MANAGE（ETH/INC）、GOVERN（策略 + ADR）、GenAI 风险覆盖（12 个 NIST 600-1 类别）

**示例：**

```bash
$ devtrail compliance --all
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
    ✓ [NIST-MAP-001] MAP function — AI actions documented (AILOG)
    ✓ [NIST-MEASURE-001] MEASURE function — Test plans exist (TES)
    ✓ [NIST-MANAGE-001] MANAGE function — Risk management documented (ETH + INC)
    ✓ [NIST-GOVERN-001] GOVERN function — Governance policy and decisions documented
    ~ [NIST-GENAI-001] GenAI risk coverage — NIST AI 600-1 (4/12 categories)

  Overall compliance: 78%

$ devtrail compliance --standard eu-ai-act --output json
[{"standard":"EuAiAct","checks":[...],"score":75.0}]
```

---

### `devtrail metrics [path] [--period <period>] [--output <format>]`

显示治理指标和文档统计。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--period` | `last-30-days` | 时间段：`last-7-days`、`last-30-days`、`last-90-days` 或 `all` |
| `--output` | `text` | 输出格式：`text`、`markdown` 或 `json` |

**包含的指标：**

- 指定时间段内按类型统计的文档数量
- 审查合规率（需要审查的文档中已达到 accepted/superseded 状态的百分比）
- 风险分布（low/medium/high/critical）
- Agent 活动（每个 Agent 的文档数）
- 与上一时期的趋势对比（↑/↓/→）

**示例：**

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

使用认知复杂度和圈复杂度指标分析代码复杂度，由 [arborist-metrics](https://crates.io/crates/arborist-metrics) 驱动。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 待分析的目标目录 |
| `--threshold` | `8`（或来自配置） | 认知复杂度阈值 |
| `--output` | `text` | 输出格式：`text`、`json` 或 `markdown` |
| `--top` | — | 仅显示复杂度最高的前 N 个函数 |

**支持的语言：** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**阈值解析顺序：** CLI 标志 → `.devtrail/config.yml` → 默认值 (8)

**配置**（可选，在 `.devtrail/config.yml` 中）：

```yaml
complexity:
  threshold: 8
```

**示例：**

```bash
# 分析当前目录
$ devtrail analyze

# 自定义阈值并显示前 10 名
$ devtrail analyze --threshold 5 --top 10

# JSON 输出用于 CI 集成
$ devtrail analyze --output json

# 分析指定项目
$ devtrail analyze /path/to/project
```

**输出示例：**

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

> **注意：** 此命令无需 `devtrail init` 即可工作。它操作源文件，而非 DevTrail 文档。`analyze` 功能可在编译时通过 `--no-default-features` 禁用。

> **文档触发：** AI Agent 使用 `devtrail analyze --output json` 作为确定何时创建 AILOG 文档的主要方法。如果 JSON 输出中 `summary.above_threshold > 0`，Agent 应创建 AILOG。当 CLI 不可用时，Agent 回退到 >20 行业务逻辑的启发式规则。

---

### `devtrail audit [path] [--from <date>] [--to <date>] [--system <name>] [--output <format>]`

生成包含时间线、可追溯性映射和合规摘要的审计跟踪报告。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--from` | — | 审计期间的开始日期（YYYY-MM-DD） |
| `--to` | — | 审计期间的结束日期（YYYY-MM-DD） |
| `--system` | — | 按系统/组件名称过滤（匹配 tags 和 title） |
| `--output` | `text` | 输出格式：`text`、`markdown`、`json` 或 `html` |

**报告包含：**

- 所有文档的时间顺序时间线，含类型、标题、Agent 和风险级别
- 显示文档关系链的可追溯性映射（例如 REQ → ADR → AILOG → TES）
- 风险分布（low/medium/high/critical）
- 合规摘要（EU AI Act、ISO 42001、NIST AI RMF 评分）

**输出格式：**

| 格式 | 用途 |
|------|------|
| `text` | 终端查看（带颜色和格式） |
| `markdown` | 包含在 PR、Wiki 或报告中 |
| `json` | 与外部工具集成 |
| `html` | 独立报告，含样式表格和 SVG 风险图表 |

**示例：**

```bash
# 完整审计报告
$ devtrail audit

# 2026 年 Q1 审计
$ devtrail audit --from 2026-01-01 --to 2026-03-31

# 按系统过滤审计
$ devtrail audit --system auth-service

# 生成 HTML 报告
$ devtrail audit --from 2026-01-01 --to 2026-03-31 --output html > audit-q1.html

# 为 PR 生成 Markdown
$ devtrail audit --output markdown
```

---

### `devtrail explore [path]`

在终端界面（TUI）中交互式浏览和阅读 DevTrail 文档。

**参数：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |

**标志：**

| 标志 | 默认值 | 描述 |
|------|--------|------|
| `--lang <代码>` | 从项目解析（见下方） | TUI 界面与框架治理文档的显示语言（`en`、`es`、`zh-CN`）。缺少翻译时静默回退到英文。 |

**语言解析顺序**（自 cli-3.5.2 起）：

1. 提供 `--lang <代码>` 标志时优先
2. `.devtrail/config.yml` 文件存在时使用其中的 `language` 字段（即便是显式的 `language: en` 也视为用户的明确选择）
3. 环境变量 `$LC_ALL` / `$LANG`，映射到受支持的语言（如 `zh_CN.UTF-8` → `zh-CN`，`es_MX.UTF-8` → `es`）。繁体中文（`zh_TW` / `zh_HK`）及其他不受支持的 locale 会跳到下一项
4. `en`

**功能特性：**

- 双面板布局：导航树 + 文档查看器
- 元数据面板，显示状态、置信度、风险、标签和关联链接
- Markdown 渲染，支持颜色、表格、代码块和标题缩进
- 通过超链接在关联文档间导航
- 按文件名、标题、标签或日期搜索
- 全屏文档模式，`j` / `k` 作为 `↓` / `↑` 的替代按键
- 本地化感知：框架文档（`QUICK-REFERENCE`、`AGENT-RULES`、中国合规指南等）按 `.devtrail/config.yml` 中的 `language` 或 `--lang` 提供对应语言版本

**快捷键：**

| 按键 | 操作 |
|------|------|
| `↑↓` / `j/k` | 导航 / 滚动 |
| `Enter` | 展开分组 / 打开文档 |
| `Tab` | 切换面板：导航 → 元数据 → 文档 |
| `f` | 切换全屏文档 |
| `/` | 搜索 |
| `L` | 切换显示语言（`en → es → zh-CN`） |
| `Esc` | 返回 / 折叠 / 清除搜索 |
| `?` | 帮助弹窗，显示所有快捷键 |
| `q` | 退出 |

**示例：**

```bash
$ devtrail explore                       # 使用 config.language（默认 en）
$ devtrail explore --lang zh-CN          # 以简体中文浏览框架文档
$ devtrail explore --lang es             # 会话内切换到西班牙语
```

> **注意：** `explore` 命令需要 `tui` feature（默认启用）。如需不含此功能编译：`cargo build --no-default-features`。

---

### `devtrail about`

显示版本、作者和许可证信息。

**示例：**

```bash
$ devtrail about
DevTrail CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.6.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/devtrail
  Website:           https://strangedays.tech
```

---

## 环境变量

| 变量 | 描述 |
|------|------|
| `GITHUB_TOKEN` | GitHub 个人访问令牌，用于经过身份验证的 API 请求。在下载版本时有助于避免速率限制。 |

---

## 退出码

| 代码 | 含义 |
|------|------|
| `0` | 成功 |
| `1` | 错误（详情输出到 stderr） |

---

<div align="center">

**DevTrail** — 因为每一次变更都值得被记录。

[返回文档](../../README.md) • [README](../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
