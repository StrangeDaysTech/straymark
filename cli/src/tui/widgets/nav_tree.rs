use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::app::{ActivePanel, App, NavSelection, SortOrder};
use crate::tui::i18n_strings::t;
use crate::tui::index::DocEntry;
use crate::tui::theme;
use crate::utils::{truncate_visual, visual_width};

pub struct NavTree<'a> {
    app: &'a App,
}

impl<'a> NavTree<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for NavTree<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_active = self.app.active_panel == ActivePanel::Navigation;
        let border_style = if is_active {
            Style::default().fg(theme::BORDER_ACTIVE)
        } else {
            Style::default().fg(theme::SUBTLE)
        };

        let lang = self.app.language.as_str();
        let block = Block::default()
            .title(format!(
                " {} {} ",
                t("Navigation", lang),
                match self.app.sort_order {
                    SortOrder::Name => t("[s:sort ↓name]", lang),
                    SortOrder::Date => t("[s:sort ↓date]", lang),
                }
            ))
            .title_style(if is_active {
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::SUBTLE)
            })
            .borders(Borders::ALL)
            .border_type(theme::BORDER_TYPE)
            .border_style(border_style)
            .style(Style::default().bg(theme::SURFACE));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<Line<'static>> = vec![Line::from("")];
        let mut selected_line: Option<usize> = None;
        let search = self.app.search_query.as_deref();
        let has_search = search.is_some();

        for (gi, group) in self.app.index.groups.iter().enumerate() {
            let is_expanded = self.app.expanded_groups[gi];
            let is_selected = self.app.selection == NavSelection::Group(gi);

            let show_children = if has_search {
                group_has_matches(group, search)
            } else {
                is_expanded
            };

            if has_search && !show_children {
                continue;
            }

            let arrow = if show_children { "▾" } else { "▸" };
            let doc_count = count_group_docs(group);

            let count_str = if doc_count > 0 {
                format!(" ({doc_count})")
            } else {
                String::new()
            };

            let style = if is_selected {
                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };

            if is_selected {
                selected_line = Some(lines.len());
            }
            lines.push(Line::from(vec![
                Span::styled(format!(" {arrow} "), Style::default().fg(theme::ACCENT)),
                Span::styled(group.label.clone(), style),
                Span::styled(count_str, Style::default().fg(theme::TEXT_DIM)),
            ]));

            if show_children {
                for (fi, entry) in group.files.iter().enumerate() {
                    if !matches_search(entry, search) {
                        continue;
                    }
                    let is_sel = self.app.selection == NavSelection::GroupFile(gi, fi);
                    if is_sel {
                        selected_line = Some(lines.len());
                    }
                    lines.push(file_entry_line(entry, "   ", inner.width as usize, is_sel));
                }

                for (si, sg) in group.subgroups.iter().enumerate() {
                    let sg_matches = subgroup_has_search_matches(sg, search);
                    if has_search && !sg_matches {
                        continue;
                    }

                    let is_sel = self.app.selection == NavSelection::Subgroup(gi, si);
                    let sg_expanded = has_search || self.app.is_subgroup_expanded(gi, si);
                    let sg_count = sg.files.len()
                        + sg.user_dirs.iter().map(|ud| ud.files.len()).sum::<usize>();
                    let sg_arrow = if sg_expanded { "▾" } else { "▸" };
                    let sg_style = if is_sel {
                        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::SUBGROUP)
                    };

                    if is_sel {
                        selected_line = Some(lines.len());
                    }
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled(format!("{sg_arrow} "), Style::default().fg(theme::SUBGROUP)),
                        Span::styled(format!("{}/", sg.label), sg_style),
                        Span::styled(
                            format!(" ({sg_count})"),
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                    ]));

                    if sg_expanded {
                        // Direct files in subgroup
                        for (fi, entry) in sg.files.iter().enumerate() {
                            if !matches_search(entry, search) {
                                continue;
                            }
                            let is_sel =
                                self.app.selection == NavSelection::SubgroupFile(gi, si, fi);
                            if is_sel {
                                selected_line = Some(lines.len());
                            }
                            lines.push(file_entry_line(entry, "     ", inner.width as usize, is_sel));
                        }

                        // User-created subdirectories
                        for (di, ud) in sg.user_dirs.iter().enumerate() {
                            let ud_has_matches =
                                ud.files.iter().any(|e| matches_search(e, search));
                            if has_search && !ud_has_matches {
                                continue;
                            }

                            let is_sel =
                                self.app.selection == NavSelection::UserDir(gi, si, di);
                            let ud_expanded =
                                has_search || self.app.is_userdir_expanded(gi, si, di);
                            let ud_count = ud.files.len();
                            let ud_arrow = if ud_expanded { "▾" } else { "▸" };
                            let ud_style = if is_sel {
                                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                            } else {
                                Style::default().fg(theme::USER_DIR)
                            };

                            if is_sel {
                                selected_line = Some(lines.len());
                            }
                            lines.push(Line::from(vec![
                                Span::raw("     "),
                                Span::styled(
                                    format!("{ud_arrow} "),
                                    Style::default().fg(theme::USER_DIR),
                                ),
                                Span::styled(format!("{}/", ud.name), ud_style),
                                Span::styled(
                                    format!(" ({ud_count})"),
                                    Style::default().fg(theme::TEXT_DIM),
                                ),
                            ]));

                            if ud_expanded {
                                // Files in user dir
                                for (fi, entry) in ud.files.iter().enumerate() {
                                    if !matches_search(entry, search) {
                                        continue;
                                    }
                                    let is_sel = self.app.selection
                                        == NavSelection::UserDirFile(gi, si, di, fi);
                                    if is_sel {
                                        selected_line = Some(lines.len());
                                    }
                                    lines.push(file_entry_line(entry, "       ", inner.width as usize, is_sel));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calculate scroll to keep selected item visible
        let visible_height = inner.height as usize;
        let scroll = if let Some(sel) = selected_line {
            if sel >= visible_height {
                // Keep selected item near the bottom with some margin
                (sel - visible_height + 3).min(lines.len().saturating_sub(visible_height))
            } else {
                0
            }
        } else {
            0
        };

        let paragraph = Paragraph::new(lines).scroll((scroll as u16, 0));
        paragraph.render(inner, buf);
    }
}

fn file_entry_line(entry: &DocEntry, indent: &str, max_width: usize, selected: bool) -> Line<'static> {
    let style = file_style(selected);
    let badge_style = Style::default().fg(theme::ACCENT);
    let date_style = Style::default().fg(theme::TEXT_DIM);

    let has_badge = !entry.doc_type.is_empty();
    // Badge (`doc_type`) is drawn from hardcoded ASCII prefixes, so its
    // visual width equals its char count; `+ 1` accounts for the trailing
    // space between badge and title.
    let badge_cols = if has_badge { visual_width(&entry.doc_type) + 1 } else { 0 };

    // Compact date: show MM-DD from YYYY-MM-DD. Only render the slice when
    // the prefix is ASCII so we never cut through a multi-byte boundary —
    // anything else is treated as missing.
    let date = extract_mmdd(&entry.created).map(|d| format!(" {d}")).unwrap_or_default();
    let date_cols = visual_width(&date);

    // `indent` is hardcoded ASCII spaces, so byte len == visual cols here,
    // but we use visual_width for consistency.
    let indent_cols = visual_width(indent);
    let title_budget = max_width
        .saturating_sub(indent_cols)
        .saturating_sub(badge_cols)
        .saturating_sub(date_cols);

    let title = truncate_visual(&entry.title, title_budget);

    let mut spans = vec![Span::raw(indent.to_string())];
    if has_badge {
        spans.push(Span::styled(entry.doc_type.clone(), badge_style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(title, style));
    spans.push(Span::styled(date, date_style));
    Line::from(spans)
}

fn file_style(selected: bool) -> Style {
    if selected {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT)
    }
}

fn count_group_docs(group: &crate::tui::index::DocGroup) -> usize {
    let direct = group.files.len();
    let sub: usize = group
        .subgroups
        .iter()
        .map(|sg| {
            sg.files.len()
                + sg.user_dirs.iter().map(|ud| ud.files.len()).sum::<usize>()
        })
        .sum();
    direct + sub
}

/// Extract the "MM-DD" slice from an ISO-8601 "YYYY-MM-DD[...]" string.
/// Returns `None` if the input is shorter than 10 chars, contains any
/// non-ASCII character in the first 10 positions, or the shape doesn't
/// match (dashes at positions 4 and 7). This keeps us safe even if the
/// `created` field came from user-edited frontmatter with exotic input.
fn extract_mmdd(created: &str) -> Option<String> {
    let mut chars = created.chars();
    let head: String = (&mut chars).take(10).collect();
    if head.chars().count() < 10 || !head.is_ascii() {
        return None;
    }
    let bytes = head.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    Some(format!("{}-{}", &head[5..7], &head[8..10]))
}

fn subgroup_has_search_matches(
    sg: &crate::tui::index::DocSubgroup,
    search: Option<&str>,
) -> bool {
    sg.files.iter().any(|e| matches_search(e, search))
        || sg
            .user_dirs
            .iter()
            .any(|ud| ud.files.iter().any(|e| matches_search(e, search)))
}

fn group_has_matches(group: &crate::tui::index::DocGroup, search: Option<&str>) -> bool {
    if group.files.iter().any(|e| matches_search(e, search)) {
        return true;
    }
    group.subgroups.iter().any(|sg| {
        sg.files.iter().any(|e| matches_search(e, search))
            || sg
                .user_dirs
                .iter()
                .any(|ud| ud.files.iter().any(|e| matches_search(e, search)))
    })
}

fn matches_search(entry: &DocEntry, search: Option<&str>) -> bool {
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
