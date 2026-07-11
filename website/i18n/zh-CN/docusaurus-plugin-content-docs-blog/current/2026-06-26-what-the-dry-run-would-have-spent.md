---
slug: what-the-dry-run-would-have-spent
title: 这次 dry run 本会花掉多少
authors:
  - jose
tags: [straymark, baton, cost-routing, speckit, telemetry, rust, sentinel]
date: 2026-06-26
description: Baton 的第二阶段终于看了钱 —— 但它什么都没路由。它对 StrayMark 已经在 Sentinel 里记录的 762 个工作单元做了分类,给每个推荐一个 tier,并打印出一套 routing 策略相对全 frontier 本会花掉多少。头条是一个 ~93% 的示意性上限。而诚实的读法推翻了这个实验自己的假设:granularity 从来不是那根杠杆。信号覆盖率才是。
---
*coherence bridge 故意不碰模型。Baton 的第二阶段终于做那件经济的事 —— 却仍然什么都不跑。它读取 StrayMark 已经在 Sentinel adopter 里记录的 762 个工作单元(charter、batch、follow-up、task),用便宜的确定性信号给每个分类,推荐一个模型 tier,并发出**经济遥测**:什么本会去哪里,以及在一套 routing 策略下相对把所有东西都发给 frontier 模型本会花掉多少。没有 agent 被派发;没有模型被调用。重点不是现在就省 —— 而是让潜在的节省可见,并检验一条硬规则:决策的成本不得吃掉节省。它没有。但数据回答的,是一个和我们所问不同的问题。*

<!-- truncate -->

> *如果 overhead 在某个 granularity 抹掉了节省,那是一个有效的结果,而不是一个要藏起来的失败。*

这是关于 **Baton** 的第二篇(在 [coherence bridge](what-the-spec-path-only-proved-existed) 之后)。Phase 1 确立了:你无法在正在 drift 的 intent 之上安全地路由。Phase 2,[`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md),把分类-与-推荐这一层构建成一次**回溯式 dry run** —— 而在此之前一个 Charter,它先把*coherence 检查何时该触发*这个环闭上了。

## 1. 激活接缝 —— 把检测挪早

Phase 1 交付了 `coherence` 和 `overlay`,作为 on-demand、read-only 的命令。但 #304 的 drift —— 那个针对假定契约构建的前端 —— 诞生于一个 agent **在任何人跑诊断之前就写代码**的那一刻。on-demand 检测来得太晚。

于是中间的一个小 Charter,[`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md)([#316](https://github.com/StrangeDaysTech/straymark/issues/316)),把信号挪到 authoring 时刻。它是一个 **SpecKit 扩展**,挂上 `before_implement` 生命周期事件并运行 Phase 1 的引擎 —— 通过一个 `--spec` flag 收窄到活跃的 feature,为了速度 —— 在 agent 写代码*之前*把高置信 finding 呈给它。这个先例早已存在并在 Sentinel 里验证过:SpecKit 的 `agent-context` 扩展跑 `after_specify`/`after_plan` 来刷新上下文。Baton 发一个类似的扩展,它在 `before_implement` 上检查 coherence,而不只是刷新。

设计上的偏向是 Phase 1 挣来的那条:**默认 advisory**,而非硬 gate。一个打断 authoring 的假阳性,比一个出现在 on-demand 报告里的假阳性花掉更多信任,所以这个 hook 呈上 blocking 和高置信的 finding,把决定权留给人类,硬 gate 只通过 config 提供。而且它优雅降级 —— 如果找不到 `straymark-baton` 二进制,hook 会告警并让开;它绝不打断 SpecKit 流程。在 Sentinel 里 dogfood 时,对 feature `005-frontend-dashboard` 的 `before_implement` 在 advisory 模式下呈出了那个真实的 C4(`services.public-visibility` 被消费却不引用其定义决策),仓库丝毫未动。这就是 issue #304 的第三个诉求 —— *"在 authoring 时刻给 drift 信号,而不只是在 audit 时刻"* —— 以它最强的形式。

## 2. dry-run router —— 先做三个构帧决定

然后是经济这一层。三个构帧决定,在写下一行之前就和 operator 定好了,因为概念文档刻意把它们留了开口:

1. **可路由单元:instrument 已有之物,并度量。**不发明词汇。一份更早的顾问草稿曾提议一个仓库里并不存在的 "Work Unit / WU-NNN" 概念;我们扔掉了那个*名字*,却保留了它所指向的真实问题。Baton 在 StrayMark **已经记录**的 granularity 上分类 —— Charter、Batch、Follow-up、Task —— 度量每一层有多异质,并让遥测揭示哪个 granularity "划得来"。那个悬而未决的子决策,用数据来回答,而不是靠命令。
2. **成本模型:config 里的示意性 tier。**`config.yml` 里的一个 `baton:` 块(和 [#279](https://github.com/StrangeDaysTech/straymark/issues/279) 同一个 config-driven 模式)按 tier 声明一个*示意性*的每百万 token 成本。那解锁了 dry run,而不必解决真实的供应商成本身份(推迟到 Phase 3)。我们度量**相对**节省,不是一张账单。
3. **分类器:只用便宜信号,不用 LLM。**跑在 StrayMark 已经计算的信号之上的确定性规则。这直接满足了那条硬的经济推论:分类必须便宜,否则分类器就变成了它本该避免的那笔成本。一个 LLM 分类器仍是未来的升级,只在便宜信号有歧义*且*节省值得时才用。

可路由单元就是 StrayMark 已经记录的工作:45 个 Charter、82 个 Batch、135 个 Follow-up、500 个 Task —— **762 个单元**。每一个都从便宜信号被分类进一个任务类别 —— Planner/Architect、Implementer、Auditor、Operator:`effort_estimate`、`analyze` 的复杂度、Charter 的 `risk_level`、一个 follow-up 的 bucket 和严重度、Loom projection 里 component 的状态、Phase 1 的 coherence finding、被触及的文件表面。typed、纯、无 I/O。一套 tier 策略以保守 fallback 把类别 → tier 映射,`route --dry-run` 打印这份经济账。它推荐。它从不执行。

### 那条硬规则,一上来就说清

统辖整个设计的约束:

> 分类、原子化和路由的成本,不得等于或超过不把所有东西都压给一个 frontier 模型所带来的节省。如果解法的复杂度与没有它的成本持平,那这个解法就不存在。

遥测**显式地**度量这一点 —— 每个 granularity 的 overhead 相对节省。如果某个 granularity 的 overhead 等于或超过它的节省,它就被报告为*不可路由*,不被硬塞。这就是诚实守卫:一个负面结果也是一个结果。

## 3. 头条经济账(示意性)

对着 Sentinel 真实的治理语料 read-only 运行,`git status` 未动:

```
ALL (762 units) — routable
  tiers: economic 617 · frontier 14 · local 131
  cost: all-frontier 1293.60 → routed 93.68  (gross saving 1199.92 ≈ 93%)
  overhead 15.24 → net saving 1184.68
  caveats: 57% low-confidence · 57% of saving on low-confidence routing · 12% conflicted
  sensitivity: breakeven overhead/unit 1.575 · robust at 2× overhead: true
```

每一个 granularity 都算出净正、且对两倍示意性 overhead 稳健。分类 overhead 是毛节省的 **1.2%** —— 所以那条硬规则并没有被决策的成本违反。按 gate 的字面意思,routing 在所有地方都划得来。如果我读到那里就停,我本会发布一个凯旋的数字。

## 4. 诚实的读法 —— 是置信度,不是 granularity

§4.2 的裁决说"到处都可路由"。诚实守卫说这份节省是**脆弱的**,而且它说这话的方式,与这个实验的立基猜想相悖。

| Granularity | 冲突 % | high+med 置信 % |
|---|---|---|
| Charter | 6% | 46% |
| Batch | 8% | 39% |
| Follow-up | 5% | 37% |
| Task | 15% | 44% |

两个发现,都与假设相反:

**异质性*不是*那个拦路的。**猜想曾是:一个粗单元 —— 一整个 Charter —— 捆着混合的工作(设计 + 代码 + 审计 + 清理),因此抗拒干净的路由,而一个更细的单元会路由得更好。数据把它翻了过来:**Task** 的冲突*最高*(15%),Charter 属于最低(6%)。原因很泄气 —— 冲突这个指标被**标题的冗长度**混淆了。task 标题是描述性的,于是浮现更多 cue,于是检测出更多"冲突";charter 标题是简短的。冲突是异质性的一个弱代理,而没有哪个 granularity 明显更同质、值得偏好。

**信号覆盖率才是那条约束的约束。**high+medium 置信在**所有 granularity 上都坐落在 37–46%** —— 平的。**每一层都有 57% 是低置信**,而且不是因为冲突(5–15%),而是因为*无 cue 默认*:45% 的单元根本不浮现任何分类 cue。高置信需要 `effort_estimate`,而只有 Charter 带着它;大多数单元靠一个简短的标题就被分类了。所以**那 93% 毛节省里的 57% 都坐在低置信的猜测上。**

对可路由单元问题的经验回答:*哪个 granularity 可路由?*按净节省,全都可;按信任,没有哪个比另一个更干净。**在这份语料上,granularity 不是那根杠杆。信号覆盖率才是。**引入一个更细的子单元 —— 那个被丢弃的 "Work Unit" 词汇所伸手去够的东西 —— 不会有帮助:Task 已经是最细的,而它并不比一个 Charter 更可信。

## 5. 毕业的是知识,不是节省

gate 恰恰允许这一点:*在某个 granularity 上、扣除 overhead 后的净正相对节省,并指名是哪个;而一个带证据的净负结果,依然让知识毕业。*

它 MET 了 —— 而它让之毕业的是知识,这结果原来更有价值。dry run 确立了 routing 的**上限**(~93% 示意性,能扛过 2× overhead)和它当前的**信任下限**(~43% 的单元以 high+medium 置信路由,所以 ~57% 的节省是猜测)。而且它**重新构帧了 Phase 3**,靠数据而非断言:通往一份*可信*节省的路,不是一个不同的可路由单元。而是**把那些被推迟的信号接上线** —— 每函数复杂度(需要把 `analyze` 从 `cli` crate 毕业进 `core`)、来自 Loom projection 的架构状态、来自 Phase 1 的 coherence finding —— 好在任何东西执行*之前*抬高置信度。B2 在 cheap-first 规则下推迟了那些更重的信号;dogfood 刚刚回溯地把它们证成为下一步。这正是那个经验循环按预期运转。

## 6. 我们刻意没做的事

仍然是**只推荐。**`route` 要求 `--dry-run`;没有模型客户端被链接,没有 agent 被派发,没有网络调用被发出。真正的执行是 Phase 3。

成本是**示意性的**,并且到处都标注为如此。93% 是*形状*,不是钱 —— 真实的供应商成本身份被推迟,直到有一份统一契约可建模,而现在还没有。报告一个假的美元数字,会是它自己的一种 mockup 遥测。

而且我们抵住了那个诱人的补丁。当便宜信号对 45% 的单元覆盖不足时,显而易见的动作是抓一个 LLM 分类器去填坑。我们没有 —— 那会违反 cheap-first 规则,并把真正的发现(信号覆盖率)埋在一个更贵的猜测之下。覆盖不足被归档为一个 follow-up,而不是藏起来。这一切仍是 **实验性的(Baton v0 / N=1)**。

## 7. 如果你读到了这里

可移植的教训,是诚实守卫对一个漂亮数字所做的事。Baton 本可以发布"便宜 93%"并且是在说真话 —— 毛节省是真的。它也本会深具误导性,因为其中一多半骑在工具在无据可依时的猜测上。真正要紧的数字不是节省;而是节省*背后的置信分布*,而这两者指向相反的方向。找出你世界里一个带着好消息的头条指标 —— 一次转化提升、一次延迟胜利、一次成本下降 —— 然后问:其中有多大比例坐在度量真正有把握的那些情形上。一堆猜测之上的真均值,仍然是一堆猜测。dry run 最有用的产出不是那个 93%。是它旁边那个 57%。

下一篇是那个 57% 的修复原来几乎不花钱的地方 —— 因为作者一直都知道答案。

---

*Baton Phase 2 —— [`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md) · [`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md) · dogfood [`04-phase2-dry-run-dogfood.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/04-phase2-dry-run-dogfood.md)。Issues [#316](https://github.com/StrangeDaysTech/straymark/issues/316) · [#323](https://github.com/StrangeDaysTech/straymark/issues/323) · [#324](https://github.com/StrangeDaysTech/straymark/issues/324) · [#325](https://github.com/StrangeDaysTech/straymark/issues/325) · [#326](https://github.com/StrangeDaysTech/straymark/issues/326)。前篇:[spec path 只证明了它存在](what-the-spec-path-only-proved-existed)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*