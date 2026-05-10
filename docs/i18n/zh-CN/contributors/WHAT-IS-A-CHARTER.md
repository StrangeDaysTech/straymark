# StrayMark 中的 "Charter" 是什么（以及它相对于 GitHub SpecKit 的位置）

**版本：** 0.2（Plan → Charter 重命名）
**日期：** 2026-04-30
**作者：** Jose Villaseñor Montfort — StrangeDaysTech
**目的：** 锁定 **Charter** 工件的概念范围——按其在 Sentinel 实验中（彼时称为 *Plan*）所呈现的样子——并解释它如何与已使用 SpecKit（spec → plan → tasks）的设计与开发流程共存。
**语言：** [English](../../../contributors/WHAT-IS-A-CHARTER.md) | [Español](../../es/contributors/WHAT-IS-A-CHARTER.md) | 简体中文
**相关文档：** `straymark-cli-roadmap.md`（§3 阶段 1，§6 Sentinel→CLI 映射）、`straymark-thesis-validation.md`、`straymark-charter-telemetry.md` — 均保存在 `docs/decisions/proposals/` 中，以供历史参考。

---

## 1. 什么是 StrayMark 中的 Charter

**一项工作单元的事前声明，规模足够大、值得签订一份可核验的契约并接受事后审计。**

Charter 同时具有三个面向：

1. **范围契约**（Context + Scope in/out + 待修改文件 + Risks）：要触动什么，以及——明确地——*不*触动什么。`Out of scope` 章节存在的目的就是让外部审计员不要将其分类为 gap。
2. **核验契约**（Local checks + Production smoke）：可执行命令的字面表述，按*何时适用*分别列出。Local/Prod 的拆分不是风格选择——它源于一个事实：在 3/3 个周期中，外部审计员都将仅在生产环境中失败的情形分类为 `real_debt`。
3. **审计锚点**（frontmatter + Charter 收尾 + drift check）：Charter 是 `straymark charter drift` 与 `straymark charter audit` 操作的对象。没有 Charter，就没有可审计的对象。

### 1.1 不只是技术债务

在 Sentinel 已关闭的 Plan（即 Charter 模式的实证依据）中，只有一个（`03-contract-bumps`）有"债务"的味道。其余是功能（`02-admin-endpoint`、`05-per-service-anomaly-thresholds`）、治理工作（`01-deploy-governance`）或运维工作（`06-baseline-recompute-job`）。`sentinel/docs/plans/` 的 README 上写得很直白：*"在 MVP 后 backlog 收尾期间识别出的工作路线图"* — 是 backlog，而不是 debt-log。

### 1.2 何时需要 Charter，何时仅需 AILOG

从语料中归纳，并非正式书写：

- **仅 AILOG**：可在单一会话内完成的变更，无需事前声明文件，结构性风险较低（typo、定点修复、局部重构）。
- **Charter + AILOG**：触及多个文件、有可枚举的事前风险、需要以可执行命令进行核验、并能从合并后的外部审计中获益的工作。规模 XS–L（数分钟到数日）。
- **ADR + 一个或多个 Charter**：根本性的架构决策，每个 Charter 执行该 ADR 的一部分。

### 1.3 原子化的 Charter 收尾（format v4）

自 fw-4.4.2 起，`charter-template.md` 采用 **format v4**：当预提交的 drift check（Tasks #7）报告 `## Files to modify` 与实际修改文件之间的偏差时，实施者必须原子地更新 Charter——在与实施同一次的 commit/PR 中——添加一个 `## Closing notes` 块，记录改了什么、为什么改、并引用对应的 AILOG。**不延后到合并后的 housekeeping PR。**

该模式经验性地源于 Sentinel 的 PLAN-07（2026-05-02 关闭）：一次合理的遗漏型 drift（一个 live test 结果对该变更不敏感，因此在 §Files to modify 中声明的某个文件未被触动）通过另一个独立的 housekeeping PR 进行了延后记录，导致 Plan-doc 数日处于陈旧状态。Sentinel 的 AIDEC-2026-05-02-001 形式化了该缺口并提议了 format v4；本文档反映上游的演化。PLAN-05（`docs/plans/05-per-service-anomaly-thresholds.md`）与 PLAN-07 本身是回顾性应用 `## Closing notes` 模式的典范——均在 Sentinel 中。

---

## 2. 为什么叫 "Charter" 而不是 "Plan"

StrayMark 在 Sentinel 实验期间（2026 年 4 月）最初将该工件称为 **Plan**。在 CLI 实施之前，该术语被重命名为 **Charter**，原因务实：GitHub SpecKit 已经将"plan"一词（`/speckit.plan`、`plan.md`）用于一个不同的工件，名义上的冲突会在同时组合两套流程的采用方文档中产生摩擦。

Sentinel 的历史记录（Plans 01–06、`sentinel/docs/plans/`、`sentinel/scripts/check-plan-drift.sh`、AILOG 与 YAML 遥测 `PLAN-NN.telemetry.yaml`）保留了原名——它们是某一特定时刻的实证证据，重写历史会伪造记录。本文档对那些文件的引用刻意保留 "Plan"。

SpecKit 所称的 "plan" 与 StrayMark 所称的 "Charter" 是不同的工件：

| | **SpecKit `plan.md`** | **StrayMark Charter** |
|---|---|---|
| 粒度 | 完整 feature（数周，多个 user story） | 有界单元（数小时到数日，一个分支） |
| 主导内容 | Stack、依赖、项目结构、宪章 gate | 具体待触动文件、核验命令、R<N> 风险 |
| 子工件 | 生成 `research.md`、`data-model.md`、`quickstart.md`、`contracts/` | （执行时）生成一个 AILOG |
| 验证 | 宪章检查（事前 gate） | Drift 检查 + 外部审计（事后 gate） |
| 优化目标 | *预先的清晰* | *事后的可追溯* |

SpecKit 所称的 "plan" 更像是 **ADR + 架构骨架**。StrayMark 所称的 "Charter" 更像是 **带有合同性核验和审计锚点的 task-card**。

---

## 3. SpecKit 流程的逐项映射

SpecKit 是线性、自上而下的：`/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → 实施。

| SpecKit | 解决什么 | StrayMark 中最接近的等价物 |
|---|---|---|
| `spec.md`（US、FR、SC） | 构建*什么*，与技术无关 | `01-requirements/REQ-*.md` |
| `plan.md`（stack、结构） | *如何*架构 | `02-design/ADR-*.md` |
| `tasks.md`（T001..TNNN） | *按什么顺序*执行 | StrayMark Charter 内的 `## Tasks` 章节 |
|（不存在） | 执行日志 | **AILOG** |
|（不存在） | 已声明 vs 已执行的偏差 | **`straymark charter drift`** |
|（不存在） | 多模型事后审计 | **`straymark charter audit`** |

最后三行是 StrayMark 的特定贡献。SpecKit 在产出 `tasks.md` 时即结束；之后发生了什么，框架并不关心——若实施偏离计划，框架内没有可检测它的机制。StrayMark 正是在此处介入。

---

## 4. StrayMark Charter 与 SpecKit 共存的三种模式

### 模式 A — SpecKit 驱动的 greenfield

```
spec.md  →  plan.md  →  tasks.md  →  ┐
                                     ├→  Charter-NN（覆盖 US1 + 核验 + 风险）
                                     ├→  Charter-NN+1（覆盖 US2 + 核验 + 风险）
                                     └→  Charter-NN+2（覆盖 US3 + 核验 + 风险）
                                                ↓
                                         每个 Charter 生成一个 AILOG
                                                ↓
                                         drift 检查 + 审计
```

每个 StrayMark Charter 取一组 task 的子集（通常是一个 user story 或一个阶段），并补上 SpecKit 不提供的三层：可执行核验、用以衡量 drift 的文件声明、以及 AILOG/审计的接驳。

### 模式 B — 维护 / MVP 之后（Sentinel 的真实情形）

没有运行 SpecKit。Sentinel 的 Plans 01–06 来自 AILOG 与 MVP 之后的 backlog，并非来自规范。Charter 的 `Origen:` 字段指向某个 AILOG，而不是某个 `specs/###-feature/`。此时 StrayMark Charter 是独立存在的。

### 模式 C — 混合（很可能是最现实的）

- 大型 feature → 完整 SpecKit 流程 + 在执行 task 时使用 StrayMark Charter。
- Bug 修复、治理、债务、契约变更、小型 feature → 仅使用 StrayMark Charter。

这与 Sentinel 已展示的情况一致：在已关闭的 Plan 中，没有上游的 SpecKit，因为 MVP 早已规范化并关闭。剩下的是演化性的工作，对其而言 Spec→Plan→Tasks 周期显得过重。

---

## 5. 根本的概念差异

**SpecKit 回答的问题：***"如何把一个产品想法分解为可执行的工作？"* — 优化目标是预先的清晰与 user story 的并行化。

**StrayMark Charter 回答的问题：***"如何让一个工作单元变得可核验、可审计，并在偏离声明时可被检测？"* — 优化目标是事后的证据。

因此它们并不竞争：它们处于周期中的不同时刻。SpecKit 在 `tasks.md` 处结束；StrayMark Charter 从那里开始（适用时）或独立成立（无前置规范时）。Sentinel 的论点不是"这取代 SpecKit"——而是"这覆盖 SpecKit 不覆盖的 post-tasks 阶段，并以三层模式（声明 + drift + 异质审计）来覆盖；这一模式有实证证据能够捕获实施者未文档化的缺口"。

---

## 6. 关于 Studio 愿景的说明："Stage" 不是 Charter 的同义词

`straymark-studio-vision.md`（一份仅供内部使用的文档）引入了一个额外术语 **Stage**，不应与 Charter 混淆。它们不是同义词：所处的周期层级不同。

- **Stage** = 一段工作阶段，贯穿 Studio 的六个认知阶段（Spec → Implementation → Audit → Remediation → Governance → Closure & Deliver），并以一个签名且哈希链式的 Closure Bundle 收尾。愿景中提议的命令：`straymark stage open/close/status`（未实施；对应迈向 Studio 演化的里程碑 2）。
- **Charter** = 带有核验 + drift + 审计锚点的有界单元——本文件所定义的工件。Charter 存在于 Stage *之内*；一个 Stage 通常包含一个或多个 Charter，加上其相关 task。

愿景所暗含的分类是三层结构：**Stage > Charter > Task**。当前 CLI 路线图（`straymark-cli-roadmap.md`，保存在 `docs/decisions/proposals/` 中，阶段 1–3）完全在 Charter 层级运作（即 Sentinel 所称的 "Plan"）——这是 Sentinel 的实证证据所在的层级。Stage 将来会作为附加层进入 CLI，而不是对 Charter 的重命名。

## 7. 对 StrayMark CLI 路线图的启示

值得在阶段 1 落地（`straymark-cli-roadmap.md` §3.2，保存在 `docs/decisions/proposals/` 中）时，反映在 `straymark charter new` 命令的文档之中。今天计划中的标志 `--from-ailog ID` 覆盖了模式 B。也许还应该有一个 `--from-spec specs/###-feature/` 标志覆盖模式 A——预填 `Origen:` 指向 SpecKit 的 `spec.md`，并可选地把相关 user story 继承到 Context 章节。这是阶段 1 的练习，而不是今天要解决的问题，但值得在本文档中作为一项开放决策记录在案，供子命令设计时参考。

---

*本文档捕获了 Charter 工件（在 Sentinel 实验中最初称为 "Plan"）在 2026 年 4 月底所被理解的概念范围。该定义将随 Sentinel 之外的采用方在其他领域演练此模式而继续演化——版本 0.3 将在观察到至少一个采用方项目以 `straymark charter` 完成一个 Charter（阶段 1 的退出标准）之后撰写。*
