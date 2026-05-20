---
slug: pattern-1-and-pattern-2-chain-evolution
title: "Pattern 1 与 Pattern 2 —— 链演化"
authors:
  - jose
tags: [straymark, charters, 链演化, 治理]
date: 2026-05-16
description: 二十七小时后，手工纪律被命名为两个模式：Pattern 1 —— pre-declare SpecKit refresh；Pattern 2 —— post-close audit-driven Batch N.4。正典文档、遥测 schema、CLI helper 随之落地，再过一小时，一个把两者向上重新吸收的元模式出现了。
---
*二十七小时后，这套手工纪律被命名为两个模式：模式 1 —— pre-declare SpecKit refresh；模式 2 —— post-close audit-driven Batch N.4。正典文档、遥测 schema、CLI helper —— 再过一小时，一个把两者向上重新吸收的元模式出现了。*

<!-- truncate -->

## 1. 二十七小时，为它们命名

PR [#152](https://github.com/StrangeDaysTech/straymark/pull/152) 关闭了第 9 篇的故事，于 5 月 14 日 21:39 UTC 合并。它带来了 `fw-4.14.3` 以及手动 refresh 的三个已正典化的关卡。

PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) —— 本篇要讲述的那个 —— 于 5 月 16 日 00:31 UTC 合并，整整二十七小时之后。它带来了 `fw-4.16.0 / cli-3.14.0`，以及两个具名的元模式：

- **Pattern 1 —— *pre-declare SpecKit refresh***。在链中声明下一个 Charter 之前，先刷新 spec。
- **Pattern 2 —— *post-close audit-driven Batch N.4***。当外部审计在关闭后发现问题时，修正一个已关闭的 Charter，而无需开启新的 Charter。

第 9 篇记录了手动应用于 CHARTER-18 的纪律。本篇记录之后发生的事：两个具名模式、为它们命名的正典文档（`CHARTER-CHAIN-EVOLUTION.md`）、度量它们的遥测 schema、两个用于搭建脚手架的 *CLI helper* —— 以及作为整个编辑弧线的收尾，七十七分钟后，一个哲学层面的元模式将两者向上重新吸收。

这是本弧线的最后一篇。第 1 篇从前方打开的博客，在此从后方合拢。

---

## 2. Pattern 1 —— 声明前 refresh

正典文档中的字面定义：

> *"一个专用的 refresh PR 在 Charter-N 关闭与 Charter-(N+1) 声明之间落地。它只触及 SpecKit 制件的非锁定章节（plan.md、data-model.md、contracts、quickstart.md、research.md）。"*

这正是 Sentinel 在第 9 篇中为 CHARTER-18 手动执行的操作。第 9 篇与第 10 篇的区别在于性质：第 9 篇记录流程；第 10 篇记录其*条件激活* —— 框架在何时向采用者推荐应用 Pattern 1。

激活标准是可量化的，文档毫不含糊地列出：

- 该模块在同一 spec 上已有 **≥ 3 个已关闭的 Charter**。
- 遥测中 `agent_quality.r_n_plus_one_emergent_count` 字段最近 3 个 Charter 的滚动均值**大于 6**。
- 自上一个 *branch point* 以来，没有 *refresh PR* 落地。

`> 6` 的阈值并非随意为之。它是 Sentinel CHARTER-18 转化为规则的结果：CHARTER-15、16、17 期间涌现发现（Charter 执行过程中发现的、原计划未声明的风险）的平均值，正是触发"spec 累积了过多技术债"这一感觉的临界点。框架不要求采用者凭直觉判断，而是给出一个遥测数据可以与之比较的数字。

Pattern 1 应用后产出：

- `research.md` 中一张分类表，将先前 Charter 中的发现归入四类：*可复用模式*、*代码缺口*、*纪律模式*、*经验修正*。
- 若分类揭示了权衡取舍，则给出明确的操作决策（`D1`、`D2` 等）。
- 在下一个 Charter 的 `§Context` 中，以 `id` 引用已整合条目。
- 在接收 refresh 的 Charter 的 `charter-telemetry.yaml` 中，一个可选的 `pre_declare_refresh:` 遥测块。

支撑该模式的度量指标直接引自 Sentinel CHARTER-18，在 `CHARTER-CHAIN-EVOLUTION.md` 中字面引用：

> *"Sentinel CHARTER-18 是一条 7-Charter 链中第一个无需中途补救 Charter 便干净关闭的 Charter。`estimation_drift_factor: 1.0`，`pre_work.items_discovered_during_planning: 0`，`overall_satisfaction: 5/5`。"*

单一领域，`N=1`。文档毫不掩饰地说明了这一点：该模式在 *Sentinel* 中得到了经验验证。等第二个领域的数据到来，再将其晶化至更高层级。

---

## 3. Pattern 2 —— 涌现时修正

字面定义：

> *"修正提交搭载与原始 Charter 相同的 execute 分支（该分支仍可合并至 main；修正提交叠加在其上）。"*

其思路是：当一个已关闭的 Charter 收到关闭后外部审计发现的 *critical* 或 *high* 问题时，框架不强制开启新的 Charter。操作者可以在 `execute` 分支仍可合并至 `main` 的时间窗口内，*修正*已关闭的 Charter。

理由，同样字面引自文档：

> *"开启一个新 Charter 将为约 6 小时的集中工程工作带来数周的治理开销。"*

这种不对称 —— 数周治理对比六小时工程 —— 正是 Pattern 2 所要解决的。为五个关闭后发现开启新 Charter 意味着：声明新上下文、重复外部审计、生成新 AILOG、从头注册遥测。而代码层面的修正只需一个下午。Pattern 2 的回答是：不必如此。

激活标准，同样可量化：

- Critical/High *发现*在关闭后、外部审计后出现。
- 这些发现使 Charter 的关闭标准实质上未被满足。
- 修复涉及文件少于 25 个。
- 不需要重新打开架构层面的问题（原始 Charter 视为已定论的设计决策）。

四个标准全部成立，Pattern 2 适用。任一不符，则必须开启新 Charter。

应用后产出：

- 与原始 Charter 相同的 `execute` 分支上的一个*修正*提交。
- 一个新的 AILOG（不修改原始 AILOG），包含 `risk_level: high`、`review_required: true`，以及一个指向原始 AILOG 的 `amends:` 字段。
- 原始 AILOG 中的 `## Historical correction (YYYY-MM-DD)` 章节，向*前*指向新的 AILOG。这是存档性的：原始 AILOG 不被改写，而是被链接。
- `charter-telemetry.yaml` 中的 `post_close_amendment:` 遥测块。

支撑 Pattern 2 的经验指标，字面引用：

> *"Sentinel CHARTER-18 于 2026-05-15 关闭，审计报告于 2026-05-15 至 05-17 到达。五个发现（4 个来自 gpt-5.3-codex，1 个来自 gemini-2.5-pro）均为代码层面的修复 —— DI 接线、重试头部解析、多租户过滤器、超时默认值。Batch 7.4 修正在一个内聚的提交中（19 个文件，+2257/-106 行）关闭了全部五个发现。"*

一次提交，19 个文件，两个审计模型汇聚于相似发现，五项修复。若开启新 Charter：声明、外部审计、关闭之间至少两周。替代方案：同一分支上的一次*修正*，六小时。

---

## 4. 组合，而非排斥

文档中最有趣的部分 —— 我认为也是最能收尾整个弧线的部分 —— 是关于两个模式如何关联的章节。字面引用：

> *"收到 Pattern 1 的 Charter 更可能规避 Pattern 2，因为 refresh 吸收了执行前的风险，否则这些风险会以关闭后发现的形式浮现。但 CHARTER-18 两者都需要 —— refresh 处理了 spec 层面的漂移；修正处理了 refresh 触及不到的运行时层面的漂移。"*

它们不是竞争性的模式，而是同一原则在周期不同时刻的应用。Pattern 1 吸收 *spec 层面*的漂移 —— 原始计划已不再准确描述的内容。Pattern 2 吸收*运行时层面*的漂移 —— 只有代码运行起来、外部审计员以全新视角审视、问题从代码本身涌现后才会暴露的内容。

Sentinel CHARTER-18 两者都需要。刷新后的 spec 消除了后续审计周期原本需要原子性修复的涌现发现；后续*修正*吸收了 refresh 无法预见的内容 —— DI 接线、重试头部解析、多租户过滤器、超时默认值。这些是计划无从得知的事情，只有在代码存在之后才会涌现。

框架通过命名两个模式并明确声明其组合关系，做了一件我想着重指出的事：它将纪律不是线性的这一事实记录在案。不存在单一的检查点；在每个 Charter 之前有一个（Pattern 1），之后也有一个（Pattern 2），同一个 Charter 中两者都可能是必要的。链在演化；模式也在演化。

---

## 5. Schema 与 helper

`fw-4.16.0` 在遥测 schema 中新增了两个可选的子对象，位于 `charter-telemetry.schema.v0.json` v0.2 中：

```yaml
pre_declare_refresh:
  enabled: boolean
  refresh_pr: string | null
  refresh_aidec: string | null
  reusable_patterns_integrated: integer  # ≥ 0
  code_gaps_integrated: integer
  discipline_patterns_integrated: integer
  empirical_corrections_integrated: integer
  operator_decisions_ratified: integer

post_close_amendment:
  applied: boolean
  trigger: external_audit | production_incident | deferred_implementation
  ailog_id: string
  findings_closed: integer
  files_modified: integer
  effort_hours: number
```

整数计数器，而非叙述性文字。这是有意为之的设计：遥测 schema 所捕获的，是智能体或操作者可以机械计数的内容。无法计数的 —— *问题有多深、修复有多巧妙* —— 存在于 AILOG 中，而非遥测里。Schema 度量工作的*形态*；文档承载工作的*内容*。

`cli-3.14.0` 新增了两个 helper，均不是变更器：

- **`straymark charter refresh-suggest <module> [--threshold N]`** —— *只读*。它读取该模块最近三个已关闭 Charter 的遥测，计算 `r_n_plus_one_emergent_count` 的滚动均值，并给出建议：*refresh-now*、*not-needed* 或 *insufficient-data*。*始终以 Exit 0 退出*。它是信息性的，永远不是 CI 门控。

- **`straymark charter amend <CHARTER-ID> --trigger <kind> --ailog-title <title> [--findings-closed N] [--merge-into <PATH>]`** —— 脚手架。它创建一个包含正确字段的 AILOG stub（`risk_level: high`、`review_required: true`、指向原始 AILOG 的 `amends:`），在原始 AILOG 中添加 `## Historical correction` 章节，并渲染 `post_close_amendment:` YAML 块。**不触及 git。**

让两个 helper 保持谦逊 —— 一个只推荐，另一个只准备 —— 与第 4 篇的 A1 决策一脉相承：*"CLI 编排，但不调用 API"*。在此：CLI 建议，但不决定；搭建脚手架，但不变更。人类仍然是纪律发生的节点。框架只是把工具放在触手可及之处。

---

## 6. 七十七分钟后

在弧线收尾之前，还有一件值得记录的事。PR #157 于 5 月 16 日 00:31 UTC 合并。PR #160 —— 第 1 篇的那个，`EMERGENT-OBSERVATION-DESIGN.md`，fw-4.17.0 —— 于同日 01:48 UTC 合并。七十七分钟后。

这七十七分钟里发生的事，是弧线的倒置收尾。第 1 篇所命名的哲学元模式 —— *由正式链接加上允许标记的文化许可所构成的涌现观察* —— 恰好在本篇的两个操作性元模式之后**一小时十七分钟**晶化。哲学元模式不先于纪律存在；它在纪律之后出现。它是对同一证据的升序解读。

`CHARTER-CHAIN-EVOLUTION.md` 在其 `§Related` 章节中明确记录了这一点：

> *"EMERGENT-OBSERVATION-DESIGN.md —— Pattern 1 与 Pattern 2 作为其应用的元模式（正式交叉引用 + 允许浮现的文化许可）。"*

Pattern 1 是正式链接（遥测中的 `r_n_plus_one_emergent_count`、`§Context` 中按 `id` 的引用、分类的 AILOG）与文化许可（智能体发现 spec 与代码之间的漂移时应当标记的规则）的组合。Pattern 2 是类似的组合（带 `amends:` 的 AILOG、向前链接的 `Historical correction` 章节）与允许标记关闭后发现而非静默吸收的许可。两个模式都是第 1 篇从上方命名的同一文化+结构原则的应用。

这是一个我持续喜欢的结尾：以"智能体看见了没人要求它看见的东西"开篇的博客，以在时间上更早的这一篇收尾，说的是："这里是智能体因为框架为它们命了名而能够看清的两件事"。

---

## 7. 合拢弧线

十篇文章。十一个故事片段（`H-01` 至 `H-13`，其中若干合并成篇）。一个编辑决策 —— 追溯发布 —— 由第 2 篇引入，此后九篇严格遵守。一项贯穿始终的双语承诺。

写这个博客过程中学到的一些东西，值得在收尾前记录下来：

1. **模式诞生于纪律。** 这句话在第 9 篇中已经出现，但整个弧线都在印证它。本篇的元模式不是被设计出来的，而是被提炼出来的。每个记录的里程碑都先有操作性的故事，后有名字。

2. **节奏并不均匀。** 从第一次提交（第 2 篇，1 月 27 日）到第一个有外部审计的有界单元（第 3 篇，4 月 25 日）之间隔了十九天。从手动应用纪律到正典化纪律（第 9 篇，5 月 14 日）之间只有五小时。从两个元模式到它们的哲学解读（第 10 篇与第 1 篇，5 月 16 日）之间只有七十七分钟。项目经历了每一种速度，博客将它们全部记录在案。

3. **框架对其采用者并非中立。** Sentinel 使得每个模式在晶化之前都能得到经验验证。博客不掩盖这一出处；它将其作为方法论前提加以强调。

4. **未被命名的，就不会被看见。** 第 5 篇将其表述为*结构性可见性*；第 7 篇将其应用于横向技术债；第 8 篇将其指向语言异常值。这是整个弧线中最持久的规律性。作为技术制件存在是不够的。名字激活制件。

博客要讲述的弧线 —— 里程碑 `H-01` 至 `H-13`、项目的四个名字、六个 Plan、一等公民 Charter、外部审计周期、Issue #113、更名为 StrayMark、TDE、审计提示异常值、手动纪律，以及两个元模式 —— 在此合拢。剩余的明确技术债：`H-14`，`straymark explore` TUI（在第 8 篇中作为可见性工具横向出现），以及 `PLAN-INVESTIGACION.md §1.43-55` 列出的待处理候选里程碑 `H-?-A` 至 `H-?-E`。可能的未来第二批次。不作承诺。

最后一个观察，与第 1 篇对称。那篇以"智能体看见了没人要求它看见的东西"开篇。博客在十篇之后收尾所记录的，是这句话的另一半：智能体之所以能看见，是因为框架让它具备了看见的条件。第 1 篇的涌现观察是证明；本篇的具名模式是前提条件。智能体在弧线覆盖的四个月里浮现的每一件事，都因为某个先前的故事片段在原本空无一物之处放置了一个制件、一个触发器、一个正式链接、一个可见的表面而成为可能。

博客讲完了这个故事。框架继续前行。

---

*锚点：[Issue #156](https://github.com/StrangeDaysTech/straymark/issues/156)。PR [#157](https://github.com/StrangeDaysTech/straymark/pull/157) —— `fw-4.16.0 / cli-3.14.0`（合并于 2026-05-16 00:31 UTC）。正典文档：[`CHARTER-CHAIN-EVOLUTION.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md)。Schema：`dist/.straymark/schemas/charter-telemetry.schema.v0.json` v0.2。后继元模式：[`EMERGENT-OBSERVATION-DESIGN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/EMERGENT-OBSERVATION-DESIGN.md)，`fw-4.17.0`（合并于 2026-05-16 01:48 UTC，77 分钟后）。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*

*—— StrayMark 博客 H-01 → H-13 弧线终结。*
