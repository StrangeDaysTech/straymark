# StrayMark CLI 参考手册

**`straymark` 命令行工具的完整参考。**


---

## 目录

1. [安装](#安装)
2. [版本管理](#版本管理)
3. [命令](#命令) — init, update, remove, status, repair, validate, new, charter, followups, compliance, metrics, analyze, audit, explore, about
4. [环境变量](#环境变量)
5. [退出码](#退出码)

---

## 安装

使用以下方法之一安装 StrayMark CLI。完整安装说明参见 [README](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/zh-CN/README.md#快速开始)。

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
| Framework | `fw-` | `fw-4.35.0` | 模板（12 种类型）、治理文档、指令 |
| CLI | `cli-` | `cli-3.36.1` | `straymark` 二进制文件 |
| Loom（实验性） | `loom-` | `loom-0.4.2` | `straymark-loom` 可视化服务器，由 `straymark loom serve` 按需下载 |

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
4. 配置 AI Agent 指令文件（`AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、`.cursorrules` 等）
5. 复制 CI/CD 工作流
6. *(`--hooks`)* 安装 pre-PR 钩子

**示例：**

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

将 Framework 和 CLI **同时**更新到最新版本。等同于依次运行 `update-framework` 和 `update-cli`。

如果当前目录中不存在 `.straymark/`，Framework 更新将被跳过并显示警告。

**示例：**

```bash
$ straymark update
Updating framework...
✔ Framework updated to fw-4.15.0
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
✔ Framework updated to fw-4.15.0
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

---

#### `straymark status --where [path] [--out <dir>]` *(cli-3.25.0+，实验性)*

Loom 架构计划视图的文本版**"你在这里"**伴侣（仪表板的终端那一半）。加载 `architecture/model.yml`，并从你实时的治理信号中投影出每个组件的状态，因此你无需打开浏览器即可回答*"我们到哪儿了？"*。

> ⚠️ **实验性（Loom v0）。** 需要一个 `architecture/model.yml`（参见 `straymark architecture generate`）。输出形态可能在没有弃用周期的情况下更改。

| 标志 | 默认值 | 描述 |
|---|---|---|
| `--where` | 关 | 将 `status` 切换到架构投影视图 |
| `--out <dir>` | `.straymark/architecture` | 存放 `model.yml` 的目录（指向别处可投影保存在 `.straymark/` 之外的模型） |

每个组件都会被标记一个或多个状态，纯粹从治理派生（无需新的 frontmatter）：

- **`active`** —— 某个 `in-progress` 的 Charter 声明了匹配该组件的文件（← *你在这里*）；
- **`in-progress`** —— 在一个活跃组件中，已被改动的声明文件（声明 ∩ git 已修改）；
- **`implemented`** —— 某个已关闭的 Charter（及其 AILOG）触及了该组件；
- **`has-debt`** —— 某个开放的 `TDE` 与该组件相关；
- **`uncharted`** —— 磁盘上没有任何 Charter 或文档引用的文件。

该视图以一段**"我们到哪儿了"摘要**结尾：活跃的 Charter、声明与已修改的进度、近期的 AILOG，以及开放的债务。按构造，`active`/`implemented` 标志与 `straymark charter list --status in-progress`/`--status closed` + `charter drift` 一致（共享同一个投影）。

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
  Using version: fw-4.15.0
✓ Restored 16 file(s) from framework
→ Updating checksums...

✓ StrayMark repaired successfully!
```

---

### `straymark install-skills --agent <codex|claude|gemini> [--path .] [--dry-run] [--symlink]` *(cli-3.16.0+)*

将 StrayMark skills 安装到 AI 代理的**用户级**skills 目录。目前仅 `--agent codex` 执行实际工作：将 `<path>/.codex/skills/` 中的每个 `straymark-*` skill（由 `straymark init` 或 `straymark update` 生成）复制到 `$CODEX_HOME/skills/`（若未设置 `CODEX_HOME` 则使用 `$HOME/.codex/skills/`）。

对于 `--agent claude` 和 `--agent gemini`，命令会带说明错误退出：这些代理直接从项目树读取 skills（`.claude/skills/`、`.gemini/skills/`），因此无需用户级安装。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `--agent` | 必填 | `codex`、`claude` 或 `gemini` 之一。仅 `codex` 执行工作；其他选项打印说明后退出。 |
| `--path` | `.` | 源 `.codex/skills/` 所在的项目目录。 |
| `--dry-run` | off | 只打印将要安装的内容，不写入任何文件。 |
| `--symlink` | off | 对每个 skill 使用符号链接代替复制（仅 Unix；适合迭代 skill 内容的框架开发者）。 |

重新运行该命令会替换目标位置已有的 `straymark-*` 目录；非 `straymark-*` 的 skill（例如 Codex 的 `.system/` 包）不会被触动。

---

### `straymark validate [path] [--fix] [--staged] [--agent <codex>] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

验证 StrayMark 文档的合规性和正确性。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|-----------|--------|------|
| `path` | `.`（当前目录） | 目标项目目录 |
| `--fix` | — | 自动修复简单问题（例如为高风险文档添加缺失的 `review_required: true`） |
| `--staged` | — | 仅验证已暂存（git add）的文件。适合 pre-commit 钩子。 |
| `--agent` *(cli-3.16.0+)* | — | 切换到代理模式，检查用户级 skills 安装而非项目文档。目前仅支持 `codex` —— 校验 `~/.codex/skills/straymark-*` 是否存在、frontmatter YAML 可解析、`name`/`description` 必填、以及不含 `allowed-tools` 等 Claude 专用键（出现这些键意味着有人误从 `.claude/` 复制了 skill）。 |
| `--include-charters` | — | 同时根据章程 JSON Schema 和引用完整性（`originating_ailogs` 中的 ID 解析；`originating_spec` 路径存在）验证 `.straymark/charters/` 中的章程。包含 **`CHARTER-FILES-EXIST`** *(cli-3.17.0+)*：当 `## 要修改的文件` 中某行所指路径在磁盘上不存在且未标记为"新建"时发出警告 — 捕获基于假设的、未读代码所撰写的章程（发现 #210）。仅警告;不同于 `charter drift`（后者比较声明的文件与 git 修改的文件）。Opt-in，默认 `false`，确保未使用章程模式的项目不受影响。目前仅在非 `--staged` 模式下生效；staged 模式的章程验证将在 cli-3.10.0 中加入。 |
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

管理**章程（Charter）**：事前声明、事后审计的有界工作单元。一个章程将声明性范围（要修改的文件、风险、可执行的验证命令）与事后审计锚点（漂移检测、多模型审计）配对。章程位于 `.straymark/charters/NN-slug.md`（项目根目录级别，**不在** `.straymark/` 之下）。

> **命名历史。**在使该模式定型的 Sentinel `/plan-audit` 实验中（2026 年 4 月，6 个周期），章程被称为 *Plans*。StrayMark CLI 从此版本开始使用 **Charter** 以避免与 GitHub SpecKit 的 `plan.md` 命名冲突。Sentinel 的历史文件刻意保留 "Plan" 命名。完整的概念范围与重命名理由见 `Propuesta/que-es-un-charter.md`。

**子命令：**

- `straymark charter new` — 从框架模板创建新的章程
- `straymark charter list` — 用可选过滤器枚举章程
- `straymark charter status` — 显示章程详情，或最近的 5 个章程
- `straymark charter close` *(cli-3.7.0+)* — 记录执行后遥测并将章程移至 `closed`
- `straymark charter drift` *(cli-3.7.0+)* — 检查文件 vs 提交的漂移，带 AILOG 感知抑制和 Batch Ledger 门控 *(门控在 cli-3.13.2 添加)*
- `straymark charter batch-complete` *(cli-3.13.0+, fw-4.14.0+)* — 在 AILOG 的 `## Batch Ledger` 中将章程批次标记为已完成
- `straymark charter audit` *(cli-3.8.0+)* — 编排多模型外部审计（仅编排，详见下文）
- `straymark charter refresh-suggest` *(cli-3.14.0+, fw-4.16.0+)* — 多 Charter 模块的预声明 SpecKit 刷新启发式建议（详见 [CHARTER-CHAIN-EVOLUTION.md](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/i18n/zh-CN/CHARTER-CHAIN-EVOLUTION.md) 模式 1）。默认阈值 `r_n_plus_one_emergent_count` > 6；仅可通过 `--threshold N` 覆盖（v0.2 中 `config.yml` 暂无等效字段）。
- `straymark charter amend` *(cli-3.14.0+, fw-4.16.0+)* — 关闭后审计驱动的 Batch N.4 修订脚手架（在原 execute 分支上落地，不开新 Charter — 详见 [CHARTER-CHAIN-EVOLUTION.md](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/i18n/zh-CN/CHARTER-CHAIN-EVOLUTION.md) 模式 2）

#### `straymark charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <title>] [path]`

从框架模板将章程创建到 `.straymark/charters/NN-slug.md`。如果未传入 `--title`，会以交互方式提示。两个来源标志在 clap 级别互斥。

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

  ── Follow-ups ──
  ✔ Extracted 2 follow-ups from 1 AILOG(s) into the registry (`## Bucket: ready`).
    Review against the TDE-promotion criteria (AGENT-RULES.md §3): ...
  Promote FU-078 — wire the retry budget into the sync loop to a TDE? [y/N]
```

**Follow-up 调和** *(cli-3.22.0+,RFC #135 Tier 3)*。**交互式**关闭后,该命令对最近写入的 AILOG 运行默认的 `followups drift` 扫描(已提交的 git 范围 ∪ 工作树),将**尚未在注册表中**的 `§Follow-ups` / `R<N> (new)` 内容提取到 `## Bucket: ready`,然后针对 §3 的四条标准(先前 Charter 的遗留、跨多个模块/Charter、需要专用 Charter、需要人工排优先级)**逐条提供 TDE 提升**。拒绝某个提示会保留该 follow-up 的提取状态(已捕获进注册表,只是未提升);接受则运行 `followups promote` 流程(创建带 `promoted_from_followup` 溯源的 TDE)。当项目没有 follow-ups 注册表、或没有任何未提取内容时为**空操作**,并在 `--from-template` 路径上**跳过**(无交互提示上下文)—— 在那里请运行 `straymark followups drift --apply`。

#### `straymark charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--no-batch-ledger-check] [--path <dir>]`

在章程关闭时检测文件 vs 提交漂移。**自 cli-3.23.0 起为原生 Rust**(#237)—— 不再委托给(现已弃用的)`.straymark/scripts/check-charter-drift.sh`,因此可在 Windows 原生环境运行(无需 WSL、无需 Git Bash)。零误报特性(Sentinel PLAN-05 回顾性 + PLAN-06 前瞻性)由脚本等价性测试套件保留。CLI 在原始脚本之上的附加价值是 **AILOG 感知**：报告为"已声明但未修改"的路径，如果出现在被章程 `originating_ailogs` 引用的任何 AILOG 的 `## Risk` / `## Riesgos` / `## 风险` 节中，则被静默。使用 `--no-ailog-suppress` 来禁用。

**Batch Ledger 门控** *(cli-3.13.0+)*。当章程状态为 `in-progress` 或 `closed` 时，该命令还检查每个 originating AILOG 中仍为 `(pending)` 的 `## Batch Ledger` 条目，并以清晰的诊断列出缺失的批次。没有 ledger 的 AILOG 不参与 — 该章节是 opt-in 的。使用 `--no-batch-ledger-check` 来绕过（用于在 close 后合并 ledger 的 adopter）。

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `CHARTER-ID` | — | 与 `charter status` 相同的解析规则。 |
| `--range` | `HEAD~1..HEAD` | 要检查的 git 修订范围。 |
| `--no-ailog-suppress` *(cli-3.10.0+ 始终输出确认 INFO 行)* | false | 禁用 AILOG 感知抑制（显示每条已声明但被遗漏的路径）。传入此标志时，CLI 始终打印 `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` 行 — 即使 N=0 — 以便诊断模式即使在干净运行时也在输出中可见。 |
| `--no-batch-ledger-check` *(cli-3.13.0+)* | false | 禁用 Batch Ledger 门控。当章程的 AILOG 在 close 时显式选择不使用 ledger 模式时使用。 |
| `--path` | `.` | 目标项目目录。 |

**退出码：** `0` 没有漂移（或仅 AILOG 抑制）且无 pending 批次；`1` 存在未计入的漂移或任何 `### Batch N` 仍为 `(pending)`；`2` 用法错误（章程未找到、bash 缺失等）。

**示例：**

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

> **平台说明。** 漂移检查委托给 `bash`。在 Linux/macOS/WSL/Git Bash 上开箱即用。Windows 原生且无 WSL 时需安装 Git Bash；纯 Rust fallback 在路线图上但不在 fw-4.6.x 中。

#### 已声明路径的通配符支持 *(fw-4.9.0+)*

漂移检查在 `## Files to modify` 中解析两种通配符形式：

| 形式 | 示例 | 用例 |
|---|---|---|
| 省略号 | `` `.straymark/07-ai-audit/agent-logs/AILOG-...md` `` | 任何带该前缀的修改路径满足通配符。历史上用于执行期间会创建未知数量 AILOG 的情况。 |
| Glob | `` `AILOG-*.md` `` 或 `` `src/services/foo-*.rs` `` | 任何匹配该 glob（`*` → `.*`）的修改路径满足通配符。用于参数化集合被触动的批量章程声明。在 fw-4.9.0 中加入，源于 Sentinel CHARTER-04 暴露的摩擦（[issue #81](https://github.com/StrangeDaysTech/straymark/issues/81)）。 |

两种形式都双向处理：声明的通配符既抑制"已声明但未修改"警告（当至少一个匹配文件被修改时），也抑制"已修改但未声明"警告（当一个修改路径匹配某个已声明通配符时）。

#### 设计：治理路径始终在 scope 内

`.straymark/charters/*` 和 `.straymark/07-ai-audit/*` 下的路径**永远不会**被报告为"已修改但未声明"。这是有意的设计 — 当章程本身或执行的 AILOG 被触动时，这些路径总是合法的。在 Sentinel CHARTER-04 中实证验证：一次意外的 `git add -A` 暂存了无关的用户未跟踪文件（`.claude/skills/`、`cmd/sentinel/sentinel`）；该规则正确抑制了治理噪声而没有掩盖真正的项目文件扩展（[issue #81 W2](https://github.com/StrangeDaysTech/straymark/issues/81#issuecomment-update)）。

如果你运行的章程显式 scope 是治理 churn（例如仅触动 `.straymark/07-ai-audit/` 的批量批准章程），漂移检查将报告 0 个修改文件，你需要通过阅读 AILOG 来验证 scope。一个 `--strict-scope` 标志（禁用"始终在 scope"规则）在桌面上，用于未来 minor 版本，前提是真实的 adopter 报告这种不对称为摩擦。

#### `straymark charter batch-complete <CHARTER-ID> <N> [--note <body>] [--non-interactive] [--path <dir>]`

*自 **cli-3.13.0** + **fw-4.14.0** 起可用。*

在 originating AILOG 的 `## Batch Ledger` 中将一个章程批次标记为已完成。该命令将 `### Batch <N>` 下的 `(pending)` 占位符替换为以交互方式（默认）或通过 `--note`（一次性 / 脚本化）捕获的批次说明。关闭时的 drift 门控（`straymark charter drift`）拒绝任何仍为 `(pending)` 的 `### Batch N`，使按批次的更新变为强制性而非纪律提醒。

**何时使用。** 仅用于跨越 3+ 批次或 >1 天执行的多批次章程。单批次 AILOG 在 `## 执行的操作` 中记录一切，完全跳过 ledger。Charter 模板的 §Tasks 提醒作者在适用时维护 ledger。

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `CHARTER-ID` | — | 与 `charter status` 相同的解析规则。 |
| `N` | — | 批次号（从 1 开始）— 与 AILOG `## Batch Ledger` 中的 `### Batch <N>` 标题匹配。 |
| `--note <body>` | — | 预填充的批次说明正文。使用此标志，该命令非交互式地写入说明并跳过 prompt。为 agent 和脚本设计。 |
| `--non-interactive` | false | 禁用 prompt。需要 `--note`（缺少 `--note` 会中止而非挂起）。 |
| `--path` | `.` | 目标项目目录。 |

该命令读取章程 frontmatter 的 `originating_ailogs[0]` 来定位目标 AILOG。当出现以下情况时，以清晰错误拒绝执行：

- 章程没有 `originating_ailogs` 条目（无法解析目标文件）；
- AILOG 文件在 `.straymark/07-ai-audit/agent-logs/` 中缺失；
- AILOG 没有 `## Batch Ledger` 章节（先添加该章节，或如果是单批次 Charter 则跳过命令）；
- `## Batch Ledger` 下没有 `### Batch <N>` 标题；
- 目标批次已完成（拒绝覆盖 — 如需更正请手动编辑 AILOG）。

**交互流程。** 三个 prompt：touched 文件（单行）、添加的 tests 或状态（单行）、多行设计说明（用单行 `.` 或 Ctrl-D 结束）。空字段在输出中被跳过。结果正文写入 `### Batch <N>` 标题下；`(pending)` 占位符被替换。

**示例：**

```bash
# 交互式 — 人类粘贴批次说明
$ straymark charter batch-complete CHARTER-17 5
✔ AILOG: .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md
✔ ### Batch 5 — Migration 022 + handlers

OK `### Batch 5` written.
Reminder: `git add .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md` before pushing.

# 一次性 — agent / 脚本
$ straymark charter batch-complete CHARTER-17 5 \
    --note "Migration 022 + handlers. 文件: migrations/022.sql, services/handler_x.go. Tests passing. R8 (CHECK constraint missing ARCHIVED), 原子修复."
OK `### Batch 5` written.
```

**Workflow 集成。** 根据 Charter §Tasks 模板的指引，在批次的 commit 落地**之后**但推送**之前**运行 `batch-complete`。AILOG 的更新和它所记录的工作随后乘同一次 push。关闭时的 drift 门控（`straymark charter drift CHARTER-NN`）拒绝任何 pending 批次并打印列表 — 将"忘记更新 ledger"从审计轨迹的静默侵蚀变为硬失败。

#### `straymark charter audit <CHARTER-ID> [--range <REV..REV>] [--prepare | --merge-reports] [--merge-into <PATH>] [--path <dir>]` {#straymark-charter-audit}

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
| `--range` | `origin/main..HEAD`（回退到 `origin/master..HEAD`，再到 `HEAD~1..HEAD` 并告警） | 审计员将审查的 git 修订范围。**多批次陷阱：** 对于早期阶段已合并到基础分支的章程，做按阶段审计时，默认的 `origin/main..HEAD` 会*排除*已合并的提交，从而静默地覆盖不足——请传入显式的 `--range <章程首个提交>..HEAD` 以覆盖整个阶段。当 `--prepare` 检测到已完成批次且未指定显式范围时会打印告警。 |
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

### `straymark followups <subcommand>` *(cli-3.19.0+)*

管理**后续事项积压注册表**（`.straymark/follow-ups-backlog.md`）—— 这是跨 AILOG 聚合 `§Follow-ups` 与 `R<N> (new, not in Charter)` 条目的一等产物。Schema：`.straymark/schemas/follow-ups-backlog.schema.v1.json`（实验性 v1）。约定：`FOLLOW-UPS-BACKLOG-PATTERN.md` 与 `STRAYMARK.md §16`；随附的代理指令见 `AGENT-RULES.md §13`。

解析是**宽容的**：v0 注册表（fw-4.21.0 之前）可被无误读取；首个写入命令（`drift --apply` —— 即使没有可提取内容,cli-3.20.0+ ——、`recount` 或 `promote`）会就地、非破坏性地将其升级为 v1。frontmatter 的 `total_*` 计数器由 **CLI 拥有** —— 每次写入时重新计算；切勿手工编辑。

- `straymark followups list` — 枚举条目 *(cli-3.19.0+)*
- `straymark followups status` — 注册表概览 / 条目详情 *(cli-3.19.0+)*
- `straymark followups drift` — 将注册表与 AILOG 同步（已弃用的 adopter 侧 `check-followups-drift.sh` 的原生替代）*(cli-3.19.0+)*
- `straymark followups recount` — 手动分诊会话后重新计算 CLI 拥有的计数器 *(cli-3.20.0+)*
- `straymark followups promote` — 将条目提升为 TDE 文档 *(cli-3.19.0+)*

#### `straymark followups list [--bucket <name>] [--status <s>] [--severity <s>] [--label <tag>] [path]`

条目表格：FU id、status、severity、bucket、destination、description。解析告警（格式错误的 `### FU-` 标题）输出到 stderr 而不会使命令失败。

| Flag | Default | Description |
|------|---------|-------------|
| `--bucket <name>` | — | 按 bucket 过滤：`ready`、`time-triggered`、`charter-triggered`、`phase-blocked`、`operational` |
| `--status <s>` | — | 按 status 过滤：`open`、`in-progress`、`suspected-closed`、`closed`、`superseded`、`promoted` |
| `--severity <s>` | — | 按 severity 过滤：`normal`、`blocking` |
| `--label <tag>` | — | 按 label 过滤（对单个 tag 的大小写不敏感精确匹配） |

```bash
$ straymark followups list --severity blocking
  FU      STATUS  SEV       BUCKET  DEST          DESCRIPTION
  FU-010  open    blocking  ready   mini-charter  Harden staging probe
```

#### `straymark followups status [FU-NNN] [--path <dir>]`

不带 id：注册表概览 —— 计数器从实际条目状态**即时重新计算**（即使文件 frontmatter 已陈旧也可信；分歧会被标记）、按 bucket 的 open 明细、blocking 与 suspected-closed 告警、咨询性 schema 校验。带 id：该条目的完整字段详情。

#### `straymark followups drift [--apply] [--scan-all] [--range <REV..REV>] [--path <dir>]`

检测其后续事项内容尚未提取到注册表的 AILOG。粒度为**按 AILOG**（frontmatter 的 `fully_extracted_ailogs` 列表）—— 经实证验证的设计选择（在参考 adopter 的 76 个 AILOG 中 0 误报）。

| Flag | Default | Description |
|------|---------|-------------|
| *(default)* | — | 扫描在 `origin/main..HEAD` 中变更的 AILOG（回退到 `origin/master..HEAD`，再回退到带告警的 `HEAD~1..HEAD`）。有漂移时告警并 **exit 1**。 |
| `--apply` | off | 将缺失的条目提取到 `## Bucket: ready`，使用自动编号的 `FU-NNN` id，把 AILOG id 追加到 `fully_extracted_ailogs`，**重新计算计数器**，并就地把 v0 注册表升级为 v1。注册表不存在时从框架模板播种。自 cli-3.20.0 起,**即使没有可提取的内容也会重新计算计数器**(#222 Finding 1)。 |
| `--scan-all` | off | 扫描项目中的每一个 AILOG，而非 git 范围。 |
| `--range <REV..REV>` | — | 默认扫描的显式 git 范围。 |

**抗噪声精炼**（issue #214 Signal 1）：其 AILOG 文本带有显式闭合标记的条目 —— `closed in-Charter`、`fixed in batch N`、反引号包裹的 commit hash，或 *(cli-3.20.0+,#222 Finding 2)* born-resolved 习语（关闭动词 `updated`/`corrected`/`remediated`/`resolved`/`fixed`/`closed` 后跟 `in this PR` / `in this commit`,如 `updated atomically in this PR`）—— 会被提取为 **`suspected-closed`** 而非 `open`，从而避免已解决的工作以 TBD 噪声污染 `ready` bucket。操作员在下次分诊时确认（→ `closed`）或重开。

```bash
$ straymark followups drift --scan-all --apply
✓ Extracted 4 entries from 1 AILOG(s) into `## Bucket: ready`.
  ! 1 extracted as suspected-closed (closure marker in source AILOG) — confirm at the next triage.
  Counters recomputed: 3 open / 1 suspected-closed / 0 promoted (total 4).
```

#### `straymark followups recount [--path <dir>]` *(cli-3.20.0+)*

根据实际条目状态重新计算 CLI 拥有的 `total_*` 计数器并重写 frontmatter —— 不扫描 AILOG、不提取、不触碰条目。这是在**手动分诊会话**之后（按照被认可的分诊/消费生命周期手动翻转状态,没有可提取或可提升的内容 —— #222 Finding 1,首个外部 adopter）符合 §13 地调和计数器的方式。幂等:第二次运行会报告计数器已同步。像其他写入命令一样,就地将 v0 注册表升级到 v1。

```bash
$ straymark followups recount
✓ Counters recomputed: 0 open / 0 suspected-closed / 2 promoted (total 2).
```

#### `straymark followups promote <FU-NNN> [--title <title>] [--path <dir>]`

自动化 FU → TDE 的提升（`FOLLOW-UPS-BACKLOG-PATTERN.md` §Promotion to TDE）：从框架模板创建带 `promoted_from_followup: FU-NNN` 溯源的 TDE 文档，将条目翻转为 `Status: promoted` 并使 `Destination`/`Promoted to` 指向该 TDE id，再重新计算计数器。非交互式（对 agent 友好）；除非 `--title` 覆盖，否则 FU 描述将成为 TDE 标题。优先级排序与分配仍由人工决定（`AGENT-RULES.md §3`）。

```bash
$ straymark followups promote FU-010
✓ FU-010 promoted → TDE-2026-06-04-001
  TDE created: .straymark/06-evolution/technical-debt/TDE-2026-06-04-001-harden-staging-probe.md
```

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

> **由 arborist-metrics 驱动：** 认知复杂度与圈复杂度因子由 [`arborist-metrics`](https://github.com/StrangeDaysTech/arborist-metrics/) 计算 —— 这是我们开源的 Rust 多语言代码度量库，同样由 StrangeDaysTech S.A.S. de C.V. 开发。也可在 [crates.io](https://crates.io/crates/arborist-metrics) 作为独立库使用。

---

### `straymark analyze declared-vs-wired [path] [--profile <名称> | --declared-glob … --wired-glob … --declared-pattern … --wired-pattern …] [--show-orphans] [--output <格式>]` *(cli-3.18.0+)*

标记在实现侧**没有接线对应物**的已声明符号 —— 即"声明了表层但未接线"反模式的子类 5（客户端 IPC/RPC proxy 方法 vs 服务端接口）。由 LNXDrive 的 N=2 验证（发现 [#209](https://github.com/StrangeDaysTech/straymark/issues/209)/[#210](https://github.com/StrangeDaysTech/straymark/issues/210)）结晶而来；见 `.straymark/00-governance/POLISH-CHARTER-PATTERN.md`。

它是一个**由配置驱动的 set-difference**，本质上与语言/IPC 无关：你提供一个*声明*侧和一个*接线*侧，各为 `(glob, regex)` 对；每个 regex 的**捕获组 1** 即符号名。命令报告 **D \ W**（已声明但未接线），以及在 `--show-orphans` 下报告 **W \ D**（已接线但从未声明）。

**参数与标志：**

| 参数/标志 | 默认 | 说明 |
|---------------|---------|-------------|
| `path` | `.` | 目标目录 |
| `--profile` | — | 来自 `.straymark/config.yml` 的命名 profile（`declared_vs_wired.profiles`）。可替代四个内联标志。 |
| `--declared-glob` | — | 声明文件的 glob（相对于 `path`，proxy/stub/客户端）。 |
| `--wired-glob` | — | 实现文件的 glob（相对于 `path`，daemon/服务端接口）。 |
| `--declared-pattern` | — | 对声明文件的 regex；**捕获组 1** = 符号名。 |
| `--wired-pattern` | — | 对接线文件的 regex；**捕获组 1** = 符号名。 |
| `--show-orphans` | off | 同时报告已接线但从未声明的符号（`W \ D`）。 |
| `--output` | `text` | `text`、`json` 或 `markdown`。 |

必须传入 `--profile` **或** 四个内联 glob/pattern 之一。

**退出码：** `0` 干净（每个已声明符号都已接线）；`1` 至少一个已声明符号没有接线对应物（一个发现）—— 适合作为 CI 门。

**配置**（可选，在 `.straymark/config.yml`）：

```yaml
declared_vs_wired:
  profiles:
    - name: dbus
      declared_glob: "client/**/*.rs"     # GTK 客户端的 D-Bus proxy
      declared_pattern: "fn (\\w+)"
      wired_glob: "daemon/src/interface.rs" # daemon 已实现的接口
      wired_pattern: "fn (\\w+)"
```

**示例：**

```bash
# 内联（一次性）
$ straymark analyze declared-vs-wired \
    --declared-glob "client/**/*.rs" --declared-pattern 'fn (\w+)' \
    --wired-glob "daemon/**/*.rs"    --wired-pattern 'fn (\w+)'

# 命名 profile（提交一次），JSON 供 CI 使用
$ straymark analyze declared-vs-wired --profile dbus --output json
```

> **v0 范围：** 这是机械化可处理的 cross-stack 检查（子类 5）。子类 1–4 的基于 AST 的变体（env-var 文档、metric instrument、HTML embed、公共路由标记）以及动态 runtime 检查仍是 project-local —— 见模式文档的未决问题。

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
  Framework version: fw-4.15.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/straymark
  Website:           https://strangedays.tech
```

---

### `straymark architecture <generate|sync|validate>` *(cli-3.25.0+，实验性)* {#straymark-architecture-generatesyncvalidate-cli-3250-experimental}

编写并维护驱动 Loom 架构计划视图（Spec 002）的**架构模型**：一个语义化的 `model.yml`（组件 → 文件 glob + 层级 + 链接），搭配一份 `plan.drawio` 蓝图，二者由一个稳定的 `component_id` 关联（即 BIM 中"模型与图纸"的拆分）。该模型之上的文本投影是 `straymark status --where`；其可视化叠加层是 Loom（A2）。

> ⚠️ **实验性（Loom v0）。** 模型 schema、DrawIO 约定以及命令接口都可能在没有弃用周期的情况下更改。三个子命令默认都操作 `.straymark/architecture/`；`--out <dir>` 可覆盖该路径。

**`straymark architecture generate [path] [--force] [--out <dir>]`** —— 通过挖掘你的代码库结构（每个源目录对应一个组件，glob `dir/**`）并结合 ADR 信号（C4 Mermaid 图 + "Affected Components" 表可改进标签并添加链接）来写出 `model.yml` + `plan.drawio` 的初稿。挖掘器会向下穿过*容器*目录（`internal/`、`src/`、`pkg/`、`lib/`、`app/`、`modules/` 等），使得像 `internal/modules/<x>` 这样的树会拆分为真实的组件，而不是挤成一个大块；并会跳过构建*脚手架*（`src/main/java`、`src/main/kotlin` 等），从而把 Maven/Gradle 模块归属到其模块目录，而不是塌缩成单个 `main` 方块。组件会落在占位的 `unassigned` 层，而 `.straymark` 的 00–09 阶段为层级列表提供种子 —— 之后你再手动精修（重新分配/重命名层级、收紧 glob、添加链接）。`--force` 会覆盖已有的产物。

> **按你的技术栈调整扫描（`config.yml`）。** 种子开箱即用就具备语言与结构感知，但你可以通过 `.straymark/config.yml` 中的 `architecture:` 区块为非默认生态扩展它。这些列表是**叠加式**的 —— 它们扩展内置默认值，绝不替换：
>
> ```yaml
> architecture:
>   source_extensions:   [rb, ex, exs, gleam]   # 添加种子应统计的语言
>   container_dirs:      [services, domains]     # 额外要向下穿过的目录
>   scaffolding_prefixes: [src/main/java]        # 额外要跳过的构建脚手架（已含 Maven/Gradle 默认值）
>   excluded_dirs:       [generated]             # 额外要跳过的目录
> ```
>
> 状态叠加本身（glob → 文件匹配）已经与语言无关；这里只塑造生成的种子。

**`straymark architecture sync [path] [--out <dir>] [--apply]`** —— **仅追加**的对账：检测尚未被模型覆盖的新顶层源目录 / ADR 组件，并将它们追加到 `model.yml` + `plan.drawio`，**绝不**破坏人工编辑或 DrawIO 几何布局。默认为试运行；`--apply` 才会写入。

**`straymark architecture validate [path] [--out <dir>] [--output <text|json|markdown>]`** —— 报告模型与计划之间的完整性信号：**undrawn**（组件没有对应的 DrawIO 单元格）、**unmodeled**（DrawIO 单元格在模型中不存在）、**empty**（glob 在磁盘上未匹配到任何文件）。一旦发现任何信号即**以 1 退出**（可用于 CI 门禁）。当 `plan.drawio` 缺失时，降级为仅做 glob 覆盖检查。

| 子命令 | 关键标志 | 是否写入？ |
|---|---|---|
| `generate` | `--force`、`--out` | 是（没有 `--force` 时拒绝覆盖） |
| `sync` | `--apply`、`--out` | 仅在 `--apply` 时（仅追加） |
| `validate` | `--output`、`--out` | 否（只读；任何信号都以 1 退出） |

**示例：**

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

### `straymark loom serve [path] [--port <端口>] [--no-open]` *(cli-3.24.0+，实验性)*

启动 **Loom** —— 实验性知识图谱可视化服务器：一个仅回环、只读的 Web 仪表板，将项目的 StrayMark 文档渲染为实时力导向图（节点按文档类型着色、按连接度调整大小；选中节点会点亮其完整的关系线索；对被监视的 `.md` 文件的编辑会在一秒内更新已打开的浏览器）。

> ⚠️ **Loom 是实验性的（v0）。** 其 API、CLI 接口乃至其存在本身都可能在没有弃用周期的情况下更改或移除。`straymark-loom` 二进制文件**不**捆绑在 CLI 中 —— 首次使用时从 `loom-*` GitHub 发布按需下载，并缓存在 `~/.straymark/bin/`。下载门槛*就是*选择加入的边界。

**参数和标志：**

| 参数/标志 | 默认值 | 描述 |
|---|---|---|
| `path` | `.`（当前目录） | 项目目录。如果存在 `.straymark/` 子目录则监视它，否则监视目录本身 |
| `--port` | `7700` | 在 `127.0.0.1` 上服务的端口 |
| `--no-open` | 关 | 不自动打开浏览器 |

**安全态势：** 仅绑定 `127.0.0.1`（否则拒绝启动），拒绝非回环 `Host` 头（防 DNS 重绑定），且从不写入被监视的目录。

**示例：**

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

StrayMark 提供一组 skills（slash 命令）供 AI 助手内使用（Claude Code、Gemini Code、Codex CLI、Cursor、通用 Agent 运行时）。每个 skill 在 `straymark init` 时以 4 种平行形式安装：

- `.claude/skills/<skill>/SKILL.md`（Claude — frontmatter 含 `allowed-tools`）
- `.gemini/skills/<skill>/SKILL.md`（Gemini — frontmatter 不含 `allowed-tools`）
- `.codex/skills/<skill>/SKILL.md` *(fw-4.19.0+)*（Codex — 最小 frontmatter，仅 `name`+`description`；由 Claude 变体生成）
- `.agent/workflows/<skill>.md`（通用 Agent — 仅 `description` frontmatter）

Claude 和 Gemini 直接从项目树发现 skills。**Codex 从 `~/.codex/skills/`（用户级）读取 skills，而不是从项目树读取** —— 在 `straymark init` 之后（以及每次框架更新之后）运行一次 `straymark install-skills --agent codex`，从 `.codex/skills/` 填充该目录。

| Skill | 用途 | 产生的文件 |
|---|---|---|
| `/straymark-status` | 检查近期变更的文档合规状态。 | 无（read-only） |
| `/straymark-new` | 交互式创建任意类型的文档。从上下文建议最佳匹配。 | `.straymark/<type-dir>/<TYPE>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | 快速 AILOG 创建快捷方式。 | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | 快速 AIDEC 创建快捷方式。 | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | 快速 ADR 创建快捷方式。 | `.straymark/02-design/decisions/ADR-*.md` |
| `/straymark-mcard` | 交互式 Model Card 创建流程。 | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | 交互式 SEC（安全评估）流程。 | `.straymark/08-security/SEC-*.md` |
| `/straymark-charter-new` *(fw-4.12.0+)* | 搭建 Charter —— 声明式事前工作单元。封装 `straymark charter new`（slug 派生、顺序编号、模板替换）；skill 负责来源/工作量选择以及声明前侦察纪律。 | `.straymark/charters/NN-slug.md` |
| `/straymark-followups` *(fw-4.22.0+)* | 维护 follow-ups backlog 注册表（`AGENT-RULES.md §13`）：会话开始时从规范注册表回答“有什么待办？”，提交前运行 `followups drift --apply` 使注册表与 AILOG 同一提交，Charter 关闭后分诊以及操作员审批的 `promote`。`straymark followups` 之上的薄封装 —— 绝不手动编辑 CLI 拥有的计数器。 | 无直接产物（写入经由 `straymark followups drift --apply` / `promote` → `.straymark/follow-ups-backlog.md`，promote 时生成 TDE） |
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
