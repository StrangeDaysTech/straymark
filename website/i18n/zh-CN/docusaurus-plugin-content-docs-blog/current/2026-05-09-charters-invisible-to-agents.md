---
slug: charters-invisible-to-agents
title: 对智能体不可见的 Charter
authors:
  - jose
tags: [straymark, charters, 智能体, 可观测性, 治理]
date: 2026-05-09
description: "Issue #113 —— 拥有一个制品与智能体能够看见它之间的差距"
---
*六个小时的工作会话，使用一个能力强的智能体，加载了框架的全部入门资料 —— 一次都没有主动建议使用 Charter。直接被问到时五分钟内就能套用。这不对称的不是能力，是可见性。*

<!-- truncate -->

## 1. 六小时看不见，五分钟看见了

Issue [#113](https://github.com/StrangeDaysTech/straymark/issues/113) 开头有这样一句话，事后回看显得既显而易见、又来之不易：

> *"检测耗时：约 6 小时的会话工作；从未被自主发现。一旦用户提示后解决耗时：约 5 分钟。"*

六小时，一个能力强、专注的智能体，加载了框架完整的入门资料 —— `STRAYMARK.md`、项目章程、`CLAUDE.md` 清单、可用的 `/straymark-*` 技能、开始时运行了 `/straymark-status` —— 却在整整六小时里，从未主动建议使用 Charter。当我直接要求时，五分钟内流程就被纳入了。这种不对称不在于能力，而在于可见性。

这篇文章记录一段简短、具体的事件：Issue #113，于 2026 年 5 月 7 日开启，距 Charter 在框架中正式成为一等实体（fw-4.4.0，5 月 2 日）仅三天。它是本博客第一篇文章的天然对应。那篇文章命名了一切就位时涌现的属性；这篇文章记录了当一个结构性组件存在、但*不在智能体首先读取的表面上*时会发生什么。

---

## 2. 意外的实验

这个设置并非刻意设计成实验。它是一个真实项目：用 Rust 从零开始构建的 CLI+TUI 套件，处于第一个 `v0.1` 版本。智能体是 Claude Opus 4.7，拥有 1M token 的上下文窗口，足以将整个仓库保留在上下文中而无需分页。流程是规范的 SpecKit 流程：`/speckit-specify`、`/speckit-plan`、`/speckit-tasks`，而在任务与实现之间，拆分为 Charter 的想法本应浮现 —— *理论上如此*。

Issue 以近乎冷静的精确措辞表述如下：

> *"一个能力强、专注的智能体，遵循规范的项目入门资料（`DEVTRAIL.md`、项目章程、`CLAUDE.md` 清单、可用的 `/devtrail-*` 技能、`/devtrail-status`），在 plan/tasks 生成过程中，未能自主识别 Charter 作为工作流概念，尽管该项目明显符合其使用场景（多会话实现、多阶段任务、审计价值）。"*

*(该 Issue 仍使用 `DEVTRAIL.md`，因为按照时间线，更名为 StrayMark 的工作几乎是并行合并的。混合命名是日历上的残留，而非疏忽。)*

该项目满足 Charter 作为正确单元的每一个条件：多会话实现、多个阶段、审计价值。智能体做了一切正确的事 —— 阅读了章程、运行了 status、识别了可用技能、生成了合理的 SpecKit 计划 —— 却始终没有关联到 Charter。仿佛这个概念根本不存在。对于智能体来说，在它对仓库的模型中，确实不存在。

关于这个 Issue，事后最令人不安的是：智能体并非因能力不足而失败。它失败，是因为框架在无意间构建了一张地图，而其中没有标注 Charter 的位置。

---

## 3. 九个差距汇聚为一

当操作者（我）开启 Issue 并着手审计问题原因时，问题并非一个，而是九个。而所有这些都是同一个缺陷的不同版本：

| # | 差距 | 位置 |
|---|---|---|
| 1 | 框架的规范文档未将 Charter 列为概念 | `STRAYMARK.md`（当时为 `DEVTRAIL.md`）§6/9/10/11/13/15 |
| 2 | `CLAUDE.md` 清单中没有触发器提示建议使用 Charter | `dist/dist-templates/directives/CLAUDE.md`（以及 `GEMINI.md`、`copilot-instructions.md`）|
| 3 | 项目章程继承了框架的差距 | 任何通过 `straymark init` 安装的 `*-constitution.md` |
| 4 | 不存在 `/straymark-charter-new` 技能 | 技能目录 |
| 5 | `/straymark-status` 的输出中未列出 Charter | 技能 + CLI 子命令 `straymark status` |
| 6 | 审计技能将 Charter 定义为*外部*制品，而非待生成的表面 | `/straymark-audit-*` 技能 |
| 7 | Charter 模板与其他模板混放，无法区分 | `dist/.straymark/templates/`（根目录）|
| 8 | SpecKit（`plan.md`）与 Charter 之间没有概念桥梁 | 无显式文档 |
| 9 | 智能体最终构建的心智模型是二元的：要么 SpecKit，要么什么都没有 | 累积效应 |

九个失败，一个单一缺陷：Charter 以技术制品的形式存在（有效的 schema、可运行的命令、从 Sentinel 移植的模板），但在智能体入门时构建项目心智模型的任何位置，都没有被命名。

智能体读取 `STRAYMARK.md` 和章程来理解项目是什么。如果 Charter 不出现，它就不存在。智能体查看可用技能。如果没有 `/straymark-charter-new`，它就不存在。智能体运行 `/straymark-status` 来了解仓库中存在哪类文件。如果输出中没有提及 Charter，它们就不存在。智能体在入门过程中经过的每一个表面都重复着同样的沉默。而这些沉默的累积说服了智能体：这个概念不属于这个项目。

---

## 4. 为何这个事件超越了 Issue 本身

值得在此停下来，因为这正是 fw-4.17.0 元模式 —— *涌现观察*模式 —— 需要*先*发生才能被表述出来的事件。

#113 之后一周，在 `EMERGENT-OBSERVATION-DESIGN.md` 中，框架命名了两个属性，其组合能让智能体主动发现没有人要求它发现的事物：制品之间的正式链接（前置数据中的 `originating_charter`、`originating_ailog`、`originating_spec`），以及标记来源之间不一致的文化许可。这两个属性*组合*后，产生了智能体读取 AILOG、统计 `R<N>(new, not in Charter)` 条目、并在无人要求的情况下标记差异的效果。

但这一表述有一个前提条件：智能体*在进入仓库时能看到* Charter。如果 Charter 不在可见表面上，制品之间任何正式链接都无法组合成涌现观察 —— 因为这个组合的其中一个项对智能体来说不存在。

Issue #113 是从反面记录这一前提条件的事件。fw-4.17.0 在该模式存在时命名了它；fw-4.12.0（五天前）填补了妨碍它存在的差距。没有 Issue #113 及其修正，这个元模式就没有立足之处。框架无法发现它看不见的，它也看不见表面没有命名的。

---

## 5. 回应：倍增表面

PR [#122](https://github.com/StrangeDaysTech/straymark/pull/122)（`fw-4.12.0`，5 月 9 日）以一个单一的操作策略关闭了九个差距：*在智能体构建项目模型的每一个位置命名 Charter*。PR 的数字是诚实的：+1211 行，−92 行，36 个文件被修改，几乎全是文档和模板。没有代码重构。有的是可见性的重新分配。

主要变更，按组归类：

- **在规范文档中**（`STRAYMARK.md`）：新增专门介绍 Charter 作为概念的章节（§15），以及在列出文档类型但缺少 Charter 的 §6/9/10/11/13 表格中新增了相应行。
- **在智能体指令中**（`CLAUDE.md`、`GEMINI.md`、`copilot-instructions.md`）：添加了明确的触发器，教导智能体何时*建议*使用 Charter，而不仅仅是在被要求时才创建。
- **在技能中**：新增 `/straymark-charter-new`（Claude、Gemini、通用版本）。更新 `/straymark-status` 以列出活跃的 Charter。
- **在 CLI 中**：向 `straymark status` 输出添加了 `Charters` 块，使运行该命令的智能体能在初始信号中看到它。
- **在模板中**：移至专用子目录 `dist/.straymark/templates/charter/`，不再与其他模板混放。
- **新文档**：`SPECKIT-CHARTER-BRIDGE.md`（171 行，EN/ES/zh-CN）。明确建立了 SpecKit 的 `plan.md` 与 StrayMark 的 Charter 之间的桥梁 —— SpecKit 功能产生一个或多个 Charter 的四个标准、三种不适用的情形、四个粒度启发式规则，以及 `originating_spec ↔ originating_charter ↔ originating_ailog` 前置数据链接。

`SPECKIT-CHARTER-BRIDGE.md` 的结语毫无隐喻地记录了引发整个变更的案例：

> *"引用了经验背景（issue #113）：Greenfield Rust CLI/TUI 套件，Claude Opus 4.7 通过规范入口点入门。Charter 最终被采用（2 个 Charter：基础 + MVP），但仅在用户明确提示后 —— 确认差距是系统性的，而非会话特有的。本文档消除了这一差距。"*

*差距是系统性的，而非会话特有的。* 这句话是这个 Issue 的操作性教训。不是某天一个分心的智能体；是框架在智能体查看的每一个地方都没有向它讲述 Charter。

---

## 6. 关于结构性可见性的所学

教训，以散文而非表格的形式书写：

在框架中创建一个制品，与使其对在该框架仓库中工作的智能体可见，并不是同一件事。它们是两个步骤。第一步 —— schema、命令、模板、示例 —— 是可见的步骤：它出现在 CHANGELOG 中，随版本发布，出现在 `git log` 里。第二步 —— 将制品锚定在智能体进入仓库时的每一个表面上 —— 是不可见的：第二步中没有任何内容本身作为*功能*出现；它是交叉引用、清单中的行、输出中的条目、文档中的章节。但第二步决定了第一步对智能体来说是否存在。

框架治理的是从制品创建所在的会话之外读取仓库的智能体。每个进入仓库的智能体都从头构建其心智模型，读取规范文档、可用技能、status 输出。如果制品在那里没有被命名，它就不存在于模型中。而没有该制品的模型不会建议使用它，不会审计其缺失，不会标记相关的不一致。如果仓库的表面在入门时没有给智能体正确的名词，智能体的技术能力就无关紧要了。

这就是我现在内部所称的*结构性可见性*。它不是智能体的属性。它是仓库的属性，进而是框架的责任：每当它将一个新概念具体化时，必须倍增命名该概念的表面。单一表面 —— schema、命令、模板 —— 是不够的。至少有九个，而它们正是 Issue #113 中的那九个。

---

## 7. 结语

从这一过程中得到的收获，以四点陈述：

1. **作为制品存在与对智能体可见不是同一件事。** Charter 有 schema、命令、模板、示例 —— 却依然不可见。两种状态之间的差距，就是 Issue #113 的九个表面。

2. **六小时与五分钟。** 一个能力强的智能体可以花数小时未能检测到某件事，而一旦被命名，整合只需数分钟。这种不对称不是在诊断智能体，而是在诊断仓库。

3. **结构性可见性是框架的责任。** 当框架将一个概念具体化时，不以创建它的命令为终点。终点是该概念被命名在智能体构建仓库模型的每一个表面上：规范文档、指令、技能、status、模板、与其他框架的桥梁。

4. **没有结构性可见性，就没有涌现观察。** 后来让智能体能够发现来源之间不一致的属性 —— 正式链接 + 文化许可 —— 作为前提条件，需要制品从入门的第一刻起就是可见的。Issue #113 记录了当这一前提条件缺失时会发生什么。

接下来，在下一篇文章中：一个与此相邻的事件 —— *更名为 StrayMark*（`H-08`）。Issue #113 的解决与从 DevTrail 更名为 StrayMark 处于同一时间弧内，它们共享一件值得命名的事情 —— 一个框架向其智能体和其采用者清晰声明自身身份的责任。

---

*锚点：[Issue #113](https://github.com/StrangeDaysTech/straymark/issues/113)（开启于 2026-05-07）。[PR #122](https://github.com/StrangeDaysTech/straymark/pull/122) —— `fw-4.12.0`（合并于 2026-05-09）。PR 生成的文档：[`SPECKIT-CHARTER-BRIDGE.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md)。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
