---
slug: the-rebrand-to-straymark
title: 更名为 StrayMark
authors:
  - jose
tags: [straymark, 品牌迁移, 治理, 身份认同]
date: 2026-05-09
description: 第四次改名与前三次不同——它不是在寻找概念，而是源于一次商标冲突调查。八天、一份 ADR、四十三分钟内的五个 PR。这是项目第一次有纪律的品牌迁移，也展示了把 "in scope / out of scope" 应用到自己的过去意味着什么。
---
*第四次改名与前三次不同 —— 它不是在寻找概念，而是源于一次商标冲突调查。八天、一份 ADR、四十三分钟内的五个 PR。这是项目第一次有纪律的品牌迁移，也展示了把 "in scope / out of scope" 应用到自己的过去意味着什么。*

<!-- truncate -->

## 1. 这次改名并非在寻找概念

ADR `2026-05-08-001` 以一句话开篇，对于读过本博客第 2 篇的人来说，这句话显得有些突兀：

> *"此决策的动因是对商标的法律确定性，而非产品战略或用户反馈。"*

1 月份的三次改名（Chronicle → Monimen → DevTrail）是一场仓促的概念探寻。每一次都确认了一种关于产品本质的直觉，却没有任何一次写过 ADR——提交信息是当时存在的全部文档依据。

5 月份的第四次改名性质截然不同。它不是源于寻找概念，而是来自一次商标冲突调查——于 5 月初通过 Claude.ai 网页会话委托进行——调查结果揭示了 *"随着项目获得使用者，商标归属存在法律不确定性"*。1 月份项目尚无任何使用者，改名可以即兴而为。到了 5 月，项目已有一位使用者（Sentinel）、286 次 *crate* 下载、两个未解决的 Issue，以及一份公开承诺结构化治理的宣言。再度即兴更名的门槛已经越过。

本文涵盖三件事：5 月 8 日做出的有纪律的决策；翌日四十三分钟内五个 PR 的操作弧线；以及两天后浮现的残余问题。在这一切之下，还有一个更微妙的区分：当品牌迁移纯粹停留在表面时，与当它在无意间也改变了身份认同时，两者有何不同。

---

## 2. 第一次有纪律的改名

5 月 8 日的新意在于形式。这一次：

- **有前期调查。** 不是先决定改名再找理由，而是先发现冲突，再由此推出决策。
- **有公开的 ADR。** 明确考量了三个备选方案：保留"DevTrail"并接受商标风险；采用遗留名称与新名称并行的混合品牌策略；以 StrayMark 为名、以"新项目"身份重置到 v0.1.0。三者均以明确论据驳回。
- **有时间成本测算。** ADR 本身这样量化代价：*"改名窗口随时间收窄——每增加一位使用者，日后更改的成本就增加一分……法律风险占主导地位。现在行动，只有一位自有使用者，在实质上比日后被迫行动要廉价得多。"*

测算之所以重要，在于其结构性意义。若品牌迁移再推迟两个月——等到第二位、第三位使用者出现——迁移成本将成倍增长。每个使用者的仓库都需要执行 `mv .devtrail .straymark`，更新 `CLAUDE.md`/`AGENT.md`，重新生成流水线。而只有一位使用者（操作者本人）时，成本是可控且可逆的。ADR 所说的 *"现在行动在实质上更廉价"* 正是如此：这次品牌迁移是一次最小化未来技术债务的操作，而非一项市场营销决策。

这里与第 2 篇文章之间存在一个无意的呼应。那篇以 *"大概不会再有另一次品牌迁移了"* 收尾。这篇写于 5 月，知道它发生了。承诺与追认的区别在于：第四次改名不同于前三次，它是由外部环境所迫——商标冲突——任何操作者都无法预见，且以 1 月份所没有的纪律加以应对。

---

## 3. *In scope*，*out of scope*——ADR 的精妙之处

这篇文章后续所有内容的骨架，正是 ADR 中区分哪些内容需要改名、哪些内容需要保留的那一节。原文引用如下：

> *"In scope（项目的'活跃状态'）：源代码中的所有标识符、Cargo 元数据、真实来源路径、30 个技能/工作流文件、公开文档、CI 工作流、GitHub 仓库名称。"*

> *"Out of scope（不可变历史，逐字保留）：所有提交、提交信息和 git 历史；本 ADR 之前发布的所有标签；之前发布的所有版本标题和正文；所有先前的 CHANGELOG.md 章节。"*

这一区分是整个品牌迁移的骨架。*活跃的*内容——路径、命令、URL、发布资产、README——一次性完成改名。*历史性的*内容——git log、`devtrail-cli@3.10.0` 标签、先前的 CHANGELOG 章节、已关闭的 Issue——按原样保留。项目不向后改写自己的历史。

这与第 3 篇文章为 Plan → Charter 改名所记录的档案纪律如出一辙，现在被应用于全框架范围的操作性品牌迁移。以表格呈现更为直观：

| 层级 | 执行改名 | 保留不动 |
|---|---|---|
| 文件系统路径 | `.devtrail/` → `.straymark/` | — |
| Rust 源码 | `devtrail-cli` → `straymark-cli` | — |
| 技能/工作流 | 30 个文件 `devtrail-*` → `straymark-*` | — |
| 公开文档 | README、CLAUDE.md、ADOPTION-GUIDE | — |
| CI/CD | 资产名称、二进制路径 | 标签前缀（`fw-`、`cli-`）不变 |
| GitHub 仓库 | `StrangeDaysTech/devtrail` → `StrangeDaysTech/straymark` | — |
| CHANGELOG | 顶部新增 `fw-4.11.0` 条目 | 先前章节逐字保留 |
| 已发布标签 | — | `fw-4.10.0`、`cli-3.10.0` 等保留 |
| 整个 git log | — | 含"DevTrail"的提交信息完整保留 |
| 遗留 *crate* | — | crates.io 上的 `devtrail-cli@3.10.0` 不 *yank* |

版本连续性是最显而易见的证明。下一个版本不是 `0.1.0` 或 `1.0.0`，而是 `fw-4.11.0` 和 `cli-3.11.0`。版本号无需任何比喻地宣告：这个项目依然是同一个项目。

---

## 4. 五个 PR，四十三分钟

5 月 9 日，UTC 时间 06:05 至 06:48 之间，五个 PR 依次合并：

| PR | UTC 时间 | 内容 |
|---|---|---|
| [#114](https://github.com/StrangeDaysTech/straymark/pull/114) | 06:05 | 合并 ADR 本身。零代码改动——先固定决策。 |
| [#115](https://github.com/StrangeDaysTech/straymark/pull/115) | 06:42 | 174 次 *rename* + 74 次修改。Rust 源码（38 个文件）、测试（18 个）、`dist/.devtrail/` → `dist/.straymark/`（通过 `git mv` 迁移 122 个文件）、30 个技能 × 3 个平台。 |
| [#116](https://github.com/StrangeDaysTech/straymark/pull/116) | 06:44 | 三语言（EN/ES/zh-CN）公开文档。CHANGELOG 前言更新：*"StrayMark（前身为 DevTrail；于 2026-05-08 更名）"*。 |
| [#117](https://github.com/StrangeDaysTech/straymark/pull/117) | 06:45 | 发布工作流。资产名称（`straymark-cli-*`）、版本标题。标签前缀不变。 |
| [#118](https://github.com/StrangeDaysTech/straymark/pull/118) | 06:48 | 版本升至 `fw-4.11.0` / `cli-3.11.0`。新增 CHANGELOG 章节。 |

顺序是刻意为之，由外而内：先是法律依据（ADR），再是内部代码，然后是使用者所见（文档），接着是分发（CI），最后是既成事实（发布）。这一序列具有档案意义：六个月后有人按时间顺序回顾 `git log`，会先看到决策，再看到实施。因果关系就活在仓库里。

有一处值得特别记录，它打乱了原有计划。ADR 原本设想了九个独立的 PR（每个逻辑层一个）。实际执行时，CLI 测试依赖于*硬编码*路径——`include_str!("../../dist/.devtrail/...")`——在不同步更新测试的情况下修改路径，会让 CI 变红。计划压缩为五个，是因为各层在技术上耦合，而非因为概念纪律失败。PR #115 的备注毫不掩饰地记录了这一点：*"合并是被紧耦合强制的：测试通过 `include_str!` 使用 `dist/.devtrail/` 路径。"*

四十三分钟内完成了一次触及约两百六十个文件的品牌迁移。这样的速度之所以可能，是因为 ADR 在触碰代码之前已关闭了每一个决策。框架原则第 6 条——*"当提案写得足够清晰时，实施就是执行，而非设计"*——再次得到验证，与第 4 篇文章中外部审计循环的案例如出一辙。

---

## 5. 为何 StrayMark 改变的不只是名字

到目前为止，ADR 的立场是明确的：品牌迁移停留在表面，概念身份得以保留。但有一个细节值得指出。

PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120)——在主弧线结束数小时后合并，同样是 5 月 9 日——在 README 中新增了"Why StrayMark?"一节。这不是代码，而是一份宣言。它说出了项目此前任何文档都未曾明确阐述的内容：

> *"代码不过是一场心智博弈留下的化石痕迹。真正的工程发生在决策的混沌之中，发生在经过测算的冒险之中，发生在那些你选择不走的路径之中。传统上，所有这些人类足迹都被当作项目历史中的 stray marks（意外涂抹的痕迹）而丢弃。在 Strange Days Tech，我们相信那些痕迹才是信号，而非噪音。"*

从这份宣言的视角来看，名字的选择并非随意。那些 *stray marks*——不规则的痕迹、大多数项目丢弃的意外轨迹——正是框架着力呈现的内容：AIDEC、AILOG、TDE、Charter。在任何传统项目中都可被视为可丢弃噪音的东西，在这个项目中是审计的原材料。这个名字以 *DevTrail* 从未做到的方式，将产品论点具象化在了其中。*DevTrail* 是一条中性的路径；*StrayMark* 是一种主张。

宣言以三行话收尾，后来成为编辑意义上的操作性 *tagline*：

> *"Capture the noise. Weave the signal. Humanize the machine."*

这里有品牌迁移的一个小小的有趣张力。ADR 说只有表面改变了；同一天的宣言则表明，身份认同也被阐明了——或许是第一次清晰地阐明。这并不矛盾：操作性品牌迁移的时机被用来写下项目中早已存在、却尚未被命名的东西。身份没有改变，它变得可读了。

---

## 6. 留下的未竟之事

即便有公开的 ADR 和有纪律的操作弧线，*"完成"* 依然是一个近似值。

品牌迁移两天后，当 Sentinel 正在准备其下一轮外部审计时，智能体发现了三类残余问题。这段时间线——在 `CHANGELOG.md` 中以内联勘误的形式诚实记录——值得公开呈现：

- **PR [#137](https://github.com/StrangeDaysTech/straymark/pull/137)**（5 月 11 日，22:55）——*孤立的技能目录*。品牌迁移改写了每个 `SKILL.md` 内部的 `name:` 字段，但未重命名三个平台的技能目录（`.gemini/skills/devtrail-*`、`.claude/skills/devtrail-*`、`.agent/workflows/devtrail-*.md`）。三十个孤立产物：名称已新，路径仍旧。Gemini CLI 在启动时报告了十条冲突警告。
- **PR [#138](https://github.com/StrangeDaysTech/straymark/pull/138)**（5 月 11 日，23:32）——*遗留 Charter 路径*。三个框架产物在 `fw-4.12.0` 已将规范路径迁移至 `.straymark/charters/` 之后，仍引用 `docs/charters/`。最糟糕的是：`pre-pr.sh` 钩子以 `[ ! -d docs/charters ]` 作为门控条件，导致任何 4.12.0 之后的使用者都会静默 *exit 0*。这个漂移检查已安装七个月，却一直是空操作。
- **PR [#139](https://github.com/StrangeDaysTech/straymark/pull/139)**（5 月 12 日，12:02）——*安装脚本失效*。`install.sh` 和 `install.ps1` 仍指向 `REPO=StrangeDaysTech/devtrail`、`BINARY=devtrail`、资产名 `devtrail-cli-v*`。README 已完成品牌迁移并指向新 URL。结果：任何用户从 5 月 9 日起复制 README 命令，运行安装脚本时都会遭遇 GitHub 404。*产品在生产环境中宕机了七十二小时。*
- **PR [#140](https://github.com/StrangeDaysTech/straymark/pull/140)**（5 月 12 日，16:49）——*微小残余*。`.gitignore` 第一行仍写着 `# DevTrail - .gitignore`。另外在内部开发模式中新增了 `Aparador/` 条目。

教训不在于纪律不足，而在于纪律*不能阻止*残余——它让残余变得可发现、可记录、可在公开内联勘误中修复。前三次改名（1 月份）大概也有同等程度的残余；我们从未整理它们，因为那时还没有文档记录的习惯。第四次品牌迁移整理了它们。这是项目第一次在公开仓库和 CHANGELOG 本身中承认，操作者没有在第一遍就发现所有问题。

最令人不安的细节——安装脚本宕机七十二小时——是我最想记录的。如果彼时不是一位独自使用的采用者（操作者本人），而是一个团队，那个窗口期将真实地伤害用户。*现在行动，只有一位自有使用者，在实质上更廉价*，ADR 如此说。这句话买到的正是：有权在七十二小时内犯错，而不伤及任何人。

---

## 7. 结语

从这一过程中，我得出四点结论：

1. **并非所有改名都属于同一类型。** 1 月份的三次是在寻找概念；5 月份的那次是在抵御项目未来的法律债务。只有 5 月份的那次能够借助公开 ADR 来完成，因为只有它诞生于外部证据，而非内部直觉。

2. **表面与身份的区分属于文档，而非修辞。** ADR 本身宣告了哪些内容被改名、哪些内容被保留。那份清单——路径是的、git log 否；版本标题是的、先前标签否——是品牌迁移的骨架。没有这份明确的清单，*"品牌迁移"* 可以意味着任何你想要的东西。

3. **身份可以在不改变的情况下变得可读。** *"Why StrayMark?"* 宣言没有发明项目原本是什么；它第一次清晰地将其阐明。操作性品牌迁移被用来写下那些已经在代码中存在、却尚未进入 README 的内容。身份没有被重写——它被发布了。

4. **纪律不能阻止残余；它让残余变得可发现。** 两天后的四个*勘误* PR 不是品牌迁移的失败；它们是证据，证明这次品牌迁移足够诚实，承认了漏网之鱼。1 月份的三次改名大概也有同等情况；它们从未被记录。

下一篇文章，将涉及一个与最后这点直接相关的情节：*TDE 作为命名横切面技术债务的机制*（`H-09`）。这是框架用来阐明如何暴露债务的第一个真实案例——并且留下了一个关于 PR 堆叠的惨痛教训，等到谈及那里时，值得专门用一段来讲述。

---

*参考锚点：ADR [`2026-05-08-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-05-08-001-rebranding-straymark.md)。核心弧线：PR [#114](https://github.com/StrangeDaysTech/straymark/pull/114) · [#115](https://github.com/StrangeDaysTech/straymark/pull/115) · [#116](https://github.com/StrangeDaysTech/straymark/pull/116) · [#117](https://github.com/StrangeDaysTech/straymark/pull/117) · [#118](https://github.com/StrangeDaysTech/straymark/pull/118)。宣言：PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120)。残余修复：PR [#137](https://github.com/StrangeDaysTech/straymark/pull/137) · [#138](https://github.com/StrangeDaysTech/straymark/pull/138) · [#139](https://github.com/StrangeDaysTech/straymark/pull/139) · [#140](https://github.com/StrangeDaysTech/straymark/pull/140)。版本：`fw-4.11.0` / `cli-3.11.0`。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
