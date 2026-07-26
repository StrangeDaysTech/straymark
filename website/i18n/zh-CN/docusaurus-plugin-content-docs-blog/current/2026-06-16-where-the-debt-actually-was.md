---
slug: where-the-debt-actually-was
title: 债务究竟在哪里
authors:
  - jose
tags: [straymark, loom, architecture, technical-debt, visualization, rust, sentinel]
date: 2026-06-16
description: 操作员每天真正的问题不是"把文档给我看",而是"我们现在在哪里?"。Loom 长出了第二个表面 —— 把架构当作一栋可以走进去的建筑,既有 2D 也有 3D —— 而它的技术债 overlay 却是空的。不是因为没有债务。是因为它不知道该把债务画在哪里。
---
*Loom 起初是一张文档的知识图谱。然后参考 adopter 大致这样说:那是我文书工作的一幅美丽图画,但我每天的问题是"我们现在在哪里?" —— 针对**系统**而言 —— 它的 component、它的 layer、什么已经建好、什么还欠着。于是 Loom 长出了第二个表面:把架构当作一栋建筑,只 author 一次,同时渲染为 2D 平面图和 3D exploded 模型,配上一层让 component 点亮的状态 overlay。技术债的 overlay 在第一次运行时是空的。债务有的是。这层 overlay 只是完全不知道哪些 component 背着它 —— 而把这件事做对花了三次尝试,其中两次看上去都已经完成了。*

<!-- truncate -->

> *那层空 overlay 是这个月最诚实的 bug:它对债务的判断没有错。它错在该把债务画在哪里。*

这是 Loom 系列的第三篇 —— 在[抽出共享解析器](what-the-second-reader-demanded)和[让文档图谱的边真正解析](what-the-graph-couldnt-draw-yet)之后。在这一篇里,Loom 不再只是文档的一个视图,而成为*系统*的一个视图;也是在这一篇里,一层看似简单的 overlay 教会了我们一个关于每一块曾经向你展示自信"零"的仪表盘的道理。

## 重新构帧

这次转向记录在 [`ADR-2026-06-02-002`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-02-002-architecture-plan-format.md)。Loom 原本被定义为文档的知识图谱视图;来自 Sentinel 的操作员反馈把它重新构了帧。对一个大型项目而言,操作员每天的问题不是"把文档之网给我看",而是*"我们现在在哪里?"* —— 针对**实现地图**而言,也就是架构。这个隐喻来自土木工程和 BIM:**一个模型,多种视图**,layer 点亮以显示状态。

这次重新构帧撞上了探索早已标好的一堵墙:**StrayMark 没有 `component` 的概念。**它把文档映射到*文件* —— Charter 的"Files to modify"、AILOG 的"Modified Files"、`api_changes`。所以一个架构表面需要三样新东西:一种*model* component 和 layer 的方式,一种把活的治理状态 *project* 到它们之上的方式,以及一种能承载人类 author 的布局、又不被工具踩烂的可编辑格式。使能的事实其实早已在手 —— Charter 状态(`declared` / `in-progress` / `closed`)和声明对比修改的 drift 今天就在计算,ADR 已经内嵌 C4 图和"Affected Components"表,而 `.straymark/` 的 00–09 各 stage 形成了一种天然的 layering。

## 从终端看"你在这里"

前一半*完全没有任何图形*就发布了 —— [`cli-3.25.0` / `core-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.25.0),Loom 轨道 A1。这一注下在:先在终端里回答"我们现在在哪里?",证明这个 projection,然后再去画它。

- **`straymark architecture generate`** 把代码库结构挖成初稿 `model.yml` 加一份 `plan.drawio`,并用 ADR 的信号加以丰富(C4 图、Affected-Components 表改善了标签和链接)。
- **`straymark architecture sync`** 是 append-only 的 —— 它检测新的源目录或 ADR component 并把它们加进去,**绝不踩烂人类编辑或 DrawIO 几何**。默认 dry-run。
- **`straymark architecture validate`** 报告 model↔plan 的完整性信号(`undrawn` / `unmodeled` / `empty`),任一出现即以非零退出,因此可以 CI-gate。
- **`straymark status --where`** 是文本版的"你在这里":它从活的治理信号把每个 component 的状态 —— `active` / `in-progress` / `implemented` / `has-debt` / `uncharted` —— project 出来并打印一份摘要。

承重的决定在 `straymark-core` 里:这个 projection 是一个**纯**的 `(model + GovernanceState) -> Projection` 函数,零 I/O,由 CLI 的 `status --where` 和 Loom 服务器共享。这不是为整洁而整洁 —— 它正是驱动整个系列的那条原则。终端的答案和视觉的答案由*同一份代码*计算出来,所以它们**不可能彼此矛盾**(规范称之为 NFR3)。一块和 CLI 自相矛盾的仪表盘比没有仪表盘更糟;这条原则让矛盾在构造上无法被表示出来。

## 一个模型,两种视图

然后是两种视图,都读取那一份 projection。[`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0)(A2)用 **maxGraph** 渲染人类 author 的 `plan.drawio`,保留你摆出的几何(NFR1),并把 §4 的状态以非破坏性的单元颜色 overlay 上去。[`loom-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.0)([#268](https://github.com/StrangeDaysTech/straymark/issues/268))加入了 Spec 002 的 north star:一个 `2D | 3D` 开关,把平面图换成一幕 **Three.js axonometric 场景** —— 每个 layer 是一层堆叠的半透明 floor,每个 component 是一个用同一套"你在这里"调色板上色的盒子,依赖边画在盒子之间,还有一个 **explode 滑块**沿垂直轴把各层 floor 剥开,同时依赖线重新布线到盒子的新位置。这就是字面意义上的 BIM exploded view:CLI author 结构,Loom 把它渲染为 2D 平面图*和* 3D 模型,而状态 overlay 始终是那一份共享的 projection。切换视图会干净地释放 GL 上下文;点击一个盒子打开的是 2D 平面图所用的同一块 component 详情面板。

<figure>
<img src="/img/blog/loom/arch-2d-thumb.png" data-zoom-src="/img/blog/loom/arch-2d.png" alt="Loom 2D Architecture Plan:扁平的 component 盒子 —— cmd(binaries)、Agents(AIOps)、Audit Trail、CommsHub、Identity、MCP Server —— 汇入 Core(eventbus / storage / ids),每个盒子按 has-debt 标红或按 implemented 标蓝" loading="lazy" />
<figcaption><em>2D Architecture Plan 的一处细节 —— 由 maxGraph 渲染的人类 author 的 <code>plan.drawio</code>,实时 overlay 上共享的状态 projection(红 = <code>has-debt</code>,蓝 = <code>implemented</code>)。点击查看完整平面图。</em></figcaption>
</figure>

<figure>
<img src="/img/blog/loom/arch-3d-thumb.png" data-zoom-src="/img/blog/loom/arch-3d.png" alt="Loom 3D axonometric BIM 视图:架构 layer 作为堆叠的半透明 floor(CORE、MODULES、ENTRYPOINTS),component 作为按 has-debt 标红或按 implemented 标蓝的盒子,依赖线画在它们之间" loading="lazy" />
<figcaption><em>同一个模型作为 3D exploded view —— 每个 layer 一层半透明 floor,每个 component 一个用同一套状态调色板上色的盒子,依赖边在盒子之间;explode 滑块沿垂直轴把各层 floor 剥开。点击查看完整场景。</em></figcaption>
</figure>

绕着你自己的架构飞行、看着那个 active 的 component 发光,真的是一种享受。而在第一次真正运行时,那层本该是重点的 overlay —— *技术债在哪里?* —— 是空白的。

## 债务究竟在哪里

`has-debt` overlay 在说出真相之前撒了三次谎,而每一次谎言都像一个完成的 feature。

**谎言 #1 —— 永远是空的。**一个 open 的 TDE(Transversal Debt Entry)在 frontmatter 里声明它的 `related` 文档。那些是*治理文档* —— AILOG、审计 review、Charter —— **不是源路径**。所以当 projection 试图把一个 TDE 的债务匹配到某个 component(它由文件 glob 定义)时,它每次都匹配不到任何东西。这层 overlay 报告的不是"没有债务";它报告的是"我在一个债务从不被记录的地方找债务"。[#273](https://github.com/StrangeDaysTech/straymark/issues/273)([`cli-3.26.0` / `core-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.26.0))架起了这道桥:把每个 `related` AILOG 解析到*它*记录为已修改的文件,再把那些源路径喂进匹配。其中有一处微妙之处立刻咬了人 —— 一个 AILOG 的已修改文件是它 `files_modified` frontmatter 列表与它 `## Modified Files` 表的**并集**,因为较旧的日志往往只带列表,只读表会静默地把它们漏掉(它在 Sentinel 上点亮了 Identity/Core/Database,却漏掉了 AuditTrail/CommsHub)。

**谎言 #2 —— 过度归因。**现在债务出现了,而且*到处*都是。一个关于 AuditTrail 模块的 TDE 也点亮了 `cmd`、`db` 和 `integration` —— 因为某一个 AILOG 在同一次变更里把它们全都接上了线,而 projection 继承了那个 AILOG 的整个 footprint。overlay 从空白变成了一摊涂抹。技术上讲,它把债务归给了"被该 TDE 所引用的 AILOG 所触及的 component",这是一句站得住脚的话,却是错误的答案。债务是关于 AuditTrail 的;footprint 却是整个子系统。

**谎言 #3 —— 精确了。**[#276](https://github.com/StrangeDaysTech/straymark/issues/276)([`fw-4.28.0` / `cli-3.28.0` / `core-0.8.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.28.0))给了 TDE 一种说出它究竟是关于什么的方式:一个 **`affects`** 字段,一组文件 glob(`["internal/modules/audittrail/**"]`)。存在时,projection 把债务**恰好**归给那些路径,并忽略更宽的 AILOG footprint。缺少 `affects` 时,footprint 归因作为 fallback 保留 —— 加性的,无迁移,对不为自己划界的 TDE 也没有 regression。框架在 `TEMPLATE-TDE.md` 里发布这个字段;Loom 的 [`loom-0.6.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.2) 完全不需要改代码 —— 它只是对着 `core-0.8.0` 重新构建,因为 projection 是共享的,视觉 overlay 搭上了终端搭上的同一趟修复。

还有第四次更安静的纠正值得点名。即便债务落到了正确的 component 上,一个*既* implemented *又*处于债务中的模块会被涂成平静的"implemented"蓝,债务永远不显示。于是 overlay 获得了一种**优先级**:`has-debt` 和 `wiring-gap` 的排序高于 `implemented`,在 2D 平面图、3D 视图和图例里都是如此。一个 component 涂上的是那个需要关注的状态,而不是那个让人感觉良好的状态。

<figure>
<img src="/img/blog/loom/arch-3d-explode-thumb.png" data-zoom-src="/img/blog/loom/arch-3d-explode.png" alt="Loom 3D 视图的一次贴近环绕:一排向下排列的 component 盒子,背着技术债的红色盒子(CommsHub、Identity)挨着 implemented 的蓝色盒子(MCP Server、Policy Engine、Status Center)" loading="lazy" />
<figcaption><em>环绕这同一栋建筑:红盒子背着 open 债务,蓝盒子是 implemented —— 这层 overlay 花了三次尝试才把颜色落到正确的 component 上。点击查看完整场景。</em></figcaption>
</figure>

## 把它开放出去

两次 release 让这个架构表面在孕育它的那两个仓库之外也可用。[#279](https://github.com/StrangeDaysTech/straymark/issues/279)([`cli-3.27.0` / `core-0.7.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.27.0))让种子扫描器变得**与语言和结构无关**:一个 `ScanConfig`,从 `.straymark/config.yml` 里一个可选的 `architecture:` 小节解析而来,开箱识别约 30 种语言,并跳过像 `src/main/java` 这样的构建 scaffolding,使得一个 Maven/Gradle multi-module 项目能为每个模块得到一个 component,而不是坍缩成单独一个 `main` 盒子。一个用非默认语言写的项目不再是一张空白地图。而 [#280](https://github.com/StrangeDaysTech/straymark/issues/280) 交付了 adopter 指南 —— [`docs/adopters/LOOM.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md),含 EN、ES 和 zh-CN 三版,已在 [straymark.dev](https://straymark.dev) 上线 —— 附带推荐的 `generate → refine → sync → validate` 工作流。

## 我们刻意没做的事

克制的小节。这一切仍是**实验性的(Loom v0 / N=1)**,可能不经弃用周期就改变 —— 架构模型、CLI 动词、overlay 语义、3D 视图,都是如此。这个与语言无关的扫描器对现有 Go/Rust/JS 布局的默认行为没有改变(Sentinel 仍然种出同样的 14 个 component)—— `#279` 只*增加*触达范围,并不重塑已经工作的东西。而 `affects` 是加性的:footprint fallback 保留下来,所以这个字段对想要精确的 TDE 是一项 opt-in,对不想要的并非新义务。面对一个 north-star feature 的诱惑,是因为它终于看上去够惊艳而把它稳定下来。它看上去惊艳,*而且*它会保持 v0,直到 Sentinel 之外的第二个项目走过它自己的那栋建筑。

## 如果你读到了这里

可移植的练习正是那层空 overlay 交到我们手上的。找出你世界里某块向你展示令人安心的数字的仪表盘、报告或指标 —— 零个 open 事故、零个失败检查、这个服务上零债务。现在问问:那个零的意思是"我找过了,确实没有",还是"我在一个这东西从不被记录的地方找过了"。`has-debt` overlay 并没有在债务的*数量*上撒谎;它撒谎的是它自己找到债务的*能力*,而且它是带着一抹完全平静的蓝色撒的谎。那三次迭代 —— 空白、涂抹、划界 —— 正是把颜色变成读它的人所假设的那个含义所付出的代价。每一个你没有亲手追踪过的自信的零,都配得上同样的怀疑:不是*这个数字对不对*,而是*产出它的那个东西到底知不知道该往哪里看?*

---

*StrayMark `cli-3.25.0` → `cli-3.28.0`、`core-0.5.0` → `core-0.8.0`、`fw-4.27.0` → `fw-4.28.0`、`loom-0.5.0` → `loom-0.6.2` — ADR [2026-06-02-002](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-02-002-architecture-plan-format.md) · Spec [002-architecture-plan](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-loom/specs/002-architecture-plan/spec.md) · Adopter 指南 [LOOM.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md) · Issues [#268](https://github.com/StrangeDaysTech/straymark/issues/268) · [#273](https://github.com/StrangeDaysTech/straymark/issues/273) · [#276](https://github.com/StrangeDaysTech/straymark/issues/276) · [#279](https://github.com/StrangeDaysTech/straymark/issues/279) · [#280](https://github.com/StrangeDaysTech/straymark/issues/280)。前篇:[第二个读者提出的要求](what-the-second-reader-demanded) · [这张图还画不出来的东西](what-the-graph-couldnt-draw-yet)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*
