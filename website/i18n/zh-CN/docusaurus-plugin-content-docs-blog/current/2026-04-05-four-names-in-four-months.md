---
slug: four-names-in-four-months
title: 四个月里的四个名字
authors:
  - jose
tags: [straymark, 早期历史, 身份认同, 治理]
date: 2026-04-05
description: 寻找概念，而非寻找名字
---
*Chronicle → Monimen → DevTrail → Strange Days Tech → StrayMark。四个月、四次改名 —— 一月里仓促的三次，五月里有纪律的一次。理念没有动；名字动了。*

<!-- truncate -->

> **编者按**：这是本博客第一篇以追溯方式发布的文章。frontmatter 中的 `date` 对应的是文章所叙述的时间弧线中的某一时刻，而非实际撰写当天。整个博客都以这种方式构建——从后往前，从触发我提笔的那个事件开始（`emergent-observation-design`，2026 年 5 月 16 日）。假装情况不是这样，对任何人都没有帮助。

---

## 1. 第二天

2026 年 1 月 28 日 17:29（中部时间），这条 commit 落入了仓库：

> `chore: rebrand Chronicle to Monimen`

仅仅一天。确切地说，是项目*初始 commit* 后的二十九小时。框架刚刚存在，就已经在改名了。

我用过去时态来叙述，但这件事发生在三个月前。而在三个月前，又经历了两次改名之后，它依然还不叫 StrayMark。这个今天以*面向 AI 辅助软件开发的文档治理框架*自居的项目，有着一段简短、凌乱的早期历史，值得在继续前行之前把它说清楚。

---

## 2. 五个事件，四个月

| 日期（UTC-6） | 时间 | 事件 | 锚点 |
|---|---|---|---|
| 2026 年 1 月 27 日 | 12:14 | *初始 commit*：**Enigmora Chronicle Framework v1.0.0** | [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) |
| 2026 年 1 月 28 日 | 17:29 | 改名：Chronicle → **Monimen** | [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) |
| 2026 年 1 月 29 日 | 19:30 | 改名：Monimen → **DevTrail**（`BREAKING CHANGE`） | [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) |
| 2026 年 3 月 1 日 | 19:05 | 组织改名：Enigmora → **Strange Days Tech**，同时 Rust CLI 诞生 | [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) |
| 2026 年 5 月 8/9 日 | — | DevTrail → **StrayMark**（ADR + 五个 PR 的弧线） | ADR `2026-05-08-001`，PR #114-#118 |

三天内三次改名。然后是长达一个月的停顿。然后是捆绑 Rust CLI 的组织品牌迁移。再然后又是两个月的停顿。最后是 StrayMark，这次有一份公开的 ADR 作为决策的锚点。

有一个细节值得注意：2026 年 1 月 28 日的 commit（Chronicle → Monimen）没有标注 `BREAKING CHANGE`；而 1 月 29 日的那次（Monimen → DevTrail）标注了。commit 正文写道：*"BREAKING CHANGE: Complete rebrand from Monimen to DevTrail"*。差别虽小，却说明问题：第二次改名显得更具决定性。它确实如此——坚持了整整两个月。

---

## 3. 不曾移动的东西

这是我最感兴趣的一节。项目在四个月内四次改名；理念却纹丝未动。

*初始 commit* 的 README —— 也就是项目还叫 *Enigmora Chronicle Framework* 时仓库里的那份 —— 以这行字开头：

> **Documentation Governance for AI-Assisted Software Development**

这也是今天 StrayMark README 的第一行。四个名字，一句标语。

稍往下，同一份 v1.0.0 README 在一个引用块中明确陈述了项目的核心主张：

> *"No significant change without a documented trace."*

这句话今天是框架的第一原则。它存在于 `PRINCIPLES.md §1 — Total Traceability` 中。翻译成西班牙语时措辞略有调整，但主张的分量完整保留：没有文档记录的重大变更，就不应发生。

v1.0.0 README 列出的八种文档类型 —— REQ、ADR、TES、INC、TDE、AILOG、AIDEC、ETH —— 全部留存至今。后来新增了一些（MCARD、SBOM、SEC、DPIA），但没有一个取代了原有的类型。一月份的分类体系经受住了时间的考验。

**名字动了四次；理念没有动。** 那四个月里改变的不是*内容*，而是贴在它上面的标签。

---

## 4. 年份打错的 AIDEC

有一个小细节，我总是会回想起来。

项目的第一份结构化文档 —— 第一份 AIDEC，即框架自身分类体系中用于记录借助智能体协助所做决策的文档 —— 是 `AIDEC-2025-01-27-001-i18n-strategy.md`。它存在于 commit [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e)，添加于 2026 年 1 月 27 日 18:01，也就是*初始 commit* 后六小时。

ID 写的是 **2025**。commit 日期是 **2026**。

正是这份确立了 `[TYPE]-[YYYY-MM-DD]-[NNN]` 标识符规范的文档，在自己的年份上打了个错字。框架从不完美中起步，没有人及时修正它。任何克隆仓库后仔细查看这个 ID 的人都会注意到 —— 同时也会看到引入它的 commit 日期是正确的。这是我掌握的最好证据，证明审计纪律是后来才会养成的东西，并不会随仓库一起交付。

但比这个错字更有意思的，是那份 AIDEC 做出的决策。2026 年 1 月，José 借助 Claude Opus 4.5 的帮助提出了这个问题：如何对一个为人类而存在、但配置文件由智能体读取的框架进行国际化？该 AIDEC 的论证部分给出了答案：

> *"AI agents (Claude, Gemini, Copilot, Cursor) process instructions equally well in any language, so translating their config files provides no functional benefit."*

那份文档做出的决策 —— 翻译人类阅读的内容（文档、模板），保留智能体读取的内容（CLAUDE.md、GEMINI.md、.cursorrules）为英文 —— 是一个早期直觉：框架有两类截然不同的受众。这个博客本身今天也印证了这一直觉：面向阅读的人类提供中西双语，而编排智能体的技能和提示词则仅保留英文。三个月后，这仍然是正确的选择。

---

## 5. CLI 的诞生，以及那个缺失的版本号

### 5a. 项目成为软件的那一天

2026 年 3 月 1 日 19:05，commit [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) 落入仓库：

> *"feat: rebrand to Strange Days Tech, add CLI scaffolder, restructure repo"*

在那天之前，这个项目只是文字。是模板、技能、治理规则，以及一堆 Markdown —— 依赖操作者（我本人）手工将其复制到想要使用它的仓库中。有一个叫 `copy-devtrail.sh` 的 bash 脚本。那是个玩具。

那次 3 月 1 日的 commit 引入了 `cli/Cargo.toml` 和 `cli/src/main.rs`。第一个 CLI 叫做 `devtrail-cli`，版本 `2.0.0`，有三个命令：

```rust
enum Commands {
    Init { path: String },
    Update,
    Remove { full: bool },
}
```

`init`、`update`、`remove`。三个操作。其余的都是后来的事：`status`、`repair`、`validate`、`new`、`compliance`、`metrics`、`analyze`、`audit`、`explore`。但最初的三个依然还在，几乎原封未动。

那次 commit 的意义不在于那三个命令，而在于：从那天起，这个项目不再是一堆存活于自身仓库里的文档，而成为了一个可以安装到其他仓库中的可执行工具。*"文档治理项目"* 与 *"带工具链的框架"* 之间的边界，在那个三月的周日被跨越了。这比之前任何一次改名都更为重要。

那天也是项目迁移 GitHub 账号的日子：从 `enigmora/devtrail` 到 `StrangeDaysTech/devtrail`。组织在同一个 commit 里将自身从 Enigmora 改名为 Strange Days Tech S.A.S.，同时也发布了 CLI。两个层面的身份认同同步迁移：产品的（Monimen 到 DevTrail）发生在一月；公司的，发生在三月。两个不同层次的身份，按照各自的节拍移动。

（一个对存档准确性重要的补充说明：智能体联合署名 —— 出现在 commit 中的 `Co-Authored-By: Claude Opus X.Y` 行 —— 并不始于三月。2026 年 1 月 27 日的*初始 commit* 就已经由 Claude Opus 4.5 联署。AI 联合署名的透明化是从第一行代码起就确立的规则。3 月 1 日改变的不是这个实践，而是规模：CLI 是联合署名第一次产出*可执行代码*，而非文档。）

### 5b. 那个缺失的版本号

框架版本编号有一个细节，直接看 `git tag` 就能一目了然：

```
fw-2.0.0
fw-2.1.0
fw-4.0.0
fw-4.1.0
...
```

没有 `fw-3.x.x`。项目刻意从 2 跳到了 4。

这个跳跃对应的是 commit [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2)，日期为 2026 年 3 月 27 日，其正文如此描述这次变更：

> *"Reposition DevTrail from 'documentation helper' to 'ISO 42001-aligned AI governance platform' across all user-facing docs. Lead with regulatory urgency (EU AI Act Aug 2026) and compliance value proposition."*

在三月之前，DevTrail 将自己定位为一个*文档辅助工具*；从那次 commit 起，它将自己定位为一个*符合 ISO 42001 的 AI 治理平台*。版本号的跳跃（2 → 4，跳过 3）标志着这一主张转变的幅度。这是一个有意识的选择：仅仅一个大版本的跃升，不足以体现这种差异。

而这正是早期历史开始从回顾的角度产生意义的地方。那个自一月起就存在于仓库中的直觉 —— 一个健壮、规范、可对齐标准的记录体系的构想 —— 直到三月才被明确命名。我们将在下一节谈到第二个名字时，更清楚地看到这一点。

---

## 6. 关于那些名字

四次改名中，三次没有 ADR。理由只存活在 commit 信息里，有时甚至连那里也找不到：commit `chore: rebrand Chronicle to Monimen` 没有说明为什么是 Monimen；紧接着的那条 `chore: rebrand Monimen to DevTrail` 也没有说明为什么是 DevTrail。将 ADR 作为有纪律的制品的实践，是后来才形成的；在一月，名字随着一个 commit 而改变，理由则随着产生它的那场对话一同消散。

但部分词源学是可以追溯的。

- **Chronicle**（1 月 27 日）。历史记录。航海日志。这个名字不言自明：你用一部*chronicle*（编年史）所做的事，就是按顺序写下发生了什么，以便日后查阅。没有 ADR，但这个词自带含义。

- **Monimen**（1 月 28 日）。这是唯一一个我确实记得其直觉、且值得讲述的名字。*Monimen* 来自对 *Monumento*（纪念碑）的文字游戏。驱动这个建议的类比是：彼时已经隐约感知到的那些规范 —— AIDEC、AILOG，以及可对齐于新兴 ISO 42001 治理体系的一切 —— 代表着某种坚实而持久的东西。*纪念碑*是为了长存而竖立的；是一个社群中不可移动的参照点。这正是框架想要成为的。截短的后缀 —— *Monimen* 而非 *Monument* —— 是赋予这个词更敏捷、更具软件气质的尝试，少一些大理石的厚重感。它只存活了二十五小时。但那个直觉没有消亡：三月将被明确命名为*符合 ISO 42001 的 AI 治理平台*的东西，正是那同一冲动的成熟版本。Monimen 是一个有着正确主张、却说错了语言的名字。

- **DevTrail**（1 月 29 日）。开发者的足迹。比 Monimen 少了些庄重，多了些具体。那个 commit 标注了 `BREAKING CHANGE` —— 从五月的视角来看，颇具讽刺意味：前三次改名中最具决定性的那次，恰恰是在三个半月后被替换的那次。

- **StrayMark**（5 月 8-9 日）。那个标记，那个留下的痕迹。但这次品牌迁移值得单独成文：已经有了一份公开的 ADR（`2026-05-08-001`）、五个核心 PR 的弧线（#114-#118），以及 README 中的一份*"为什么叫 StrayMark？"*宣言，值得另行论述。在此，我只是将它作为那次终于有了文档纪律支撑的品牌迁移，画上句号。

重点不在词源本身，而在于：每个名字都印证了一种关于这个产品是什么的独特直觉——**日志**（Chronicle）、**规范支柱**（Monimen）、**开发者足迹**（DevTrail）、**残留痕迹**（StrayMark）。四个名字是对这个概念的四种假设。当概念稳定下来，搜索才会终止 —— 而只有到那时，名字才能安定下来。

---

## 7. 结语

我从这个过程中得出的结论，凝炼为四点：

1. **我们不是在寻找名字，而是在寻找概念。** 四次改名是探索的证据，而非优柔寡断的证据。概念在名字的轮替中不断清晰。

2. **四次改名中，三次没有 ADR。** 将 ADR 作为有纪律的制品的实践是后来才形成的。一月时还没有这个习惯。这不是债务 —— 这是对节奏的诚实。治理习惯是想要记录已然发生之事的产物，而非启动的前提。

3. **这个项目在 2026 年 3 月 1 日成为了软件。在此之前，它只是文字。** Rust CLI 是那道边界。任何关于 StrayMark *作为框架*的讨论，都必须承认那次 commit 之前和之后的区别。

4. **大概不会再有下一次改名了。** 但这个博客不作此承诺。这个项目对自身的可变性保持诚实，做出相反的承诺将背叛第一篇文章的第一个主张。

当我两周前开始写这个博客时，并没有打算回溯早期历史。本意是谈 Charter、谈 emergent observation、谈框架今天实际在做的事情。但这个博客本身就是一个追溯性的行为 —— 它的存在，是因为某件事让我足够惊讶，以至于我开始动笔。而一旦我决定从后往前写，抵达一月就成了必然。抵达三天内的三次改名。抵达年份打错的 AIDEC。抵达 *Monimen*。

下一篇：框架的第一个真正的方法论实验 —— Sentinel 的六个 Plan，以及 *Plan* 被重命名为 *Charter* 的那一天。那才是这个博客本来要讲述的故事真正开始的地方。

---

*锚点：commit [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) · [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) · [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) · [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e) · [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) · [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2)。原始 README：`git show v1.0.0:README.md`。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
