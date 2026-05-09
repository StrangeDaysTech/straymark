# StrayMark CLI 参考手册

**`straymark` 命令行工具的完整参考。**

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

使用以下方法之一安装 StrayMark CLI。完整安装说明参见 [README](../README.md#快速开始)。

**快速安装（预编译二进制文件）：**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

**从源码安装：**

```bash
cargo install straymark-cli
```

---

## 版本管理

StrayMark 为每个组件使用**独立的版本标签**：

| 组件 | 标签前缀 | 示例 | 包含内容 |
|------|----------|------|----------|
| Framework | `fw-` | `fw-4.10.0` | 模板（12 种类型）、治理文档、指令 |
| CLI | `cli-` | `cli-3.10.0` | `straymark` 二进制文件 |

Framework 和 CLI 独立发布。Framework 更新不需要 CLI 更新，反之亦然。

**检查已安装的版本：**

```bash
straymark about    # 显示 CLI 版本 + Framework 版本（如已安装）
straymark status   # 显示完整的安装状态，包括版本
```

---

## 命令

### `straymark init [path] [--hooks]`

在项目目录中初始化 StrayMark。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--hooks` *(cli-3.7.0+)* | off | 在 `init` 后将框架的 pre-PR 钩子（`.straymark/hooks/pre-pr.sh`）安装为 `.git/hooks/pre-push`。每次 push 之前自动运行 `straymark charter drift`。Opt-in，遵循原则 #6（认知纪律 > 原始生产力）。如果已存在 `pre-push` 钩子则拒绝覆盖；非 git 仓库时静默跳过。 |

**功能：**

1. 从 GitHub 下载最新的 Framework 版本（`fw-*`）
2. 创建 `.straymark/` 目录结构
3. 创建包含治理规则的 `STRAYMARK.md`
4. 配置 AI Agent 指令文件（`CLAUDE.md`、`GEMINI.md`、`.cursorrules` 等）
5. 复制 CI/CD 工作流
6. *(`--hooks`)* 安装 pre-PR 钩子

**示例：**

```bash
$ straymark init .
✔ Downloaded StrayMark fw-4.10.0
✔ Created .straymark/ directory structure
✔ Created STRAYMARK.md
✔ Configured AI agent directives

StrayMark initialized successfully!
Next: git add .straymark/ STRAYMARK.md && git commit -m "chore: adopt StrayMark"
```

---

### `straymark update`

将 Framework 和 CLI **同时**更新到最新版本。等同于依次运行 `update-framework` 和 `update-cli`。

如果当前目录中不存在 `.straymark/`，Framework 更新将被跳过并显示警告。

**示例：**

```bash
$ straymark update
Updating framework...
✔ Framework updated to fw-4.10.0
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `straymark update-framework`

仅更新 Framework 文件。在 GitHub 上查找最新的 `fw-*` 版本。

**冲突处理：** 如果你修改了 Framework 文件（例如治理文档或模板），更新会保留你的修改并报告冲突以便手动解决。

**示例：**

```bash
$ straymark update-framework
✔ Framework updated to fw-4.10.0
```

---

### `straymark update-cli`

自动更新 `straymark` 二进制文件。自动检测安装方式并使用相应的更新机制：

- **预编译二进制文件**（通过 `install.sh` / `install.ps1` 安装）：从 GitHub Releases 下载最新二进制文件
- **Cargo**（通过 `cargo install` 安装）：运行 `cargo install --force straymark-cli`

使用 `--method` 覆盖自动检测：`--method=github` 或 `--method=cargo`。

**示例：**

```bash
$ straymark update-cli
✔ CLI updated to cli-3.5.2

$ straymark update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `straymark remove [--full]`

从当前项目中移除 StrayMark。

**标志：**

| 标志 | 描述 |
|------|------|
| `--full` | 移除所有内容，包括你在 `.straymark/` 中创建的文档。会请求确认。 |

**默认行为**（不带 `--full`）：移除 Framework 结构但保留你在 `.straymark/` 中创建的文档。

**示例：**

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
$ straymark status

  ╔════════════════════════════════════════════════╗
  ║ StrayMark Status                                ║
  ╚════════════════════════════════════════════════╝

  Project
  ┌───────────┬──────────────────────────┐
  │ Path      │ /home/user/my-project    │
  │ Framework │ fw-4.10.0                 │
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

通过恢复缺失的目录和 Framework 文件来修复损坏的 StrayMark 安装。

**参数：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |

**功能：**

1. 检查缺失的目录并使用 `.gitkeep` 恢复
2. 如果需要恢复文件（模板、治理文档、配置），**一次性**下载 Framework 版本
3. 如果 `STRAYMARK.md` 缺失，重新注入指令
4. 修复后重新计算校验和
5. 不会修改或删除用户生成的文档

**示例：**

```bash
$ straymark repair
Repairing StrayMark in /home/user/my-project
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .straymark/templates/
→ Downloading framework to restore missing files...
  Using version: fw-4.10.0
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ StrayMark repaired successfully!
```

---

### `straymark validate [path] [--fix] [--staged] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

验证 StrayMark 文档的合规性和正确性。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--fix` | — | 自动修复简单问题（例如为高风险文档添加缺失的 `review_required: true`） |
| `--staged` | — | 仅验证已暂存（git add）的文件。适合 pre-commit 钩子。 |
| `--include-charters` | — | 同时根据章程 JSON Schema 和引用完整性（`originating_ailogs` 中的 ID 解析；`originating_spec` 路径存在）验证 `docs/charters/` 中的章程。Opt-in，默认 `false`，确保未使用章程模式的项目不受影响。目前仅在非 `--staged` 模式下生效；staged 模式的章程验证将在 cli-3.10.0 中加入。 |
| `--check-pending-reviews` *(cli-3.7.0+)* | off | 列出所有 `review_required: true` 且没有 `review_outcome`、年龄超过 `--max-pending-days` 的文档。**仅警告** — 永不影响 validate 的退出码；适合用于 CI 仪表板上的审批积压视图。 |
| `--max-pending-days` *(cli-3.7.0+)* | `14` | `--check-pending-reviews` 的天数阈值。 |

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

### `straymark new [path] [-t <type>] [--title <title>]`

从模板创建新的 StrayMark 文档。

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
$ straymark new

# 创建带标题的 AILOG（非交互式）
$ straymark new -t ailog --title "Implement JWT authentication"

# 创建 ADR
$ straymark new --doc-type adr --title "Use PostgreSQL for persistence"
```

**输出示例：**

```
$ straymark new -t ailog --title "Implement JWT authentication"

  ✔ Created: .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-implement-jwt-authentication.md
```

---

### `straymark approve <doc-id> --outcome <outcome> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."] [--path <dir>]`

*自 **cli-3.7.0** + **fw-4.6.0** 起可用。`--quiet` 与高风险警告于 cli-3.8.0 加入。*

在带 `review_required: true` 的文档上记录正式的人工审批。原子地写入三个 frontmatter 审批字段（`reviewed_by`、`reviewed_at`、`review_outcome`）**并**追加规范的 `## Approval` 正文段落。实现 `DOCUMENTATION-POLICY.md §3.5` 中规范化的关闭信号。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `<doc-id>` | — | 文档 ID。接受裸前缀（`AIDEC-2026-05-02-001`）或带 slug 的完整 ID（`AIDEC-2026-05-02-001-foo`）。 |
| `--outcome` | — | 之一：`approved`、`revisions_requested`、`rejected`。TTY 中缺失时提示。 |
| `--reviewer` | — | 评审者身份：邮箱、GitHub handle 或 DID。TTY 中缺失时提示。 |
| `--at` | 今天 | 审批日期（`YYYY-MM-DD`）。 |
| `--notes` | — | 评审者可选备注（追加到正文段落）。 |
| `--path` | `.` | 目标项目目录。 |

**行为：**

- 如果文档没有 `review_required: true`，发出警告（不失败）— 追溯式签核是真实用例。
- **Frontmatter 修改**（最新优先）：替换已存在的 `reviewed_by/_at/outcome`；不存在时插入到 `review_required:` 之后。实现 §3.5 的多评审者约定：frontmatter 持有*最新*审批。
- **正文修改**（按时间顺序）：在任何模板尾签之前追加新的 `## Approval` 块。重新运行 `approve` 保留之前的块，因此正文显示完整的评审历史。
- 审批后**不会**将 `review_required: true` 切换为 `false` — 它作为为何需要评审的历史记录保留。

**示例：**

```bash
# Flag 驱动（CI / 脚本）
$ straymark approve AIDEC-2026-05-02-001 \
    --outcome approved \
    --reviewer pepe@example.com \
    --notes "Reviewed against ADR-007. LGTM."

  ✔ AIDEC-2026-05-02-001 marked as approved.
    Reviewer: pepe@example.com
    Date:     2026-05-02
    File:     .straymark/07-ai-audit/decisions/AIDEC-2026-05-02-001-foo.md

# 迭代评审周期：revisions_requested → 重新批准
$ straymark approve AIDEC-... --outcome revisions_requested --reviewer reviewer@x.io
# (作者迭代)
$ straymark approve AIDEC-... --outcome approved --reviewer reviewer@x.io
# Frontmatter 显示最新（approved）；正文按时间顺序保留两个块。

# 积压可见性
$ straymark validate --check-pending-reviews --max-pending-days 14
```

> 完整流程定义（关闭语义、正文格式、多评审者约定）见 `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` §3.5 "Recording Approval"。

---

### `straymark charter <子命令>`

管理**章程（Charter）**：事前声明、事后审计的有界工作单元。一个章程将声明性范围（要修改的文件、风险、可执行的验证命令）与事后审计锚点（漂移检测、多模型审计）配对。章程位于 `docs/charters/NN-slug.md`（项目根目录级别，**不在** `.straymark/` 之下）。

> **命名历史。**在使该模式定型的 Sentinel `/plan-audit` 实验中（2026 年 4 月，6 个周期），章程被称为 *Plans*。StrayMark CLI 从此版本开始使用 **Charter** 以避免与 GitHub SpecKit 的 `plan.md` 命名冲突。Sentinel 的历史文件刻意保留 "Plan" 命名。完整的概念范围与重命名理由见 `Propuesta/que-es-un-charter.md`。

**子命令：**

- `straymark charter new` — 从框架模板创建新的章程
- `straymark charter list` — 用可选过滤器枚举章程
- `straymark charter status` — 显示章程详情，或最近的 5 个章程
- `straymark charter close` *(cli-3.7.0+)* — 记录执行后遥测并将章程移至 `closed`
- `straymark charter drift` *(cli-3.7.0+)* — 检查文件 vs 提交的漂移，带 AILOG 感知抑制
- `straymark charter audit` *(cli-3.8.0+)* — 编排多模型外部审计（仅编排，详见下文）

#### `straymark charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

从框架模板将章程创建到 `docs/charters/NN-slug.md`。如果未传入 `--title`，会以交互方式提示。两个来源标志在 clap 级别互斥。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--type`, `-t` | `M` | 工作量估计。`XS`、`S`、`M`、`L` 之一。 |
| `--title` | — | 章程标题。用于构造 slug 和文件名。缺失时提示。 |
| `--from-ailog` | — | 来源 AILOG ID（如 `AILOG-2026-04-28-021`）。在前置元数据中预填充 `originating_ailogs`。**与 `--from-spec` 互斥。** |
| `--from-spec` | — | SpecKit 规范文件路径（如 `specs/001-feature/spec.md`）。在前置元数据中预填充 `originating_spec`。创建时会校验路径存在性。**与 `--from-ailog` 互斥。** |

未传入任何来源标志时，`originating_ailogs` 和 `originating_spec` 在生成的前置元数据中均保持注释状态——章程"无显式来源"地创建，由用户在状态变为 `in-progress` 之前手动填写。

#### `straymark charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

以表格形式枚举章程。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.` | 目标项目目录 |
| `--status` | `all` | 按生命周期状态过滤 |
| `--origin` | `any`（无过滤） | 按来源类型过滤：`ailog`、`spec` 或 `any` |

无法解析的文件作为警告输出到 stderr，不会中断命令——表格显示能列出的内容。

#### `straymark charter status [CHARTER-ID] [--path <dir>]`

带 ID：打印完整的章程详情（前置元数据、文件位置、正文章节列表）。无 ID：按 NN 降序打印最近的 5 个章程。

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `CHARTER-ID` | — | 章程标识符。接受完整的 `charter_id`（`CHARTER-01-test`）、`CHARTER-NN` 前缀（`CHARTER-01`），或仅数字 NN（`01` 或 `1`）。数字匹配对零填充宽容。 |
| `--path` | `.` | 目标项目目录。使用标志（而非位置参数）以避免与可选位置参数 `CHARTER-ID` 混淆。 |

#### `straymark charter close <CHARTER-ID> [--from-template] [--non-interactive] [--path <dir>]`

记录执行后遥测并将章程状态移至 `closed`。遥测被写入 `.straymark/charters/CHARTER-NN.telemetry.yaml`（独立文件，**不**嵌入到章程的 frontmatter — frontmatter 是事前声明性的；遥测是事后大量数据）。形状根据 `.straymark/schemas/charter-telemetry.schema.v0.json` 验证。

两种模式：

| 模式 | 标志组合 | 何时使用 |
|---|---|---|
| **交互式**（默认） | （无） | 按 schema 字段逐项 prompt。目标时间：5–10 分钟。 |
| **From template** | `--from-template` | 在章程旁复制 YAML 骨架供手动编辑。预填 `charter_id`、标题、`closed_at`。 |
| **From template, scripted** | `--from-template --non-interactive` | CI / 批量使用。完全跳过 prompts；重运行幂等。 |

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `CHARTER-ID` | — | 与 `charter status` 相同的解析规则。 |
| `--from-template` | false | 复制模板骨架而非运行交互式流程。 |
| `--non-interactive` | false | 跳过所有 prompt。需要 `--from-template`。 |
| `--path` | `.` | 目标项目目录。 |

**示例：**

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

在章程关闭时检测文件 vs 提交漂移。封装框架的 `.straymark/scripts/check-charter-drift.sh`（在 Sentinel PLAN-05 回顾性 + PLAN-06 前瞻性中实证验证零误报）。CLI 在原始脚本之上的附加价值是 **AILOG 感知**：报告为"已声明但未修改"的路径，如果出现在被章程 `originating_ailogs` 引用的任何 AILOG 的 `## Risk` / `## Riesgos` / `## 风险` 节中，则被静默。使用 `--no-ailog-suppress` 来禁用。

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `CHARTER-ID` | — | 与 `charter status` 相同的解析规则。 |
| `--range` | `HEAD~1..HEAD` | 要检查的 git 修订范围。 |
| `--no-ailog-suppress` *(cli-3.10.0+ 始终输出确认 INFO 行)* | false | 禁用 AILOG 感知抑制（显示每条已声明但被遗漏的路径）。传入此标志时，CLI 始终打印 `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` 行 — 即使 N=0 — 以便诊断模式即使在干净运行时也在输出中可见。 |
| `--path` | `.` | 目标项目目录。 |

**退出码：** `0` 没有漂移（或仅 AILOG 抑制）；`1` 存在未计入的漂移；`2` 用法错误（章程未找到、bash 缺失等）。

**示例：**

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

> **平台说明。** 漂移检查委托给 `bash`。在 Linux/macOS/WSL/Git Bash 上开箱即用。Windows 原生且无 WSL 时需安装 Git Bash；纯 Rust fallback 在路线图上但不在 fw-4.6.x 中。

#### 已声明路径的通配符支持 *(fw-4.9.0+)*

漂移检查在 `## Files to modify` 中解析两种通配符形式：

| 形式 | 示例 | 用例 |
|---|---|---|
| 省略号 | `` `.straymark/07-ai-audit/agent-logs/AILOG-...md` `` | 任何带该前缀的修改路径满足通配符。历史上用于执行期间会创建未知数量 AILOG 的情况。 |
| Glob | `` `AILOG-*.md` `` 或 `` `src/services/foo-*.rs` `` | 任何匹配该 glob（`*` → `.*`）的修改路径满足通配符。用于参数化集合被触动的批量章程声明。在 fw-4.9.0 中加入，源于 Sentinel CHARTER-04 暴露的摩擦（[issue #81](https://github.com/StrangeDaysTech/straymark/issues/81)）。 |

两种形式都双向处理：声明的通配符既抑制"已声明但未修改"警告（当至少一个匹配文件被修改时），也抑制"已修改但未声明"警告（当一个修改路径匹配某个已声明通配符时）。

#### 设计：治理路径始终在 scope 内

`docs/charters/*` 和 `.straymark/07-ai-audit/*` 下的路径**永远不会**被报告为"已修改但未声明"。这是有意的设计 — 当章程本身或执行的 AILOG 被触动时，这些路径总是合法的。在 Sentinel CHARTER-04 中实证验证：一次意外的 `git add -A` 暂存了无关的用户未跟踪文件（`.claude/skills/`、`cmd/sentinel/sentinel`）；该规则正确抑制了治理噪声而没有掩盖真正的项目文件扩展（[issue #81 W2](https://github.com/StrangeDaysTech/straymark/issues/81#issuecomment-update)）。

如果你运行的章程显式 scope 是治理 churn（例如仅触动 `.straymark/07-ai-audit/` 的批量批准章程），漂移检查将报告 0 个修改文件，你需要通过阅读 AILOG 来验证 scope。一个 `--strict-scope` 标志（禁用"始终在 scope"规则）在桌面上，用于未来 minor 版本，前提是真实的 adopter 报告这种不对称为摩擦。

#### `straymark charter audit <CHARTER-ID> [--range <REV..REV>] [--prepare | --merge-reports] [--merge-into <PATH>] [--path <dir>]`

*自 **cli-3.8.0** + **fw-4.7.0** 起可用。v1 统一流程在 **cli-3.10.0** + **fw-4.9.0** 中发布 — 用两步（PREPARE/MERGE-REPORTS）替换 v0 的三步（PREPARE/CALIBRATE/FINALIZE），统一审计员模板，并将规范路径迁移到 `.straymark/audits/`。*

编排章程执行的多模型外部审计。**仅编排** — CLI 准备 prompts、根据 schema 验证 outputs，并打印可粘贴到章程遥测中的 findings。**它不调用 LLM API。** 操作员在自己选择的审计器（Copilot、Gemini、Claude 等）中运行 prompts，并将响应保存到规范路径。

三步，每步可独立调用：

| 步骤 | 标志 | 发生什么 |
|---|---|---|
| 1. PREPARE | （默认） | 根据章程 + git diff + 来源 AILOGs 解析 `auditor-primary` 和 `auditor-secondary` prompts。写入到 `audit/charters/<CHARTER-ID>/prompts/` 之下。 |
| 2. CALIBRATE | `--calibrate` | 读取 `auditor-primary.md` 和 `auditor-secondary.md`（操作员必须在步骤 1 和 2 之间保存它们）。根据 `audit-output.schema.v0.json` 验证。解析 calibrator prompt 并嵌入两个响应。 |
| 3. FINALIZE | `--finalize` | 读取 calibrator 响应。验证全部 3 个 outputs。打印一个 YAML 格式的 `external_audit` 数组块，可粘贴到章程遥测中。 |

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `<CHARTER-ID>` | — | 与 `charter status` 相同的解析规则。 |
| `--range` | `HEAD~1..HEAD` | 审计员将审查的 git 修订范围。 |
| `--calibrate` | off | 运行步骤 2。与 `--finalize` 互斥。 |
| `--finalize` | off | 运行步骤 3。与 `--calibrate` 互斥。 |
| `--path` | `.` | 项目目录。 |

##### 异质性建议（在 v0 中不强制）

按照设计理由（`straymark-cli-roadmap.md` §5.2），auditor pair 应该是**不同模型族**：一个 Anthropic + 一个 Google + 一个 OpenAI，任意组合，永远不要两个同族。跨族异质性是使 findings 上的趋同成为高信号的原因 — 同族审计员共享盲点。

Calibrator-reconciler 可以是任何家族（包括实现者的家族），因为它的任务是定义性的（在已产生的判定上应用 schema），而非发现性。异质性对 auditor pair 重要，对 calibrator 不重要。

v0 文档化此建议但不自动检测或强制。一个 `--implementer-family X` 标志（拒绝单色配置）是 v1 候选项，等到 adopter 报告真实情况。

##### 产生的布局

```
audit/charters/CHARTER-NN/
├── prompts/
│   ├── auditor-primary.prompt.md      # 由步骤 1 解析，被发送的内容
│   ├── auditor-secondary.prompt.md    # 由步骤 1 解析
│   └── calibrator-reconciler.prompt.md  # 由步骤 2 解析
├── auditor-primary.md                 # 操作员粘贴 auditor 1 的响应
├── auditor-secondary.md               # 操作员粘贴 auditor 2 的响应
└── calibrator-reconciler.md           # 操作员粘贴 calibrator 的响应
```

`prompts/` 子目录持久化 API 调用*之前*发送给每个 auditor 的内容（关闭关于审计可见性的 [RFC #82](https://github.com/StrangeDaysTech/straymark/issues/82)）。Adopters 可以 `git add` 整个 `audit/` 目录得到完全版本化的审计轨迹，或 `.gitignore` 它（如果偏好周期是临时的）。

**示例：**

```bash
$ straymark charter audit CHARTER-05
  Step 1/3: PREPARE (CHARTER-05)
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-primary.prompt.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-secondary.prompt.md

  Next:
    1. Paste each prompt into your auditor of choice (use a model
       of a different family per auditor — see CLI-REFERENCE).
    2. Save the auditor responses to:
         audit/charters/CHARTER-05/auditor-primary.md
         audit/charters/CHARTER-05/auditor-secondary.md
    3. Run: straymark charter audit CHARTER-05 --calibrate

# (操作员在 Copilot 中运行 auditor 1，保存响应。在 Gemini 中运行 auditor 2，保存响应。)

$ straymark charter audit CHARTER-05 --calibrate
  Step 2/3: CALIBRATE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/calibrator-reconciler.prompt.md

  Next:
    1. Run the calibrator prompt in a model of your choice (calibrator
       may be of any family).
    2. Save the response to: audit/charters/CHARTER-05/calibrator-reconciler.md
    3. Run: straymark charter audit CHARTER-05 --finalize

# (操作员在 Claude 中运行 calibrator，保存响应。)

$ straymark charter audit CHARTER-05 --finalize
  Step 3/3: FINALIZE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md (5 findings)
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md (4 findings)
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

> **为什么仅编排？** 实现 3 个 HTTP 客户端（OpenAI / Google / Anthropic）需要 1-2 周 + 当 API 变化时的永久维护。Phase 3 v0 是实验性的 — CLI 的价值是 canon（prompt 形状 + output schema + 与遥测的集成），而非 API 调用本身。当 adopter 报告真实需求时，v1 可能加入 HTTP 客户端；在此之前，人在环模式与激发 Phase 3 的 Sentinel 实证 `/plan-audit` 模式相符。

> **Skill 替代方案 *(fw-4.9.0+)*。** 当与 AI 助手在循环中协作时（Claude Code、Gemini Code、Cursor 等），skills `/straymark-audit-prompt CHARTER-ID` 和 `/straymark-audit-review CHARTER-ID` 封装此命令并在对话中内联展示 prompts。Skills 还处理校准器步骤（驱动对话的 Agent 运行校准器）并触发 `--finalize --merge-into`，使得 `external_audit:` 数组直接追加到遥测中无需手动复制粘贴。详见下方的 [Skills](#skills) 章节。CLI 仍是唯一真相来源 — skills 仅添加 UX-inline。

---

### `straymark compliance [path] [--standard <name>] [--region <name>] [--all] [--output <format>]`

检查法规合规状态。默认评估 `.straymark/config.yml` 中 `regional_scope` 所列区域的标准(默认 `[global, eu]`)。在 `regional_scope` 中加入 `china` 后,六个中国法规框架可用。

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
$ straymark compliance

# 六个中国框架(需要 regional_scope: china)
$ straymark compliance --region china

# 单一中国框架
$ straymark compliance --standard china-pipl --output json

# 强制运行全部标准,忽略 regional_scope
$ straymark compliance --all
```

> **启用方式**:在 `.straymark/config.yml` 中添加:
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
$ straymark compliance --all
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
    ✓ [NIST-MAP-001] MAP function — AI actions documented (AILOG)
    ✓ [NIST-MEASURE-001] MEASURE function — Test plans exist (TES)
    ✓ [NIST-MANAGE-001] MANAGE function — Risk management documented (ETH + INC)
    ✓ [NIST-GOVERN-001] GOVERN function — Governance policy and decisions documented
    ~ [NIST-GENAI-001] GenAI risk coverage — NIST AI 600-1 (4/12 categories)

  Overall compliance: 78%

$ straymark compliance --standard eu-ai-act --output json
[{"standard":"EuAiAct","checks":[...],"score":75.0}]
```

---

### `straymark metrics [path] [--period <period>] [--output <format>]`

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

使用认知复杂度和圈复杂度指标分析代码复杂度，由 [arborist-metrics](https://crates.io/crates/arborist-metrics) 驱动。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 待分析的目标目录 |
| `--threshold` | `8`（或来自配置） | 认知复杂度阈值 |
| `--output` | `text` | 输出格式：`text`、`json` 或 `markdown` |
| `--top` | — | 仅显示复杂度最高的前 N 个函数 |

**支持的语言：** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**阈值解析顺序：** CLI 标志 → `.straymark/config.yml` → 默认值 (8)

**配置**（可选，在 `.straymark/config.yml` 中）：

```yaml
complexity:
  threshold: 8
```

**示例：**

```bash
# 分析当前目录
$ straymark analyze

# 自定义阈值并显示前 10 名
$ straymark analyze --threshold 5 --top 10

# JSON 输出用于 CI 集成
$ straymark analyze --output json

# 分析指定项目
$ straymark analyze /path/to/project
```

**输出示例：**

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

> **注意：** 此命令无需 `straymark init` 即可工作。它操作源文件，而非 StrayMark 文档。`analyze` 功能可在编译时通过 `--no-default-features` 禁用。

> **文档触发：** AI Agent 使用 `straymark analyze --output json` 作为确定何时创建 AILOG 文档的主要方法。如果 JSON 输出中 `summary.above_threshold > 0`，Agent 应创建 AILOG。当 CLI 不可用时，Agent 回退到 >20 行业务逻辑的启发式规则。

---

### `straymark audit [path] [--from <date>] [--to <date>] [--system <name>] [--output <format>]`

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
$ straymark audit

# 2026 年 Q1 审计
$ straymark audit --from 2026-01-01 --to 2026-03-31

# 按系统过滤审计
$ straymark audit --system auth-service

# 生成 HTML 报告
$ straymark audit --from 2026-01-01 --to 2026-03-31 --output html > audit-q1.html

# 为 PR 生成 Markdown
$ straymark audit --output markdown
```

---

### `straymark explore [path]`

在终端界面（TUI）中交互式浏览和阅读 StrayMark 文档。

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
2. `.straymark/config.yml` 文件存在时使用其中的 `language` 字段（即便是显式的 `language: en` 也视为用户的明确选择）
3. 环境变量 `$LC_ALL` / `$LANG`，映射到受支持的语言（如 `zh_CN.UTF-8` → `zh-CN`，`es_MX.UTF-8` → `es`）。繁体中文（`zh_TW` / `zh_HK`）及其他不受支持的 locale 会跳到下一项
4. `en`

**功能特性：**

- 双面板布局：导航树 + 文档查看器
- 元数据面板，显示状态、置信度、风险、标签和关联链接
- Markdown 渲染，支持颜色、表格、代码块和标题缩进
- 通过超链接在关联文档间导航
- 按文件名、标题、标签或日期搜索
- 全屏文档模式，`j` / `k` 作为 `↓` / `↑` 的替代按键
- 本地化感知：框架文档（`QUICK-REFERENCE`、`AGENT-RULES`、中国合规指南等）按 `.straymark/config.yml` 中的 `language` 或 `--lang` 提供对应语言版本

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
$ straymark explore                       # 使用 config.language（默认 en）
$ straymark explore --lang zh-CN          # 以简体中文浏览框架文档
$ straymark explore --lang es             # 会话内切换到西班牙语
```

> **注意：** `explore` 命令需要 `tui` feature（默认启用）。如需不含此功能编译：`cargo build --no-default-features`。

---

### `straymark about`

显示版本、作者和许可证信息。

**示例：**

```bash
$ straymark about
StrayMark CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.10.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/straymark
  Website:           https://strangedays.tech
```

---

## Skills

StrayMark 提供一组 skills（slash 命令）供 AI 助手内使用（Claude Code、Gemini Code、Cursor、通用 Agent 运行时）。每个 skill 在 `straymark init` 时以 3 种平行形式安装：

- `dist/.claude/skills/<skill>/SKILL.md`（Claude — frontmatter 含 `allowed-tools`）
- `dist/.gemini/skills/<skill>/SKILL.md`（Gemini — frontmatter 不含 `allowed-tools`）
- `dist/.agent/workflows/<skill>.md`（通用 Agent — 仅 `description` frontmatter）

| Skill | 用途 | 产生的文件 |
|---|---|---|
| `/straymark-status` | 检查近期变更的文档合规状态。 | 无（read-only） |
| `/straymark-new` | 交互式创建任意类型的文档。从上下文建议最佳匹配。 | `.straymark/<type-dir>/<TYPE>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | 快速 AILOG 创建快捷方式。 | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | 快速 AIDEC 创建快捷方式。 | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | 快速 ADR 创建快捷方式。 | `.straymark/04-architecture/decisions/ADR-*.md` |
| `/straymark-mcard` | 交互式 Model Card 创建流程。 | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | 交互式 SEC（安全评估）流程。 | `.straymark/08-security/SEC-*.md` |
| `/straymark-audit-prompt CHARTER-ID` *(fw-4.9.0+，在 fw-4.9.0 中重构)* | 在规范路径处生成章程的统一审计 prompt。封装 `straymark charter audit --prepare`。操作员随后在同一仓库中打开 N 个审计员 CLI，在每个中调用 `/straymark-audit-execute` — 无需复制/粘贴。 | `.straymark/audits/<CHARTER-ID>/audit-prompt.md` |
| `/straymark-audit-execute [CHARTER-ID]` *(fw-4.9.0+)* | **在审计员 CLI 中运行**（gemini-cli、claude-cli、copilot-cli、codex-cli 等）。从磁盘读取已准备的 prompt，使用 tool use 进行审计并引用 `path:line`，写入以审计员模型 ID 为键的 report。CHARTER-ID 参数可选 — 自动发现尚未由此模型生成 report 的 prompts。 | `.straymark/audits/<CHARTER-ID>/report-<sluggified-model-id>.md` |
| `/straymark-audit-review CHARTER-ID` *(fw-4.9.0+，在 fw-4.9.0 中扩展)* | `/straymark-audit-prompt` 的对应。读取 `.straymark/audits/<CHARTER-ID>/` 下的 N 个 reports，对每个 finding 与实际代码进行交叉验证（并行 Explore agents），生成六节合并的 `review.md`（执行摘要、范围、按审计员评估、修复计划 P0-P4、丢弃的 findings、审计员评分），并运行 `straymark charter audit --merge-reports --merge-into` 将 `external_audit:` 追加到章程遥测中。如果遥测尚不存在（章程未关闭），写入 `external-audit-pending.yaml` 供 close 时合并。 | `.straymark/audits/<CHARTER-ID>/review.md`，`external_audit:` 数组合并入遥测（或 pending YAML） |

### Skill vs CLI

三个审计 skill 是 CLI 命令和流程纪律的**封装**。`.straymark/audits/` 下的规范路径、统一的 prompt 模板、schema 验证、`external_audit` 形状全部在 CLI + framework 中 — skills 处理 UX-inline 部分：调度操作员通过审计周期，无需手动管理文件。**操作员从不复制/粘贴 prompts 或 reports** — skills 通过 `.straymark/audits/` 下的规范文件系统路径交换 artefacts。

不在循环中使用 AI 助手的 adopter 可直接通过 `straymark charter audit`（`--prepare` / `--merge-reports [--merge-into <path>]`）驱动相同工作流。`.straymark/audits/<id>/audit-prompt.md` 中的审计 prompt 在没有审计员 CLI 时也可粘贴到 chat 类 LLM 中使用 — skill 只是自动化文件交换。

### 审计检查点 *(fw-4.9.0+)*

`.straymark/00-governance/AGENT-RULES.md` §12 编码了一个工作流检查点，其中 Agent 在某个特定时刻主动提议审计 — 当 Charter 实现完成、drift 干净，且 `charter close` 尚未调用时。推荐基于启发式给出 是/否（安全面、新组件、AILOG 风险、复杂度）。外部审计**完全可选**；检查点是**软性**的 — 永不阻塞 `charter close`，永不强制（v0+v1 永久设计决策）。

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

**StrayMark** — 因为每一次变更都值得被记录。

[返回文档](../../README.md) • [README](../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
