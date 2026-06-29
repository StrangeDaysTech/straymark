---
slug: what-the-binary-couldnt-hide
title: 二进制藏不住的东西
authors:
  - jose
tags: [straymark, charters, governance, polish-charter, sentinel, anti-pattern]
date: 2026-05-23
description: 一次 polish Charter 浮现了十个潜伏 gap，以及一个终于得到命名的反模式
---
*一次本应是美容清理的 polish Charter，在六个小时内浮现了十个生产 gap —— 其中两个已经交付到 `main` 并且整整十天从未真正工作过。反模式获得了名字，模式作为 `fw-4.18.0` 被写入文档，而 CLI helper 被有意推迟。*

<!-- truncate -->

> *"polish Charter 尝试做的第一件事 —— 从 `main` 启动 `./sentinel` 来执行 §5 的 smoke —— 以两个不同的 panic 失败了。"*

5 月 22 日，Sentinel adopter 以 RFC 形式提交了 Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199)，正是这样开头。当 thread 在五条评论更新后收尾时，计数已经从两个增长到十个。polish Charter —— Etapa 2 的关闭 Charter，原本被规划为一份 WCAG 审计与 quickstart 验证的清单 —— 用六个小时做了另外一件事：捕获了一类潜伏回归，而之前八个 Charter 的所有 per-Charter 测试套件都没能抓住它，*也*不可能抓住它，因为它们的写法决定了这一点。

这篇文章是对那个早晨为何发生的重建，对底层那个反复出现的形状是什么的追问，以及在 PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) 中所做的深思熟虑的决定：**为反模式命名，但暂时不为它打造工具**。这个循环对追读过前几篇 —— 关于[涌现观察](emergent-observation-design)与[链条演化](pattern-1-and-pattern-2-chain-evolution) —— 的读者来说很熟悉：这是两周内第三个沿着同一弧线结晶的模式：在 N=1 中浮现 → 为元模式命名 → 在 N=2 验证之前，推迟跨项目工具化。

## 真正分量重的两个案例

在 polish 会话浮现的十个 gap 中，八个或者是环境依赖腐烂（`huma` 升级在 `*string` 参数上引入了 panic；Go 1.22 收紧了 `http.ServeMux` 的 wildcard 语义），或者是 runbook 漂移（§boot 中缺失的 env vars、混淆了互斥模式的 smoke 配方、关于 fake provider 标准输出的声明而 fake 实际上从未写过任何东西）。每个都值得修。没有哪个改写了规则手册。

真正改写了规则手册的那两个不一样。它们共享一个形状。

**US3 Preference Center，5 月 12 日交付。** 当 Sentinel 发送邮件的 recipients 点击 footer 中类似 unsubscribe 的链接时，他们落在一个 JWT-in-path 的 URL 上，形如 `/preferences/<token>`。handler 是有意公共-按-合约的：JWT *就是* auth；不期望 `Authorization` header。handler 的 doc-comment 这样写着。集成测试用 `humatest` 写成，通过测试 adapter 直接挂载 handler 并确认：给定 path 中一个合法的 JWT，handler 返回正确的 HTML。CI 通过。feature 被交付到 `main`。

从未发生的事：任何在那个 handler 前面执行生产 middleware 链的事情。`internal/core/middleware/auth.go` 有一个 `publicPrefixes` 列表，列出绕过 `Authorization` 检查的路由前缀。`/preferences/` 不在这个列表里。整整十天，每个点击 unsubscribe 链接的 recipient 都从 middleware 那里收到一个 `401 missing authorization header`，请求甚至没有触达那个本来知道不该期望 Authorization 的 handler。Preference Center 在测试中可达，在生产中不可达。

**发送 pipeline 的 OTel 可观测性 instruments，同一周交付。** 八个 metric instruments —— counters、histograms、gauges —— 在 `internal/core/metrics/commshub.go` 中声明，在启动时与 OpenTelemetry meter 注册。Charter 的 `Constitution Check §IV` 正式宣布 FR-039..042 可观测性门已满足：API 层是正确的，注册都通过了，dashboards 已经规划好。这八个 instruments 中有七个在**整个 commshub 模块中拥有零个 `.Add()` / `.Record()` 调用位置**。声明了，注册了，从未被调用。ops 在这些 series 之上构建的 dashboards 已经收了十天的零数据。polish Charter 的手动 smoke —— 启动一个 OTel Collector，生成几个 sends，grep collector 输出查找 FR-039..042 名称 —— 立即浮现了这一点。

两个案例都不奇特。两个都不是硬 bug。两个都是同一个机械错误的实例：一个 artifact 在一处被声明，而本应连线它的实现住在另一处，并且代码库或 CI 里没有任何东西在关联两者。

## 反模式想要的那个名字

当同一个形状出现得足够多以至于值得一个名字时，名字就重要起来。在 #199 的第三条评论更新时，Sentinel 作者已经数出了同一个底层错误的四个子类：

| 声明位置 | 连线位置 | 机械检查 |
|---|---|---|
| operator runbook 中记录的 env var | 代码中的 `os.Getenv(...)`（或对应 stack 的等价物） | 每个记录的 env var 都至少有一个消费者 |
| metrics 包中声明的 metric instrument | handler 或 worker 代码中的 `.Record()` / `.Add()` 调用位置 | 每个声明的 instrument 至少被记录一次 |
| 渲染/嵌入 HTML 中引用的 URL（`<script src=...>`、`<link href=...>`） | 注册在同一 API surface 上的路由 | 被服务的 HTML 中的每个 `src=`/`href=` 都解析到一个已注册的路由 |
| 标记为公共-按-合约的路由（doc-comment、专用标记） | auth middleware 的公共前缀列表中的条目 | 每个公共-按-合约的 handler 都有匹配的前缀条目 |

那个一句话的统一者 —— 新治理文档以此开篇的那一句 —— 是：

> *每个被声明的 surface artifact 都至少有一个可从真实 request 触达的连线位置。*

这个反模式的名字 —— 它在 `dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md` 中作为典范落地 —— 是**声明了表层但未连线**（Surface declaration without wiring）。那才是交付物。polish Charter 是发现工具，不是论点本身。

## 为什么集成测试会系统性地漏掉这些

这一节值得展开，因为它塑造了这篇文章其余部分的形状。四个子类共有的失败模式是：标准集成测试 harness —— Go 中的 `humatest.NewTestAdapter`、TypeScript、Python、Rust 中的对应物 —— *直接*通过测试 API 挂载 handler。被测的 handler 由 fixture 正确连线了。生产组合的那一步 —— 路由注册在那里与 middleware 链相遇，与 env-var 清单相遇，与嵌入资源表相遇 —— 才是坏掉的那一步。CI 的绿灯对它声明的内容是诚实的：handler 在正确的 request 下返回正确的响应。它对以下事情什么也没说：request 在生产中是否能触达 handler，以及 handler 所依赖的 artifact 是否曾经被连线到 runtime。

把这件事读成对 `humatest` 的批评是诱人的。它不是。`humatest`（以及它的等价物）正在做它被设计来做的事：让你以隔离方式、快速地、不需要立起整棵组合树就测试 handler 逻辑。那种隔离是一个 feature。隔离的代价就是我们刚刚命名的那个 —— 而代价只在隔离之外的某些事浮现出分歧时才变得可见。polish Charter 是浮现那种分歧最便宜的方法，因为它做了没有任何 test fixture 在做的事：它启动真实的 binary，对它执行真实的、已记录的 operator 配方。

这是新 pattern doc 那项承重的主张。不是说 polish Charter 出于美容原因而有价值。而是说它是**唯一**让生产组合在外部可读的规范（operator runbook）面前被 end-to-end 执行的地方。如果你把 polish Charter 当作美容清理，你也就把那种浮现能力当作美容。Sentinel 的数据论证了相反的方向。

## 决定：B′，不是 B

RFC 提出了三个选项。PR #200 中的决定是它们都不照原样采纳 —— 称之为 B′。值得说出原因，因为结构性的决定比表面看上去更重要。

**RFC 的 Option B** 要求一个 pattern doc 放在 `docs/patterns/` 之下。那个目录今天在 StrayMark 中并不存在，创建它会分裂项目的文档约定。已经住在正典里的经验模式 —— `FOLLOW-UPS-BACKLOG-PATTERN.md`、`CHARTER-CHAIN-EVOLUTION.md`、`EMERGENT-OBSERVATION-DESIGN.md`、`SPECKIT-CHARTER-BRIDGE.md` —— 都住在 `dist/.straymark/00-governance/`。它们共享一个 i18n 镜像基础设施（每个治理 doc 都有英文、西班牙文、简体中文的兄弟版本）。增加第三个文档 surface 会成倍放大 i18n 开销，并把项目的模式撒在两个家之间。把新 doc 留在 `00-governance/` 内部以零边际成本复用了既有基础设施。

**RFC 的 Option C** 要求一个 `straymark charter polish-checklist` 或 `straymark analyze declared-vs-wired` 的 CLI helper。Sentinel 作者已经原型化了一个（CHARTERs 25/26/27，三个预备性 CI 守卫），并愿意把它种到 upstream。这里的决定是保守的：推迟。不是因为原型没有价值 —— 它有 —— 而是因为 Sentinel 是 N=1。那四个子类是*一个 adopter、一个 stack* 所浮现的。framework 必须预先考虑才能交付一个有用的跨项目 CLI 的第五、第六、第七个子类，目前还不存在，因为没有第二个 adopter 把它们推上来。我们走过这条路。[`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) 作为 v0 在 `fw-4.10.0` 中交付，并在那里住了一个半月，等待第二个 adopter 在它毕业为 `straymark followups` 子命令之前完成验证。同一道闸门在这里适用。新 doc 的 `## Open questions` 明确说了：「结晶为 `straymark analyze declared-vs-wired` CLI 子命令 …… 闸门：N=2 adopters。」

**B′ 是落地的版本。** 一个新的治理 pattern doc 放在典范位置，以三种语言。Charter 模板里的第七个 `Format conventions` 项目符号，当作者关闭一个 Etapa 或 SpecKit `Polish` 阶段时把他们指向它。QUICK-REFERENCE `## Patterns` 表格里的一行。一个 `fw-4.18.0` 版本号 bump，附带在三十几份文档间的页脚级联。没有 CLI 子命令。没有新的 frontmatter 字段。没有 schema 变更。反模式有了名字；发现仪式有了一份文档；工具化在等。

## 反复重演的弧线

三周前，follow-ups backlog 模式沿同一弧线结晶：一个 adopter 浮现了一个需求，模式在 v0 被命名，CLI helper 被推迟。两周前，链条演化的两个模式结晶：Pattern 1（pre-declare SpecKit refresh）和 Pattern 2（post-close audit-driven Batch N.4），由同一个 Sentinel adopter 浮现，在 `fw-4.16.0` 中被命名，并以 `straymark charter refresh-suggest` 作为软 helper 而非硬闸门。一周前，元模式 —— `EMERGENT-OBSERVATION-DESIGN.md` —— 编码了让上述所有观察首先成为可能的东西：形式化的交叉引用加上文化上的许可。这周，这个模式。

这是一个月里的四次结晶，全都来自同一个 adopter，全都遵循同一个形状：**在 N=1 中浮现 → 为元模式命名 → 只在 N=2 之后才工具化**。当一个模式以鲜明的方式浮现 —— 六小时里十个生产 gap 就是鲜明的 —— 的诱惑是跳过命名，直接去做工具。纪律恰好相反。名字承担了大部分工作。工具化在到来时，是被至少两个 adopter 所浮现的内容参数化的；建造在 N=1 之上时，它是对一个 stack 与一个团队失败模式的外推，并倾向于把那些选择固化成不通用的 framework 默认值。

这一轮迭代里有一个真正新的、值得标出来的东西。新 pattern doc 命名了一个 Sentinel 作者已经承诺要发表的可证伪预测：下一个 Etapa 的 polish Charter，在刚刚在 Sentinel CHARTERs 25/26/27 中落地的三个预备性 CI 守卫的对照下执行，应该浮现大约少 80% 的 gap。如果那个预测成立，把 Option C 从「open question」升级为「CLI 子命令」就获得了定量支撑。如果它失败 —— 如果下一个 polish Charter 仍然浮现十个 gap 但落在没被预料到的第五个子类里 —— 失败本身就是下一个数据点，并会在打造工具之前重塑规范。任何一种结果都有用。两种都还没被观察到。我们在等。

## 关于刻意没有做的事的一则说明

Charter frontmatter 里没有 `phase: polish` 字段。没有 `straymark charter polish-checklist <ID>`。没有自动化 analyzer 在扫描 Go（或任何其他语言）代码中已声明但未连线的符号。pattern doc 把上述每一项都列为候选演化，明确以 N=2 或 Etapa-3 之后的回顾为闸门。在 `fw-4.18.0` 中它们一个都不存在。

这是有意的，而且值得公开说出来，因为对一个鲜明发现的自然反应是过度建造响应。每一项推迟都是用短期完备性换长期可移植性。一个 frontmatter 字段把 framework 锁定到下一个 adopter 可能不共享的词汇上。一个 CLI helper 把 framework 锁定到一个映射某个具体 stack 失败模式的 runtime 上。一个 analyzer 把 framework 锁定到扫描一种语言，而它的约定可能与下一个 adopter 的截然不同。任何这样的承诺都不应基于一个领域的信号来做，无论信号多干净。pattern doc 本身可以被修订；一个 CLI 子命令不能在不破坏 adopters 的情况下被收回。

## 如果你读到这里，我会建议你看什么

如果你运营一个有基于 mock-adapter 的集成测试的代码库 —— 大多数代码库都是 —— 你可以做一项有用的内部练习，完全不需要采纳 StrayMark 的任何东西。挑你的团队最近交付的、同时具有（a）docs 一侧声明（env vars、OpenAPI specs、嵌入式 HTML、metric instruments）与（b）住在另一份文件里的连线位置的最新 feature。在一个干净的 shell 里启动 binary。把 runbook 中面向 operator 的配方端到端跑一遍。如果第一次就跑通了，你的代码库还没有这个模式所捕获的潜伏债务 —— 或者它有但你这次走运。如果跑不通，数一下 gap。这个数字就是你需要的校准，用来判断 polish-Charter-作为债务检测的仪式在你的项目里是否值得它的开销。

如果你已经采纳了 StrayMark 并且正在关闭一个用 `humatest` 风格 adapter 测试 handler 的 Etapa，新 pattern doc 已经为你 scoped 好四个子类检查。把 polish Charter 预算为 L，不是 XS 或 S。预期会有 emergent 的 follow-on Charter，而不是残余清理 scope creep。在 scoped 工作之前去读 [`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md)，而不是在之后。

而如果你是 Go 之外某个 stack —— TypeScript、Python、Rust、Elixir —— 上的 adopter，那四个子类是与语言无关的，但具体的检查形态不是。在你的 stack 里浮现一个第五或第六子类正是模式要从 v0 毕业所需要的第二-adopter 信号。从「在你的项目里一个有趣的观察」到「在 StrayMark 正典中被命名」的路径与 #199 走过的是同一条：开一个 issue。开它的成本很低；让模式处于欠验证状态的成本是真实的。

---

*StrayMark `fw-4.18.0` —— Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199) · PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) · tag [`fw-4.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.18.0)。Sentinel 锚点：[`AIDEC-2026-05-22-001`](https://github.com/StrangeDaysTech/sentinel/pull/93) · [CHARTERs 25/26/27 PR](https://github.com/StrangeDaysTech/sentinel/pull/94)。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
