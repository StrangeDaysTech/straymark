---
slug: what-the-second-reader-demanded
title: 第二个读者提出的要求
authors:
  - jose
tags: [straymark, loom, knowledge-graph, architecture, rust, cli, sentinel]
date: 2026-06-12
description: 在 Loom 能画出任何东西之前，StrayMark 的文档模型必须先停止做 CLI 的私产。那次零用户可见变更的重构，是之后一切可视化工作的前提 —— 也是框架一直以来向 adopter 反复宣讲的一个小教训。
---
*StrayMark 文档一直构成一张图 —— 每一条 `related`、`supersedes`、`originating_ailogs` 链接都是一条边。CLI 在内部为 `straymark audit` 构建那张图，而人类可以在 `explore` TUI 里一次一份文档地读它。然后第二个消费者出现了 —— Loom，一个对整个语料库的实验性浏览器视图 —— 它要求解析同样的文档。对"它能复用 CLI 的 parser 吗？"这个问题，诚实的答案是不能，因为那个 parser 并不是一个库；它埋在 `cli/src/document.rs` 里。修复以 `cli-3.23.1` 交付，**零用户可见行为变更**。它也是本月最重要的一次 release。*

<!-- truncate -->

> *"同一份语料库不能有两个 parser。"*

这句话就是整篇文章，但要让它变得具体，需要一个新组件。在 StrayMark 大部分生命里，读 StrayMark 文档的程序恰好只有一个：CLI。一个 parser，一个消费者，没有产生分歧的可能 —— 因为根本没有可供分歧的对象。把 Charter 连到 AILOG、连到 TDE、连到 ADR 的那张图是存在的，但它住在 CLI 的私有内存里，按需为 `straymark audit` 装配，从未整体地展示给任何人。

三位独立的框架评审者标出了同一个缺口：随着语料库增长 —— 参考 adopter [Sentinel](https://github.com/StrangeDaysTech/sentinel) 正是不断把这些问题推到台面上的那个案例 —— 在 TUI 里一次一张卡片地读文档网，已经无法传达*形态*。你看不见聚簇、看不见桥接文档、看不见孤儿、看不见环。答案注定是一个图形化视图。而那个视图成为现实的那一刻，StrayMark 就有了它的第一个第二读者。

## 先把台面清干净

新组件就是一条新战线。在未竟之事上面再开一条，正是一个项目积累起那种它日后要写博文道歉的债务的方式。所以在 Loom 第一行代码之前的那一周，三件被推迟的事收了尾 —— 每件都很小，每件都在"早就该做了"的清单上。

- **可移植的已安装 skill**（[`fw-4.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.24.0)，[#232](https://github.com/StrangeDaysTech/straymark/issues/232)）。一次 Codex（`gpt-5.5`）对 StrayMark 自己那些在 Sentinel 里跑在 Codex CLI 下的 skill 的评审，找出了五个可移植性 bug —— 最尖锐的一个是：为 Codex 生成的 skill 仍然往它们产出的 frontmatter 里写 `agent: claude-code-v1.0`，*在一个全部工作就是溯源的工具上扭曲了溯源遥测*。skill 现在从 `AGENT-RULES.md §1` 解析自己的运行时身份，而不再硬编码一个。
- **`charter close` 闭合 follow-ups 的环**（[`cli-3.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.22.0)，[#135](https://github.com/StrangeDaysTech/straymark/issues/135) Tier 3）。follow-ups 路线图最后一个未闭合的层级 —— 关闭一个 Charter 现在会扫描它的 AILOG 寻找被推迟的工作，提取注册表里没有的部分，并逐条提供 TDE 提升。它一直被闸在 drift 检测变得可靠之前；它可靠了，于是它交付了。
- **`charter drift` 现在是原生 Rust 了**（[`fw-4.26.0` / `cli-3.23.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.0)，[#237](https://github.com/StrangeDaysTech/straymark/issues/237)）。这条命令过去要 shell out 到一个 bash 脚本，这意味着它在没有 WSL 或 Git Bash 的 Windows 上根本不跑。把 declared-vs-modified 的集合差异移进进程内，闭合了最后一个功能性的 Windows-native 缺口 —— 而删掉那个解析脚本 stdout 的中间层，去掉了一整类 bug。早已固定住这一行为的集成测试套件，现在在每个平台上运行，并兼任脚本等价性的保证。

这里头没有一件是头条。但合在一起，它们就是"从一个干净的基座起步一个新组件"和"从一个你暗自羞愧的基座起步"之间的差别。

## 一个 parser，还是两个真相

接着是真正的转向：[`cli-3.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.1)，记录在 [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md)，Loom 里程碑 M0。

仓库根目录变成了一个 Cargo **workspace**（`core` + `cli`）。文档模型 —— `DocType`、`Frontmatter`、`StrayMarkDocument`、`parse_document`、`discover_documents` —— 原封不动地从 `cli/src/document.rs` 移进一个新 crate，**`straymark-core`**，发布到 crates.io。而 `audit_engine::build_traceability`，CLI 那个私有的图装配器，泛化成了 `core::graph`：一张在 frontmatter 交叉链接之上的、带类型、双向、保留孤儿的 knowledge graph，外加一个稍后才显出分量的刻意设计抉择 —— **悬空引用被保留为一等的 `resolved: false` 边**，而不是丢弃。一条指向不存在文档的链接是数据，不是一个该被吞掉的错误。

为什么要费劲搞一个 workspace 和一个发布的 crate，而不是干脆让 Loom 拷一份解析逻辑过去？因为拷贝正是你得到两个真相的方式。如果 Loom 用自己的代码解析 StrayMark 文档，这两个 parser 就会 drift —— 一个新的 frontmatter 字段被其一识别、另一个不识别，一个标题变体在这里匹配上、在那里漏掉 —— 于是可视化会悄悄地和 audit 不一致。这*恰恰*是 StrayMark 存在的目的就是要在 adopter 的项目里防止的失败模式：同一个事实有两个真相来源，无声地分岔。把它发到我们自己的工具里，会是一种面不改色的渎职。

所以 M0 的门槛就是这条原则所要求的门槛：**CLI 的行为前后必须逐字节一致。**完整测试套件原封不动地通过；`straymark audit` 产出完全相同的输出。release notes 写着"无用户可见行为变更"，读起来像是一桩无事发生，而它事实上正是全部要点。一次不改变任何用户能看见之物的重构，为的是保证两个程序永远无法分歧 —— 这是想象得到的最具 StrayMark 形态的 release。

## 我们刻意没做的事

克制小节，因为这个系列每篇都有一节。

`straymark-core` 发布到了 crates.io —— 它现在是一个真正的共享库了。**Loom 没有。**它只走 GitHub-release，挂在 `loom-*` 标签下，标记为实验性（v0 / N=1），藏在 `straymark loom serve` 一个需要主动确认的下载闸门之后。CLI 没有从这一切里多出**任何** `axum` 或 `tokio` 依赖 —— 服务器整个住在 `experimento/` 里，CLI 对它的全部认知只是一个在首次使用时拉取二进制的 launcher。共享的*文法*在第二个读者需要它的那一刻晋升为一个发布的 crate；实验性的*组件*则保持隔离，直到它挣得出场资格。这两个决定中一个是永久的、另一个是可逆的，而它们被刻意保持在那条线的两侧。

## 如果你读到了这里

可移植练习：找出你项目里那个在不止一处被解析的格式。被 app 和一个 CI 脚本同时读的配置文件。被一个发送端和一个 dashboard 消费的日志形态。一个在一侧被序列化、又在另一侧被手写代码反序列化的 API 契约。现在问那个 M0 为之而建要回答的问题：那两个读者共享一个 parser，还是它们今天仅仅*彼此相像*？相像不是保证 —— 它是一个有有效期的巧合，而那个日期就是某人改了一侧、忘了另一侧的那一刻。唯一持久的修复是那个不光鲜的办法：把文法抽出来，发布一次，让分岔变得不可能，而不只是不大可能。

---

*StrayMark `fw-4.24.0` → `fw-4.26.0`，`cli-3.22.0` → `cli-3.23.1` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) · Charter [`CHARTER-01-loom-server`](https://github.com/StrangeDaysTech/straymark/blob/main/experimento/CHARTER-01-loom-server.md) · Issues [#135](https://github.com/StrangeDaysTech/straymark/issues/135) · [#232](https://github.com/StrangeDaysTech/straymark/issues/232) · [#237](https://github.com/StrangeDaysTech/straymark/issues/237) · PRs [#238](https://github.com/StrangeDaysTech/straymark/pull/238) · [#239](https://github.com/StrangeDaysTech/straymark/pull/239)。前篇：[bash 脚本口中的"in sync"](what-the-bash-script-said-was-in-sync)。下一篇：[这张图还画不出来的东西](what-the-graph-couldnt-draw-yet)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*
