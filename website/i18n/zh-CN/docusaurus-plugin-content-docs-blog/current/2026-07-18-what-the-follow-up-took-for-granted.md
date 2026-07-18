---
slug: what-the-follow-up-took-for-granted
title: follow-up 想当然的那件事
authors:
  - jose
tags: [straymark, governance, followups, charters, cli]
draft: false
date: 2026-07-18
description: 一个 greenfield adopter 在发布前清空了一个 follow-up backlog,发现其中三个 —— 那三个是真正工作的 —— 各自都带着一个在你核查它的那一刻就为假的前提。不是因为代码在它们之下发生了变化,而是因为一个 follow-up 恰恰写于你最没有能力验证它的那一刻。教训不是"把 follow-up 写得更好"。而是一个 follow-up 是一个有日期的假设,而检验它的廉价之处在你读它时,而非写它时 —— 所以那正是 StrayMark 现在放置核查的地方。
---

*一个 adopter 在发布前清空了一个 follow-up backlog —— 从七个条目降到一个。四个是已解决的噪声。在那三个真正的、未解决的工作里,每一个都带着一个虚假的前提:一个要"复制"却从不存在的测试、一个要针对无法存在的参照物去构建的 gate、一个要修复一个根本不在那里的成本的优化。三者中没有一个是因为代码在其之下漂移而出错的。它们在被写下的那一刻就是错的,而这份虚假只在几个月后一次三十秒的核查中才浮现。那个时机就是整个故事。一个 follow-up 恰恰写于你最无法验证它的那一刻 —— 而读于验证几乎免费的那一刻。StrayMark 的注册表过去把条目当作要执行的指令。它们更应被理解为要重新测试的有日期的假设 —— 而且,从这个 release 起,这正是工具所说的,以及它放置核查的地方。*

<!-- truncate -->

## 背景

这个 adopter 就是 [#345](https://github.com/StrangeDaysTech/straymark/issues/345)/[#346](https://github.com/StrangeDaysTech/straymark/issues/346)/[#355](https://github.com/StrangeDaysTech/straymark/issues/355)/[#360](https://github.com/StrangeDaysTech/straymark/issues/360) 背后那个 greenfield 的 .NET 10 / Rust CRDT 库 —— 即 [Adopter] Weft 讨论。Milestone 3 除了由操作员把关的发布之外都已 code-complete。在拉下那个不可逆的操纵杆之前,计划是清空 follow-up backlog:七个打开的条目,好让发布出去的东西不会带着已知却未接线的缺口被冻结进一个公开包里。

Triage 很快把它们分开了。四个不是工作:两个是提取器刮取到的已解决的风险,一个是推迟到某个尚未触发的 trigger 的决定,一个被一个项目控制之外的 upstream merge 阻塞。剩下**三个是真正的、未解决的、可执行的工作。** 三个都拿到了一个 Charter。三个在检查之下,都原来建立在某个并不为真的东西之上。

## 三个 follow-up,三个虚假前提

**那个不存在的一致性测试。** 一个 follow-up 实际上写的是"为 Loro shim 复制 yrs shim 已经有的那个 header↔binding 一致性测试"。合理 —— 只不过 yrs 那个测试并不存在。存在的是两条*注释* —— 一条在 C# binding 里,一条在 C header 里 —— 都声称"一个 CI 测试验证这些声明与此 header 匹配"。从没写过这样的测试;那句话是憧憬性的,可追溯到一条说该核查*可以*被生成的研究笔记。这个 follow-up 的作者把一条注释读成了一个事实。follow-up 继承了注释的虚假,并加了一跳权威。"照原样"构建它,就意味着移植一个没有原本的测试。

**那个针对无法存在的参照物的 gate。** 另一个要求给 Loro 引擎做一个确定性 gate,镜像已有的 yrs↔Yjs 一致性 gate。这种对称很诱人:yrs 有一个独立的参考实现(Yjs)可以逐字节核对,那么 Loro 想必也该有它的对应物。它不能。Loro 格式没有第二个独立实现 —— 那个 npm 包是同一个 Rust core 的 WebAssembly build,所以拿它来比对就是拿这个 crate 跟自己比。这个 follow-up 靠类比来推理,而类比悄然失效了。可实现的 gate 是另一个更朴素的东西(跨运行的自我确定性,一个回归见证而非一致性证明)—— 那正是最终构建出来的,但只因为前提先被抓住了。

**那个针对根本不在那里的成本的优化。** 第三个担心把 relay 重排为 persist-before-broadcast 会把 I/O 推到 actor 的 hot path 上并损害吞吐。追踪真实的 `await` 链表明,持久化调用*已经*在 receive loop 上被 await 了,在连接读取下一帧之前 —— 重排并没有把 I/O 加到任何它本就不在其上的 hot path。一个负载 harness 证实了这点:"安全"的排序在 p50/p99 上的成本实际为零。这个 follow-up 编码的是一份架构的*心智模型*,而不是被构建出来的架构。这是三者中最微妙的一个:没有撒谎的注释,没有破裂的类比 —— 只是某人脑中一份已经偏离领土一格的地图,以及一条忠实记录了那份地图的笔记。

三个 follow-up。一条被轻信的注释、一个被过度信任的类比、一个略微过时的心智模型。不同的失败模式,一个共同的形状:**每个前提在写下的那一刻就为假,而在读取的那一刻却便宜可证伪。**

## 两个时刻

放置一条"验证前提"规则的地方恰好有两个:follow-up 被写下时,或它被读取时。

写下发生在验证的最糟时刻。一个 follow-up 是在*完成别的事情*时随手记下的一条给自己的笔记 —— 正在关闭 Charter N,满脑子都是当前的子系统,侧眼瞥一眼你正要离开的另一个。验证那一瞥意味着从你正试图落地的工作上做一次完整的上下文切换。它恰恰*在那时*昂贵。

读取发生在最好的时刻。当你终于对这个 follow-up 采取行动时,你已经身处那个子系统之中,代码就开着。核查"这个测试真的存在吗 / 这个参照物真的存在吗 / 这个成本真的存在吗"就是一次 `grep`、读一个文件、追踪一条调用链 —— 数秒。上面三个虚假前提正是在这些核查上崩塌的。

所以验证不仅在读取时更便宜 —— 它是*范畴性地*更便宜,因为在读取时你已经因为别的原因付了上下文切换的成本。经济学只指向一个方向。

## 重构:一个 backlog 是一个推测性缓冲区

这里是改变整个功能该如何被理解的部分。人们很容易得出"作者应该在写下前更用力地验证 follow-up"的结论。那是错误的教训,而且会让工具更糟。

一个 follow-up backlog 是一个*推测性缓冲区*。它的职责是廉价地捕获"某件事*可能*值得做" —— 好让信号在注意力移开时不丢失。如果你要求在捕获时验证,你就会把每个 Charter 的收尾都花在你正在放弃的子系统里探洞,而理性的反应将是干脆停止撰写 follow-up。**急于验证会破坏缓冲区的目的。** 未经充分验证的条目不是作者的缺陷;它是推测性缓冲区中任何事物*预期的认识论地位*。

这意味着那些虚假前提并不是 follow-up 撰写方式中的 bug。它们是一个从未被测试过的假设的自然状态 —— 而一个 follow-up 就是一个假设。唯一真正的 bug 会是*在不重新测试的情况下执行它*。而这恰恰是注册表的框架所设下的陷阱:它把条目呈现为一份待办清单、一套指令、一份计划。当作指令来读,虚假前提就变成被浪费的 Charter。当作**有日期的假设**来读,它们就成了它们本来的样子 —— 在你有条件的那一刻去重新核查的廉价赌注。

## 我们交付了什么

现场报告以 [#365](https://github.com/StrangeDaysTech/straymark/issues/365) 落地。它干净地分成两个改动,而且 —— 恰如其分地 —— 重构本身也被当作一个*有日期的假设*来处理:它被记录为一个决定([`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md)),由一个人审查,并在交付的文档改动一行之前被签署。一个关于如何对待主张的主张,值得它所论证的那份同样的纪律。

### 把条目当作假设来框定,并把验证移到执行

作为 [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0) 发布([#369](https://github.com/StrangeDaysTech/straymark/pull/369)),分三层:

**措辞。** follow-ups 模式文档新增了一个一等公民的*"认识论地位"*小节,把过去闷着没说的话讲出来:注册表是一个推测性缓冲区,一个条目是一个有日期、会衰减的假设而非一条指令,一个未经充分验证的条目是*预期*状态而非撰写缺陷 —— 而唯一真正的 bug 是在不重新测试其前提的情况下执行它。代理指令(`AGENT-RULES.md §13`)新增了对应规则:**在捕获时廉价地写;在提升或行动时重新验证前提 —— 绝不在捕获时。** 框架才是承重的改动。其余一切只是给它装上牙齿。

**字段。** 一个条目现在可以携带一个显式的 `Premise`(支撑它的假设)和一个 `Verified-at` 日期。二者皆可选,且 schema 仍为 `v1`,所以没有任何现有注册表会改变。声明前提正是把"重新验证"从一个模糊的提醒变成一个具体靶子的东西:*"yrs shim 已经有一个一致性测试"* 是一句你能用一次 `grep` 证伪的话。`Verified-at` 缺失意味着"自捕获以来从未重新核查" —— 诚实的默认;它的存在是一种溯源证据,表明在有人为它花一个 Charter 之前,该假设已针对现实被检验过。

**检查点。** 两个 CLI 可供性(affordance)把核查放在它便宜的地方:

- `straymark followups verify FU-NNN` 浮现前提,可选地记录或更新它(`--premise "..."`),并在你确认重新核查时盖上 `Verified-at`(`--verified`)。不带 flag 时它是只读的 —— 只把假设展示给你,并问它是否仍然成立。这是常见路径:一个作为杂务被执行、从不成为正式债务文档的条目。
- `straymark followups promote FU-NNN --premise-verified` 在一个 follow-up 毕业为 TDE 的那一刻做同样的事:它打印前提并附上一句*"这还成立吗?针对代码重新验证"*的提醒,并在确认时盖上 `Verified-at`。

二者之下的设计规则:**CLI 提醒并记录;它从不设卡。** 无论带不带 flag,提升都会进行;`verify` 从不阻塞任何东西。它不会裁定你的前提是否为真 —— 那是人的工作,站在那个核查几乎免费的唯一位置上。任何更严格的做法都会重建重构所要避免的那笔捕获时税。

### 机器写的那个标题

次要的发现更小更具体,而且先交付了,在 [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2) 中([#366](https://github.com/StrangeDaysTech/straymark/pull/366))。当 `followups drift --apply` 从一个 AILOG 的 `## Follow-ups` 小节自动提取一个条目时,它把 bullet 的*第一个物理行*当作标题。AILOG 的 bullet 是硬换行的散文,所以一句开头会在作者的换行列被切断 —— 这次会话中有三个条目字面上被命名成诸如 *"**被 `test-hooks` 污染的本地 pack 的 footgun** —— 该 pack 读取自"* 之类的东西,在半句处被截断。一台机器抓一个行片段会丢掉手写标题所携带的细微差别,而一个其*标题*就歪曲了自己的 follow-up 一出生就已经有点不对 —— 这恰恰放大了本文其余部分所讲的"当作指令来读"的危险。

修复会把 bullet 展开,优先取一个够分量的开头 `**加粗**` span 作为标题(作者本就会用的约定),否则取第一*句*,并在词边界处截断。微妙之处在于保持它**哈希中性**:注册表按一个从原始首行派生的内容哈希来去重,所以一个更好看的标题必须与去重键解耦 —— 否则每个 adopter 注册表中每个已提取的条目都会在下一次扫描时作为重复项重新出现。标题变清晰了;什么都没有重新重复。

## 可移植的版本

如果你保留着任何一份延期工作的 backlog —— 一个 follow-ups 注册表、一个 `// TODO(later)`、一个打了 `someday` 标签的 issue —— 你就在保留一个假设的缓冲区,无论你是否这么叫它。这些条目写起来便宜,而且是在你无法核查它们时写下的。错误不在于把它们写得随意;那是对的,而在捕获时要求严谨只会让你干脆不再捕获。错误在于把它们当作一份计划来读、并凭信念去执行。在你行动时重新测试前提 —— 你正站在它几乎免费的那个唯一位置上 —— 让一个虚假前提花掉你一次 `grep`,而不是一个 Charter。

---

*经验基础:在 [Adopter] Weft 项目中清空一个 follow-up backlog 的三个 Charter,2026-07-16 → 2026-07-18(7 个打开 → 1 个)。交付于 StrayMark [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0)(重构,[#365](https://github.com/StrangeDaysTech/straymark/issues/365)/[#369](https://github.com/StrangeDaysTech/straymark/pull/369),[`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md))与 [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2)(标题保真,[#366](https://github.com/StrangeDaysTech/straymark/pull/366))。相关:[#360](https://github.com/StrangeDaysTech/straymark/issues/360)、[#355](https://github.com/StrangeDaysTech/straymark/issues/355)、[#346](https://github.com/StrangeDaysTech/straymark/issues/346)。*

*本文档在生成式 AI 工具(Claude Opus 4.8)的协助下产生;内容的全部责任由人类作者承担。*
