# Follow-ups Backlog 模式 - StrayMark

> 用于跨多个 AILOG 和 Charter 管理累积的 `§Follow-ups` 与 `R<N> (new, not in Charter)` 条目的一等公民注册表。

**语言**: [English](../../FOLLOW-UPS-BACKLOG-PATTERN.md) | [Español](../es/FOLLOW-UPS-BACKLOG-PATTERN.md) | 简体中文

---

## 状态

**v1 — 自 fw-4.21.0 / cli-3.19.0 起为一等公民实体**（实验性；硬性稳定化以第二个 adopter 为门槛，依据设计原则 #12 与 ADR-2026-06-03-001）。

成熟历程，与 Charter 通道相对应：

| 阶段 | 发布版本 | 落地内容 |
|------|---------|---------|
| 约定 (v0) | fw-4.10.0 | 模式文档 + adopter 端 bash 脚本（Sentinel CHARTER-12,N=47） |
| 精化 (v0.1) | fw-4.13.1 | FU → TDE 提升路径（2 种形态）、`total_promoted` 计数器 |
| **一等公民 (v1)** | **fw-4.21.0 / cli-3.19.0** | JSON schema、原生 `straymark followups` CLI、`explore`/`status` 集成、随 `AGENT-RULES.md §13` 发布的代理指令、注册表模板 |

该注册表是一个**一等公民工件**,与 Charter 一样 —— 不属于 16 种文档类型之一。它有自己的规范路径、自己的 schema、自己的 CLI 命名空间,以及在 `explore` TUI 中自己的合成分组。

---

## 何时适用此模式

StrayMark 的 per-AILOG `§Follow-ups` 约定在写入时有效 —— 创建 AILOG 时,实施者记录推迟到后续 Charter 或操作触发器的内容。在累积列表超出操作员可凭记忆扫描范围之前,这种方式都能正常工作。

当满足**任一**以下条件时,采用此模式:

- 项目已累积 **约 20 个或更多 AILOG**,带有非平凡的 `§Follow-ups` 部分。
- 操作员反复要求代理"列出项目中所有待处理事项",答案需要多文件扫描。
- 一个"当 X 到来时执行"的 follow-up 几乎丢失,因为在 X 到来后从未重读过原始 AILOG。
- Charter 回顾揭示出本应在数周前被分类为 `closed`、但从未被索引的 follow-ups。

低于此规模时,仅 per-AILOG 约定就足够了 —— 过早采用此模式只会增加维护开销而无回报。

### 注册表作为规划输入

来自参考 adopter 的经验教训（issue #214,N=91 个条目）:backlog 不仅仅是一份延期杂务清单。Follow-ups 不仅源自规划（ex-ante,事前）,也源自**执行现实** —— 测试运行、遥测读数、staging 事故、在真实（非模拟）环境中观察到的 bug —— 并且它们反过来反哺规划:它们变成杂务（chore）、迷你 Charter,甚至重塑已经规划好的 Charter。该注册表是 **SpecKit 的 ex-post（事后）对应物**:SpecKit 从意图反哺规划;backlog 从执行反哺规划。v1 维度（`Origin-class`、`Severity`、`Labels`、`Destination` 词汇表）的存在正是为了让这个规划闭环可被查询。

---

## 形式

### 注册表文件

规范路径下的单个 markdown 文件:

```
.straymark/follow-ups-backlog.md
```

一个带有空 frontmatter 和五个 bucket 标题的模板随 `.straymark/templates/follow-ups-backlog.md` 一起发布。

### Frontmatter (YAML)

```yaml
---
last_scan: 2026-06-03
last_scan_range: AILOG-NNNN-NN-NN-NNN..AILOG-NNNN-NN-NN-NNN  # 可选 —— 涵盖的首个..末尾 AILOG
schema_version: v1
total_open: 0                # CLI-owned —— 每次写入时重新计算
total_promoted: 0            # CLI-owned
total_closed_in_session: 0   # CLI-owned
total_phase_blocked: 0       # CLI-owned
total_suspected_closed: 0    # CLI-owned（v1 新增）
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-04-11-001
  - AILOG-2026-04-12-001
  # ... 每个 follow-ups 已被处理的 AILOG 一个条目
---
```

**自 v1 起,`total_*` 计数器为 CLI-owned。** 每个写入命令（`straymark followups drift --apply`、`straymark followups promote`）都会根据实际条目状态重新计算它们。不要手工维护它们 —— 陈旧的手工编辑值会在下一次写入时被纠正。这关闭了在 N=91 处观察到的静默计数器漂移失败模式（声明 `total_open: 47`,而 4 周后实际为 65 —— issue #214 信号 2）。`straymark followups status` 始终显示即时重新计算的计数,因此即便文件陈旧,脉搏也是可信的。

`fully_extracted_ailogs` 列表记录所有 `§Follow-ups` 和 `R<N>` 条目已被转移到注册表(或被显式分类为 superseded)的 AILOG。自 cli-3.21.0 起它是**信息性的**(由 `followups status` 显示);漂移检测按 follow-up 的内容哈希去重,而非依据此列表 —— 见下文"按 follow-up 的内容哈希去重"。

正式的 frontmatter schema 是 `.straymark/schemas/follow-ups-backlog.schema.v1.json`（实验性 v1 —— 见上方"状态"）。

### Buckets

五个 bucket 按触发类型组织条目 —— *何时可执行*:

- `ready` — 现在可执行,无外部触发器依赖。
- `time-triggered` — 基于日历的触发器(审计周期、周期性审查)。
- `charter-triggered` — 由触及相关领域的未来 Charter 阻塞。
- `phase-blocked` — 由尚不存在的未来组件或阶段阻塞。
- `operational` — 操作员手动决策或外部系统操作。

在参考 adopter 的 N=91 个条目处,该词汇表是稳定的 —— 不需要第六个 bucket。Severity（*跳过它有多痛*）有意**不是**一个 bucket:它是一个正交的 per-entry 字段(见下文)。

### 条目 schema (v1)

bucket 内的每个条目遵循以下形式（标注了 v1 字段;所有这些字段都是可选的 —— v0 条目仍然有效）:

```markdown
### FU-NNN — <简短描述>
- **Origin**: AILOG-NNNN-NN-NN-NNN <指向源部分的指针>
- **Source-hash**: <12 位十六进制>                                                     (cli-3.21.0+,自动管理 —— 漂移检测的去重键;请勿手动编辑)
- **Origin-class**: ex-ante-planning | testing | telemetry | staging | real-env-bug   (v1, 可选)
- **Status**: open | in-progress | suspected-closed | closed | superseded | promoted
- **Severity**: normal | blocking                                                     (v1, 可选;默认 normal)
- **Work verb**: design | implement | audit | operate                                 (可选;声明的工作分类，Baton #332)
- **Design provenance**: new | upstream                                               (可选;仅用于 implement —— upstream 降级为 operator)
- **Trigger**: ready | <日历日期> | when <X> | <其他>
- **Destination**: chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>
- **Cost**: <工作量估计>
- **Labels**: <自由标签,逗号分隔>                                                     (v1, 可选)
- **Notes**: <自由格式上下文>
- **Promoted to**: <TDE id,当 Status: promoted 时 — 见下方"提升为 TDE">
```

`FU-NNN` 在注册表整个生命周期内单调递增;条目关闭时不重新编号。

**v1 维度**,每个都将一个经验观察到的需求规范化(issue #214):

- **`Origin-class`** —— 条目的诞生地:规划工件（ex-ante,事前）vs 执行现实（testing、telemetry、staging、真实环境 bug）。使 ex-post 规划闭环可被查询。
- **`Severity`** —— `blocking` 标记必须在生产切换之前落地的可靠性类问题。将参考 adopter 在 `Notes` 字段中浮现的 `PROD-BLOCKER` 散文约定规范化（信号 3）。与 bucket 正交:一个 `charter-triggered` 条目也可以是 `blocking`。
- **`Labels`** —— 用于在 triage 期间将条目分组到已规划的 Charter / 迷你 Charter / 杂务中的自由标签。可通过 `straymark followups list --label <tag>` 查询。
- **`Destination` 词汇表** —— 形式化触发后工作落地的去向:`chore`、`mini-charter`、`charter-replanning`（该条目重塑一个已规划的 Charter,而不是向其添加一个任务）、`operations`、某个具体的 Charter id,或某个 TDE id。仍接受自由格式的值（宽松解析）。

### Status 词汇表

- `open` — 待处理,尚未采取行动。
- `in-progress` — 已声明或正在执行的 Charter 处理此条目。
- `suspected-closed` *(v1 新增)* —— 由 `drift --apply` 从其文本携带显式关闭标记（`closed in-Charter`、`fixed in batch N`、某个 commit 哈希，或诸如 `updated atomically in this PR` 的 born-resolved 习语 —— 见下方"规范关闭标记习语"）的 AILOG 中自动提取。操作员在下一次 triage 时确认（→ `closed`）或重新打开（→ `open`）。见下方"漂移检测"。
- `closed` — 条目已解决(Charter 已合并、操作任务已完成、时间已过且已审查)。
- `superseded` — 由其他工作处理,该工作未直接引用此条目。
- `promoted` — 条目因满足横向债务标准而被提升为 TDE 文档(见下方"提升为 TDE")。`Promoted to:` 字段携带 TDE id。

closed、superseded 和 promoted 条目保留在文件中(可审计的历史)。操作员可以将它们移到底部的 `## Bucket: closed` 部分以进行视觉整理,但绝不删除。

---

## 提升为 TDE

某些 FU 条目不仅仅是延期任务 —— 它们描述的是值得拥有自己治理文档的**横向技术债务**(TDE)。提升标准与 `AGENT-RULES.md §3` 中的 TDE-vs-`R<N>` 判定一致:

- 该条目是*先前 Charter 的遗留*(已经历 ≥1 次 Charter 关闭仍未修复)。
- 该条目*横跨多个模块或多个 Charter* —— 中央注册表已将其碎片化为共享同一根本原因的多个 bullet。
- 该条目*需要在当前 scope 包络之外的专用 Charter* 来修复。
- 该条目*需要人工决定优先级或分配*,操作员的周期性审查无法仅从 bullet 决定(impact × effort 矩阵、所有权)。

当上述任一条件成立时,将该 FU 条目提升为 `.straymark/06-evolution/technical-debt/` 下的 TDE 文档:

```bash
straymark followups promote FU-NNN
```

该命令将 v0 中需手动执行的三步流程自动化:

1. 创建 TDE 文档(与 `straymark new --type tde` 相同的机制),从 FU 条目预填 `impact`、`effort`、`type` 与正文上下文。
2. 在 TDE 的 frontmatter 中添加 `promoted_from_followup: FU-NNN` 以便溯源。
3. 在 FU 条目中,设置 `Status: promoted`、`Destination: TDE-YYYY-MM-DD-NNN`,以及 `Promoted to: TDE-YYYY-MM-DD-NNN`;并重新计算 frontmatter 计数器。

FU 条目在提升后**不会被删除** —— 它在注册表中的存在就是显示 TDE 来源的审计轨迹。

### 两种提升形态 —— 提升已存在的 vs 创建时即追溯提升

上述工作流涵盖**标准情况**:`open` 状态的 FU 条目已存在于注册表中,并在周期性审查期间被提升为 TDE。还有一种同样有效的情况,源自 Sentinel CHARTER-13 回顾的经验:

- **提升已存在条目** —— FU 数周或数个 Charter 之前已被(通常通过 `drift --apply`)登记为 `open`,经历过 ≥1 次 Charter 关闭仍未解决,并满足上述四项标准。标准流程。
- **创建时即追溯提升** —— 在回顾(Charter 关闭仪式、审计周期、RFC 撰写)*期间* 该债务被识别为值得作为 TDE,且从未作为 `open` FU 存在。先创建 TDE;在注册表中以 *`Status: promoted`* 状态新增一个 FU 条目,提供从 TDE 回溯到原始上下文(AILOG 中的某个 `R<N>`、calibrator 的 finding、被延期的分类)的审计轨迹。

两种形态在注册表中产生相同的终态:一个 `Status: promoted` 且具有 `Promoted to: TDE-YYYY-MM-DD-NNN` 指针的条目。区别在于条目是预先以 `open` 存在,还是天生即为 `promoted`。漂移检测一视同仁;统计 `total_promoted` 的分析在两种情况下得到相同数字。

存疑时,优先创建 FU 条目 —— 即便是追溯创建 —— 因为它会把 TDE 交叉引用回触发该识别的 AILOG / R-号 / 源上下文。一个 `promoted_from_followup: FU-NNN` 指向 backlog 中实际存在的条目的 TDE,比指向一个虚构的 FU 更易导航。

### 何时提升

- **周期性审查** —— 当操作员做人工重新分类时,提升任何已经历 ≥2 次 Charter 关闭仍未解决且符合上述标准的条目。
- **Charter 关闭** —— 在审查刚关闭的 Charter 所解决的条目时,如果发现*未*被解决且符合上述标准的条目,则提升它们,而不是保留为 `open`。
- **Charter 声明前** —— 如果你即将声明一个 Charter,并注意到注册表中包含此 Charter 仅会*部分*处理的条目,那么未处理的部分可能应作为 TDE,而不是作为另一个被延期的 FU。

---

## 漂移检测 —— 自 cli-3.19.0 起原生

漂移检测使注册表与新 AILOG 保持同步。自 cli-3.19.0 起,它是一个**原生 CLI 命令** —— 无需外部脚本:

```bash
straymark followups drift              # 扫描 git diff origin/main..HEAD（回退 HEAD~1..HEAD）中修改的 AILOG,并与工作树（git status --porcelain）取并集;drift 时退出 1
straymark followups drift --apply      # 相同扫描 + 将新条目提取到注册表
straymark followups drift --scan-all   # 对每个 AILOG 的周期性完整扫描
```

自 cli-3.21.0 起,默认扫描将已提交的 git 范围与工作树（`git status --porcelain`）取并集,因此未提交/未跟踪的 AILOG 对已记录的 pre-commit 流程可见 —— 你不再需要 `--scan-all` 来查看刚写入、尚未提交的 AILOG(issue #229)。

### `--apply` 做什么

1. 提取每个**内容哈希尚未在注册表中**的 `§Follow-ups` bullet 和 `R<N> (new, not in Charter)` 风险,在 `## Bucket: ready` 下以自动生成的 `FU-NNN` id 及存储的 `Source-hash` 追加。操作员在下一次 triage 时重新分类 bucket/trigger/destination。(已提取的 AILOG 会被重新扫描并按 follow-up 去重 —— 见下文"按 follow-up 的内容哈希去重"。)
2. **反噪声精化** *(v1 —— 解决 issue #214 信号 1)*:其 AILOG 文本携带显式关闭标记（`closed in-Charter`、`fixed in batch N`、某个 commit 哈希引用）的 bullet 会以 `Status: suspected-closed` 而非 `open` 提取,而不是作为 TBD 噪声污染 `ready` bucket。在参考 adopter 的两次有记录的发生中,每批自动追加的条目中有 20–75% 已在 Charter 内解决 —— 此精化消除了 v0 工作流唯一反复出现的成本。
3. 将 AILOG id 追加到 `fully_extracted_ailogs`。
4. **根据实际条目状态重新计算所有 `total_*` 计数器**(信号 2)。
5. 如果注册表为 `schema_version: v0`,则就地将其升级到 `v1` —— 非破坏性且幂等地(所有 v1 字段都是可选的;除版本标记和计数器外什么都不重写)。

自 cli-3.20.0 起,`--apply` **即使没有可提取的内容也会重新计算计数器** —— 因此提交前的 `drift --apply` 也能修复手动分诊会话留下的过期计数器（首个外部 adopter 反馈,issue #222 Finding 1）。

### 规范关闭标记习语

反噪声精化识别一个固定词汇表（不区分大小写）。AILOG 作者应在写入时收敛到这些表述,使 born-resolved 条目以 `suspected-closed` 而非 TBD 噪声落地:

| 习语族 | 示例 |
|---|---|
| In-Charter 关闭 | `closed in-Charter`、`closed in Charter`、`resolved in-Charter`、`resolved in Charter` |
| 批次修复 | `fixed in batch 3`（需要数字） |
| Commit 引用 | 反引号包裹的 commit 哈希:`` `ab12cd34ef` ``（7–40 个十六进制字符,至少一个数字） |
| Born-resolved *(cli-3.20.0+,#222 Finding 2)* | 关闭动词 —— `updated` / `corrected` / `remediated` / `resolved` / `fixed` / `closed` —— 后跟 `in this PR` 或 `in this commit`,例如 `Charter row updated atomically in this PR` |

词汇表之外的表述（如 `done earlier`、`no longer relevant`）会以 `open` 提取;操作员在分诊时翻转。当新的关闭习语在你的 AILOG 中反复出现时,请向上游提议,而不是手动编辑提取结果。

### 按 follow-up 的内容哈希去重

自 cli-3.21.0 起,漂移检测**按 follow-up 通过稳定的内容哈希去重**(`fu_content_hash`,由源 AILOG id + 来源部分 + 描述计算),作为每个条目的 `Source-hash` 存储。已提取的 AILOG 会被重新扫描,各个 follow-up 与注册表去重 —— 因此**追加到已提取 AILOG 的 follow-up**(多批次 Charter 的情形,即一个 AILOG 的 `§Follow-ups` 跨多个批次增长)会被发现,而不会被静默遗漏(issue #231)。

对 per-bullet 匹配最初的反对是改写导致的误报:经过整理的注册表条目会改写 AILOG 的 bullet,因此从*注册表*文本重新计算哈希会重新标记已提取的内容。存储的 `Source-hash` 解决了这一点 —— 它在提取时从 AILOG 的原始文本捕获,绝不从(后来被改写的)注册表标题重新计算。对每个携带哈希的条目,零误报的特性得以保留。

cli-3.21.0 之前创建的旧条目没有 `Source-hash`;对它们,漂移检测回退为从 `Origin` + `description` 重新计算哈希 —— 尽力而为,这是唯一残留的改写脆弱性向量,会随旧条目关闭而递减。`fully_extracted_ailogs` 予以保留(它记录哪些 AILOG 已被扫描,并由 `followups status` 显示),但**不再是跳过门**(skip gate)—— 去重按内容哈希进行,而非按整个 AILOG id。

### 旧版 bash 脚本（已弃用）

v0 参考实现（`scripts/check-followups-drift.sh`,Sentinel adopter repo 中约 296 行 POSIX bash）**自 cli-3.19.0 起已弃用**。它对 v0 注册表仍可工作,但不再维护,且缺少反噪声精化和计数器重新计算。迁移路径:删除该脚本,运行一次 `straymark followups drift --scan-all --apply`(这同时将注册表升级到 v1),并更新任何 pre-commit hook 改为调用 CLI。

**即使脚本报告"in sync",首次迁移后扫描也要带上 `--scan-all`**:bash 提取器对格式敏感（同时要求 `## Risk` 标题和精确的 `- **R<N> (new` bullet 形态）,对格式变体产生**静默假阴性** —— 以纯段落书写风险的 AILOG 根本不会被识别为含有 follow-up 内容。在参考 adopter 的迁移中（[issue #225](https://github.com/StrangeDaysTech/straymark/issues/225)）,原生宽容解析器捕获了脚本前一天还报告为"in sync"的 **8 个 AILOG / 29 个条目**。漂移检测上的静默假阴性正是该工具旨在防止的失败模式 —— 这也是脚本被弃用而非继续维护的原因。

---

## CLI 接口

```bash
straymark followups list                  # 枚举条目:FU id、status、severity、bucket、destination
straymark followups list --bucket ready --status open --severity blocking --label <tag>
straymark followups status                # 注册表脉搏:计数器(即时重新计算)、按 bucket/severity 细分
straymark followups status FU-NNN         # 单个条目的详情视图
straymark followups drift [--apply|--scan-all]   # 漂移检测(见上文)
straymark followups recount               # 手动分诊会话后重新计算 CLI 拥有的计数器(cli-3.20.0+)
straymark followups promote FU-NNN        # 自动化 FU → TDE 提升(见上文)
```

注册表也在 `straymark explore` TUI 中作为一个合成的 **Follow-ups** 分组出现(每个 bucket 一个子节点),并在 `straymark status` 中作为一个计数块出现。

---

## 代理集成

自 fw-4.21.0 起,代理指令**随 framework 一起发布**,位于 [`AGENT-RULES.md §13`](AGENT-RULES.md) —— adopter 不再将一个块复制到自己的 `CLAUDE.md` / `AGENT.md`。摘要如下:

- **会话开始**:扫视 `.straymark/follow-ups-backlog.md`(或运行 `straymark followups status`)以了解项目中所有待处理事项。
- **Pre-commit**:创建或修改了任何带有 `## Follow-ups` 或 `R<N> (new, not in Charter)` 条目的 AILOG 吗? → 在同一个 commit 中运行 `straymark followups drift --apply`。
- **Charter 关闭后**:审查 Charter 解决的条目;将其标记为 `closed`(在 `Notes` 中带有关闭 Charter id)或 `superseded`;确认或重新打开任何 `suspected-closed` 条目;然后运行 `straymark followups recount`,使 CLI 拥有的计数器与分诊在同一个 commit 中;通过 `straymark followups promote` 提升符合 TDE 标准的未解决条目。

这使代理成为注册表的主要维护者,CLI 成为验证层,操作员成为周期性审查者(重新分类、确认 suspected-closed、修剪 superseded、在符合标准时提升为 TDE)。

---

## 采用流程

对于从零开始的 adopter:

1. 将 `.straymark/templates/follow-ups-backlog.md` 复制到 `.straymark/follow-ups-backlog.md`(空的 `fully_extracted_ailogs:` 列表、五个 `## Bucket:` 标题)。
2. 运行 `straymark followups drift --scan-all --apply` 从现有 AILOG 播种注册表。
3. 手动将自动生成的 `## Bucket: ready` 条目重新分类到正确的 bucket;在能增加信号之处填写 `Origin-class`/`Severity`/`Labels`。这是一次性 triage,对于约 50 个条目的 backlog 通常需要 30-60 分钟。
4. 完成 —— `AGENT-RULES.md §13` 中的代理指令已经生效;无需编辑 `CLAUDE.md`。

对于从 v0 约定迁移的 adopter:运行一次 `straymark followups drift --apply`(自动将注册表升级到 v1),删除本地 bash 脚本,并更新任何 pre-commit hook 改为调用 CLI。

---

## 参考实现

`StrangeDaysTech/sentinel` —— 起源 adopter:

- v0 模式:CHARTER-12,于 2026-05-06 合并（[sentinel#53](https://github.com/StrangeDaysTech/sentinel/pull/53)、[sentinel#54](https://github.com/StrangeDaysTech/sentinel/pull/54)）。从 CHARTER-08 → CHARTER-11 回顾播种了 47 个条目。
- 规模验证:Etapa-3 后 triage 处于 **N=91 个 FU / 提取了 76 个 AILOG / 65 个 open**（[issue #214](https://github.com/StrangeDaysTech/straymark/issues/214)）—— 驱动 v1 schema 和原生 CLI 的经验输入（ADR-2026-06-03-001）。

---

## 未决问题

在 v1 中已解决:

- ~~**Schema 验证。**~~ → `.straymark/schemas/follow-ups-backlog.schema.v1.json`（frontmatter）、CLI 解析器中的条目形式验证。
- ~~**作为 `straymark followups` CLI 的结晶化。**~~ → 自 cli-3.19.0 起原生 `list / status / drift / promote`。
- ~~**Bucket 分类启发式**（部分）。~~ → `suspected-closed` 移除了主导的噪声类;完整的 bucket 建议（使用 AILOG `tags` / Charter `effort_estimate`）仍未解决。

仍为未来修订保留:

- **与审计周期的集成**。当 `straymark charter audit --merge-reports` 产生未在关闭前原子修复的真实 debt findings 时,这些 findings 仅存在于 `.straymark/audits/<id>/review.md` 中。它们不会自动流入中央注册表。自动浮现它们将关闭一个已知差距。
- **`closed` 与 `superseded` 语义**。今天的差异在于解决工作是否显式引用了该条目。可能会出现更严格的约定。
- **与 `charter close` 的软集成**（issue #135 Tier 3）:在 Charter 关闭后自动调用 `followups drift --apply`,并带有交互式提升提示。以来自第二个 adopter 的摩擦信号为门槛。
- **硬 schema 稳定化 (v1.0)**。以另一个领域的第二个 adopter 的验证为门槛,依据设计原则 #12。

---

## 致谢

通过 [issue #111](https://github.com/StrangeDaysTech/straymark/issues/111) 由 Sentinel adopter 贡献;通过 [issue #214](https://github.com/StrangeDaysTech/straymark/issues/214) 和 ADR-2026-06-03-001 成熟为一等公民。经验基础:`StrangeDaysTech/sentinel` 中的 CHARTER-08 → CHARTER-11 链以及 Etapa-3 后 N=91 的 triage。作者:José Villaseñor Montfort。

*本文档在生成式 AI 工具（Claude 4.7 / Opus 4.8）的协助下撰写；内容的全部责任由人类作者承担。*

---

## 相关

- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) —— 此漂移检测约定在每 AILOG ↔ 注册表表面实例化的元模式。
- [CHARTER-CHAIN-EVOLUTION.md](CHARTER-CHAIN-EVOLUTION.md) —— 在链级别（Pattern 1）和周期级别（Pattern 2）运作的姐妹模式。
- [AGENT-RULES.md §3](AGENT-RULES.md) —— 可将 follow-ups 提升为专用债务条目的 TDE-vs-`R<N>` 升级标准;§13 —— 随框架发布的注册表维护代理指令。
- `STRAYMARK.md §16` —— 注册表作为一等公民工件的入门级摘要。

---

*StrayMark fw-4.34.0 | [Strange Days Tech](https://strangedays.tech)*
