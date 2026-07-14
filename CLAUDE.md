# StrayMark — Development Instructions

This is the StrayMark project repository. It contains two main components, plus one experimental:

- **Framework** (`dist/`): documentation templates, governance policies, and agent directives
- **CLI** (`cli/`): the `straymark` Rust binary that manages the framework in user projects
- **Loom** (`experiment-loom/`, EXPERIMENTAL): knowledge-graph/architecture visualization server — see `experiment-loom/README.md` and `experiment-loom/CHARTER-01-loom-server.md`

The Rust code is a **Cargo workspace** (root `Cargo.toml`, members `core` + `cli`): `straymark-core` holds the shared document model and knowledge graph (one parser for the CLI and Loom); `Cargo.lock` and `[profile.release]` live at the workspace root; build artifacts land in the root `target/`.

## Framework path discipline (self-adoption safeguard S4)

StrayMark is preparing to adopt StrayMark for its own development (decision: [`docs/decisions/AIDEC-2026-07-13-001-straymark-self-adoption.md`](docs/decisions/AIDEC-2026-07-13-001-straymark-self-adoption.md)). Two `.straymark/`-shaped things exist in this repo — **never confuse them**:

- **`dist/.straymark/` = the framework DISTRIBUTION SOURCE — product under edit.** This is the shipped template. Edit it as *product* (it's what adopters receive). **Never read it as governance-in-force for this repo, and never write generated artifacts (AILOGs/AIDECs/telemetry) into it.** Never run a mutating `straymark` command with cwd or `--path` inside `dist/` — the CLI now refuses (safeguard S1), but the intent is yours to hold.
- **`/.straymark/` (repo root) = the installed framework — governance IN FORCE** (once self-adopted; it does not exist yet). It will be **pinned to the last release** ("yesterday's tail"), so it may lag `dist/`. When it exists, *that* is the governance you follow and where your artifacts go; `straymark status` shows the skew (safeguard S3).

Rule of thumb: **`dist/` is what we ship; `/.straymark/` is what governs us.** They are one framework version apart on purpose.

## Project Structure

```
straymark/
├── Cargo.toml              # Workspace root (members: core, cli) + release profile
├── Cargo.lock
├── core/                   # straymark-core: shared document model + typed knowledge graph
│   └── src/
│       ├── document.rs     # DocType, Frontmatter, parse_document, discover_documents
│       └── graph.rs        # Typed, bidirectional, orphan-preserving graph builder
├── cli/                    # Rust CLI source code
│   ├── src/
│   │   ├── main.rs         # Entry point, command routing
│   │   ├── commands/       # Subcommands: init, update, remove, status, repair, validate, new, compliance, metrics, analyze, audit, explore, about
│   │   ├── tui/            # Terminal UI for `explore` (ratatui + crossterm)
│   │   ├── analysis_engine.rs # Code complexity analysis (arborist-metrics)
│   │   ├── config.rs       # StrayMarkConfig, Checksums, ComplexityConfig
│   │   ├── download.rs     # GitHub API, ZIP downloads
│   │   ├── inject.rs       # Directive injection system
│   │   ├── manifest.rs     # dist-manifest.yml parser
│   │   ├── platform.rs     # OS/arch detection
│   │   ├── self_update.rs  # CLI auto-update
│   │   └── utils.rs        # Output helpers, file hashing
│   ├── tests/              # Integration tests
│   └── Cargo.toml
├── experiment-loom/            # Loom (EXPERIMENTAL): straymark-loom server crate
│   ├── src/                # axum server + notify watcher
│   ├── web/                # Sigma.js + graphology frontend (Vite/TS, built in CI only)
│   ├── specs/              # SpecKit sets (001 knowledge graph, 002 architecture plan)
│   └── CHARTER-01-loom-server.md
├── dist/                   # Framework distribution files
│   ├── .straymark/          # Templates, governance, config
│   ├── STRAYMARK.md         # Unified governance rules
│   └── dist-manifest.yml   # What gets installed
├── docs/                   # Project documentation (EN + ES + zh-CN)
├── .github/workflows/      # CI/CD
│   ├── release-cli.yml     # Build + release CLI binaries (publishes straymark-core first)
│   ├── release-loom.yml    # Build + release Loom binaries (loom-* tags; npm build step; no crates.io)
│   └── release-framework.yml
└── README.md
```

## Versioning

StrayMark uses **independent versions** for framework and CLI:

| Component | Tag format | Current | Example |
|-----------|-----------|---------|---------|
| Framework | `fw-X.Y.Z` | fw-4.2.0 | `fw-4.2.0` |
| CLI | `cli-X.Y.Z` | cli-3.2.2 | `cli-3.2.2` |

Follow [semver](https://semver.org/):
- **Major**: breaking changes
- **Minor**: new features (e.g., new command)
- **Patch**: bug fixes, small improvements

## Release Workflow — CLI

### Step 1: Bump version

Edit `cli/Cargo.toml`:
```toml
version = "X.Y.Z"
```

Run `cargo check` at the repo root to update the workspace `Cargo.lock`.

Update version references in all docs that mention version numbers:
- `docs/adopters/CLI-REFERENCE.md` (EN — versioning table + example outputs)
- `docs/i18n/es/adopters/CLI-REFERENCE.md` (ES — same)
- `docs/i18n/zh-CN/adopters/CLI-REFERENCE.md` (zh-CN — same)
- `README.md` (versioning table)
- `docs/i18n/es/README.md` (ES — versioning table)
- `docs/i18n/zh-CN/README.md` (zh-CN — versioning table)

Update `CHANGELOG.md` (root) following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format:
- Add a new `## CLI X.Y.Z` section at the top (below the header)
- Use subsections: `### Added (CLI)`, `### Changed (CLI)`, `### Fixed (CLI)`, `### Removed` as applicable
- Describe user-facing changes concisely

### Step 2: Commit and merge

```bash
git checkout -b chore/bump-cli-X.Y.Z
git add cli/Cargo.toml Cargo.lock docs/
git commit -m "chore: bump CLI version to X.Y.Z"
# Push, create PR, merge to main
```

### Step 3: Create and push tag

```bash
git tag cli-X.Y.Z
git push origin cli-X.Y.Z
```

The `release-cli.yml` workflow triggers automatically:

1. Verifies `Cargo.toml` version matches the tag
2. Compiles for 4 platforms in parallel:
   - `x86_64-unknown-linux-gnu` (Ubuntu)
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS ARM)
   - `x86_64-pc-windows-msvc` (Windows)
3. Packages each as `.tar.gz` (Unix) or `.zip` (Windows)
4. Creates the GitHub release and uploads all binaries

**If CI needs re-running**, trigger manually:

```bash
gh workflow run release-cli.yml -f tag=cli-X.Y.Z
```

### Step 4: Verify

```bash
gh release view cli-X.Y.Z --json assets --jq '.assets[].name'
# Should show 4 binaries
```

Users can now run `straymark update-cli` to get the new version.

## Release Workflow — Framework

Framework releases are automated via `release-framework.yml`. The workflow triggers on tag push (`fw-*`), packages `dist/` as a ZIP, and creates the GitHub release with the asset.

### Step 1: Bump version

Edit `dist/dist-manifest.yml`:
```yaml
version: "X.Y.Z"
```

Update version references in docs:
- `docs/adopters/CLI-REFERENCE.md` (EN — versioning table)
- `docs/i18n/es/adopters/CLI-REFERENCE.md` (ES — versioning table)
- `docs/i18n/zh-CN/adopters/CLI-REFERENCE.md` (zh-CN — versioning table)
- `README.md`, `docs/i18n/es/README.md`, and `docs/i18n/zh-CN/README.md` (versioning tables)
- `dist/.straymark/00-governance/QUICK-REFERENCE.md` (EN + ES + zh-CN footer)
- `dist/.straymark/00-governance/AGENT-RULES.md` (EN + ES + zh-CN footer)
- `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` (EN + ES + zh-CN footer)
- `dist/.straymark/00-governance/C4-DIAGRAM-GUIDE.md` (EN + ES + zh-CN footer)

Update `CHANGELOG.md` (root) following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format:
- Add a new `## Framework X.Y.Z` section at the top (below the header)
- Use subsections: `### Added (Framework)`, `### Changed (Framework)`, `### Fixed`, `### Removed` as applicable
- If both CLI and Framework are bumped together, combine into `## Framework X.Y.Z / CLI X.Y.Z`

### Step 2: Commit and merge

```bash
git checkout -b chore/bump-fw-X.Y.Z
git add dist/ docs/ README.md
git commit -m "chore: bump Framework version to X.Y.Z"
# Push, create PR, merge to main
```

### Step 3: Create and push tag

```bash
git tag fw-X.Y.Z
git push origin fw-X.Y.Z
```

The `release-framework.yml` workflow triggers automatically:
1. Verifies `dist-manifest.yml` version matches the tag
2. Packages `dist/` contents into `straymark-fw-X.Y.Z.zip`
3. Creates the GitHub release with the ZIP as asset

**If CI needs re-running**, trigger manually:

```bash
gh workflow run release-framework.yml -f tag=fw-X.Y.Z
```

### Step 4: Verify

```bash
gh release view fw-X.Y.Z --json assets --jq '.assets[].name'
# Should show: straymark-fw-X.Y.Z.zip
```

Users can now run `straymark update-framework` to get the new version.

## Release Workflow — Loom (EXPERIMENTAL)

Loom releases are GitHub-release-only (no crates.io while experimental). Tag format: `loom-X.Y.Z`.

1. Bump `version` in `experiment-loom/Cargo.toml` and update `experiment-loom/CHANGELOG.md`.
2. Commit via PR (same rules as any change).
3. `git tag loom-X.Y.Z && git push origin loom-X.Y.Z` — `release-loom.yml` verifies the version matches the tag, builds the frontend (`npm ci && npm run build` in `experiment-loom/web/`, embedded via rust-embed), compiles the same 4-platform matrix as the CLI, and uploads `straymark-loom-vX.Y.Z-<target>.{tar.gz,zip}` assets. The release is marked `--latest=false` so CLI update flows are unaffected.
4. Verify: `gh release view loom-X.Y.Z --json assets --jq '.assets[].name'` (4 binaries). `straymark loom serve` picks the new version automatically (version marker in `~/.straymark/bin/`).

## Git Workflow

### Rules

- **Never commit directly to `main`.** All changes go through feature/fix/chore branches and pull requests (mirrors the rule shipped to adopters in `dist/STRAYMARK.md §5`).
- **Branch prefixes**: `feat/`, `fix/`, `chore/`, `docs/`, `refactor/`, `test/`.
- **Conventional commit subjects**: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:` (release bumps use `chore: bump fw-X.Y.Z` or `chore: bump cli-X.Y.Z`).
- **Squash merge is the project default.** Visible in `git log`: each merged PR yields one squash commit on `main` (e.g. `2ae3802 feat(framework): fw-4.13.0 — TDE activation trigger (closes #128) (#129)`).

### Stacked PRs — avoid, or use merge commits

**Lesson from the #129 / #131 / #133 incident (2026-05-11)**: when you stack PR B on top of PR A's branch (base of B is A's head, not `main`), and A is **squash-merged** to `main`, B's content gets stranded:

- The squash merge of A creates a new commit on `main` with content equivalent to A but a **different SHA** than A's original commits.
- B's branch still points at A's *original* commits, which now have no descendant on `main`.
- When B is merged, GitHub merges it into A's branch (its declared base), not `main`. The "merge to main" never happens — even though GitHub UI shows B as `MERGED`.
- A direct PR from A's branch → `main` afterwards will surface as **CONFLICTING** because git sees the same lines touched in different ways (A's original commits vs A's squash on main).

**Prevention** (pick one):

1. **Sequential, not stacked.** Wait for PR A to merge to `main`, then rebase B onto `main` and open B as a standalone PR with `base = main`. Slower but bulletproof.
2. **If you must stack, use merge commits (not squash) for the parent.** A merge commit preserves shared history, so subsequent merges of stacked PRs into `main` resolve cleanly. Pay the cost of a noisier `git log` for the stacked-PR safety.

**Recovery if you find yourself in this state** (B stranded on A's branch, B's content not in `main`):

1. `git checkout -b chore/sync-<B-content>-to-main main`
2. `git cherry-pick <B's merge commit SHA>` — should be clean because B touches files outside A's conflict zone (which is why it could be stacked in the first place).
3. Push, open PR with `base = main, head = chore/sync-...`. Cherry-pick produces a fresh commit on top of `main`, so no conflicts.
4. The branch protection may require `--admin` merge if the content was already reviewed in B (the original PR) — sync PRs are purely procedural.

### Authority to merge

- The user owns the repo. `gh pr merge --admin` is acceptable for **procedural sync PRs** where the content was already reviewed in a separate PR (e.g. recovering from a stacked-PR mishap as above).
- For substantive PRs, do not bypass review — wait for the user to merge via UI or explicit instruction.

### Tagging discipline

- Tags are created on `main` HEAD after the relevant PR is merged.
- Before pushing a tag, verify the in-file version matches: `grep '^version' dist/dist-manifest.yml` for `fw-*` tags; `grep '^version' cli/Cargo.toml` for `cli-*` tags. CI's `release-cli.yml` and `release-framework.yml` both refuse mismatches.
- Both tags can be pushed in a single command when releasing framework + CLI together: `git push origin fw-X.Y.Z cli-X.Y.Z`.

## CLI Commands Reference

| Command | Description |
|---------|-------------|
| `straymark init [path]` | Initialize StrayMark in a project |
| `straymark update` | Update both framework and CLI |
| `straymark update-framework` | Update only the framework |
| `straymark update-cli` | Update the CLI binary |
| `straymark remove [--full]` | Remove StrayMark from project |
| `straymark status [path]` | Show installation health and doc stats |
| `straymark status --where [path] [--out DIR]` | EXPERIMENTAL (Loom A1.4) — textual "you are here": load `architecture/model.yml`, project per-layer/per-component state (active/in-progress/implemented/has-debt/uncharted) from governance signals (charters + drift + open TDEs + on-disk inventory) via the pure `core::architecture::project`, highlight active components, and print the §8 "Where are we" summary. Degrades to a hint when no model exists. `--out` overrides the default `.straymark/architecture/` (lets you dogfood a non-installed repo) |
| `straymark repair [path]` | Restore missing directories and framework files |
| `straymark validate [path] [--staged]` | Validate documents for compliance and correctness |
| `straymark new [path] [-t type] [--title]` | Create a new StrayMark document from a template |
| `straymark compliance [path]` | Check regulatory compliance (EU AI Act, ISO 42001, NIST) |
| `straymark metrics [path]` | Show governance metrics and documentation statistics |
| `straymark analyze [path]` | Analyze code complexity (cognitive + cyclomatic metrics) |
| `straymark analyze declared-vs-wired [path]` | Flag declared symbols (IPC/RPC proxy methods) with no implemented wiring counterpart — config-driven set-difference (POLISH-CHARTER-PATTERN sub-class 5) |
| `straymark followups list [--bucket] [--status] [--severity] [--label]` | List follow-ups registry entries with filters |
| `straymark followups status [FU-NNN]` | Registry pulse (counters recomputed on the fly) or entry detail |
| `straymark followups drift [--apply] [--scan-all]` | Detect/extract AILOGs with unextracted follow-up content (native, anti-noise `suspected-closed`, recomputes CLI-owned counters even with zero extractions, upgrades v0→v1) |
| `straymark followups recount` | Recompute the CLI-owned counters after a manual-triage session (no AILOG scan, idempotent) |
| `straymark followups promote FU-NNN` | Elevate an entry to a TDE document with `promoted_from_followup` traceability |
| `straymark architecture generate [path] [--force] [--out DIR]` | EXPERIMENTAL (Loom A1.2) — write a first-draft `architecture/model.yml` + `plan.drawio` by mining codebase structure (top-level source dirs → components) enriched with ADR C4 diagrams + "Affected Components" tables. `--force` overwrites; `--out` overrides the default `.straymark/architecture/` |
| `straymark architecture sync [path] [--out DIR] [--apply]` | EXPERIMENTAL (Loom A1.3) — append-only: detect new top-level source dirs / ADR components not yet covered by the model and append them to `model.yml` + `plan.drawio` (never clobbers human edits/geometry). Dry-run by default; `--apply` writes |
| `straymark architecture validate [path] [--out DIR] [--output FMT]` | EXPERIMENTAL (Loom A1.3) — report model↔plan.drawio integrity signals (`undrawn`/`unmodeled`/`empty`) via `core::architecture::validate_model`. `--output text\|json\|markdown`; exits 1 when any signal is found (CI-gateable) |
| `straymark audit [path]` | Generate audit trail reports with timeline and traceability |
| `straymark explore [path]` | Interactive TUI documentation browser |
| `straymark loom serve [path] [--port] [--no-open]` | Launch Loom, the EXPERIMENTAL knowledge-graph visualization server (binary downloaded on demand from `loom-*` releases, cached in `~/.straymark/bin/`) |
| `straymark about` | Show version and license info |

## Development

### Build

```bash
cd cli
cargo build              # Debug
cargo build --release    # Release
cargo build --no-default-features  # Without TUI
```

### Test

```bash
cargo test    # Full workspace suite
```

**CLI integration tests** (`cli/tests/`) spawn the binary via `assert_cmd`. Use the **macro** `cargo_bin_cmd!("straymark")` (import `use assert_cmd::cargo_bin_cmd;`), **not** the function `Command::cargo_bin("straymark").unwrap()`. The function is deprecated since `assert_cmd` 2.1 (incompatible with a custom cargo build-dir) and the macro returns a `Command` directly — no `.unwrap()`.

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `tui` | Yes | Terminal UI for `explore` (ratatui + crossterm + pulldown-cmark) |

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `colored` | Terminal colors |
| `ratatui` | TUI framework (optional) |
| `crossterm` | Terminal backend (optional) |
| `pulldown-cmark` | Markdown parser (optional) |
| `reqwest` | HTTP client for downloads |
| `serde_yaml` | YAML parsing |
| `chrono` | Date parsing (metrics, audit) |
| `anyhow` | Error handling |

## Website (straymark.dev)

Public marketing + docs + blog site, served at https://straymark.dev. Source under `website/`. Stack: **Docusaurus 3.10**, TypeScript, React 19, Node ≥ 20. Engine of record: GitHub Pages **with the official Actions pipeline** (build_type=workflow), not the legacy `gh-pages` branch.

### Layout

```
website/
├── docusaurus.config.ts    # Site config: title, url, baseUrl, i18n, plugins, theme
├── sidebars.ts             # Docs sidebar structure
├── package.json            # npm scripts: start/build/sync:docs/migrate:blog
├── blog/                   # Blog posts (canonical EN). 14 posts.
├── src/                    # React components (custom landing, Hero, etc.)
├── static/                 # Static assets served at /, incl. CNAME
├── scripts/
│   ├── sync-docs-i18n.ts   # Copies docs/i18n/{locale}/ → website/i18n/{locale}/...
│   ├── migrate-blog.ts     # One-off migration helper (Aparador → blog/)
│   └── blog-excerpts.json  # Hand-authored blog excerpts (overrides auto-truncation)
├── i18n/                   # Per-locale overrides (UI strings + blog translations + docs mirror)
└── (build/ + .docusaurus/  # Generated, gitignored)
```

Canonical docs live at the repo root in `docs/` (read by the `path: '../docs'` setting in the classic preset config). Translations of those docs live in `docs/i18n/{locale}/` and get mirrored into `website/i18n/{locale}/docusaurus-plugin-content-docs/current/` by the `sync:docs` script — Docusaurus reads them from the mirror at build time. Edit canonical content in `docs/`, edit translated content in `docs/i18n/{locale}/`, never edit `website/i18n/.../current/` directly (it's clobbered on every build).

### npm scripts

| Script | What it does |
|---|---|
| `npm run start` | Dev server with hot reload (`prestart` runs `sync:docs` first). |
| `npm run build` | Production build into `build/`. `prebuild` runs `sync:docs`. |
| `npm run serve` | Serve the production build locally for smoke-testing. |
| `npm run sync:docs` | Mirror `docs/i18n/{locale}/` into `website/i18n/{locale}/.../current/`. Idempotent. |
| `npm run migrate:blog` | One-off — only used during the initial Aparador → Docusaurus migration. |
| `npm run clear` | Wipe `.docusaurus/` cache + `build/`. |
| `npm run typecheck` | `tsc` over the website's TypeScript. |
| `npm run write-translations` | Extract translatable strings from JSX/markdown into `i18n/{locale}/code.json` skeletons. Use when adding a new locale or new translatable UI strings. |

### Locales

Configured in `docusaurus.config.ts` under `i18n`:

```ts
i18n: {
  defaultLocale: 'en',
  locales: ['en', 'es', 'zh-CN'],
  localeConfigs: {
    en: {label: 'English'},
    es: {label: 'Español'},
    'zh-CN': {label: '简体中文'},
  },
}
```

For each non-default locale you need three things:

1. The locale code in `locales` + a `localeConfigs[code]` entry (label shown in the dropdown).
2. `website/i18n/{code}/code.json` — UI strings (hero, workflow, features section). Generate skeleton with `npm run write-translations -- --locale {code}` and translate the `message` fields. The `es` file is the reference shape.
3. `website/i18n/{code}/docusaurus-plugin-content-blog/` — blog post translations + `authors.yml`. One file per canonical post (same filename).
4. `docs/i18n/{code}/` — doc translations, copied automatically into `website/i18n/.../current/` by `sync:docs`. Also add the locale to `LOCALES` array in `website/scripts/sync-docs-i18n.ts`.
5. Optionally `website/i18n/{code}/docusaurus-theme-classic/{navbar,footer}.json` for theme overrides.

### Deploy pipeline

Workflow: `.github/workflows/deploy-website.yml`. Triggers on push to `main` when files under `website/**`, `docs/**`, or the workflow itself change. Two jobs:

1. **build** — checkout, `npm ci` (cache keyed on `website/package-lock.json`), `npm run build`, `actions/configure-pages` + `actions/upload-pages-artifact` upload the `website/build/` dir as a Pages artifact.
2. **deploy** — `actions/deploy-pages` consumes the artifact, publishes to the `github-pages` environment, surfaces the URL on the PR/run.

Permissions required (already set in the workflow): `contents: read`, `pages: write`, `id-token: write`.

Repo-side Pages config (one-time setup, can be inspected with `gh api repos/StrangeDaysTech/straymark/pages`):

| Field | Value | Why |
|---|---|---|
| `build_type` | `workflow` | Required by `actions/deploy-pages`. Was `legacy` until the migration to the official pipeline (PR #169) — see the "Pages source migration gotcha" note below. |
| `cname` | `straymark.dev` | Custom domain. Must be set via `gh api -X PUT .../pages -f cname=straymark.dev` once. Auto-detection from `website/static/CNAME` only works in legacy `build_type`. |
| `https_enforced` | `true` | Let's Encrypt cert auto-provisions after the cname is set. |

### Custom domain

`website/static/CNAME` carries `straymark.dev` into the published artifact (used by Pages for the legacy redirect from `strangedaystech.github.io/straymark/`). The authoritative source of the custom domain when running under `build_type: workflow` is the Pages API `cname` field — not the file.

To change the domain: update `website/static/CNAME`, update `url:` in `docusaurus.config.ts`, and `gh api -X PUT .../pages -f cname=<new-domain>`. Then redeploy. DNS must already point at GitHub Pages or the cert provisioning will fail.

### Build gotchas (lessons from PR #169–#171 cycle)

1. **Docusaurus build has a slow tail.** After the production build is logically complete (artifacts written to `build/`), the Node process can take 5–20 extra seconds to exit cleanly (workers, fs watchers). The command is **not hung** — wait it out. Markers of real completion: `build/index.html` and `build/docs/intro/index.html` both exist.
2. **Never run two builds in parallel.** Both write to the same `build/` dir and race. If `npm run build` seems stuck, kill it first (`pkill -f "docusaurus build"`); do not relaunch on top of it.
3. **Avoid `npm run build | tail -N` for monitoring.** `tail -N` waits for EOF on its stdin, and Docusaurus's slow tail makes that close late — making the whole command appear hung when it isn't. Either run the build without a pipe, or redirect to a file (`npm run build > build.log 2>&1`) and `tail` the file separately.
4. **`exclude` semantics.** Patterns in the docs plugin's `exclude` are matched relative to `path: '../docs'`. Use `'**/decisions/**'` (with leading `**`), not `'decisions/**'`, when you want to exclude a sub-tree across all i18n mirrors as well.
5. **Pages source migration gotcha.** Switching the workflow from `peaceiris/actions-gh-pages` to the official `actions/deploy-pages` requires `build_type=workflow` at the repo level. If you migrate the workflow without flipping the repo setting, the `deploy` job fails with zero visible steps (the environment can't bind to a legacy source). Fix: `gh api -X PUT repos/<org>/<repo>/pages -f build_type=workflow`.

### Verifying a deploy

```bash
# Pages config is sane
gh api repos/StrangeDaysTech/straymark/pages --jq '{build_type, cname, html_url, https_enforced}'
# Workflow run succeeded
gh run list --workflow=deploy-website.yml --branch=main --limit 1 --json conclusion --jq '.[0].conclusion'
# Live site responds
curl -sI https://straymark.dev/ | head -1                     # HTTP/2 200
curl -sL https://straymark.dev/ | grep -oE '<title>[^<]*</title>'   # site title present
curl -sIL https://strangedaystech.github.io/straymark/ | head -2    # 301 → straymark.dev
```

<!-- straymark:begin -->
> **Read and follow the rules in [STRAYMARK.md](STRAYMARK.md).**
> That file contains all StrayMark documentation governance rules for this project.
<!-- straymark:end -->
