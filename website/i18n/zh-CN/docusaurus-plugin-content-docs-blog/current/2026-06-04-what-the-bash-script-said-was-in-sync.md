---
slug: what-the-bash-script-said-was-in-sync
title: bash 脚本口中的"in sync"
authors:
  - jose
tags: [straymark, follow-ups, governance, adopters, sentinel, lnxdrive, cli]
date: 2026-06-04
description: Follow-ups 成为 StrayMark 的第二个一等公民实体 —— 一天之内，两次外部迁移对这个设计做了压力测试。参考 adopter 已弃用的 bash 脚本一直在报告"in sync"，而 29 个条目对它完全不可见。
---
*就在 follow-ups 成为一等公民实体 —— schema、CLI namespace、随框架发布的代理指令、一个可调用的 skill —— 的次日，两个 adopter 端到端跑完了迁移并提交了报告。一个发现了我们没有闭合的环。另一个发现了更妙的东西：它所替换的那个已弃用的 bash 脚本，不知沉默失明了多久，一边报告"in sync"，一边让 8 个 AILOG 和 29 个条目无人提取。两份报告在同一天变成了 releases。*

<!-- truncate -->

> *"……尽管 legacy 脚本**就在前一天还报告着'in sync'**，包括 `--scan-all` 的运行。"*

这行字来自 Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225)，[Sentinel](https://github.com/StrangeDaysTech/sentinel) 的更新报告，也是这篇文章标题的出处。要解释它为什么重要 —— 以及为什么它是它所闭合的那条 release 车道最有力的论据 —— 得从 follow-up *是什么*说起。

## 待办工作的问题

StrayMark 里每个 AILOG 都可以用一节 `## Follow-ups` 收尾：这次变更推迟了哪些事。Charter 中途浮现的风险也是同样的写法 —— `R4 (new, not in Charter): …`。这是一个写入时的约定，而在写入时它工作得很完美：实现者清楚自己在推迟什么，并把它写在产生推迟的那份工作旁边。

失败模式在读取时。*"项目里有什么待办？"*是操作员的问题，而 per-AILOG 约定用一次随每个 Charter 不断恶化的多文件扫描来回答它。参考 adopter 最先撞墙：到 Sentinel 的 CHARTER-12 时，答案已经深达 47 个 follow-ups、散落在几十个 AILOG 里。框架的回应 —— [`fw-4.10.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.10.0)，早在五月 —— 是一个*约定*：一个中央注册表文件 `.straymark/follow-ups-backlog.md`，五个以**条目何时可付诸行动**为键的 bucket，以及一个 adopter 侧的 bash 脚本（约 296 行 POSIX awk 和 grep），用于检测 follow-ups 尚未被提取的 AILOG。

那条 v0 车道是工作的 —— 然后它积累了你能想到的、一个在 N=91 规模下手工维护的约定会积累的全部债务。Issue [#214](https://github.com/StrangeDaysTech/straymark/issues/214) 从操作员的座位记录了这些信号。手工维护的 frontmatter 计数器已经漂移成了虚构：声明 `total_open: 47`，实际 65，距离上次对账四个星期。而每一批提取都携带 20–75% 的噪声 —— 脚本追加时就*已经解决*的条目，因为 bullet 文本明明写着（`closed in-Charter`、`fixed in batch 3`），而脚本读不懂。

## 一等公民，带着闸门

回应是 [ADR-2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-03-001-followups-first-class.md)：follow-ups backlog 不再是一个约定，而成为 StrayMark 的第二个一等公民工件，走 Charter 走过的同一条车道 —— 自己的规范路径、自己的 schema、自己的 CLI namespace、自己在 `explore` TUI 中的分组。

[`fw-4.21.0` / `cli-3.19.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.19.0) 交付了实质内容：

- **一个 JSON schema（v1，实验性）**，覆盖注册表 frontmatter，外加操作员此前一直在散文里即兴发挥的 v1 条目维度：`Severity: blocking`（将笔记里的 `PROD-BLOCKER` 约定规范化）、`Origin-class`（规划工件 vs 执行现实）、自由的 `Labels`、一套 `Destination` 词汇。
- **`straymark followups list / status / drift / promote`** —— 原生实现，替换 bash 脚本。`drift --apply` 以 per-AILOG 粒度提取未提取的 AILOG（这一粒度在参考 adopter 的 76 个 AILOG 上产生了 0 个假阳性），`promote` 以可溯源的方式自动化 FU → TDE 的提升，`status` 是注册表的脉搏。
- **CLI 拥有的计数器。**每次写入时，`total_*` 字段都从实际条目状态重新计算。静默计数器漂移这一失败模式不是被"劝阻" —— 而是被机制性地杀死。
- **反噪声提取。**源文本携带关闭标记的 bullet 以 `suspected-closed` 落地，而不是作为 TBD 噪声污染 ready bucket。操作员在分诊时确认或重开。
- **随框架发布的代理指令。**`AGENT-RULES.md §13` 随框架交付：会话开始时看一眼注册表，与 AILOG 同一个 commit 中运行 `drift --apply`，Charter 关闭时分诊。adopter 们不再往自己的代理配置里复制一段块。

还有一个深思熟虑的刹车，写在 ADR 本身：schema 以**实验性 v1** 发布。硬性稳定化以第二个 adopter 为闸门 —— 与[上一篇文章](what-the-feature-flag-compiled-away)用整个中段为之辩护的 N=2 纪律相同。一个 adopter 的工作流，无论记录得多好，都只是单一 stack 的形状。

[`fw-4.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.22.0) 用必须等待框架 release 的那部分补完了表面：`/straymark-followups` skill，覆盖全部四个代理表面，封装 §13 指令。设计上就是薄封装 —— 它的 `allowed-tools` 刻意省略了 `Write`，因为注册表的每次变更都经由 CLI，计数器始终归 CLI 所有。skill 驱动纪律；它不拥有逻辑。

## 然后两个 adopter 同时出现

接下来发生的事值得写下来，因为它把闸门所等待的反馈循环压缩到了一天之内。

**[LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) 冷启动采用了注册表** —— 首次外部采用，74 份文档，Issue [#222](https://github.com/StrangeDaysTech/straymark/issues/222)。提取本身是干净的（与手工 grep 交叉验证：零遗漏 AILOG）。两个发现都在边缘上：

- **我们没有闭合的环。**生命周期明确认可*手动*分诊 —— 操作员手工翻转状态。但唯二会重新计算 CLI 拥有的计数器的命令是 `drift --apply` 和 `promote`，而当没有东西可提取或可提升时，两者都是 no-op。一场纯分诊会话之后，注册表带着明知过期的计数器搁浅，且没有合规的修复方式：手工编辑违反 §13，而 `status` 警告的承诺（"下一次写入会重新计算"）只有在未来恰好发生一次写入时才为真。被认可的工作流与不变量没有交汇。
- **一个我们不会说的关闭习语。**两个被提取的条目生来就已解决 —— 源文本写着 `Charter row updated atomically in this PR`。反噪声词汇表认识 `closed in-Charter`、`fixed in batch N` 和 commit 哈希；它不认识这个表述，于是两个条目都以 `open` TBD 噪声落地。恰恰是这项精化为之而生要杀死的回归，通过一个未被识别的习语回来了。

[`fw-4.23.0` / `cli-3.20.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.20.0) 当天发布：一个专用的 **`followups recount`** 动词（重新计算计数器，不碰其他任何东西，幂等），`drift --apply` 即使零提取也会调和计数器，关闭词汇表中加入 **born-resolved 习语族** —— 以及更持久的一项：规范习语表写入 pattern doc，让 AILOG 作者在写入时就收敛到可识别的表述，而不是在提取时才发现不被识别的那些。

**Sentinel 跑了官方迁移** —— Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225)：删除脚本，`drift --scan-all --apply`，重指 pre-commit 钩子。原生解析器立刻从 8 个 AILOG 中提取出 29 个条目 —— 而 bash 脚本就在前一天还报告它们完全 in sync，包括 `--scan-all` 的运行。

根因（对照 git 历史里被删除的脚本验证过）属于那种让"已弃用"一词显得不够分量的东西。awk 提取器同时要求 `## Risk` 小节标题*和*精确的 `- **R<N> (new` bullet 形态。那 8 个 AILOG 把风险写成了纯段落 —— `R4 (new, not in Charter): …` 顶格书写，没有 bullet，没有标题。对格式敏感的匹配，零命中，没有报错。这些 AILOG 甚至从未被登记为*含有* follow-up 内容。脚本缺的不是 feature。它在产出**漂移检测本身的静默假阴性 —— 它存在的唯一目的就是防止这件事。**

宽容解析器全部捕获了它们，因为宽容是一个设计决定，不是偶然。`cli-3.19.1` 早已出于同样的原因放宽了状态解析：操作员会就地批注（`open — **OVERDUE** (…)`），而一个精确匹配的解析器会把 Sentinel 65 个条目中的 4 个降级为 `unknown`，并在第一次迁移时就写下错误的计数器。解析人类实际书写的东西，让 schema 验证去提建议。每一种被严格工具丢在地上的格式变体，最终都被证明是承重的。

[`fw-4.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.23.1) 收尾了这份报告：pattern doc 的迁移段落现在要求首次迁移后扫描带上 `--scan-all`，*即使 legacy 脚本说"in sync"* —— 并引用 8/29 这个数据点作为理由。（Sentinel 的另一个发现 —— v0 → v1 升级在 no-op `--apply` 上不触发 —— 已在数小时前被 LNXDrive 车道修复 —— 两份报告在互不知情的情况下解决了彼此的边缘。）

Sentinel 的报告里还埋着一个安静的确认，值得给它一句话的聚光灯。分诊进行到一半、29 个条目正被手工重新归类时，`status` 打印出 *"frontmatter says total_open: 91 but the real count is 77 — run `straymark followups recount`"* —— 时机分毫不差 —— 一条命令调和了一切。开启 issue #214 的那个计数器漂移失败模式死了，而当初报告它的 adopter 亲眼看着它死去。

## 我们仍然刻意没做的事

克制的小节 —— 这个系列的每篇文章都有一节，而纪律只有在好消息面前活下来才算数。

Sentinel 的报告又提供了两个关闭习语（`Validated: … No vulnerabilities found`、`Bundled into CHARTER-19`）—— 各出现一次。它们被*记录在案*，而非发布。pattern doc 新的词汇小节写明了我们对自己持守的规则：当一个习语**反复出现**时再向上游提议。一个吸收每个一次性表述的词汇表就不再是词汇表了。

schema 仍是**实验性 v1**。两次生产迁移、零 schema 层面的发现，恰恰是闸门为测量而建的有利信号 —— 两份报告都是工效与词汇，没有结构性问题 —— 但硬性稳定化是一个要在头脑冷静时做的深思熟虑的决定，不是与 feature 同一周交付的反射动作。而 `charter close` 软集成（Charter 关闭时自动运行 `drift --apply`）依然闸在一个尚无 adopter 发出的摩擦信号之后。

## 如果你读到了这里

这次的可移植练习小得近乎不好意思。挑出你项目里回答*"有什么待办？"*的那个文件 —— 或 issue 标签，或 TODO 约定。现在检查两件事。第一：它的汇总数字由谁维护，人还是工具？如果是人，它们就是错的；唯一的问题是错多少（我们量过：18，四个星期之后）。第二：如果你的提取工具今天报告"无事可做"，你能分辨那是*真的*，还是你最近推迟的三件事只是没匹配上它的正则？这两种状态之间的差异在构造上就不可见 —— 这正是*静默*的含义 —— 唯一的出路是对读取宽容、对所有权排他的工具。

如果你维护着一个我们还没见过的延迟工作约定 —— 不同的 bucket 词汇、不同的关闭习语、对计数器归谁所有的不同回答 —— 渠道与 #214、#222 和 #225 走过的是同一条：[开一个 issue](https://github.com/StrangeDaysTech/straymark/issues)。v1 schema 之所以是实验性的，恰恰因为你的注册表是它还没见过的那一个。

---

*StrayMark `fw-4.21.0` → `fw-4.23.1`，`cli-3.19.0` → `cli-3.20.0` — ADR [2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-03-001-followups-first-class.md) · Issues [#214](https://github.com/StrangeDaysTech/straymark/issues/214) · [#220](https://github.com/StrangeDaysTech/straymark/issues/220) · [#222](https://github.com/StrangeDaysTech/straymark/issues/222) · [#225](https://github.com/StrangeDaysTech/straymark/issues/225) · PRs [#217](https://github.com/StrangeDaysTech/straymark/pull/217) · [#218](https://github.com/StrangeDaysTech/straymark/pull/218) · [#221](https://github.com/StrangeDaysTech/straymark/pull/221) · [#223](https://github.com/StrangeDaysTech/straymark/pull/223) · [#227](https://github.com/StrangeDaysTech/straymark/pull/227)。Pattern：[`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md)（v1，实验性）。前篇：[二进制藏不住的东西](what-the-binary-couldnt-hide) · [What the feature flag compiled away](what-the-feature-flag-compiled-away)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*
