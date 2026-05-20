---
slug: charters-and-the-external-audit-cycle
title: Charter 作为一等实体与外部审计周期
authors:
  - jose
tags: [straymark, charters, 审计, cli, 治理]
date: 2026-05-06
description: 从 Sentinel 中的手工仪式到正式 CLI 命令
---
*每个 Charter 在两小时工作之上还要 40 分钟的复制粘贴。纪律在起作用；仪式却变成了问题。Charter 如何成为 CLI 的一等实体，以及那一条把审计编排留给框架、把 prompt 评估交给任何第三方的架构决策。*

<!-- truncate -->

## 1. 当纪律开始变成仪式

4 月末，Sentinel 以一种写出来略显狼狈的感受关闭了当月第四个 Charter：外部审计是有效的——Copilot 9.25，Gemini 9.5，漂移脚本零误报——但每次关闭都要我手动打开三个文件，把三段 prompt 复制进三个不同的窗口，等待回复，再把三份标定 YAML 粘回遥测文件，然后逐一核对哈希值以确认匹配。每个 Charter 在两小时正式工作之外还要 40 分钟的复制粘贴。纪律在起作用；仪式却变成了问题。

5 月 3 日撰写的 `audit-skills-design.md` 提案毫不掩饰地点明了这一点：

> *"如果每次关闭时操作员都必须在文件间手动搬运数据，吞吐量将会崩溃，纪律也将从有益的摩擦变成纯粹的仪式。凡是可以可逆地自动化的，都应当自动化；文档保留用于事后人工审计。"*

本文涵盖三天——从 2026 年 5 月 2 日到 6 日——期间两条并行主线同时收尾。其一：Charter 从手工实践变成了 CLI 的一等实体。其二：外部审计周期从使用临时 prompt 的手工仪式变成了一条正式命令（背后有一个反直觉的架构决策，值得单独成节）。

---

## 2. Sentinel 已经在做的，与 StrayMark 尚未拥有的

5 月 3 日的提案用一句话点明了当时的状况，措辞算不上优雅，却精准：

> *"Sentinel 有这些技能，StrayMark 没有。"*

Sentinel 已有的：`sentinel/.claude/skills/plan-audit/` 和 `plan-audit-review/`——本地技能，能生成 prompt，收到回复后自动标定，并合并到遥测文件中。它们可用，已经过六个周期的验证。但它们耦合了 Sentinel 特有的路径（`docs/plans/`、`internal/modules/`、`go vet`）以及 *"Plan"* 词汇——而这个词汇在一周前刚刚被重命名为 Charter。

StrayMark 尚未拥有的：没有任何等价物。截至 4 月，框架只有文档 + 技能 + 带 `init`、`update`、`remove` 的 CLI。在 Sentinel 中称为 *Plan* 的单元——预先声明、事后审计的有界工作单元——作为 CLI 工件并不存在，只以实践的形式存在。

fw-4.4.0（5 月 2 日）以及 fw-4.7 → 4.9（5 月 3-5 日）的演进弧线，几乎是逐字地将 Sentinel 的手工做法移植成任何框架使用者都可以调用的通用命令。从 `cli-roadmap.md` 摘抄的迁移对照表直白地说明了来源：

| Sentinel 工件 *(2026 年 4 月)* | StrayMark 等价物 *(2026 年 5 月)* |
|---|---|
| `docs/plans/` 中的 `TEMPLATE.md`（v3 版） | `dist/.straymark/templates/charter-template.md` |
| `scripts/check-plan-drift.sh`（145 行 bash） | `straymark charter drift`（Rust 子命令） |
| 本地技能 `plan-audit` | 通用技能 `straymark-audit-prepare` |
| `audit/plans/05,06/{copilot,gemini,claude}.md` 中的双份报告 | `straymark charter audit` 的正式输出 |
| 手动编辑的遥测 YAML | `dist/.straymark/schemas/charter-telemetry.schema.v0.json` |

fw-4.4.0 的 CHANGELOG 直白表述：*"将 Charter 模式固化——预先声明、事后验证的有界可审计工作单元——该模式源自 Sentinel 六周期 /plan-audit 实验。"* 固化才是关键。4 月还是配了 bash 脚本的定制做法，5 月已是带可验证 schema 的正式命令。

---

## 3. 成为"一等实体"意味着什么

PR #65（fw-4.4.0 / cli-3.6.0，5 月 2 日）做了三件具体的事：

- 创建 `straymark charter new` 命令。自动递增编号（`NN-slug.md`），预填来源（若源自已有 AILOG 则填 `originating_ailogs`；若源自 SpecKit `plan.md` 则填 `originating_spec`），并支持 `--type X|S|M|L` 标志以在第一时间确定规模。
- 引入 `charter-template.md`，移植自 Sentinel 的 `TEMPLATE.md` v3。模板内嵌了 4 月实验逐周期固化的六项规范：本地/生产检查分离、以时间而非故事点计量投入、结构化子章节、`R<N>` 风险记录、通过 AILOG 的合并后协调进行闭环、以及自动核查清单漂移。
- 随附 `dist/.straymark/schemas/charter.schema.v0.json`——schema 刻意标注为 `v0` 而非 `v1`，因为框架原则 #12 规定：未经第二个领域验证的 schema 不得固化。Sentinel 是一个领域，第二个尚缺。

PR #68（5 月 3 日）看似动作小，却举足轻重：*原子 Charter 关闭模式*，格式 v4。在 Sentinel 中，*"若 AILOG 记录了偏差，则在合并后更新 Plan 文档"* 这一步骤在 TEMPLATE 中仅作为备注存在，没有系统性触发机制。实践中，每当声明内容与交付结果存在偏差，操作员（我本人）要靠记忆来协调文档——而记忆会失效。偏差会在仓库中停留数天乃至数周，直到下一个周期才被发现。格式 v4 将协调变成关闭的强制步骤，而非边注。

PR #69（同日）解决了一个摩擦细节：手动编号。此前需要自行判断下一个 Charter 是 11 还是 12；现在 `straymark charter new` 读取目录并提议下一个编号。纯粹的用户体验改进。但这类摩擦累积到二十个 Charter 时，决定的是框架被使用还是被抛弃。

三个 PR 在同一个周日和周一的提交潮中发布。这种节奏是刻意为之：一旦模式固化，用户体验细节必须*一起*关闭，而不是分散在几个 bump 中，让使用者拿到半成品流程。

---

## 4. 一天三个版本

随后的 fw-4.7.0、fw-4.8.0、fw-4.9.0 是外部审计周期的连续版本。`audit-skills-rollout.md` 提案如实记录了这一节奏：

> *"第一阶段在 1 个日历日内完成（2026 年 5 月 3 日连续 5 个 PR），远快于 1.5-2 周的重点估算。吞吐量来自 `audit-skills-design.md` 中决策已提前固化（D1/D2/D3），以及带有明确优雅降级的 arborist 启发式——实施期间无待定决策。这为原则 #6 提供了额外的操作验证：提案写得好，实施就是执行，而非设计。"*

具体的外部审计周期由三条链式命令组成：

1. `straymark charter audit prepare <CHARTER-NN>` ——从已关闭的 Charter、其关联的 AILOG 以及被修改文件的 diff 生成规范审计 prompt。输出：`audit/<CHARTER-NN>/{copilot,gemini,claude}.prompt.md` 中的三个 `.prompt.md` 文件。
2. *（人工将这些 prompt 带到所选审计工具，收到回复后粘贴回来。）*
3. `straymark charter audit collect <CHARTER-NN>` ——接收回复，对照审计 schema 逐一验证，将标定值合并进 Charter 的遥测文件，并输出审计员之间收敛（或发散）的摘要。

提案中的决策 D1 是整个体系的基础：

> *"技能通过 `Bash(straymark charter audit *)` 委托给正式实现。模板只存放在 `dist/.straymark/audit-prompts/` 中。技能与 CLI 之间零漂移可能。"*

prompt 只有一个存放位置，流程只有一个实现，技能（Claude、Cursor）通过 Bash 调用它。这是朴素的模式——CLI 是唯一来源——却避免了任何拥有两个接口（技能 + CLI）的框架最终都会出现的顽疾：技能和 CLI 因各自独立更新而说出不同的话。

---

## 5. 为什么 CLI 只做编排而不调用 API

这是 5 月最奇特的决策，也是我花费最多精力作出的决策。它在 `cli-roadmap.md` §0 中被字面记录为 *"架构决策 A1"*：

> *"A1（第三阶段）：仅编排，v0 中不含 HTTP API 客户端。路线图 §5.4 原本建议'v0 支持 OpenAI/Google/Anthropic'并处理 API 密钥。实际实现是：CLI 准备 prompt，对照 schema 验证输出，并与遥测集成——但不调用 API。操作员手动将 prompt 粘贴到所选审计工具。"*

其理由，同样字面呈现：

> *"实现 3 个 HTTP 客户端需要 1-2 周加上 API 变更时的持续维护（对于实验性 v0 而言过于超前）；人在回路的模式与 Sentinel 的 `/plan-audit` 一致；符合原则 #10（'不是 LLM 网关'）；在设计上关闭了 RFC #82。HTTP 客户端推迟到 v1，届时真实使用者以数据证明其必要性。"*

三个论点，各有分量。

**第一个是务实层面的。** 三个 HTTP 客户端——OpenAI、Anthropic、Google——需要一到两周的实现工作，加上每当三方之一有所变更时的持续维护。对于一条手工形式在 Sentinel 中已有经验验证的命令来说，这是荒谬的成本。这是为一个没有任何使用者提出的问题而做的工程。

**第二个关乎延续性。** Sentinel 在 4 月验证的，恰恰是人在回路模式：操作员*手动*将 prompt 粘贴进 Copilot/Gemini/Claude，阅读回复，再粘回来。那个诞生了整条演进弧线的六个 Plan 实验*从未*调用过 API。如果 CLI 现在自行调用 API，就是用一个未经验证的模式取代一个已经验证的模式，毫无经验依据。

**第三个关乎身份。** StrayMark 不是 *LLM 网关*。框架原则 #10 明确说明了这一点。LangChain、LiteLLM、LiteLLM Proxy、OpenRouter，以及各类包装器——有数十个产品*是*这个定位——它们对自身所做的事情是有用的。StrayMark 做的是另一件事：它围绕*使用*这些模型所完成的工作构建纪律，无论是哪个模型。CLI 是否调用 API 改变的是框架*是什么*；保持*仅编排*的定位，使身份保持清晰。

这段话中我最在意的部分：*"HTTP 客户端推迟到 v1，届时真实使用者以数据证明其必要性。"* 这个决策不是被教条关闭的，而是被设置在一个证据门槛之后延后处理的。如果六个月后某个使用者证明人在回路流程破坏了他们的吞吐量，HTTP 客户端就会落地。在那之前，手动将 prompt 粘贴到另一个窗口所需的四十秒摩擦，与维护三个活跃 SDK 的成本相比，微不足道。

---

## 6. 框架决定*不*自动化什么

值得用同一决策的另一面来收尾。`audit-skills-design.md` 提案写道：

> *"凡是可以可逆地自动化的，都应当自动化；文档保留用于事后人工审计。"*

前半句解释了 5 月的三个版本：prompt 自动生成，标定值自动合并，漂移自动检测。后半句——*"文档保留用于事后人工审计"*——解释了哪些内容是刻意保留为手工的。

框架*不*自动化的内容：

- **操作员在关闭 Charter 前阅读 AILOG。** `straymark charter audit prepare` 命令会生成 prompt，但操作员*必须*自行打开 AILOG 并阅读。如果我们将这一步自动化——由智能体来阅读、总结、决策——就会失去那个使 Charter 存在有意义的人类判断时刻。
- **接受或拒绝外部审计的决策。** CLI 将标定值合并进遥测文件，但不对其采取行动。如果某位审计员给 Charter 打了 4 分，Charter 不会自动重新开启：操作员决定是重新开启、提出异议，还是声明其为*可接受的已知债务*。这个决策就是纪律；将其自动化会摧毁纪律。
- **调用模型。** 如上所述：操作员复制 prompt 的暂停时刻，正是他们决定使用哪个模型、调整哪段 prompt、判断问题措辞是否合理的时刻。那个暂停是功能，不是缺陷。

贯穿三者的标准只有一个：*自动化可逆的、机械的；将需要判断的保留为手工*。四十秒的复制粘贴并不算多，如果换来的是让判断留在人手中。

---

## 7. 结语

我从这个过程中得到的四点结论：

1. **经验验证有效的实践无需重新发明即可被正式化。** Sentinel 在 4 月完成了验证工作；5 月只是将其移植进框架。固化是迁移，不是从零设计。

2. **提案写得好，实施就是执行。** 一天三个版本只有在所有决策都在触碰代码之前关闭的情况下才有可能。原则 #6 的表述更为平实，但这就是它的含义。

3. **并非所有可以自动化的都应该自动化。** 决策 A1——CLI 做编排但不调用 API——是一个哲学决策的技术表达：人在回路不是需要消除的遗留做法，而是支撑纪律的功能特性。

4. **框架的身份由它做什么和它不做什么共同定义。** *"不是 LLM 网关"* 是框架原则 #10 中少数几句值得逐字记住的话之一。StrayMark 之所以把自己做的事情做好，恰恰是因为有许多事情它拒绝去做。

下一篇文章：与本博客起点直接相连的方法论事件——*Issue #113：智能体看不见的 Charter*——框架在此发现，创建命令和 schema 还不够，如果仓库的*接触面*不能向智能体表达自身。这是本文的自然对立面：这里我们谈的是什么被正式化了；那里我们谈的是什么没有被看见。

---

*锚点：PR [#65](https://github.com/StrangeDaysTech/straymark/pull/65) · [#68](https://github.com/StrangeDaysTech/straymark/pull/68) · [#69](https://github.com/StrangeDaysTech/straymark/pull/69)（Charter 一等实体）。提案 [`audit-skills-design.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-design.md) · [`audit-skills-rollout.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-audit-skills-rollout.md) · [`audit-cli-flow.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-04-audit-cli-flow.md) · [`cli-roadmap.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/proposals/2026-05-03-cli-roadmap.md)（决策 A1 §0）。版本 fw-4.4.0 → fw-4.9.0。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
