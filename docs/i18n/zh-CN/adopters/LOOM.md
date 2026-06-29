# StrayMark — Loom 与架构地图（实验性）

**看见你的项目：文档之网，以及标示工作正在何处发生的"你在这里"地图。**

> ⚠️ **EXPERIMENTAL — Loom v0 (N=1)。** Loom 是一项可选启用、不稳定的实验。在它毕业之前，它的 API、CLI 表面、磁盘上的模型格式乃至其本身都可能在没有弃用周期的情况下变更或被移除。它**不**属于受支持的 Framework 或 CLI 契约——暂时不要基于它构建自动化。`architecture` 模型与 `status --where` 随 CLI 发布，但带有同样的实验性注意事项。

---

## Loom 是什么

Loom 是 StrayMark 的第三个组件（与 Framework 和 CLI 并列）：一个仅回环、只读的**开发仪表盘**，让 StrayMark 项目对人眼变得可读。它对同一个项目提供两个视图：

1. **知识图谱（Knowledge Graph）** — 你的 StrayMark *文档*之网，根据其 frontmatter 链接（`related`、`originating_ailogs`……）渲染为一张实时的力导向图。选中一个节点，它的整条关系链路就会点亮。这个视图不言自明，无需任何设置。
2. **架构地图（Architecture Map）** — *系统*的实现地图：组件与层（你真实的架构），叠加一层从治理状态（活跃的 Charter、drift、已关闭的工作、未结的技术债务）计算得出的实时**"你在这里"状态覆盖层**。它以可视化方式回答每日的*"我们在哪里？"*。**这个视图由你编写并精修的模型驱动**——本指南的其余部分讲的就是它。

Loom 从不写入你的项目——它只*可视化* CLI 所计算的内容。CLI 命令（`validate`、`audit`、`charter drift`）始终是事实来源与关卡。

---

## 心智模型：一个模型，多个视图（BIM）

架构地图借用了 **BIM"一个模型，多个视图"**的理念。只有**一个**模型，表达为 `.straymark/architecture/` 下两个相互关联的文件：

| 文件 | 它持有什么 | 由谁拥有 |
|---|---|---|
| `model.yml` | **语义** — 组件 → 文件 glob、层、链接 | 你（由 CLI 播种） |
| `plan.drawio` | **布局** — 方框与箭头组成的图（DrawIO/mxGraph） | 你（由 CLI 播种） |

从这一个模型出发，StrayMark 渲染**三种投影**，全部来自*同一份*状态计算，因此它们不可能彼此矛盾：

- **文本** — `straymark status --where`（终端中的"你在这里"）。
- **2D 平面图** — Loom 的 Architecture 标签页（你的 `plan.drawio`，叠加实时状态）。
- **3D 轴测图** — Loom 的 `2D | 3D` 切换（一个爆炸式的 BIM 风格视图）。

**状态覆盖层是实时计算的，从不手工维护。** 你编写*结构*（哪些文件属于哪个组件、组件之间如何关联）；每次你查看时，StrayMark 都会依据治理信号为其上色（`active` / `in-progress` / `implemented` / `has-debt` / `uncharted`）。

---

## 推荐工作流

```
generate  →  refine (human or AI)  →  validate  →  sync (as code grows)  →  loom serve
   draft        the real model         CI gate       append-only            visualize
```

### 1. Generate — 生成初稿种子

```bash
straymark architecture generate
```

挖掘你代码库的结构（每个源目录对应一个组件），并用你的 ADR（C4 图 + "Affected Components" 表）加以丰富以改善标签并添加链接。它会写出 `model.yml` + `plan.drawio`，其中每个组件都处于占位的 `unassigned` 层，而 `.straymark` 的 00–09 阶段则作为占位层。

种子是**语言与结构感知的**（它会下钻进入 `internal/`、`src/`、`pkg/`……并跳过 Maven/Gradle 脚手架，这样一个 Java 模块就不会变成单独一个 `main` 方框）。对于非默认技术栈，你可以在 `.straymark/config.yml` 中用一个 `architecture:` 小节来扩展它——参见 [CLI-REFERENCE](./CLI-REFERENCE.md#straymark-architecture-generatesyncvalidate-cli-3250-experimental)。

**种子是草稿，而非答案。** 它捕获了形态；你在下一步让它变得有意义。

### 2. Refine — 真正重要的环节（人工*或* AI）

正是在这里，一张生成的地图变成一张真实的地图。`model.yml` 是纯 YAML，`plan.drawio` 是标准的 DrawIO 文件，因此精修可以**由人、由 AI Agent，或两者共同**完成：

- 把占位阶段**层重命名**为你真实的架构（例如 `entrypoints`、`domain`、`persistence`、`web`）。
- 把组件从 `unassigned` **重新分配**到这些层中。
- 把标签**修正**为人类可读的名字（`internal-modules-commshub` → "CommsHub"）。
- **收紧 glob**，使每个组件恰好拥有它自己的文件。
- 在组件之间**添加 `links`**，以捕获真实的依赖关系。
- 在 DrawIO 中**排布图形**（打开 `plan.drawio`），使 2D 平面图易读。

> **AI 辅助精修。** 由于模型只是 YAML + DrawIO XML，AI 编码 Agent 可以直接完成这件事：把它指向 `model.yml` 和你的代码库，让它分配层、修正标签并添加依赖链接。它操作的是同一套 CLI（`generate` / `validate` / `sync`），编辑的是你本会编辑的同一批文件。精修就是一次普通的审查与编辑循环——对的保留，错的纠正。

### 3. Validate — 捕捉模型↔平面图的偏差

```bash
straymark architecture validate            # text; exits 1 on any signal (CI-gateable)
straymark architecture validate --output json
```

报告完整性信号：**undrawn**（某个组件没有对应的图元）、**unmodeled**（某个图元在模型中缺失）、**empty**（匹配不到任何文件的 glob）。错误现在是可读的（它们会指明确切原因——例如某个组件 id 与某个层 id 冲突），因此一个格式错误的模型会告诉你该修什么。

### 4. Sync — 随代码增长持续跟进（仅追加）

```bash
straymark architecture sync            # dry-run: shows what's new
straymark architecture sync --apply    # appends new dirs / ADR components
```

检测尚未纳入模型的新源目录和 ADR 组件，并将它们**追加**进去——它**绝不**覆盖你的编辑或你的 DrawIO 几何布局。当你新增一个模块时运行它；用同样的方式精修新条目。

### 5. Serve — 实时查看

```bash
straymark loom serve            # downloads the loom-* binary on first use, opens the browser
```

在 `localhost` 上打开两个视图。Architecture 标签页显示你精修过的平面图及实时覆盖层；`2D | 3D` 切换给出轴测视图。对 `.straymark/`（或模型）的编辑会在一秒之内更新已打开的浏览器。还有一个无需服务器、仅在终端的答案：

```bash
straymark status --where        # the textual "you are here"
```

---

## 诚实的局限（这是种子，而非权威输出）

- **生成的种子是一个起点。** 它不会推断出你意图中的层或组件边界——按设计，那是精修环节的事。
- **覆盖度取决于你的技术栈。** 扫描默认识别约 25 种语言，并处理常见布局（Go 的 `internal/`、JS 的 `src/`、多模块 Maven/Gradle）。非默认语言或不寻常的布局需要一个 `architecture:` 配置项以及更多手工精修——但种子从不*误导*；它只是给的起步优势更少。
- **`has-debt` 覆盖层的精度取决于你的债务记录。** 默认情况下，债务归属于一个未结 TDE 所引用的 AILOG 实际修改过的组件；粗粒度的 `files_modified` 列表会把它摊得比债务本身更宽。若要精确限定，可在 TDE 上声明 **`affects`** 字段（文件 glob，例如 `affects: [internal/modules/audittrail/**]`）—— 存在时，债务只归属于这些路径，忽略 AILOG 足迹。
- **Loom 是 N=1。** 它已通过在本项目和 Sentinel 参考上的自用（dogfooding）得到验证。请预期会有粗糙之处；发现了请反馈。

---

## 另见

- [CLI-REFERENCE](./CLI-REFERENCE.md) — `straymark architecture <generate|sync|validate>`、`status --where` 与 `loom serve` 的标志，以及 `architecture:` 配置小节。
- [WORKFLOWS](./WORKFLOWS.md) — 架构地图如何融入 StrayMark 的日常节奏。
- [ADOPTION-GUIDE](./ADOPTION-GUIDE.md) — 首先如何把 StrayMark 引入一个项目。
