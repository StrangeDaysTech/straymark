---
slug: what-the-author-already-knew
title: 作者本就知道的事
authors:
  - jose
tags: [straymark, baton, cost-routing, framework, schema, sentinel]
date: 2026-06-28
description: dry run 证明了:是信号覆盖率 —— 而非 granularity —— 让 57% 的路由停留在低置信的猜测上。修复几乎免费。创建一个工作单元的人本就知道它是设计、实现、审计还是机械劳动;他知道它,只需敲一个词的代价。于是 Baton 彻底停用了标题扫描,把一个声明的动词变成唯一权威信号 —— 然后把它毕业到框架。
---
*Phase 2 结束在一个令人不安的数字上:57% 的路由坐在猜测上,因为 45% 的工作单元不浮现任何一个便宜分类器读得到的 cue。本能是去抓一个更强的分类器。正确的答案却相反 —— 干脆别再推断这件事。创建一个工作单元的人本就知道它是设计、实现、审计还是机械劳动。他免费就知道,在 authoring 时刻,只需一个声明词的代价。于是 Baton 把标题扫描作为分类输入扔掉,把一个声明的 `work_verb` 变成唯一权威信号,对着一份真实的 762 单元语料把词汇 ratify,并把它作为一等字段毕业到框架。*

<!-- truncate -->

> *Undeclared 是一个诚实的状态,不是一个错误。一个没有动词的单元会保守地向上路由,并得到一个 nudge —— 绝不会从它的标题得到一个低置信的猜测。*

这是关于 **Baton** 的第三篇,也是一个兄弟实验做了 Loom 此前所做之事的那一篇:把一个干净的基本构件交回 core。在 [coherence bridge](what-the-spec-path-only-proved-existed) 和 [dry-run router](what-the-dry-run-would-have-spent) 之后,这道弧线不是以一个 routing 引擎收尾,而是以一个**字段** —— 其实是两个 —— 加进了其他所有人都在用的框架。

## 1. 转向:别再猜作者本可直接说出来的东西

dry run 的发现很具体([#328](https://github.com/StrangeDaysTech/straymark/issues/328)):可信路由的杠杆是*结构化信号*,不是更细的单元。而分类器最弱的输入,正是它最依赖的那个 —— **标题**。标题被扫描找 cue(`scan_cues`、一张 `CUE_TABLE`),而标题在两个方向上都撒谎:一个冗长的 task 标题制造出幻影"冲突",一个简短的 charter 标题什么都不浮现。更糟,标题扫描是轻易可被 **game** 的,且天生**校准到某一个技术栈**([#321](https://github.com/StrangeDaysTech/straymark/issues/321)) —— 在一个 Go/TypeScript 仓库里管用的 cue 词并不迁移。

于是那个架构转向([#332](https://github.com/StrangeDaysTech/straymark/issues/332),在 #333 里做原型)彻底停用了标题扫描作为 routing 输入,并在它的位置留下唯一一个权威信号:一个**声明的动词**,由作者在单元创建时说出。它的经济性无可匹敌 —— 分类花费 **≈ 0 token**,因为作者已经确定地知道答案,并把它敲一次。标题没有消失;它被**降级**为一个 advisory 交叉核对,只在周期性的校准工具里用,**绝不**在 routing 路径里用。

## 2. 两个字段

**`work_verb`** —— 权威信号,一个封闭的、受控的词汇:

| 值 | 含义 |
|---|---|
| `design` | 开放的架构 / 模式 / 可扩展性决策;spec 的 authoring。 |
| `implement` | 服务逻辑;一个带真实诊断的修复;一个复杂查询;新工具;**定义一个有界的基础契约**。 |
| `audit` | 独立的 review / 验证 / 对着现实的对照。 |
| `operate` | 机械工作:测试、迁移、scaffolding、文档、收尾仪式、批量编辑。 |

**`design_provenance`** —— `{new, upstream}`,可选,默认 `new`。它捕捉残余的认知负荷:一个其工作只是*instrument 一个已在上游做好的设计*的单元是机械的,哪怕它表面的动词看起来是 `implement`。`new` 意味着那个难的决策发生*在这个单元里*;`upstream` 意味着设计已经存在,这个单元只是把它转录或接线。

## 3. 判定规则 —— 光有 enum 不够的地方

光有一个 enum 解决不了那个真正会咬人的歧义。ratify([`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md))固定了四条规则,每一条都是被真实数据浮现出来的:

1. **基础契约规则。**"定义一个有界的基础契约/接口"是 `implement`,**不是** `design`。`design` 保留给*开放的*架构 —— 一个分区模式、一个可扩展性决策 —— 以及 spec 的 authoring。在校准过的 Sentinel 样本里,真正的 `design` 单元有**零**个,与 dry run 里 `planner` 类别为空的发现一致。大多数*感觉*像设计的工作,其实是有界的实现。
2. **残余认知负荷会降级 tier。**`implement` + `design_provenance: upstream` **降级为 operator**。这正是把一个接线*在更早的 charter 里声明过的*指标的 charter(机械 → operator)与一个实现了新逻辑的 charter(implementer)分开的东西。难的思考已经在上游花掉了;instrument 它是便宜的工作。
3. **`design_provenance` 只对 `implement` 有意义。**一个其设计"已在上游存在"的 `design` 是自相矛盾 —— 如果设计已经做好,工作就不是设计,而是实现。所以降级锚定在 `implement` 上;`audit` 和 `operate` 忽略 provenance。
4. **无工作映射到 `operate`。**卫生、收尾仪式、meta-note —— 不引入第五个值。它们作为 `operate` 便宜地路由;"严格意义上的非工作"是一个遥测观察,不是一个 tier。

由此得出的动词 → tier 映射小而清晰:`design` → planner,`implement` → implementer(是 upstream 时降级为 operator),`audit` → auditor,`operate` → operator。

## 4. 同质粒度,与诚实的 undeclared

动词**只声明一次,在最宽的同质粒度上**,在那个已经有声明槽位的最细单元上 —— 不为没有槽位的单元发明新 frontmatter:

| 单元 | 声明槽位 |
|---|---|
| Charter | Frontmatter:`work_verb:` / `design_provenance:` |
| Follow-up | 条目行:`- **Work verb**:` / `- **Design provenance**:` |
| Batch | AILOG ledger 里的一行 `- **Work verb**:` |
| Task | **没有槽位。**从父 charter/spec 继承。`tasks.md` 里零新 frontmatter。 |

Batch 和 Task 从父继承;你只在一个单元真的混合了工作(设计 + 一个机械部分)时,才 override 到更细的粒度。你不会为了迁就 schema 而把一个同质单元切碎。(诚实的现状:原型还没实现继承 —— 它从 charter frontmatter 和 follow-up 行采集动词,把 batch/task 留作 `undeclared`。继承是一条*ratify 过的规则*,待毕业实现来落地。现在把它写下来,是为了阻止毕业去发明一个这条规则明确拒绝的 per-task 机制。)

而那个承重的设计选择:**undeclared 是一个诚实的状态。**一个没有声明动词的单元是不可分类的 —— 它保守地*向上*路由到 frontier 并发出一个 nudge("声明动词"),而且它**绝不**从标题捏造一个低置信的猜测。这个缺口变成一个*动作* —— 一个卫生任务 —— 而不是一个假造的数字。遥测报告 `undeclared_fraction`,那个可行动的指标,而不是标题扫描产出的 `conflict_fraction` 假象。dry run 那个令人不安的 57% 没有被藏到一个更聪明的猜测器背后;它变成了一张待办清单。

## 5. 毕业 —— fw-4.31.0 / cli-3.30.0

这里有一条让这件事花了一整个额外 Charter 而不是一个 commit 的纪律。Sentinel adopter 立了一条明确约束:*不要对着一个未 ratify 的 schema 去 instrument。*所以词汇、规则、placement 和 enforcement 姿态先在一份 ratify 文档里固定 —— 对着 Sentinel 真实的 762 单元治理语料校准 —— 然后*才*让字段在 [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0)([#332](https://github.com/StrangeDaysTech/straymark/issues/332) 第 2 步)毕业到框架。

毕业刻意地安静:

- `work_verb` 和 `design_provenance` 作为**可选 string** 属性加进 `charter.schema.v0.json` —— 刻意**不**是一个 `enum`,好让一个词汇外的值成为一个 advisory 警告,而绝不是一个阻塞的 schema 错误。Charter 模板(EN/es/zh-CN)把它们作为注释指引带上;follow-up 条目 schema 得到对应的 `Work verb` / `Design provenance` 行。
- `straymark validate` 得到两条**只警告**的规则,`CHARTER-WORK-VERB` 和 `CHARTER-DESIGN-PROVENANCE`,它们*只*在一个 Charter 用一个词汇外的值声明该字段时触发。一个缺失的字段**什么都不发**。这个检查从不触碰 exit code。

最后这条性质就是全部重点。Sentinel 的遗留语料是 100% undeclared,而它保持完全安静 —— 不破坏 CI、不产生噪声、不需迁移。声明 schema 不会惩罚任何还没采用它的人。`undeclared` 在框架层面上也是一个诚实的状态。

## 6. 我们刻意没做的事

我们没有发布**执行。**真正的模型派发、真正的定价、config 文件、一块监视 dashboard —— 全是 Phase 3,全被推迟。这道弧线毕业的是一套*分类词汇*,不是一个 router。

我们没有把字段做成**必填**,也没做成一个 CI gate。enforcement 姿态是一个 nudge,契合 v0 实验的谨慎和 adopter 明确的"别破坏遗留语料"这条线。

我们没有宣告 **forward-validation** 已完成。作者在采用后是否真的在一份多样语料上*声明得好*,是 #332 的第 3 步 —— StrayMark 的责任,在 ratify 的范围之外。一个易于声明的词汇也易于被错误地声明,只有真实的采用才会显示这四条规则在 Sentinel 之外是否成立。

而且我们没有为"非工作"重新引入第五个动词,尽管那很诱人。四个值、一个小的 anti-gaming 表面、周期性校准。只在未来的校准显示 `operate` 正在熔掉要紧的信号时才重新考虑。

## 7. 如果你读到了这里

可移植的动作,是那个几乎不花钱就把缺口合上的动作。Baton 花了一整个阶段去建造机器来*推断*某样东西 —— 每个单元是哪一类工作 —— 而这机器封顶在 43% 的置信度,因为它在从一个标题重建一个作者一直揣在脑子里、却从没被要求写下来的事实。一个系统里最便宜、质量最高的信号,常常是某个人在创建那一刻就已确定知道、却根本没被提示去记录的那个。在你为了找回数据的某个属性而去建一个分类器、一个启发式或一个 ML 模型之前,先查一查:上游是否有人确定地、免费地知道它 —— 而唯一缺的,是一个放它的字段。知道一件事最贵的方式,是事后去推断它。最便宜的方式,是在答案还显而易见的时候,问一次。

下一篇是来自框架另一个角落的一段尾声 —— audit cycle —— 那里的问题不是*什么*被验证了,而是框架以为*谁*做了那次验证。

---

*Baton Phase 3(schema 毕业)—— [`fw-4.31.0` / `cli-3.30.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.31.0) · ratify [`06-work-verb-schema-ratification.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/06-work-verb-schema-ratification.md)。Issues [#321](https://github.com/StrangeDaysTech/straymark/issues/321) · [#328](https://github.com/StrangeDaysTech/straymark/issues/328) · [#331](https://github.com/StrangeDaysTech/straymark/issues/331) · [#332](https://github.com/StrangeDaysTech/straymark/issues/332)。前篇:[spec path 只证明了它存在](what-the-spec-path-only-proved-existed) · [这次 dry run 本会花掉多少](what-the-dry-run-would-have-spent)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*