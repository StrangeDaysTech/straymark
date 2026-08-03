# StrayMark — Baton: declared work routing (Experimental)

**Tell StrayMark what *kind* of work a change is — and get honest, cost-aware routing advice back.**

> ⚠️ **EXPERIMENTAL — Baton 0.x (Track C: forward-validation with adopters).** Baton is an opt-in, unstable experiment. Its CLI surface, report format, and very existence may change or be removed without a deprecation cycle until it graduates. It is **not** part of the supported Framework or CLI contract — don't build automation against it yet. It is **read-only**: Baton never writes into your project.

---

## What Baton is

Baton is StrayMark's experimental **Coherence Bridge**: it reconciles intent (charters, specs) against governance and code, and classifies work units into cost-aware routing tiers:

- **Frontier** — genuine design / unknown territory: route to the strongest model.
- **Economic** — known work with clear contracts: route to a mid-tier model.
- **Local/operator** — mechanical or upstream-defined work: cheap or local models suffice.

Since decision **#332**, classification is **declaration-based**: the author declares the work class in frontmatter, and Baton never guesses from titles or text. An undeclared unit is an *honest state* — Baton routes it to the frontier tier and nudges you to declare, rather than fabricating a classification.

## What Track C asks of you

Forward-validation (gate #3 for graduating Baton) needs real governance traffic. Your part is small:

1. **Declare the work class** on new units, exactly as documented in the [CLI Reference](./CLI-REFERENCE.md#straymark-validate):
   - Charter frontmatter: `work_verb: design | implement | audit | operate` and, only when significant for `implement`, `design_provenance: new | upstream`.
   - Follow-up backlog entries: the same optional fields.
2. **Work as usual.** Nothing else in your cadence changes; the fields are advisory and absence is silent.
3. **After 2–4 weeks**, run the simplified calibration protocol from the [adopter kit](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md) and report back (see [Adopter Feedback](./ADOPTER-FEEDBACK.md)).

The full kit — vocabulary, determination rules, calibration protocol, friction questions — lives in [`experiment-baton/07-track-c-adopter-kit.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md).

## Getting the binary

Baton ships as **GitHub-release-only** assets (no crates.io), like Loom:

1. Open the [`baton-*` release](https://github.com/StrangeDaysTech/straymark/releases) — only the latest is kept.
2. Download the asset for your platform:

   | Platform | Asset |
   |---|---|
   | Linux x86_64 | `straymark-baton-v{version}-x86_64-unknown-linux-gnu.tar.gz` |
   | macOS Intel | `straymark-baton-v{version}-x86_64-apple-darwin.tar.gz` |
   | macOS Apple Silicon | `straymark-baton-v{version}-aarch64-apple-darwin.tar.gz` |
   | Windows x86_64 | `straymark-baton-v{version}-x86_64-pc-windows-msvc.zip` |

3. Extract and put `straymark-baton` on your `PATH`.

Alternative: compile from the repository — `cargo build --release --manifest-path experiment-baton/Cargo.toml`.

Quick sanity check (read-only, mutates nothing):

```bash
straymark-baton --version
straymark-baton classify .          # declared classes of your recorded work units
straymark-baton route . --dry-run   # tier routing advice; never executes anything
```

## Read-only guarantees

- `classify` and `route` only **read** your `.straymark/` tree; `route` requires `--dry-run` and there is no execution path.
- No network calls to model providers — Baton classifies from declarations, it doesn't run models.
- Nothing in your governance documents or code is modified. The CLI (`validate`, `status`) remains the gate.

## Honest limitations

- **Baton is N=1+ (Sentinel dogfood).** Expect rough edges; report them via the Adopter feedback channel.
- Routing advice is **advice**: it never blocks, mutates, or decides for you.
- While in Track C, releases replace each other (only the latest `baton-*` survives) — re-download when a new release appears.

---

## See also

- [Adopter kit — Track C](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md) — declaration placement, determination rules, simplified E1 calibration.
- [Adopter Feedback](./ADOPTER-FEEDBACK.md) — how findings and telemetry flow upstream.
- [CLI Reference](./CLI-REFERENCE.md) — the advisory `work_verb` vocabulary checks in `straymark validate`.

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
