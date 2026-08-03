# StrayMark — Baton：声明式工作路由（实验性）

**告诉 StrayMark 一次变更是*哪一类*工作——获得诚实、成本敏感的路由建议。**

> ⚠️ **EXPERIMENTAL — Baton 0.x（Track C：与采纳者进行前瞻性验证）。** Baton 是一个自选参与（opt-in）、不稳定的实验。在其毕业之前，它的 CLI 接口、报告格式乃至存在本身都可能随时变更或移除，且不提供弃用周期。它**不**属于受支持的 Framework 或 CLI 契约——请勿基于它构建自动化。它是**只读**的：Baton 从不写入你的项目。

---

## Baton 是什么

Baton 是 StrayMark 的实验性 **Coherence Bridge**：它对照治理与代码来校验意图（charter、spec），并将工作单元分类为成本敏感的路由层级：

- **Frontier** — 真正的设计 / 未知领域：路由到最强的模型。
- **Economic** — 契约明确的已知工作：路由到中档模型。
- **Local/operator** — 机械性或上游定义好的工作：廉价或本地模型即可胜任。

自 **#332 决策** 起，分类**基于声明**：作者在 frontmatter 中声明工作类别，Baton 绝不从标题或正文猜测。未声明的单元是一种*诚实状态* — Baton 将其路由到 frontier 层级并提醒你补声明，而不是编造一个分类。

## Track C 需要你做什么

前瞻性验证（Baton 毕业的第 #3 道关卡）需要真实的治理流量。你的部分很轻：

1. **在新单元上声明工作类别**，与 [CLI 参考](./CLI-REFERENCE.md#straymark-validate) 中的文档完全一致：
   - Charter frontmatter：`work_verb: design | implement | audit | operate`，以及仅当对 `implement` 有意义时的 `design_provenance: new | upstream`。
   - follow-up backlog 条目：相同的可选字段。
2. **照常工作。** 你的节奏没有任何其他变化；这些字段只是建议性的，缺省时完全静默。
3. **2–4 周后**，按[采纳者工具包](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md)中的简化校准协议执行并反馈（见 [Adopter Feedback](./ADOPTER-FEEDBACK.md)）。

完整工具包 — 词汇表、判定规则、校准协议、摩擦问题 — 位于 [`experiment-baton/07-track-c-adopter-kit.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md)。

## 如何获取二进制

与 Loom 一样，Baton **仅以 GitHub Release 资产**分发（不发布到 crates.io）：

1. 打开 [`baton-*` release](https://github.com/StrangeDaysTech/straymark/releases) — 仅保留最新一个。
2. 下载你所在平台的资产：

   | 平台 | 资产 |
   |---|---|
   | Linux x86_64 | `straymark-baton-v{version}-x86_64-unknown-linux-gnu.tar.gz` |
   | macOS Intel | `straymark-baton-v{version}-x86_64-apple-darwin.tar.gz` |
   | macOS Apple Silicon | `straymark-baton-v{version}-aarch64-apple-darwin.tar.gz` |
   | Windows x86_64 | `straymark-baton-v{version}-x86_64-pc-windows-msvc.zip` |

3. 解压并将 `straymark-baton` 放入 `PATH`。

备选：从仓库编译 — `cargo build --release --manifest-path experiment-baton/Cargo.toml`。

快速自检（只读，不改动任何内容）：

```bash
straymark-baton --version
straymark-baton classify .          # 你已记录工作单元的声明类别
straymark-baton route . --dry-run   # 层级路由建议；绝不执行任何操作
```

## 只读保证

- `classify` 与 `route` 只**读取**你的 `.straymark/` 树；`route` 必须带 `--dry-run`，且不存在执行路径。
- 不会向模型供应商发起网络调用 — Baton 基于声明进行分类，不运行模型。
- 你的治理文档与代码均不会被修改。CLI（`validate`、`status`）仍是唯一的门槛。

## 诚实的局限

- **Baton 目前是 N=1+（Sentinel 自用验证）。** 请预期粗糙之处，并通过 Adopter Feedback 渠道上报。
- 路由建议只是**建议**：它从不阻塞、修改或替你决策。
- Track C 期间，release 会相互替换（仅最新的 `baton-*` 保留）— 出现新 release 时请重新下载。

---

## 另请参阅

- [采纳者工具包 — Track C](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md) — 声明位置、判定规则、简化版 E1 校准。
- [Adopter Feedback](./ADOPTER-FEEDBACK.md) — 发现与遥测如何上报。
- [CLI 参考](./CLI-REFERENCE.md) — `straymark validate` 中建议性的 `work_verb` 词汇检查。

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
