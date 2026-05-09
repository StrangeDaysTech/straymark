use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::document::Document;
use super::index::DocIndex;

/// Which panel is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Navigation,
    Metadata,
    Document,
}

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Two-panel layout (≥100 cols) or single-panel nav (<100 cols)
    Normal,
    /// Document takes full screen
    Fullscreen,
    /// Help popup is shown
    Help,
}

/// What is selected in the navigation tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavSelection {
    /// A top-level group is selected
    Group(usize),
    /// A subgroup within a group is selected
    Subgroup(usize, usize),
    /// A file in a group's direct files
    GroupFile(usize, usize),
    /// A file within a subgroup's direct files
    SubgroupFile(usize, usize, usize),
    /// A user-created directory within a subgroup
    UserDir(usize, usize, usize),
    /// A file within a user-created directory
    UserDirFile(usize, usize, usize, usize),
}

/// What is selected within the Metadata panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaSelection {
    Tag(usize),
    Related(usize),
}

/// Sort order for file listings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Name,
    Date,
}

/// Navigation history entry for hyperlinked navigation
#[derive(Debug, Clone)]
struct HistoryEntry {
    selection: NavSelection,
    scroll_offset: u16,
}

pub struct App {
    pub index: DocIndex,
    pub active_panel: ActivePanel,
    pub view_mode: ViewMode,
    pub selection: NavSelection,
    /// Which groups are expanded in the tree
    pub expanded_groups: Vec<bool>,
    /// Which subgroups/user_dirs are expanded: (gi, si) for subgroups, (gi, si, di) encoded as string
    pub expanded_nodes: HashSet<String>,
    /// Scroll offset for the document viewer
    pub doc_scroll: u16,
    /// Total lines in current document (for scroll bounds)
    pub doc_total_lines: usize,
    /// Currently loaded document
    pub current_doc: Option<Document>,
    /// Document cache: path -> Document
    doc_cache: HashMap<PathBuf, Document>,
    /// Navigation history stack for hyperlinked navigation
    nav_history: Vec<HistoryEntry>,
    /// Sort order
    pub sort_order: SortOrder,
    /// Search/filter query
    pub search_query: Option<String>,
    pub search_input: String,
    pub is_searching: bool,
    /// Current selection within the Metadata panel
    pub meta_selection: Option<MetaSelection>,
    /// Temporary notification message shown in the status bar
    pub notification: Option<String>,
    /// Should the app quit
    pub should_quit: bool,
    /// Project root path
    pub project_root: PathBuf,
    /// Whether we're using a fallback path (repo root instead of cwd)
    pub is_fallback: bool,
    /// Active display language ("en", "es", "zh-CN"). Persisted on the app
    /// so refresh ('r') rebuilds the index in the same locale.
    pub language: String,
}

impl App {
    pub fn new(project_root: &Path, is_fallback: bool, language: &str) -> Self {
        let straymark_dir = project_root.join(".straymark");
        let index = DocIndex::build(&straymark_dir, language);
        let num_groups = index.groups.len();

        Self {
            index,
            active_panel: ActivePanel::Navigation,
            view_mode: ViewMode::Normal,
            selection: NavSelection::Group(0),
            expanded_groups: vec![false; num_groups],
            expanded_nodes: HashSet::new(),
            doc_scroll: 0,
            doc_total_lines: 0,
            current_doc: None,
            doc_cache: HashMap::new(),
            nav_history: Vec::new(),
            sort_order: SortOrder::Name,
            search_query: None,
            search_input: String::new(),
            is_searching: false,
            meta_selection: None,
            notification: None,
            should_quit: false,
            project_root: project_root.to_path_buf(),
            is_fallback,
            language: language.to_string(),
        }
    }

    /// Languages the live switcher cycles through. Order matters — this is
    /// what `cycle_language` walks. New locales should be appended here AND
    /// have entries in `tui::i18n_strings::t`.
    pub const SWITCHER_LANGUAGES: &'static [&'static str] = &["en", "es", "zh-CN"];

    /// Cycle to the next supported display language without leaving the TUI.
    /// Rebuilds the document index in the new locale (so group labels and
    /// translated docs swap immediately) and shows a status-bar notification
    /// with the new locale code. Resets the cached document body — file
    /// paths change with the locale, so the previously-rendered body would
    /// otherwise stay in the old language.
    pub fn cycle_language(&mut self) {
        let current = self.language.as_str();
        let idx = Self::SWITCHER_LANGUAGES
            .iter()
            .position(|l| *l == current)
            .unwrap_or(0);
        let next = Self::SWITCHER_LANGUAGES[(idx + 1) % Self::SWITCHER_LANGUAGES.len()];
        self.language = next.to_string();

        let straymark_dir = self.project_root.join(".straymark");
        self.index = DocIndex::build(&straymark_dir, &self.language);
        self.doc_cache.clear();
        self.current_doc = None;
        self.doc_scroll = 0;
        // Selection may now be out of bounds if a group disappeared; clamp.
        if self.expanded_groups.len() != self.index.groups.len() {
            self.expanded_groups = vec![false; self.index.groups.len()];
        }

        let prefix = crate::tui::i18n_strings::t("Language", &self.language);
        self.notification = Some(format!("{prefix}: {next}"));
    }

    /// Move selection up in the navigation tree
    pub fn nav_up(&mut self) {
        let items = self.build_nav_items();
        if items.is_empty() {
            return;
        }
        let current = items.iter().position(|s| *s == self.selection);
        match current {
            Some(0) | None => self.selection = items.last().unwrap().clone(),
            Some(i) => self.selection = items[i - 1].clone(),
        }
    }

    /// Move selection down in the navigation tree
    pub fn nav_down(&mut self) {
        let items = self.build_nav_items();
        if items.is_empty() {
            return;
        }
        let current = items.iter().position(|s| *s == self.selection);
        match current {
            Some(i) if i + 1 < items.len() => self.selection = items[i + 1].clone(),
            _ => self.selection = items[0].clone(),
        }
    }

    /// Enter/expand: toggle group expansion or open a file
    pub fn nav_enter(&mut self) {
        match &self.selection {
            NavSelection::Group(gi) => {
                let gi = *gi;
                self.expanded_groups[gi] = !self.expanded_groups[gi];
            }
            NavSelection::Subgroup(gi, si) => {
                let key = format!("sg:{gi}:{si}");
                if self.expanded_nodes.contains(&key) {
                    self.expanded_nodes.remove(&key);
                } else {
                    self.expanded_nodes.insert(key);
                }
            }
            NavSelection::GroupFile(gi, fi) => {
                let gi = *gi;
                let fi = *fi;
                if let Some(entry) = self.index.groups.get(gi).and_then(|g| g.files.get(fi)) {
                    self.load_document(&entry.path.clone());
                }
            }
            NavSelection::SubgroupFile(gi, si, fi) => {
                let gi = *gi;
                let si = *si;
                let fi = *fi;
                if let Some(entry) = self
                    .index
                    .groups
                    .get(gi)
                    .and_then(|g| g.subgroups.get(si))
                    .and_then(|sg| sg.files.get(fi))
                {
                    self.load_document(&entry.path.clone());
                }
            }
            NavSelection::UserDir(gi, si, di) => {
                let key = format!("ud:{gi}:{si}:{di}");
                if self.expanded_nodes.contains(&key) {
                    self.expanded_nodes.remove(&key);
                } else {
                    self.expanded_nodes.insert(key);
                }
            }
            NavSelection::UserDirFile(gi, si, di, fi) => {
                let gi = *gi;
                let si = *si;
                let di = *di;
                let fi = *fi;
                if let Some(entry) = self
                    .index
                    .groups
                    .get(gi)
                    .and_then(|g| g.subgroups.get(si))
                    .and_then(|sg| sg.user_dirs.get(di))
                    .and_then(|ud| ud.files.get(fi))
                {
                    self.load_document(&entry.path.clone());
                }
            }
        }
    }

    /// Go back: collapse group or go to parent
    pub fn nav_back(&mut self) {
        if self.view_mode == ViewMode::Fullscreen {
            self.view_mode = ViewMode::Normal;
            return;
        }

        // Try navigation history first
        if let Some(entry) = self.nav_history.pop() {
            self.selection = entry.selection;
            self.doc_scroll = entry.scroll_offset;
            // Reload the document at that selection if it was a file
            match &self.selection {
                NavSelection::GroupFile(gi, fi) => {
                    if let Some(e) = self.index.groups.get(*gi).and_then(|g| g.files.get(*fi)) {
                        self.load_document(&e.path.clone());
                    }
                }
                NavSelection::SubgroupFile(gi, si, fi) => {
                    if let Some(e) = self
                        .index
                        .groups
                        .get(*gi)
                        .and_then(|g| g.subgroups.get(*si))
                        .and_then(|sg| sg.files.get(*fi))
                    {
                        self.load_document(&e.path.clone());
                    }
                }
                _ => {
                    self.current_doc = None;
                }
            }
            return;
        }

        match &self.selection {
            NavSelection::GroupFile(gi, _) | NavSelection::Subgroup(gi, _) => {
                let gi = *gi;
                self.selection = NavSelection::Group(gi);
            }
            NavSelection::SubgroupFile(gi, si, _) | NavSelection::UserDir(gi, si, _) => {
                let gi = *gi;
                let si = *si;
                self.selection = NavSelection::Subgroup(gi, si);
            }
            NavSelection::UserDirFile(gi, si, di, _) => {
                let gi = *gi;
                let si = *si;
                let di = *di;
                self.selection = NavSelection::UserDir(gi, si, di);
            }
            NavSelection::Group(gi) => {
                let gi = *gi;
                self.expanded_groups[gi] = false;
            }
        }
    }

    /// Toggle between panels, or cycle related links when in document panel
    /// Cycle panels forward: Navigation → Metadata → Document → Navigation
    pub fn toggle_panel(&mut self) {
        if self.current_doc.is_some() {
            match self.active_panel {
                ActivePanel::Navigation => {
                    self.active_panel = ActivePanel::Metadata;
                    self.enter_metadata();
                }
                ActivePanel::Metadata => {
                    self.meta_selection = None;
                    self.active_panel = ActivePanel::Document;
                }
                ActivePanel::Document => {
                    self.active_panel = ActivePanel::Navigation;
                }
            }
        }
    }

    /// Cycle panels reverse: Navigation → Document → Metadata → Navigation
    pub fn toggle_panel_reverse(&mut self) {
        if self.current_doc.is_some() {
            match self.active_panel {
                ActivePanel::Document => {
                    self.active_panel = ActivePanel::Metadata;
                    self.enter_metadata();
                }
                ActivePanel::Metadata => {
                    self.meta_selection = None;
                    self.active_panel = ActivePanel::Navigation;
                }
                ActivePanel::Navigation => {
                    self.active_panel = ActivePanel::Document;
                }
            }
        }
    }

    /// When entering Metadata panel, preselect first navigable item
    fn enter_metadata(&mut self) {
        let tags = self.tag_count();
        let related = self.related_count();
        if tags > 0 {
            self.meta_selection = Some(MetaSelection::Tag(0));
        } else if related > 0 {
            self.meta_selection = Some(MetaSelection::Related(0));
        } else {
            self.meta_selection = None;
        }
    }

    /// Move selection down in metadata (tags then related)
    pub fn metadata_down(&mut self) {
        let tags = self.tag_count();
        let related = self.related_count();
        match self.meta_selection {
            None => {
                if tags > 0 {
                    self.meta_selection = Some(MetaSelection::Tag(0));
                } else if related > 0 {
                    self.meta_selection = Some(MetaSelection::Related(0));
                }
            }
            Some(MetaSelection::Tag(idx)) => {
                if idx + 1 < tags {
                    self.meta_selection = Some(MetaSelection::Tag(idx + 1));
                } else if related > 0 {
                    self.meta_selection = Some(MetaSelection::Related(0));
                }
            }
            Some(MetaSelection::Related(idx)) => {
                if idx + 1 < related {
                    self.meta_selection = Some(MetaSelection::Related(idx + 1));
                }
            }
        }
    }

    /// Move selection up in metadata (related then tags)
    pub fn metadata_up(&mut self) {
        let tags = self.tag_count();
        match self.meta_selection {
            None => {}
            Some(MetaSelection::Tag(0)) => {}
            Some(MetaSelection::Tag(idx)) => {
                self.meta_selection = Some(MetaSelection::Tag(idx - 1));
            }
            Some(MetaSelection::Related(0)) => {
                if tags > 0 {
                    self.meta_selection = Some(MetaSelection::Tag(tags - 1));
                }
            }
            Some(MetaSelection::Related(idx)) => {
                self.meta_selection = Some(MetaSelection::Related(idx - 1));
            }
        }
    }

    /// Execute action on the selected metadata item
    pub fn metadata_enter(&mut self) {
        match self.meta_selection {
            Some(MetaSelection::Tag(idx)) => {
                // Search by this tag
                if let Some(ref doc) = self.current_doc {
                    if let Some(ref fm) = doc.frontmatter {
                        if let Some(tag) = fm.tags.get(idx) {
                            self.search_query = Some(tag.clone());
                            self.meta_selection = None;
                            self.active_panel = ActivePanel::Navigation;
                        }
                    }
                }
            }
            Some(MetaSelection::Related(idx)) => {
                if let Some(ref doc) = self.current_doc {
                    if let Some(ref fm) = doc.frontmatter {
                        if let Some(related_id) = fm.related.get(idx) {
                            let id = related_id.clone();
                            self.meta_selection = None;
                            self.navigate_to_id(&id);
                        }
                    }
                }
            }
            None => {}
        }
    }

    /// Get the number of tags in the current document
    fn tag_count(&self) -> usize {
        self.current_doc
            .as_ref()
            .and_then(|doc| doc.frontmatter.as_ref())
            .map(|fm| fm.tags.len())
            .unwrap_or(0)
    }

    /// Get the number of related links in the current document
    fn related_count(&self) -> usize {
        self.current_doc
            .as_ref()
            .and_then(|doc| doc.frontmatter.as_ref())
            .map(|fm| fm.related.len())
            .unwrap_or(0)
    }

    /// Toggle fullscreen mode for document
    pub fn toggle_fullscreen(&mut self) {
        if self.current_doc.is_some() {
            self.view_mode = match self.view_mode {
                ViewMode::Fullscreen => ViewMode::Normal,
                _ => ViewMode::Fullscreen,
            };
        }
    }

    /// Toggle help popup
    pub fn toggle_help(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Help => ViewMode::Normal,
            _ => ViewMode::Help,
        };
    }

    /// Scroll document down
    pub fn scroll_down(&mut self, amount: u16) {
        if self.doc_total_lines > 0 {
            let max = self.doc_total_lines.saturating_sub(5) as u16;
            self.doc_scroll = (self.doc_scroll + amount).min(max);
        }
    }

    /// Scroll document up
    pub fn scroll_up(&mut self, amount: u16) {
        self.doc_scroll = self.doc_scroll.saturating_sub(amount);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.doc_scroll = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        if self.doc_total_lines > 5 {
            self.doc_scroll = (self.doc_total_lines - 5) as u16;
        }
    }

    /// Jump to a specific group by number (1-8)
    pub fn jump_to_group(&mut self, num: usize) {
        let idx = num.saturating_sub(1);
        if idx < self.index.groups.len() {
            self.selection = NavSelection::Group(idx);
            self.expanded_groups[idx] = true;
        }
    }

    pub fn is_subgroup_expanded(&self, gi: usize, si: usize) -> bool {
        self.expanded_nodes.contains(&format!("sg:{gi}:{si}"))
    }

    pub fn is_userdir_expanded(&self, gi: usize, si: usize, di: usize) -> bool {
        self.expanded_nodes.contains(&format!("ud:{gi}:{si}:{di}"))
    }

    /// Cycle sort order and re-sort all files in the index
    pub fn cycle_sort(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Name => SortOrder::Date,
            SortOrder::Date => SortOrder::Name,
        };
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        let sort = self.sort_order;
        for group in &mut self.index.groups {
            sort_entries(&mut group.files, sort);
            for sg in &mut group.subgroups {
                sort_entries(&mut sg.files, sort);
                for ud in &mut sg.user_dirs {
                    sort_entries(&mut ud.files, sort);
                }
            }
        }
    }

    /// Navigate to a document by its ID (hyperlinked navigation)
    pub fn navigate_to_id(&mut self, id: &str) {
        let path = match self.index.find_by_ref(id) {
            Some(p) => p,
            None => {
                self.notification = Some(format!("Document not found: {id}"));
                return;
            }
        };

        // Save current position to history
        self.nav_history.push(HistoryEntry {
            selection: self.selection.clone(),
            scroll_offset: self.doc_scroll,
        });

        // Find the selection that corresponds to this path
        if let Some(sel) = self.find_selection_for_path(&path) {
            self.selection = sel;
            // Ensure parent group is expanded
            match &self.selection {
                NavSelection::GroupFile(gi, _)
                | NavSelection::Subgroup(gi, _)
                | NavSelection::SubgroupFile(gi, _, _)
                | NavSelection::UserDir(gi, _, _)
                | NavSelection::UserDirFile(gi, _, _, _) => {
                    self.expanded_groups[*gi] = true;
                }
                _ => {}
            }
        }

        self.load_document(&path);
    }

    /// Navigate to next document in the current group/subgroup
    pub fn next_document(&mut self) {
        self.navigate_sibling(1);
    }

    /// Navigate to previous document in the current group/subgroup
    pub fn prev_document(&mut self) {
        self.navigate_sibling(-1);
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.search_input.clear();
    }

    /// Cancel search
    pub fn cancel_search(&mut self) {
        self.is_searching = false;
        self.search_input.clear();
        self.search_query = None;
    }

    /// Apply search filter
    pub fn apply_search(&mut self) {
        if self.search_input.is_empty() {
            self.search_query = None;
        } else {
            self.search_query = Some(self.search_input.clone());
        }
        self.is_searching = false;
    }

    // ── Private helpers ──

    fn load_document(&mut self, path: &Path) {
        if let Some(cached) = self.doc_cache.get(path) {
            self.current_doc = Some(cached.clone());
        } else if let Some(doc) = Document::load(path) {
            self.doc_cache.insert(path.to_path_buf(), doc.clone());
            self.current_doc = Some(doc);
        }
        self.doc_scroll = 0;
        self.meta_selection = None;
        self.active_panel = ActivePanel::Document;
    }

    fn navigate_sibling(&mut self, direction: i32) {
        match &self.selection {
            NavSelection::GroupFile(gi, fi) => {
                let gi = *gi;
                let fi = *fi;
                let len = self.index.groups[gi].files.len();
                if len == 0 {
                    return;
                }
                let new_fi = (fi as i32 + direction).rem_euclid(len as i32) as usize;
                self.selection = NavSelection::GroupFile(gi, new_fi);
                let path = self.index.groups[gi].files[new_fi].path.clone();
                self.load_document(&path);
            }
            NavSelection::SubgroupFile(gi, si, fi) => {
                let gi = *gi;
                let si = *si;
                let fi = *fi;
                let len = self.index.groups[gi].subgroups[si].files.len();
                if len == 0 {
                    return;
                }
                let new_fi = (fi as i32 + direction).rem_euclid(len as i32) as usize;
                self.selection = NavSelection::SubgroupFile(gi, si, new_fi);
                let path = self.index.groups[gi].subgroups[si].files[new_fi].path.clone();
                self.load_document(&path);
            }
            NavSelection::UserDirFile(gi, si, di, fi) => {
                let gi = *gi;
                let si = *si;
                let di = *di;
                let fi = *fi;
                let len = self.index.groups[gi].subgroups[si].user_dirs[di].files.len();
                if len == 0 {
                    return;
                }
                let new_fi = (fi as i32 + direction).rem_euclid(len as i32) as usize;
                self.selection = NavSelection::UserDirFile(gi, si, di, new_fi);
                let path = self.index.groups[gi].subgroups[si].user_dirs[di].files[new_fi]
                    .path
                    .clone();
                self.load_document(&path);
            }
            _ => {}
        }
    }

    fn find_selection_for_path(&self, target: &Path) -> Option<NavSelection> {
        for (gi, group) in self.index.groups.iter().enumerate() {
            for (fi, entry) in group.files.iter().enumerate() {
                if entry.path == target {
                    return Some(NavSelection::GroupFile(gi, fi));
                }
            }
            for (si, sg) in group.subgroups.iter().enumerate() {
                for (fi, entry) in sg.files.iter().enumerate() {
                    if entry.path == target {
                        return Some(NavSelection::SubgroupFile(gi, si, fi));
                    }
                }
                for (di, ud) in sg.user_dirs.iter().enumerate() {
                    for (fi, entry) in ud.files.iter().enumerate() {
                        if entry.path == target {
                            return Some(NavSelection::UserDirFile(gi, si, di, fi));
                        }
                    }
                }
            }
        }
        None
    }

    /// Build a flat list of all visible navigation items in order
    fn build_nav_items(&self) -> Vec<NavSelection> {
        let mut items = Vec::new();
        let search = self.search_query.as_deref();
        let has_search = search.is_some();

        for (gi, group) in self.index.groups.iter().enumerate() {
            let show_children = if has_search {
                group_has_matches(group, search)
            } else {
                self.expanded_groups[gi]
            };

            // When searching, skip groups with no matches
            if has_search && !show_children {
                continue;
            }

            items.push(NavSelection::Group(gi));

            if show_children {
                // Direct files
                for (fi, entry) in group.files.iter().enumerate() {
                    if !entry_matches_search(entry, search) {
                        continue;
                    }
                    items.push(NavSelection::GroupFile(gi, fi));
                }
                // Subgroups, their files, and user dirs
                for (si, sg) in group.subgroups.iter().enumerate() {
                    let sg_has_matches = subgroup_has_matches(sg, search);
                    if has_search && !sg_has_matches {
                        continue;
                    }
                    items.push(NavSelection::Subgroup(gi, si));

                    let sg_expanded = has_search || self.is_subgroup_expanded(gi, si);
                    if sg_expanded {
                        for (fi, entry) in sg.files.iter().enumerate() {
                            if !entry_matches_search(entry, search) {
                                continue;
                            }
                            items.push(NavSelection::SubgroupFile(gi, si, fi));
                        }
                        for (di, ud) in sg.user_dirs.iter().enumerate() {
                            let ud_has_matches =
                                ud.files.iter().any(|e| entry_matches_search(e, search));
                            if has_search && !ud_has_matches {
                                continue;
                            }
                            items.push(NavSelection::UserDir(gi, si, di));

                            let ud_expanded =
                                has_search || self.is_userdir_expanded(gi, si, di);
                            if ud_expanded {
                                for (fi, entry) in ud.files.iter().enumerate() {
                                    if !entry_matches_search(entry, search) {
                                        continue;
                                    }
                                    items.push(NavSelection::UserDirFile(gi, si, di, fi));
                                }
                            }
                        }
                    }
                }
            }
        }
        items
    }
}

fn group_has_matches(
    group: &crate::tui::index::DocGroup,
    search: Option<&str>,
) -> bool {
    group.files.iter().any(|e| entry_matches_search(e, search))
        || group
            .subgroups
            .iter()
            .any(|sg| subgroup_has_matches(sg, search))
}

fn subgroup_has_matches(
    sg: &crate::tui::index::DocSubgroup,
    search: Option<&str>,
) -> bool {
    sg.files.iter().any(|e| entry_matches_search(e, search))
        || sg
            .user_dirs
            .iter()
            .any(|ud| ud.files.iter().any(|e| entry_matches_search(e, search)))
}

fn sort_entries(entries: &mut Vec<crate::tui::index::DocEntry>, order: SortOrder) {
    match order {
        SortOrder::Name => {
            entries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
        SortOrder::Date => entries.sort_by(|a, b| b.created.cmp(&a.created)),
    }
}

fn entry_matches_search(
    entry: &crate::tui::index::DocEntry,
    search: Option<&str>,
) -> bool {
    let Some(q) = search else {
        return true;
    };
    let query = q.to_lowercase();
    entry.filename.to_lowercase().contains(&query)
        || entry.title.to_lowercase().contains(&query)
        || entry.tags.iter().any(|t| t.to_lowercase().contains(&query))
        || (!entry.created.is_empty() && entry.created.contains(&query))
        || entry.id.to_lowercase().contains(&query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_app(language: &str) -> App {
        let tmp = tempfile::TempDir::new().unwrap();
        let governance = tmp.path().join(".straymark").join("00-governance");
        std::fs::create_dir_all(&governance).unwrap();
        std::fs::write(governance.join("AGENT-RULES.md"), "# Rules").unwrap();
        // Hold the TempDir alive by leaking — fine for test scope.
        let path = tmp.keep();
        App::new(&path, false, language)
    }

    #[test]
    fn cycle_language_walks_full_cycle() {
        let mut app = fixture_app("en");
        assert_eq!(app.language, "en");
        app.cycle_language();
        assert_eq!(app.language, "es");
        app.cycle_language();
        assert_eq!(app.language, "zh-CN");
        app.cycle_language();
        assert_eq!(app.language, "en");
    }

    #[test]
    fn cycle_language_emits_translated_notification() {
        let mut app = fixture_app("en");
        app.cycle_language(); // → es
        let note = app.notification.as_ref().expect("notification set");
        // Spanish prefix and the locale code must both be present.
        assert!(note.contains("Idioma"), "notification {note:?} missing es prefix");
        assert!(note.contains("es"), "notification {note:?} missing locale");
    }

    #[test]
    fn cycle_language_unknown_starting_locale_recovers() {
        // If for some reason `language` was set to something not in the
        // cycle list, cycling should land on the first known entry instead
        // of panicking on unwrap.
        let mut app = fixture_app("fr");
        app.cycle_language();
        assert_eq!(app.language, "es", "cycle from unknown should advance from index 0");
    }

    #[test]
    fn cycle_language_clears_doc_cache_and_current() {
        let mut app = fixture_app("en");
        app.current_doc = None;
        app.cycle_language();
        // Cache must be empty so the next open() reads the localized file.
        assert!(app.current_doc.is_none());
    }
}
