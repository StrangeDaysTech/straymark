---
slug: what-the-spec-path-only-proved-existed
title: spec path 只证明了它存在
authors:
  - jose
tags: [straymark, baton, speckit, coherence, cost-routing, rust, sentinel]
date: 2026-06-25
description: 第四个实验从一个经济问题开始 —— 当订阅补贴结束时,全程全 frontier 就不再负担得起 —— 而它立刻撞上了一个结构性问题。如果工作正在偏离它自己的 plan,你就无法把它路由到更便宜的模型;一个跑在被遗忘的 intent 之上的更便宜的 router,只会更快地把 drift 自动化。所以 Baton 的第一步完全不碰模型。它去读那份 StrayMark 一直以来只证明了*存在*的 plan。
---
*第四个实验 Baton,从一个令人不安的经济预测出发:每月的 AI 订阅是靠一笔消费补贴撑着的,而当补贴结束 —— 按百万 token 计费,就像中国的供应商已经在做的那样 —— 把每一项任务都跑在 frontier 模型上,对独立开发者就不再负担得起。显而易见的解法是:把便宜的工作路由给便宜的模型,把 frontier 留给设计和裁决。不那么显而易见的问题是:一个跑在被遗忘的 intent 之上的更便宜的 router,只会更快地把 drift 自动化。所以 Baton 的第一个交付物什么都不路由。它去读那份 StrayMark 至今为止只检查过是否**存在**的 SpecKit plan —— 并把它和真正被构建出来的东西对账。*

<!-- truncate -->

> *一个跑在被遗忘的 intent 之上的更便宜的 router,只会更快地把 drift 自动化。所以:先 coherence,后 routing。*

这是关于 **Baton** 的第一篇 —— 它是最新的兄弟实验,住在 `experiment-baton/` 里,与 [Loom](where-the-debt-actually-was) 和 OKF 对比并列,拥有自己的 charter、自己的 tag 和自己的 graduation gate。和 Loom 一样,它从第一天起就被设计成:一旦证明了自己的价值,就把一个干净的、typed 的 core 毕业回 StrayMark。和 Loom 不同的是,它从钱开始。

## 1. 为什么是现在

这个预测很无聊,也很难反驳。一个 MAX 档订阅今天大约 \$100/月,而这个价格被普遍理解为坐在供应商目前正吸收的 \$400–600/月消费之上。中国的实验室已经转向了计量的、按百万 token 计费的方式;西方的看上去是时间问题。当补贴蒸发时,我们如今这种工作方式的真实成本 —— 每一次 commit、每一份 PR 摘要、每一次交给那个设计了架构的同一个 frontier 模型去做的原子任务分类 —— 对一个单人 operator 就变得可能负担不起。而这里的 operator,就是我。

整体换供应商是一场高方差的赌注。从今天就能动手的做法,是改变*怎么*花这份算力。不是所有东西都需要 frontier 模型:commit、PR、原子任务清单、分类 —— 低一档的模型做得很好。把 Opus / GPT-5.5 留给设计、架构、裁决,以及开放式的诊断。论点很简单:**把 token 支出对准适合该任务类型的模型**,并依靠 StrayMark 已经产出的治理纪律来判断每个单元是哪一类任务。

这就是全部的 pitch,而它本会是个轻松的周末项目 —— 若不是它以一种需要一个 Sentinel bug 才看得见的方式,悄悄地错了。

## 2. 那个重排了工作顺序的陷阱

这就是一个天真的 router 会径直走进的失效模式。StrayMark 治理的是*实现* —— charter、AILOG、follow-up、TDE —— 而 Loom 从这些信号加上磁盘上的代码,projection 出*涌现的*架构。但*被意图的*架构 —— 那份住在 SpecKit 工件里的全局 plan —— 从不进入这张图。这是验证过的,不是假设的:一个 Charter 可以声明 `originating_spec: specs/004/spec.md`,而 StrayMark 只验证**那个文件存在**。`validate_spec_path()` 从不解析它的内容。Charter 和它的 spec 之间的联系是名义上的。Loom 的架构 projection 读 charter、drift、AILOG、TDE、磁盘清单 —— 却从没有一次打开过 `spec.md`。

所以被意图的架构和涌现的架构,是两个没人对账的平面。intent 稀释进实现里,而这种分歧是不可见的,因为没有任何工件把它们比对。现在想象用一个 router 把这一切包起来,它欢快地把"实现这个任务"发给一个经济档的模型。如果这个任务自己的 plan 已经 drift 了,你没省下钱 —— 你只是让 drift 更便宜、更快地被生产出来。**一个跑在被遗忘的 intent 之上的更便宜的 router,只会更快地把 drift 自动化。**coherence 必须先来,否则 routing 是主动有害的。

### 真实数据上的反派

这不是假想。典型案例是 [issue #304](https://github.com/StrangeDaysTech/straymark/issues/304),Sentinel adopter 里的一个真实 bug。一个 post-MVP 决策(PM-002,躺在某个 spec 的 backlog 里)扩展了一个 per-component 的健康契约。前端 spec —— 一个*不同的* spec —— 从没引用过那个决策。它实现了一个**假定的**契约:字段错了、enum 错了、指标根本不存在。mock 把那个假定编码了进去。测试通过了,一片绿。而它只在 staging 里引爆,报 `TypeError: t.find is not a function`。

同样的机制留下了另外两道 Sentinel 的疤。**mockup 遥测**:一个 agent,需要一个本该由某个规划了但没建的模块提供的数据源,就在它需要的地方悄悄发出了 mockup —— 于是整个遥测表面都出来是假的,建在一个不存在的地基上。还有 **PolicyEngine 的分散**:一个在 `.specify/memory/` 里被记录为干净、专用组件的模块,它的函数却散落进了别的模块,因为那个做局部决策的 agent 从没查过说了另一回事的全局 plan。这些都不是孤立的人为疏忽。它们是同一个结构性窟窿:*一个从不读全局 SpecKit plan 的 agent 做出与设计相悖的局部决策,而没有任何东西告诉它。*

## 3. Phase 1 —— 读取的接缝

SpecKit 的 intent 与 StrayMark 的治理之间有三道接缝。Baton 的第一个 Charter —— [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md),effort L,在被报告的当天就关闭 —— 只构建**读取**这道接缝。摄入 intent、对账、发出一份诊断。read-only:它诊断,不 mutate SpecKit,也不跑 agent。它不碰模型。最后这条约束才是重点 —— 经济驱动是真实的,但第一个交付物对它给予的关注恰好为零,因为 coherence 问题必须在 router 可以安全构建之前先解决。

四块,按顺序在五个 batch 里构建:

- **一个带版本的、read-only 的 SpecKit adapter**([#308](https://github.com/StrangeDaysTech/straymark/issues/308))。锚定到 `speckit_version`(0.11.3,Sentinel 跑的那个),它解析 `specs/**/{spec,plan,tasks}.md`、post-MVP backlog、`contracts/**`,以及自由格式的 `.specify/memory/`。memory 文件被当作*提示*,绝不当作硬契约 —— 宽容解析、低置信 finding,好让人类散文不产生假警报。
- **一个 IntentModel —— 第三个平面**([#309](https://github.com/StrangeDaysTech/straymark/issues/309))。SpecKit 所*声明*的 component、模块和契约的 typed 表示,包括 post-MVP backlog 决策。它在能复用的地方复用 Loom 架构模型的类型,并且**不**重新实现 `glob_match` 或 drift;整个设计原则就是*不要复制 matcher,共享它*。
- **cross-spec 契约溯源边** —— #304 的核心。它们把一个消费者(一个 spec、一个 FR)与定义了它所依赖契约的生产者或决策连起来。本阶段保守地推断;宁可要一个静默的假阴性,也不要一个吵闹的假阳性。
- **一个 coherence 引擎,C1–C4**([#310](https://github.com/StrangeDaysTech/straymark/issues/310)),外加一层 Loom 可消费的 intent overlay([#311](https://github.com/StrangeDaysTech/straymark/issues/311)),把 projection 扩展到第三个平面:*intent vs. 治理 vs. 代码*。高置信 finding 类别优先:一个在 memory 里被意图、却没有文件去实现的 component;一个被消费者要求、却没有生产源的契约字段;一个针对被后续决策改过的形状而构建的消费者。CLI 镜像 `architecture validate` —— `text|json|markdown`,有 finding 就非零退出,可 CI-gate。

原型住在一个 `straymark-baton` crate 里,镜像 `straymark-loom` 独立于 `core` 之外的做法。纯的、typed 的逻辑被暴露出来,好让它日后能毕业到 `straymark-core` —— 但还不是现在,免得一个实验把共享的 core 耦合到可能被丢弃的代码上。

## 4. dogfood 真正抓到了什么

graduation gate 被刻意定得很具体:*若诊断在对着 Sentinel 的 read-only 运行中,抓到至少一个人类 review 放过的真实架构 drift,则 Phase 1 成功。*

对着 HEAD `24d5a66` 的 Sentinel 运行,两个命令前后 `git status --porcelain` 都为空 —— bridge 在目标仓库里不 mutate 任何东西 —— 第一次原始运行发出了 **90 个 finding**,多数是噪声。这正是第一次 dogfood 的预期形状:真实数据会暴露任何 fixture 都不会暴露的精度 bug。四次有针对性的校准修复把它降到了 **6 个 finding,0 blocking**,每一次修复都是只有现实才能抓到的真 bug —— 一个 backlog 决策在声称它的 spec 提到的每一个 endpoint,而不是只声称它自己那一节所引用的(单这一项就把某个 finding 类别从 84 塌成 1);测试文件被算成了契约生产者;一个来自 memory 的匹配在一个子串上误触发。

活下来的头条是一个精确的 C4:

```
[C4] spec '005-frontend-dashboard' consumes contract 'services.public-visibility'
     but never references its defining decision(s): PM-001, AILOG-2026-04-21-002
```

那就是 #304 模式,在结构上、在活的数据上:一个消费者 spec 依赖一个由*另一个* spec 里的决策所定义的契约,却不予承认 —— 正是那种从没有任何东西暴露过、人类 review 也没标记出来的 cross-spec 决策传播 drift。read-only 地、在真实代码上、用一行清晰的话抓到。

intent overlay 补齐了这幅清晰画面的其余部分 —— `DevPortal` 和 `UsageGuard` 有设计却没有模块(真实的缺口);memory 文档里的 `SentinelAgent` 对磁盘上的 `agents/` 目录(一处命名 drift)。还有一个细节让我开始信任这个工具:**PolicyEngine 现在显示 `intended & implemented`。**那个团队曾经忘掉又分散掉的模块,此后已经被建好,而 bridge 正确地*不*标记它。它反映的是当前的现实,而不是一段陈旧的轶事 —— 这正是你对一个要接进 authoring 流程的东西所需要的。

## 5. 我们刻意没做的事

克制的小节 —— Baton 比大多数实验更需要它,因为那个经济 pitch 太诱人。

我们**没碰模型。**没有 tier、没有 token、没有成本、没有 routing —— Phase 1 一样都不引入,是故意的。整个重点是证明 coherence 是先决条件,而先建 router 本会正是这个实验存在着要避免的那个错误。

我们保持**read-only。**bridge 做诊断;它从不往 SpecKit 写回一个 patch。悄悄自动纠正一个契约,恰恰就是那种把 Sentinel 拖进泥潭的"自信地错"。

而且我们交付了一个**诚实的局限**,不是一场干净的胜利。在生成的类型文件上做契约 keying(`types.gen.ts` 把每一个 API 类型都塞进一个文件)会把不同的契约塌到一起,所以字段/enum 不匹配的 finding(C2/C3)在 Sentinel 上还没隔离出来 —— 这次运行 0 blocking 是*按设计如此*,而不是因为 drift 被完全抓住了。它的消费者一侧得到了一个 call-site 绑定修复([#313](https://github.com/StrangeDaysTech/straymark/issues/313));对称的生产者一侧是一个被跟踪的兄弟 follow-up([#319](https://github.com/StrangeDaysTech/straymark/issues/319))。这一切都是 **实验性的(Baton v0 / N=1)**,可能不经弃用周期就改变。一个 coherence 工具的价值主张是信任,而信任被一个自信的假阳性摧毁的速度,远快于被十个真阳性赢得的速度 —— 所以 finding 类别起步很窄,严重度起步很低。

## 6. 如果你读到了这里

可移植的问题,正是那个名义上的 spec-path 检查交到我们手上的。找出你自己系统里一个工件*引用*另一个工件的地方 —— 一份命名了 schema 的 config、一个声明了依赖的服务、一张链接了设计文档的 ticket —— 然后问:这个引用到底验证了什么。这个检查确认被链接的东西*存在*,还是确认这两样东西仍然*彼此一致*?StrayMark 验证了 `originating_spec` 指向一个真实文件,验证了好几个月,而这一整段时间里它感觉都像是一个链接。它在证明存在、暗示一致,而这两者之间的缝隙,正是一个前端被建到一个没人更新过的契约之上的地方。每一个你没有亲手对过账的"它链上了,所以没事",都配得上同样的怀疑 —— 不是*链接在不在*,而是*这个链接除了文件名之外还检查了什么吗*。

下一篇是 Baton 终于开始看钱的地方 —— 而它发现,它以为会是那根杠杆的东西,并不是。

---

*Baton Phase 1 —— [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md) · 概念 [`01-baton-concept.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/01-baton-concept.md) · dogfood [`03-sentinel-dogfood-report.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/03-sentinel-dogfood-report.md)。Issues [#304](https://github.com/StrangeDaysTech/straymark/issues/304) · [#308](https://github.com/StrangeDaysTech/straymark/issues/308) · [#309](https://github.com/StrangeDaysTech/straymark/issues/309) · [#310](https://github.com/StrangeDaysTech/straymark/issues/310) · [#311](https://github.com/StrangeDaysTech/straymark/issues/311) · [#313](https://github.com/StrangeDaysTech/straymark/issues/313)。兄弟系列:[债务究竟在哪里](where-the-debt-actually-was)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*