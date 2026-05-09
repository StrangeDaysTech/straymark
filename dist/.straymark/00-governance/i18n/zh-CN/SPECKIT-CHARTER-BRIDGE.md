# SpecKit ↔ StrayMark Charter 桥接

> **状态**：经验性模式（`v0`）。需在第二个领域中验证后才会结晶（原则 #12）。如有新用例，通过 PR 进行细化。

## 本文档解决的问题

[SpecKit](https://github.com/StrangeDaysTech/speckit) 为一个 feature 提供 `spec.md`、`plan.md` 与 `tasks.md`。StrayMark 提供 Charter、AILOG、AIDEC、ADR。**没有任何权威文档解释一个 SpecKit feature 何时应产出一个 Charter、应使用何种粒度、由谁触发创建、何时创建。** 这正是 [issue #113](https://github.com/StrangeDaysTech/straymark/issues/113) 所报告的核心议题——一个让 Agent（Claude、Gemini、Copilot）形成二元心智模型（`SpecKit = 规划，StrayMark = 审计轨迹`）、并悄然丢弃第三层（即"工作-作为-可审计-可发布单元"，Charter 所在的层）的发现性缺口。

本文件就是答案。

## 心智模型

三层结构，含交接点：

| 层级 | 居于 | 目的 | 所有者 |
|------|------|------|--------|
| **1. 规范** | `specs/NNN-feature/{spec,plan,tasks,research,quickstart}.md` | feature 是什么、为什么存在、技术层面将如何实施。SpecKit 通过 `/speckit-specify` → `/speckit-plan` → `/speckit-tasks` 产出。 | 操作者（含 Agent 协助）。 |
| **2. 有界执行单元** | `.straymark/charters/NN-slug.md` | 一次可发布切片的合约。将事前范围（文件、风险、任务子集）与事后遥测（drift、audit、教训）配对。 | 操作者声明 Charter；Agent 在其内执行。 |
| **3. 实施轨迹** | `.straymark/07-ai-audit/agent-logs/AILOG-*.md`（以及视情况而定的 AIDEC、ADR） | 实际工作内容、原因与置信度的逐日记录。每个 AILOG 通过 `originating_charter:` 引用 Charter（或 Charter 通过 `originating_ailogs:` 聚合 AILOG）。 | Agent 在工作时创建；操作者审核。 |

**桥接就是 Charter。** 规范层级太高，不利于 drift 检查（"你交付了规范吗？"在有用的时间窗内无法回答）；AILOG 层级太低，不利于以其为单位发布（"你交付了这个 AILOG 吗？"是错误的单位）。Charter 处于恰当的粒度：你可以在数日内（而非数月内）做审计的稳定范围合约。

## SpecKit feature 何时产出 Charter？

当**任一**条件成立时，一个 SpecKit feature 应至少产出一个 Charter：

1. feature 的 `tasks.md` 包含 **5 个或更多任务**，无法在单一会话中完成。
2. feature 横跨 **2 个或更多 SpecKit 阶段**（Setup、Foundation、User Stories、Polish 等），且你打算将其作为一个单元一起发布。
3. 工作在完成时值得**外部审计**（跨模型审查、跨团队审查）。
4. 你希望在关闭时获得**可度量的遥测**（effort estimate vs. 实际、drift 数量、教训）。

**不应**产出 Charter 当：

- feature 足够小，可在一次会话内完成（<1 天，<5 个任务）。仅使用 AILOG——Charter 的开销超过其可审计性所带来的收益。
- feature **仅为规划**（尚无代码）。等到 `tasks.md` 存在再说；Charter 合约需要列出具体任务。
- feature 是**没有计划范围的维护**（例如"按需修 bug"）。对于即兴维护，AILOG 已足够。

## 粒度启发式

当一个 feature 值得使用 Charter，按**可发布单元**而非按结构单元选择粒度。具体而言：

### 启发式 1 — 每一可发布切片对应一个 Charter

如果 feature 有阶段（如 SpecKit 典型的 Foundation → US1 → US2 → US3 → Polish），**第一个 Charter 包裹基础切片**（一起作为 `v0.1` 发布的所有内容）。后续 Charter 包裹后续切片。Effort estimate **M** 是可发布切片的中位桶；**L** 用于完整 feature 的切片。

```
specs/001-peek-mvp-foundation/
├── spec.md
├── plan.md
└── tasks.md  →  CHARTER-01 (Foundation: T001-T012, effort M)
                  CHARTER-02 (peek MVP: T013-T044, effort L)
```

### 启发式 2 — 不要按 User Story 划分

User Story 太细。一个 2-3 个任务的 US 应位于 Charter *内部*，而不是其自己的 Charter。每 US 的遥测是噪音；每可发布切片的遥测是信号。

### 启发式 3 — 不要按 feature 划分

一个分两次切片发布的 feature（如 MVP → polish）应有两个 Charter，而不是一个。可做 drift 检查的 Charter 合约是"这个切片发布了什么"，而非"我们最终建造了什么"。

### 启发式 4 — 边界情况：≥10 个任务跨 4+ 阶段

当一个 feature 异常庞大时，第三个 Charter（或将基础切片再拆分为"scaffolding" + "core"）可能是合理的。以 effort estimate **L** 作为上限；如果你估计为 **XL**，那是一个信号——这个 feature 应被重新规范化。

## 创建时机

```
/speckit-specify  → spec.md
/speckit-plan     → plan.md
/speckit-tasks    → tasks.md
                    ↓
                ┌────────────────────────────────────────┐
                │   ★ Charter 声明点 ★                   │
                │                                        │
                │   操作者运行 `straymark charter new`   │
                │    --from-spec specs/NNN-feature/spec.md│
                │    --type <M|L>                        │
                │                                        │
                │   Charter 状态：declared               │
                │   → 操作者填写 scope、files、tasks     │
                │   → 执行时改为 status: in-progress     │
                └────────────────────────────────────────┘
                    ↓
/speckit-implement  → 任务被执行
                    → AILOG 被创建（`originating_charter:` → Charter）
                    ↓
straymark charter drift CHARTER-NN  → 文件 vs commit 检查
straymark charter audit CHARTER-NN  → 外部审计（可选）
straymark charter close CHARTER-NN  → 遥测，状态：closed
```

**关键不变量**：在 `/speckit-implement` 启动**之前**声明 Charter。Charter 是一份合约；在执行之后才声明会让 drift 检查失去意义。

## frontmatter 关联

Charter 的 frontmatter 显式引用 SpecKit feature：

```yaml
charter_id: CHARTER-01-workspace-foundation
status: declared
effort_estimate: M
trigger: tasks.md 在 2 个阶段内有 12 个有序任务；作为 v0.1 发布。
originating_spec: specs/001-peek-mvp-foundation/spec.md
```

反向（spec → Charter）依靠惯例——如果你的 `plan.md` 模板有 "Phase 5: Implementation Tracking" 章节，就在那里列出当前活动的 Charter。SpecKit 目前没有为此提供 schema 槽位；这是新兴的惯例。

执行期间创建的 AILOG 应引用该 Charter：

```yaml
id: AILOG-2026-05-08-005
title: T013, T016-T026 — US1 P1 MVP core + TUI + peek bin
agent: claude-code-v4.7
confidence: high
risk_level: medium
review_required: false
originating_charter: CHARTER-02-peek-mvp-foundation
```

## 生命周期映射

| SpecKit 阶段 | Charter 事件 | StrayMark CLI |
|-------------|-------------|---------------|
| `/speckit-tasks` 完成 | **声明 Charter** | Skill `/straymark-charter-new` 或 `straymark charter new --from-spec …` |
| 第一个任务开始 | 操作者把 `declared` 翻为 `in-progress` | （手动编辑 frontmatter） |
| 每个任务执行 | 产出 AILOG（当 STRAYMARK.md §6 要求时） | `/straymark-ailog` |
| 遇到主要决策 | 产出 AIDEC | `/straymark-aidec` |
| 架构变化 | 产出 ADR | `/straymark-adr` |
| 最后一个任务完成，关闭前 | Drift 检查 | `straymark charter drift CHARTER-NN` |
| 可选外部审查 | 多模型审计 | `straymark charter audit CHARTER-NN` + `/straymark-audit-prompt` + `/straymark-audit-execute` + `/straymark-audit-review` |
| 切片已发布 | 关闭 Charter | `straymark charter close CHARTER-NN`（状态：`closed`，发出 telemetry yaml） |

## 反模式

**不要"以防万一"开 Charter。** 没有清晰可发布切片的 Charter 会变成愿望清单。操作者最终会把它关为 `closed: aborted`，遥测毫无意义。

**不要按 User Story 开 Charter。** 每 US 的遥测过于嘈杂，无法指导未来的估算。要聚合。

**不要省略 `originating_spec` 字段。** 即便 Charter 包裹的工作没有 SpecKit spec，也要改为设置 `originating_ailogs:`。无来源的 Charter 是反模式（暗示动机未被记录）。

**没有就绪的审计员 CLI 时，不要运行 `straymark charter audit`。** 审计仅为编排——`straymark` 不调用 LLM API。如果没有 N 个审计员 CLI 就绪，跳过该步骤；不进行外部审计直接关闭 Charter。

**不要在 drift 检查 + telemetry yaml 之前把状态改为 `closed`。** `straymark charter close` 原子化地完成两者；手动关闭跳过了不变量。

## 此模式不适用的场景

此桥接假设的是 SpecKit 驱动的 feature 流程，包含多任务、多会话的实施。它不适用于：

- **单会话 feature** — 仅使用 AILOG。
- **仅架构、无实施的工作**（如"设计下一代 schema"）— 使用 ADR。
- **没有新行为的纯重构** — 使用 AILOG + 标记 `refactor:`。
- **事故响应与 hotfix** — 使用 INC + AILOG。
- **仅合规交付物**（如季度 DPIA 刷新）— 直接使用相关文档类型。

如果你的工作属于上述任一类，*不要声明 Charter*。当没有可发布切片需要包裹时，Charter 的成本超过其价值。

## 另请参阅

- `STRAYMARK.md` §6（何时编写文档）与 §15（Charter 作为有界工作单元）
- `.straymark/templates/charter/charter-template.md` — 声明式模板
- `.straymark/templates/charter/charter-telemetry-template.yaml` — 遥测模板
- `.straymark/schemas/charter.schema.v0.json` — 声明式 frontmatter 的 JSON Schema
- `.straymark/schemas/charter-telemetry.schema.v0.json` — 遥测的 JSON Schema
- `.claude/skills/straymark-charter-new/SKILL.md`（以及 Gemini / 通用等价物）

> **被引用的经验性背景**（issue #113）：一个全新的 Rust CLI/TUI 套件，Claude Opus 4.7 通过权威入口点（`STRAYMARK.md`、项目宪章、`CLAUDE.md` 检查清单、可用的 `/straymark-*` skills、`/straymark-status`）入门。Charter *最终*被采用（2 个 Charter：foundation + MVP），但只在用户明确提示之后——这证实了缺口是系统性的，而非会话特有的。本文档消除了该缺口。

---

*语言*：[English](../../SPECKIT-CHARTER-BRIDGE.md) | [Español](../es/SPECKIT-CHARTER-BRIDGE.md) | 简体中文
