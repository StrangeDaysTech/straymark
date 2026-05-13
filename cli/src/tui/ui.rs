use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use ratatui::Frame;

use super::app::{App, ViewMode};
use super::theme;
use super::widgets::doc_viewer::DocViewer;
use super::widgets::help_popup::HelpPopup;
use super::widgets::metadata_panel::MetadataPanel;
use super::widgets::nav_tree::NavTree;
use super::widgets::status_bar::StatusBar;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let terminal_width = area.width;

    // Fill entire background
    let bg = Block::default().style(Style::default().bg(theme::BG));
    bg.render(area, frame.buffer_mut());

    // Split: main area + status bar at bottom
    let vertical = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(area);

    let main_area = vertical[0];
    let status_area = vertical[1];

    // Render status bar
    frame.render_widget(StatusBar::new(app), status_area);

    match app.view_mode {
        ViewMode::Fullscreen => {
            DocViewer::new(app).render(main_area, frame.buffer_mut());
        }
        ViewMode::Help => {
            render_main_layout(frame, app, main_area, terminal_width);
            frame.render_widget(HelpPopup::new(&app.language), main_area);
        }
        ViewMode::Normal => {
            render_main_layout(frame, app, main_area, terminal_width);
        }
    }
}

fn render_main_layout(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect, width: u16) {
    if width >= 100 {
        // Two-column layout: left (nav + metadata) | right (document)
        let horizontal =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(area);

        // Left column: navigation on top, metadata on bottom
        // Give metadata a fair share: at least 40% of left column, but not more than needed
        let available = horizontal[0].height;
        let ideal_metadata = metadata_panel_height(app);
        // Ensure metadata gets at least 40% but cap at its ideal height
        let min_metadata = (available * 40 / 100).max(8);
        let metadata_h = ideal_metadata.max(min_metadata).min(available.saturating_sub(6));
        let left_split = Layout::vertical([
            Constraint::Min(6),
            Constraint::Length(metadata_h),
        ])
        .split(horizontal[0]);

        frame.render_widget(NavTree::new(app), left_split[0]);
        frame.render_widget(MetadataPanel::new(app), left_split[1]);
        DocViewer::new(app).render(horizontal[1], frame.buffer_mut());
    } else {
        // Single-panel mode
        if app.current_doc.is_some() {
            DocViewer::new(app).render(area, frame.buffer_mut());
        } else {
            frame.render_widget(NavTree::new(app), area);
        }
    }
}

/// Calculate how tall the metadata panel should be based on content
fn metadata_panel_height(app: &App) -> u16 {
    let base = 2; // borders

    let doc = match &app.current_doc {
        Some(d) => d,
        None => return base + 1,
    };

    let fm = match &doc.frontmatter {
        Some(fm) => fm,
        None => return base + 2,
    };

    let mut lines: u16 = 0;

    match fm {
        crate::tui::document::DocumentMetadata::Doc(fm) => {
            // Each field on its own line
            if fm.status.is_some() {
                lines += 1;
            }
            if fm.created.is_some() {
                lines += 1;
            }
            if fm.agent.is_some() {
                lines += 1;
            }
            if fm.confidence.is_some() {
                lines += 1;
            }
            if fm.risk_level.is_some() {
                lines += 1;
            }
            if fm.review_required == Some(true) {
                lines += 1;
            }
            if !fm.tags.is_empty() {
                lines += 1 + fm.tags.len() as u16; // header + one per tag
            }
            // Related: separator + header + one per link
            if !fm.related.is_empty() {
                lines += 2 + fm.related.len() as u16;
            }
        }
        crate::tui::document::DocumentMetadata::Charter(fm) => {
            // Charter ID + Status + Effort + Origin are always rendered.
            // Trigger only when non-empty (always non-empty per schema, but
            // guard anyway).
            if !fm.charter_id.is_empty() {
                lines += 1;
            }
            lines += 1; // Status
            lines += 1; // Effort
            if !fm.trigger.is_empty() {
                lines += 1; // Trigger
            }
            lines += 1; // Origin (single-line summary)

            // Materialized related = ailogs + spec.
            let related_count = fm.originating_ailogs.as_ref().map(|v| v.len()).unwrap_or(0)
                + fm.originating_spec.as_ref().map(|_| 1).unwrap_or(0);
            if related_count > 0 {
                lines += 2 + related_count as u16; // separator + header + entries
            }
        }
    }

    (base + lines).max(base + 1)
}
