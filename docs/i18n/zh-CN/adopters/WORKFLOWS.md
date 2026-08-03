# StrayMark - 推荐工作流

**日常使用 StrayMark 的模式和节奏。**


---

## 目录

1. [初始设置之后](#初始设置之后)
2. [日常开发](#日常开发)
3. [保持 StrayMark 更新](#保持-straymark-更新)
4. [检查项目状态](#检查项目状态)
5. [使用 Skills（主动文档）](#使用-skills主动文档)
6. [团队模式](#团队模式)
7. [理解版本](#理解版本)

---

## 初始设置之后

你已运行 `straymark init .` 并提交了结果。接下来呢？

1. **用你的 AI 编码助手打开项目**（Claude Code、Cursor、Gemini CLI 等）
2. 助手会**自动读取** StrayMark 指令（`CLAUDE.md`、`GEMINI.md` 等）
3. 从此刻起，助手会在正常工作流中**在 `.straymark/` 中创建文档**
4. **无需额外配置** — StrayMark 通过指令文件被动运行

---

## 日常开发

### 被动循环

1. 正常使用你的 AI 助手工作——编写功能、修复 Bug、重构
2. AI 根据治理规则在 `.straymark/` 中创建文档：
   - **AILOG** 用于重大实现（>10 行变更）
   - **AIDEC** 在多个方案间选择时
   - **ADR** 用于架构决策
   - **ETH** 当出现伦理考量时
3. 审查标记为 `review_required: true` 的文档
4. 将文档与相应的代码变更一起提交

### 何时手动创建文档

在以下情况使用主动系统（Skills）：

- AI 遗漏了对重大变更的记录
- 你（人类）做了一个应该记录的决策
- 你想创建 REQ、TES、TDE 或 INC 文档
- 你想检查文档合规状态

---

## 保持 StrayMark 更新

### 建议频率

- **每月**或当你在 GitHub 上看到新版本时
- 查看[发布页面](https://github.com/StrangeDaysTech/straymark/releases)了解变更日志

### 更新命令

| 目标 | 命令 |
|------|------|
| 同时更新 Framework 和 CLI | `straymark update` |
| 仅更新模板和治理文档 | `straymark update-framework` |
| 仅更新 CLI 二进制文件 | `straymark update-cli` |

Framework 和 CLI 有**独立的版本** — 你可以单独更新其中一个。参见[理解版本](#理解版本)。

### 更新之后

1. 检查指令文件和治理文档的变更
2. 提交更新的文件：`git add .straymark/ && git commit -m "chore: update StrayMark framework"`
3. 如果你自定义了 Framework 文件，检查是否有冲突

---

## 检查项目状态

### CLI 状态

```bash
straymark status
```

显示：Framework 版本、CLI 版本、目录结构完整性和按类型统计的文档数据。用它来验证安装是否健康。

### 文档合规（Skill）

```bash
/straymark-status
```

`/straymark-status` Skill（在 Claude Code 和 Gemini CLI 中可用）分析：

- 哪些近期代码变更缺少对应的文档
- 文档与治理规则的合规情况
- 整体文档健康状态

### 看见项目（Loom，实验性）

要以*可视化*方式回答"我们在哪里？"，**EXPERIMENTAL** 的 Loom 仪表盘会把你的项目渲染为一张实时的文档图，外加一张"你在这里"的架构地图：

```bash
straymark status --where     # textual "you are here" (no server)
straymark loom serve         # the visual dashboard (downloads the loom-* binary on first use)
```

架构地图由一个你先生成、再精修的模型驱动——推荐的 `generate → refine → validate → sync → serve` 工作流（精修步骤可以由人或 AI 辅助完成）参见 **[Loom 与架构地图指南](./LOOM.md)**。

### 声明式工作路由（Baton，实验性）

**EXPERIMENTAL** 的 Baton 根据声明的 `work_verb` 对你的工作单元分类，并给出成本敏感的路由建议——只读、仅建议：

```bash
straymark-baton classify .          # 你已记录工作单元的声明类别
straymark-baton route . --dry-run   # 层级路由建议；绝不执行任何操作
```

该二进制是仅存于 GitHub Release 的资产（与 Loom 相同）——下载方式及 Track C 对采纳者的要求参见 **[Baton 指南](./BATON.md)**。

---

## 使用 Skills（主动文档）

StrayMark 有两个文档系统：

| 系统 | 工作方式 | 何时使用 |
|------|----------|----------|
| **被动** | AI 通过指令文件自动记录 | 默认 — 自动发生 |
| **主动** | 用户调用 Skills 创建文档 | 当被动系统遗漏时，或用于人工决策 |

### 可用 Skills

| Skill | 用途 |
|-------|------|
| `/straymark-status` | 检查文档合规状态 |
| `/straymark-new` | 创建任意类型的文档（建议最佳匹配） |
| `/straymark-ailog` | 快速创建 AILOG |
| `/straymark-aidec` | 快速创建 AIDEC |
| `/straymark-adr` | 快速创建 ADR |
| `/straymark-audit-prompt CHARTER-XX` *(fw-4.8.0+，在 fw-4.9.0 中重构)* | 在规范路径 `.straymark/audits/<id>/audit-prompt.md` 处生成统一的审计 prompt。封装 `straymark charter audit --prepare`。操作员随后打开 N 个审计员 CLI 并在每个中运行 `/straymark-audit-execute` — 无需复制/粘贴。 |
| `/straymark-audit-execute [CHARTER-XX]` *(fw-4.9.0+)* | **在审计员 CLI 中运行**（gemini-cli、claude-cli、copilot-cli、codex-cli）。从磁盘读取 prompt，使用 tool use 进行审计并引用 `path:line`，写入以审计员模型 ID 为键的 report。参数可选 — 自动发现此模型待处理的 prompts。 |
| `/straymark-audit-review CHARTER-XX` *(fw-4.8.0+，在 fw-4.9.0 中扩展)* | `audit-prompt` 的对应。读取 N 个 reports，对 findings 与实际代码交叉验证，生成 `review.md` 六节合并分析（执行摘要 / 范围 / 按审计员评估 / 修复计划 P0-P4 / 丢弃 / 审计员评分），并将 `external_audit:` YAML 合并到遥测。 |
| `/straymark-architecture` *(fw-4.29.0+，实验性)* | 在一次引导式流程中驱动架构模型的 `generate → refine → validate` 弧线：生成种子、将组件重新分配到真实的层、连接依赖 `links`、同步 DrawIO 使 2D 显示箭头，并迭代 `validate` 至通过。手工 DrawIO 精炼的 agent 原生对应。 |
| `/straymark-architecture-sync` *(fw-4.29.0+，实验性)* | 封装 `straymark architecture sync`（仅追加），随代码增长保持已策展模型最新——dry-run、展示新目录/组件、确认、应用、重新 validate。绝不从零重新精炼。 |
| `/straymark-loom` *(fw-4.29.0+，实验性)* | 从 agent 窗口掌管 Loom 服务器生命周期（up / down / status）。在后台启动 `straymark loom serve --no-open` 并向操作者交付一个链接——通往 2D/3D 架构视图的无终端路径。 |

完整 Skill 详情参见 [README](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/zh-CN/README.md#skills)。

### Charter 审计检查点 *(fw-4.8.0+)*

在与人共同实现 Charter 时，Agent 会在一个特定时刻主动提议外部审计：当实现完成、drift 干净，且 `charter close` 尚未调用时。推荐基于 Charter 的风险面和复杂度给出 是/否（完整启发式见 `.straymark/00-governance/AGENT-RULES.md` §12）。

外部审计**完全可选**且**永不强制**。Charter 的声明性范围 + drift check + AILOG 纪律已为关闭提供了足够严格的支撑。审计在 Charter 触及安全面、引入新组件或 diff 中存在高复杂度函数时增加跨模型信号。如果你的情况下成本（2-3 个 LLM 审计员）与价值不匹配，可以自由拒绝。

---

## 团队模式

### PR 审查

- 检查重大代码变更是否包含 `.straymark/` 中的对应文档
- 审查任何标记为 `review_required: true` 的文档
- 验证 AILOG 是否准确描述了 AI 所做的工作

### 新成员入职

1. 引导他们查看 `.straymark/QUICK-REFERENCE.md` 快速了解概况
2. 让他们阅读近期 ADR 以理解架构背景
3. 展示近期功能的 AILOG，让他们了解文档在实践中如何运作

### Sprint 回顾

- 回顾 Sprint 中的 AILOG 和 AIDEC，了解 AI 贡献模式
- 识别应该记录但未记录的决策
- 检查 TDE 文档了解累积的技术债务

### 共享 AI 助手

当多个团队成员在同一项目中使用 AI 助手时：

- 每个助手会话产生各自的文档
- 元数据中的 `agent` 字段标识每个文档由哪个助手创建
- 在 PR 审查中关注重叠或矛盾的 AIDEC

---

## 中国合规工作流 *(opt-in)*

如果项目在中国大陆运营或处理中国大陆用户的个人信息,启用 china 范围并按以下流程操作。

### 一次性设置

1. 编辑 `.straymark/config.yml` 并将 `china` 加入 `regional_scope`:
   ```yaml
   regional_scope:
     - global
     - eu      # 如同时受 EU 约束
     - china
   ```
2. 运行 `straymark compliance --region china` 查看基线(在创建相应文档前所有检查都会失败)。
3. 阅读 `.straymark/00-governance/` 下安装的指南:
   - `CHINA-REGULATORY-FRAMEWORK.md` — 概览与覆盖矩阵
   - `TC260-IMPLEMENTATION-GUIDE.md` — 五级风险分级
   - `PIPL-PIPIA-GUIDE.md` — 何时需要 PIPIA 及其内容
   - `CAC-FILING-GUIDE.md` — 单一 vs 双重备案、状态生命周期
   - `GB-45438-LABELING-GUIDE.md` — 显式 + 隐式标识设计

### 添加生成式 AI 模型时

需一并创建并通过 `related:` 互相关联的文档集:

| 文档 | 用途 | 何时必需 |
|------|------|--------|
| `MCARD` | 含 `cac_filing_required`、`gb45438_applicable`、`tc260_risk_level` 的模型卡 | 范围内模型始终需要 |
| `TC260RA` | 风险分级(场景 × 智能 × 规模 → 5 级) | 始终 |
| `AILABEL` | 依据 GB 45438 的显式 + 隐式标识 | 模型生成内容时 |
| `CACFILE` | 算法备案记录 | `cac_filing_required: true` 时 |
| `PIPIA` | 个人信息影响评估(第55-56条) | 处理个人信息时 |
| `SBOM` | 训练数据清单 + GB/T 45652 合规 | 始终 |

`straymark compliance --region china` 确认套件完整。

### 发生事件时

`INC` 模板包含 *CSL 2026 Incident Reporting* 部分。设置:

```yaml
csl_severity_level: relatively_major   # 或 particularly_serious | major | general
csl_report_deadline_hours: 4           # particularly_serious 为 1,relatively_major 为 4
```

`straymark validate` 强制严重程度-时限一致性(`CROSS-008`、`CROSS-009`)。major+ 事件须在 30 天内关闭(状态 `accepted`)以使 `CSL-003` 检查通过。

### 跨境数据传输

当过程涉及将个人信息传输至中国大陆境外时,在 PIPIA 上设置 `pipl_cross_border_transfer: true`,并在 *Cross-Border Transfer Analysis* 部分记录所选机制(CAC 安全评估 / 认证 / 标准合同)。`CROSS-011` 在未记录任何机制时发出警告。

### 日常合规检查

```bash
# 在合并涉及 AI 服务的功能分支之前
straymark validate                    # 跨规则,包括 CROSS-004..011
straymark compliance --region china   # 各框架得分
```

---

## 理解版本

StrayMark 为两个组件使用**独立版本管理**：

| 组件 | 标签前缀 | 包含内容 | 更新方式 |
|------|----------|----------|----------|
| **Framework** | `fw-` | 模板、治理文档、指令、脚本 | `straymark update-framework` |
| **CLI** | `cli-` | `straymark` 二进制文件 | `straymark update-cli` |

### 为什么使用独立版本？

- Framework 变更（新模板、更新的规则）更加频繁
- CLI 变更（新命令、Bug 修复）遵循不同的节奏
- 你可以更新治理文档而不需要新的 CLI 二进制文件

### 检查你的版本

```bash
straymark about     # 快速版本检查
straymark status    # 完整的健康报告，包含版本信息
```

详细的 CLI 信息参见 [CLI 参考手册](CLI-REFERENCE.md#版本管理)。
