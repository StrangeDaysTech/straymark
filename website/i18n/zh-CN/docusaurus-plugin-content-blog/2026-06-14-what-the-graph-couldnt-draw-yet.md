---
slug: what-the-graph-couldnt-draw-yet
title: 这张图还画不出来的东西
authors:
  - jose
tags: [straymark, loom, knowledge-graph, visualization, rust, sentinel]
date: 2026-06-14
description: Loom 的首个 release 在不到一秒内就把文档图谱实时渲染进了浏览器。它同时也画出了 330 条通向虚空的 edge。看得见的那个 feature 是简单的部分 —— 让引用真正解析到位才是真正的工作，而它教会了我们：dangling edge 何时是 bug，何时是信号。
---
*这具能走路的骨架交付得很快，看起来也很棒：一张涵盖每一份 StrayMark 文档的 force-directed 图谱，按类型着色，在保存文件后的一秒之内就在浏览器里重建完成。然后我们把它指向一个真实的 corpus —— [Sentinel](https://github.com/StrangeDaysTech/sentinel)，395 条引用 —— 其中 330 条 dangling。不是因为文档坏了，而是因为它们以人类书写的方式互相引用，而非一个朴素的图谱构建器所匹配的方式。渲染只用了一个周末。让那些 edge 落到实处，耗掉了 Loom M1 余下的全部 follow-ups，并且改变了"断链"二字的含义本身。*

<!-- truncate -->

> *395 条引用里 330 条 dangling。这张图在技术上完全正确，在实践中毫无用处。*

这条弧线的第一篇讲的是管道工程 —— 抽出 [`straymark-core`](what-the-second-reader-demanded)，让 CLI 与 Loom 用同一份代码解析文档。这一篇讲的是当那个共享的图谱构建器遇上一个它从未被调校过的 corpus 时发生了什么，以及它如何发现：画出一张图，恰恰是谁都不该为之惊艳的那部分。

## 能走路的骨架

[`loom-0.1.0` / `cli-3.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.24.0)（[#240](https://github.com/StrangeDaysTech/straymark/issues/240)）就是 Loom M1：一个仅 loopback、read-only 的 Web 仪表盘，实时渲染项目的文档图谱。

技术栈记录在 [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) 里：一个 **`axum` + `tokio`** 服务器，通过 `straymark-core` 构建图谱，并提供一个小巧的读 API 外加一个 WebSocket；一个 **`notify`** 文件系统 watcher（250 ms debounce），在 `.md` 文件改动稳定下来后重新解析，并通过 socket 推送一次重建 —— 打开着的浏览器会在远不到一秒内反映出一次编辑（实测约 ~255 ms）；一个 **Sigma.js + graphology** 前端，force-directed，按文档类型着色，选中一个 node 会点亮它完整的传递性 thread 并淡化其余部分。整个 Web UI 都**通过 rust-embed 嵌入** —— adopter 从不运行 `npm`，他们下载的是一个二进制。

两条不可妥协的要求来自规范的安全姿态，值得点名，因为它们塑造了之后的一切：服务器**专一绑定 `127.0.0.1`** 并**拒绝非 loopback 的 `Host` 头**（防 DNS-rebinding），而且它在构造上就是 **read-only** 的。没有一条以后再补、却忘了加固的写入路径，因为根本就不存在写入路径。CLI 的 `straymark loom serve` 是一个按需下载的启动器 —— 下载这道闸门*正是*实验性的 opt-in 边界 —— 并在离线时回退到缓存的二进制。

## 从画面到仪器

M2 和 M3 把画面变成了你会真正去用的东西。[`loom-0.2.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.2.0)（[#241](https://github.com/StrangeDaysTech/straymark/issues/241)）加入了**Louvain community 检测**，配上 cluster 着色、一个 corpus 统计面板（按类型/状态/风险计数，可导航的 orphan，dangling 引用），以及服务端按类型、状态、风险、tag 和日期范围的过滤。[`loom-0.3.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.3.0)（[#243](https://github.com/StrangeDaysTech/straymark/issues/243)）加入了**增量重建** —— 一个解析 cache 只重新解析改动过的文件，SPA 原地打补丁更新图谱，保留所有未移动元素的布局，而不是每敲一个键就把整张画布重新洗牌一遍 —— 外加在有向语义 edge 上的**依赖环（SCC）报告**、基于 centrality 的 node 尺寸（默认 Betweenness，于是桥梁文档会变大）、搜索并对焦摄像机、"pin subgraph"以隔离一条 thread、在编辑器中打开的深链，以及由项目配置语言驱动的完整 UI 国际化（`en` / `es` / `zh-CN`）。

<figure>
<img src="/img/blog/loom/kg-graph-thumb.png" data-zoom-src="/img/blog/loom/kg-graph.png" alt="Loom 知识图谱视图：一张 StrayMark 文档的 force-directed 图谱，node 按 community 着色、按 centrality 调整尺寸，带有诸如 'Decommissioning Saga step 3' 和 'Observability bundle: 7 instruments' 这样的标签" loading="lazy" />
<figcaption><em>Loom 知识图谱视图的一处细节 —— 文档呈现为一张 force-directed 图谱，node 按 Louvain community 着色、按 betweenness centrality 调整尺寸，于是桥梁文档会变大。点击查看完整 corpus。</em></figcaption>
</figure>

这一切都是货真价实的，而这一切都坐落在一张大多数 edge 通向虚空的图谱之上。

## 395 里的 330

这是重新定义了整个项目的那个数字。拿 `loom-0.1.0` 对 Sentinel 做 dogfooding，**395 条引用里有 330 条 dangling** —— 被画成通向并不存在的 node 的 edge。一张连自己 corpus 的六分之五都连不起来的 knowledge graph，不是 knowledge graph；它是一张带着妄想的散点图。

成因不是数据损坏。而是 StrayMark 文档以一个人书写引文的方式互相引用，而图谱构建器却像一个期待主键的数据库那样去匹配。一份 AILOG 写着 `CHARTER-12`，或用 basename 称呼一个文件，或给出一段相对路径后缀 —— 而构建器在寻找一个精确的 node id 时，什么都没找到。更糟的是，整类目标*根本就不是 node*：图谱发现了 AILOG、TDE 和 ADR，但 Charter、PLAN 遥测和 audit 评审住在 `.straymark/` 里、从未被注入，于是每一条指向它们的引用都按定义 dangling。

[`loom-0.4.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.0)（[#246](https://github.com/StrangeDaysTech/straymark/issues/246)）修好了两半 —— 而且因为修复就住在共享的 `straymark-core` 图谱构建器里，`straymark audit` 免费获得了同样的连通性：

- **引用规范化（R1）。**构建器现在按精确 id 解析一条 edge 的目标，若失败，则按唯一的文件 basename、唯一的相对路径后缀、`CHARTER-NN` 前缀、或带日期 id 的前导前缀来解析 —— **绝不解析有歧义的匹配**（两个候选意味着它保持 dangling，因为一条错的 edge 比一条缺失的更糟）。已解析的目标会规范化为 node id。
- **实体 node（R2）。**一个新的 `core::entities` 模块发现 `.straymark/charters/*.md`、`plans/PLAN-*.telemetry.yaml` 和 `audits/*/review.md`，把它们注入为 `CHARTER` / `PLAN` / `AUDIT` node，于是指向它们的引用有了落脚之处。

在 Sentinel 上实测：dangling 引用 **330 → 87**，node **131 → 193**（+41 charter，+5 plan，+16 audit），orphan **2 → 0**。而剩下的 87 条*正确地*保持 dangling —— 它们指向治理 corpus 之外的文件（`.specify/memory/…`、`constitution.md`）。这便引出了真正的问题。

## 当一条 dangling edge 是 feature

如果 87 条未解析的引用是*正确的*，那么"dangling"就不能是"断裂"的同义词。一个对全部 87 条都大呼小叫的面板是在狼来了，而一个把它们藏起来的面板是在同一个桶里藏起了真正的断链。是这个类别本身错了。

[#262](https://github.com/StrangeDaysTech/straymark/issues/262)（与 Architecture 视图一同在 [`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0) 中交付）把未解析的目标拆成三种：**断裂的治理链接**（一个*本该*解析却没解析的 id —— 可付诸行动的情形）、**文件引用**（指向代码、规范、sidecar 的路径 —— 不是文档，正确地缺席于文档图谱之外），以及**外部链接**（一个 URL）。在 Sentinel 的 corpus 上，dangling 引用面板从 **92 个假警报降到 0** 个真正的断链。dangling edge 没有被压制；它被*分类*了。它变成了一个带类型的信号 —— 而这是一条警告值得被展示的唯一形态。

这和[上一条弧线](what-the-bash-script-said-was-in-sync)从一个报告"in sync"的已弃用 bash 脚本身上学到的是同一课：对你读取的东西宽容，对你声称的东西排他。R1 是宽容（解析一个人类实际写下的引文）；绝不解析歧义的规则与三路分类是排他（别声称一条你无法证成的 edge，别在它没断时大喊断裂）。同样的纪律，不同的表面。

## 那个吃掉点击的 bug

这段历程里的一个修复值得单独成段，因为症状令人抓狂，而成因优雅。操作员报告说 Loom 的侧栏按钮 —— dangling-ref 链接、community 开关、统计动作 —— 感觉*死了*，点击落到虚空，时有时无。[`loom-0.4.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.1)（[#247](https://github.com/StrangeDaysTech/straymark/issues/247)）找到了它：每当一个文件的修改时间发生变动，watcher 就广播一次 WebSocket "rebuild" —— **哪怕它的内容根本没变** —— 一次无改动的编辑器保存、一次格式化、一次 `touch`、一次 cloud-sync 重写。每一次这样的空操作广播都会通过 `innerHTML` 重新渲染每个打开着的客户端的面板，在刚刚绑定好的点击 handler 附加上去几分之一秒后就把它们摧毁。图谱看起来好好的；面板却在被定时切除脑叶。两个修复：watcher 现在把新图谱与旧图谱做 diff，相同时保持沉默；面板在稳定的容器上使用**事件委托**，于是点击能挺过一次合理的重新渲染。[`loom-0.4.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.2)（[#248](https://github.com/StrangeDaysTech/straymark/issues/248)）随后让 100+ node 的图谱变得可读 —— 标签密度被限制为每个屏幕区域最中心的那个 node、一个"hide isolated"开关、屏上的缩放/适配控件。

## 我们刻意没做的事

服务器**严格保持 read-only**。在编辑器中打开是一个客户端侧的深链（一个浏览器交接出去的 `vscode://` URL），copy-path 是页面内的一次剪贴板写入 —— 服务器从不触碰你的编辑器、你的文件、或 loopback socket 之外的任何东西。路线图上没有任何"Loom 也能替你编辑文档"的 feature，因为构造上 read-only 的姿态比那点便利更值钱，而你无法在不抛弃姿态的前提下加上那点便利。

## 如果你读到了这里

这次的可移植练习：拿起你自己项目里的交叉引用 —— issue 提及、`@see` 文档链接、工单里"depends on"的备注、如果你想较真的话还有 import 图谱 —— 然后问：如果一个工具真去追踪它们，有多大比例会真正解析得出来？接着问 395-里-330 那一刻强加给我们的那个更难的问题：在那些*解析不出来*的当中，你的工具能不能把一条真正断裂的链接，与一条正确地指向自身世界之外的引用区分开来？在它做不到之前，它向你展示的每一个"N 条断裂引用"的数字，都是火灾与假警报某种未知的混合 —— 而一条你已学会忽略的警告，比没有警告更糟。

---

*StrayMark `loom-0.1.0` → `loom-0.5.0`，`cli-3.24.0` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) · Spec [001-loom-server](https://github.com/StrangeDaysTech/straymark/blob/main/experimento/specs/001-loom-server/spec.md) · Issues [#240](https://github.com/StrangeDaysTech/straymark/issues/240) · [#241](https://github.com/StrangeDaysTech/straymark/issues/241) · [#243](https://github.com/StrangeDaysTech/straymark/issues/243) · [#246](https://github.com/StrangeDaysTech/straymark/issues/246) · [#247](https://github.com/StrangeDaysTech/straymark/issues/247) · [#262](https://github.com/StrangeDaysTech/straymark/issues/262)。前篇：[第二个读者提出的要求](what-the-second-reader-demanded)。下一篇：[债务究竟在哪里](where-the-debt-actually-was)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*
