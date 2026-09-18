//! Only live work is inventoried: nested checkouts are skipped by every walker
//! (#434) and follow-up examples are not entries (#431).
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use straymark_baton::codescan;
use straymark_baton::coherence::Inventory;
use straymark_baton::units::{inventory, Granularity, RoutableUnit};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "baton-inventory-hygiene-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn followups(&self, registry: &str) -> Vec<RoutableUnit> {
        self.write(".straymark/follow-ups-backlog.md", registry);
        inventory(&self.0, Some(Granularity::Followup))
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        // Never panic in Drop: a failing test is already unwinding.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// `fixture` at the project root, plus two more copies of it that are other
/// checkouts: a linked worktree (`.git` file) and a nested clone (`.git` dir).
fn project_with_nested_checkouts(fixture_name: &str) -> (Project, Project) {
    let plain = Project::new();
    copy_tree(&fixture(fixture_name), &plain.0);

    let nested = Project::new();
    copy_tree(&fixture(fixture_name), &nested.0);
    copy_tree(&fixture(fixture_name), &nested.0.join(".worktrees/feature"));
    nested.write(
        ".worktrees/feature/.git",
        "gitdir: /elsewhere/.git/worktrees/feature\n",
    );
    copy_tree(&fixture(fixture_name), &nested.0.join("vendor-clone"));
    std::fs::create_dir(nested.0.join("vendor-clone/.git")).unwrap();
    (plain, nested)
}

fn ids(units: &[RoutableUnit]) -> Vec<String> {
    units
        .iter()
        .map(|u| format!("{}:{}", u.granularity.as_str(), u.id))
        .collect()
}

#[test]
fn unit_inventory_skips_nested_checkouts() {
    let (plain, nested) = project_with_nested_checkouts("governance-corpus");
    let expected = ids(&inventory(&plain.0, None));
    assert!(
        expected.iter().any(|id| id.starts_with("task:")),
        "fixture must exercise the task walker"
    );
    assert!(
        expected.iter().any(|id| id.starts_with("batch:")),
        "fixture must exercise the batch walker"
    );
    assert_eq!(ids(&inventory(&nested.0, None)), expected);
}

#[test]
fn coherence_inventory_skips_nested_checkouts() {
    let (plain, nested) = project_with_nested_checkouts("sample-project");
    let expected = Inventory::scan(&plain.0).files;
    assert!(!expected.is_empty());
    assert_eq!(Inventory::scan(&nested.0).files, expected);
}

#[test]
fn code_scan_skips_nested_checkouts() {
    let (plain, nested) = project_with_nested_checkouts("sample-project");
    let sources = |root: &Path| -> Vec<String> {
        codescan::scan(root)
            .into_iter()
            .map(|s| s.source.file)
            .collect()
    };
    let expected = sources(&plain.0);
    assert!(!expected.is_empty(), "fixture must yield contract shapes");
    assert_eq!(sources(&nested.0), expected);
}

#[test]
fn the_shipped_empty_registry_has_no_followups() {
    let template = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../dist/.straymark/templates/follow-ups-backlog.md");
    let registry = std::fs::read_to_string(template).unwrap();
    assert!(
        registry.contains("### FU-NNN"),
        "template still documents the shape"
    );
    assert!(Project::new().followups(&registry).is_empty());
}

#[test]
fn issue_431_reproduction() {
    let registry = "<!--\n### FU-NNN — example only\n- **Work verb**: design | implement | audit | operate\n-->\n\n## Bucket: ready\n";
    assert!(Project::new().followups(registry).is_empty());
}

#[test]
fn examples_never_leak_into_live_entries() {
    let registry = "\
## Bucket: ready

### FU-007 — Real entry
- **Status**: open
- **Severity**: blocking <!-- was: normal -->

<!--
### FU-NNN — commented example
- **Work verb**: design
-->

```markdown
### FU-999 — fenced example
- **Work verb**: audit
```

### Triage notes
- **Work verb**: operate

### FU-008: Colon-separated entry
- **Work verb**: implement
- **Design provenance**: upstream
";
    let units = Project::new().followups(registry);
    let ids: Vec<&str> = units.iter().map(|u| u.id.as_str()).collect();
    assert_eq!(ids, ["FU-007", "FU-008"]);

    let (fu7, fu8) = (&units[0], &units[1]);
    assert_eq!(fu7.followup_severity.as_deref(), Some("blocking"));
    assert_eq!(
        fu7.work_verb, None,
        "examples and other headings are not FU-007's"
    );
    assert_eq!(fu8.work_verb.as_deref(), Some("implement"));
    assert_eq!(fu8.design_provenance.as_deref(), Some("upstream"));
    assert_eq!(fu8.followup_bucket.as_deref(), Some("ready"));
}
