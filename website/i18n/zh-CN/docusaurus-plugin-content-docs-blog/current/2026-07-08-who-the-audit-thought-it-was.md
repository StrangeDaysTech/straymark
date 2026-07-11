---
slug: who-the-audit-thought-it-was
title: 审计以为自己是谁
authors:
  - jose
tags: [straymark, audit, attribution, framework, governance, sentinel]
date: 2026-07-08
description: StrayMark 的审计只有在独立的模型家族各自趋同到一个 finding 时才值钱 —— 一致就是那个信号。一连串 release 把这条保证加固,以对抗三种伪造它的方式。最新的一种最锋利:一个 router CLI 注入了它自己的产品名,于是一个审计器在报告上署名 "qwen-code",哪怕 operator 早已切换了后端模型。趋同的数学被喂了一个关于谁做了这件工作的谎言。
---
*StrayMark 外部审计的价值,不在于某个模型找到了一个缺陷。而在于**独立的模型家族趋同**到了它 —— 一个 finding 只有当来自真正不同后端的审计器各自分别抵达它时,才是信号,而 review 步骤正是在这个基础上去重和评级。这意味着整套机器都坐在一个安静的假设上:每个审计器被记录的身份是真实的。最近一连串 release 发现了这个假设破裂的三种方式,并把它们堵上。最新的那种最令人不安,因为没有任何东西看起来不对:一个审计器自信地在自己的报告上,署上了它所运行于其中的那个 CLI wrapper 的名字,而不是真正做了推理的那个模型。*

<!-- truncate -->

> *Router CLI 通过 system prompt 注入一个产品身份,于是审计器会写下 `auditor: qwen-code`,哪怕 operator 已确认通过 `/model` 选了一个不同的后端 —— 腐蚀了归因,并伪造了 cross-family 的一致。*

这是一段尾声,来自与它前面那组 [Baton 三部曲](what-the-author-already-knew) 不同的框架角落 —— 但它与之押韵。Baton 的弧线是关于服务于*routing* 的趋同;这一段是关于服务于*信任* 的趋同。两者都归结为同一条纪律:一个信号只值它的溯源所值的。StrayMark 的[外部审计循环](charters-and-the-external-audit-cycle)自五月起就是一个一等命令;这些 release 是要把它的核心保证做到无法伪造。

## 1. 为什么身份是承重的

审计靠不一致和一致运作。你把同一个 charter 递给不同模型家族的审计器;一个只有一个家族提出的缺陷是弱信号,一个被几个家族各自独立提出的缺陷是强信号。consolidation 步骤(`straymark-audit-review`)在这份趋同的基础上去重 finding 并给审计器评级。为了让这一切有意义,两件事必须为真:一个审计器不得在写自己的报告之前读另一个审计器的报告,并且每份报告都必须被**诚实地归因**到产出它的那个家族。

破坏其中任何一个,趋同的数学就不只是退化 —— 它*撒谎*。两次"独立"的、暗地里是同一个模型的运行,看起来像一致。两个真正不同的后端都署上同一个 wrapper 名,会在去重里塌成一个。评级奖励或惩罚了错误的家族。审计继续产出自信的数字;它们只是不再意味着它们所说的了。

三个 release,三种伪造趋同的不同方式。

## 2. 抄兄弟的作业(fw-4.27.0,v1 加固)

第一个也是最显然的:一个审计器在写自己的 finding 之前偷看另一个审计器的 finding。那个更早被堵上了,在 v1 的 audit-prompt 加固里([#261](https://github.com/StrangeDaysTech/straymark/issues/261)) —— 审计器独立性加一个 contamination guard。这是底线:趋同只有在盲抵达时才是证据。值得在这里点名,因为那两个更新的修复是同一条原则应用到更微妙的泄漏上。

## 3. 把 mock 当成疆域(fw-4.32.0)

v1.1 的 audit-prompt 这一遍([`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0),[#303](https://github.com/StrangeDaysTech/straymark/issues/303)/[#306](https://github.com/StrangeDaysTech/straymark/issues/306))收紧了一个审计器*被允许*当作现实去信任的东西。两条加性规则:

- **审计对象 vs. 真相神谕。**prompt 现在把*你在哪里报告缺陷*(审计对象 —— git range 里的代码)与*你可以读什么来验证它们*(真相神谕 —— 包括 range 之外的东西)分开。一次审计客户端的运行,必须把 API/IPC/契约调用与**真实的服务端定义**交叉核对,哪怕服务端在 diff 之外。一个客户端↔服务端契约不匹配是客户端的一个可审计缺陷,而**绿色的客户端测试不能赦免它** —— mock 编码的是客户端自己的假定,所以一个通过的测试只是那个假定在和自己一致。(如果这个失效模式听起来眼熟,它正是 [Baton coherence bridge](what-the-spec-path-only-proved-existed) 为之而建、要去抓的 #304 drift,从审计一侧看过去。)
- **验证保真度。**对每一个"已验证 / 已解决 / 已完成"的断言,审计器现在都要问它是*对着哪个现实*被核对的 —— 真正要紧的那个条件,相对于一个方便的代理(一个本地测试、一个 mock、文档自己的断言) —— 并打开工件本身,而不是信任一份下游摘要。`real_debt` finding 类别也被重新指向一等的 follow-up registry 并带 TDE 提升,好让一个确认的缺陷落在某个持久的地方,而不是一张松散的审计后便条。

这是审计在学着不信任方便的证据。接下来两个 release 让它不信任它从没想过要质疑的东西:它自己。

## 4. 是谁做了那次验证(fw-4.33.0 与 fw-4.34.0)

这就是那个 bug,它是个好例子,因为关于它的一切看起来都正确。

Router CLI —— Qwen Code、Gemini CLI 及其同类 —— 通过 system prompt 注入一个**产品身份**。于是当一个审计器填它报告的 frontmatter 时,它自检测并写下 `auditor: qwen-code` —— 那个 *wrapper* 的名字 —— **哪怕 operator 早已通过 `/model` 切换了真正的后端模型。**报告是良构的。字段被填了。它只是命名了那个 CLI,而不是那颗心智。

这一下做了两件坏事。它**腐蚀了归因** —— 你再也说不出到底是哪个模型家族真正产出了这个 finding。而且它能**伪造 cross-family 一致**:两次跑在真正不同后端上的运行都署上同一个 CLI 产品名,或者一次运行署上错误的家族,于是整个审计所依赖的趋同-与-去重的数学在源头被毒化。审计器评级 —— 那个本该告诉你哪些家族可靠的东西 —— 给一个幻影评了级。

[`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) 把**operator 提供的身份**变成审计器的权威。`straymark-audit-execute` 的第 2 步被重写,从一个可选的第二参数取身份 —— `/straymark-audit-execute <CHARTER-ID> <AUDITOR-SLUG>` —— 或一句聊天中的陈述,以**禁止替换成 CLI 产品名**,并加一个强制的**写后 guard**,验证 `auditor:` frontmatter 和报告头都与提供的 slug 匹配。自检测只作为 operator 什么都不提供时的最后手段 fallback 存活。

[`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0) 把完全相同的修复应用到 **calibrator** —— consolidation 步骤(`straymark-audit-review`)上的身份,`calibrator:` / `**Reviewer:**` 字段 —— 用同一个可选第二参数(`/straymark-audit-review <CHARTER-ID> <CALIBRATOR-SLUG>`)和同一个写后 guard。因为*裁判*趋同的那个 reviewer,必须和产出趋同的审计器一样被诚实地归因;一个被贴错标签的裁判,不比一个被贴错标签的选手更好。

两个修复都跨全部四份 runtime 副本发布 —— `.agent` workflow 和 `.claude` / `.gemini` / `.codex` skill —— 因为这个 bug 住在 prompt 运行的任何地方,而一个只改一份副本的修复,在 operator 一切换 CLI 的那一刻就失效。

## 5. 这刻意不是什么

它不是一个**检测器。**框架无法伸进一个 CLI 去读真正被选中的是哪个后端;operator 可以,而这个修复让 operator 成为权威,而不是假装工具能说得出。guard 验证的是*一致性* —— 写下来的东西与 operator 声明的东西相符 —— 而不是关于硅片的地面真相。

它不是**乔装的自检测。**自检测只作为 fallback 留着,而且它明确是最不被信任的路径。整个动作是把工具对自己的猜测降到人类的陈述之下,正如 [Baton 的 work-verb 毕业](what-the-author-already-knew)把一次标题扫描降到一个声明动词之下。同一个教训,不同的角落:权威信号是 operator 声明的那个,而不是机器对自己身份推断出的那个。

## 6. 如果你读到了这里

可移植的问题,是关于任何聚合独立判断的系统里的溯源 —— code review 的签核、red-team 的 finding、模型 ensemble 的投票、第二意见。这个聚合只和附在每个输入上的*身份*一样可信,而身份恰恰是没人复核的那个字段,因为它被自动填上,而且它总是看起来合理。`auditor: qwen-code` 是一个完全良构的值。它只是在回答"是什么工具跑了这个",而数学需要的是"是什么心智产出了这个",而这两者在一个 wrapper 让你把底下的后端换掉的那一刻就分道扬镳了。audit-prompt 花了几个 release 学着信任*什么*被验证了。这两个让它信任*谁*做了那次验证。去找出你自己流水线里那个免费被填上的身份字段 —— 然后问:它命名的是你真正倚仗的那个东西,还是只是碰巧握着笔的那个东西。

---

*StrayMark Framework [`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0) → [`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) → [`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0),建立在 fw-4.27.0 的 v1 加固之上。Issues [#261](https://github.com/StrangeDaysTech/straymark/issues/261) · [#303](https://github.com/StrangeDaysTech/straymark/issues/303) · [#306](https://github.com/StrangeDaysTech/straymark/issues/306)。相关:[Charter 作为一等实体,以及外部 audit cycle](charters-and-the-external-audit-cycle) · [作者本就知道的事](what-the-author-already-knew)。*

*本文档在生成式 AI 工具（Claude Opus 4.8）的协助下完成；内容的全部责任由人类作者承担。*