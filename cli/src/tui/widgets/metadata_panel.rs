use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use straymark_core::charter::{CharterFrontmatter, CharterStatus, EffortEstimate};
use crate::tui::app::{ActivePanel, App, MetaSelection};
use crate::tui::document::{ConfidenceLevel, DocFrontMatter, DocStatus, DocumentMetadata, RiskLevel};
use crate::tui::i18n_strings::t;
use crate::tui::theme;
use crate::utils::{pad_right_visual, truncate_visual};

/// Visual columns reserved for the field label + colon. Sized to fit the
/// widest English label ("Confidence:" = 11 cols) plus a small gutter.
/// Spanish ("Confianza:") and zh-CN labels (≤ 6 cols) all fit comfortably.
const LABEL_WIDTH: usize = 13;

/// Render a metadata field label padded to LABEL_WIDTH visual columns so
/// values stay aligned across all locales.
fn label_block(label: &str, style: ratatui::style::Style) -> ratatui::text::Span<'static> {
    ratatui::text::Span::styled(format!(" {}", pad_right_visual(label, LABEL_WIDTH)), style)
}

pub struct MetadataPanel<'a> {
    app: &'a App,
}

impl<'a> MetadataPanel<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for MetadataPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_active = self.app.active_panel == ActivePanel::Metadata;
        let lang = self.app.language.as_str();
        let border_style = if is_active {
            Style::default().fg(theme::BORDER_ACTIVE)
        } else {
            Style::default().fg(theme::SUBTLE)
        };

        let block = Block::default()
            .title(format!(" {} ", t("Metadata", lang)))
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

        let doc = match &self.app.current_doc {
            Some(d) => d,
            None => {
                let line = Line::from(Span::styled(
                    format!(" {}", t("No document selected", lang)),
                    Style::default().fg(theme::TEXT_DIM),
                ));
                Paragraph::new(vec![line]).render(inner, buf);
                return;
            }
        };

        let lines: Vec<Line<'static>> = match &doc.frontmatter {
            None => {
                vec![
                    Line::from(vec![
                        Span::styled(
                            format!(" {}  ", t("File:", lang)),
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                        Span::styled(doc.filename.clone(), Style::default().fg(theme::TEXT)),
                    ]),
                    Line::from(Span::styled(
                        format!(" {}", t("No frontmatter", lang)),
                        Style::default().fg(theme::TEXT_DIM),
                    )),
                ]
            }
            Some(DocumentMetadata::Doc(fm)) => doc_lines(fm, lang, self.app, inner.width),
            Some(DocumentMetadata::Charter(fm)) => charter_lines(fm, lang, self.app, inner.width),
        };

        // Empty/no-frontmatter cases bypass scroll arithmetic — just paint.
        if doc.frontmatter.is_none() {
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .render(inner, buf);
            return;
        }

        // Calculate scroll: find the line with ▸ marker to keep it visible
        let total_lines = lines.len() as u16;
        let scroll = if total_lines > inner.height {
            if let Some(selected_pos) = lines.iter().position(|line| {
                line.spans
                    .first()
                    .map(|s| s.content.contains('▸'))
                    .unwrap_or(false)
            }) {
                let sel = selected_pos as u16;
                if sel >= inner.height {
                    sel.saturating_sub(inner.height.saturating_sub(3))
                        .min(total_lines.saturating_sub(inner.height))
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        Paragraph::new(lines)
            .scroll((scroll, 0))
            .render(inner, buf);
    }
}

fn doc_lines(
    fm: &DocFrontMatter,
    lang: &str,
    app: &App,
    inner_width: u16,
) -> Vec<Line<'static>> {
    let l = Style::default().fg(theme::TEXT_DIM);
    let v = Style::default().fg(theme::TEXT);
    let mut lines: Vec<Line<'static>> = vec![Line::from("")];

    // Status
    if let Some(ref status) = fm.status {
        let (indicator, color) = status_style(status);
        lines.push(Line::from(vec![
            label_block(t("Status:", lang), l),
            Span::styled(
                format!("{indicator} {status}"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Created
    if let Some(ref created) = fm.created {
        lines.push(Line::from(vec![
            label_block(t("Created:", lang), l),
            Span::styled(created.clone(), v),
        ]));
    }

    // Agent
    if let Some(ref agent) = fm.agent {
        lines.push(Line::from(vec![
            label_block(t("Agent:", lang), l),
            Span::styled(agent.clone(), v),
        ]));
    }

    // Confidence
    if let Some(ref confidence) = fm.confidence {
        let (filled, total, color, label) = confidence_bar(confidence);
        lines.push(Line::from(vec![
            label_block(t("Confidence:", lang), l),
            Span::styled(
                format!("{}{}", "█".repeat(filled), "░".repeat(total - filled)),
                Style::default().fg(color),
            ),
            Span::styled(format!("  {label}"), Style::default().fg(color)),
        ]));
    }

    // Risk
    if let Some(ref risk) = fm.risk_level {
        let (filled, total, color, label) = risk_bar(risk);
        lines.push(Line::from(vec![
            label_block(t("Risk:", lang), l),
            Span::styled(
                format!("{}{}", "█".repeat(filled), "░".repeat(total - filled)),
                Style::default().fg(color),
            ),
            Span::styled(format!("  {label}"), Style::default().fg(color)),
        ]));
    }

    // Review required
    if let Some(true) = fm.review_required {
        lines.push(Line::from(vec![
            label_block(t("Review:", lang), l),
            Span::styled(
                t("⚠ REQUIRED", lang).to_string(),
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Tags
    if !fm.tags.is_empty() {
        let tag_hint = match app.meta_selection {
            Some(MetaSelection::Tag(_)) => t(" (Enter: search)", lang),
            _ => "",
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {}", t("Tags:", lang)), l),
            Span::styled(tag_hint.to_string(), Style::default().fg(theme::TEXT_DIM)),
        ]));
        let tag_style = Style::default()
            .fg(theme::TEXT_DIM)
            .bg(Color::Rgb(45, 45, 60));
        let tag_selected_style = Style::default()
            .fg(Color::Rgb(220, 224, 242))
            .bg(Color::Rgb(45, 50, 80))
            .add_modifier(Modifier::BOLD);
        for (i, tag) in fm.tags.iter().enumerate() {
            let is_sel = app.meta_selection == Some(MetaSelection::Tag(i));
            let marker = if is_sel { " ▸ " } else { "   " };
            let st = if is_sel { tag_selected_style } else { tag_style };
            lines.push(Line::from(vec![
                Span::styled(marker, l),
                Span::styled(format!(" {tag} "), st),
            ]));
        }
    }

    // Separator before related
    if !fm.related.is_empty() {
        push_related_block(
            &mut lines,
            &fm.related,
            lang,
            app,
            inner_width,
        );
    }

    lines
}

fn charter_lines(
    fm: &CharterFrontmatter,
    lang: &str,
    app: &App,
    inner_width: u16,
) -> Vec<Line<'static>> {
    let l = Style::default().fg(theme::TEXT_DIM);
    let v = Style::default().fg(theme::TEXT);
    let mut lines: Vec<Line<'static>> = vec![Line::from("")];

    // Charter ID
    if !fm.charter_id.is_empty() {
        lines.push(Line::from(vec![
            label_block(t("Charter ID:", lang), l),
            Span::styled(fm.charter_id.clone(), v),
        ]));
    }

    // Status (charter vocabulary)
    let (indicator, status_color) = charter_status_style(&fm.status);
    lines.push(Line::from(vec![
        label_block(t("Status:", lang), l),
        Span::styled(
            format!("{indicator} {}", fm.status.as_str().to_uppercase()),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Effort estimate
    let (effort_color, effort_label) = effort_style(&fm.effort_estimate);
    lines.push(Line::from(vec![
        label_block(t("Effort:", lang), l),
        Span::styled(
            effort_label.to_string(),
            Style::default().fg(effort_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Trigger (one-line, truncated to remaining width)
    if !fm.trigger.is_empty() {
        let max_width = (inner_width as usize)
            .saturating_sub(LABEL_WIDTH + 2);
        let trimmed = fm.trigger.replace('\n', " ");
        lines.push(Line::from(vec![
            label_block(t("Trigger:", lang), l),
            Span::styled(truncate_visual(&trimmed, max_width), v),
        ]));
    }

    // Origin (single-line summary). The materialized "Related" block below
    // makes each origin link navigable; this top line just gives an at-a-glance
    // hint of where the charter came from.
    let origin = straymark_core::charter::display_origin(fm);
    let origin_style = if origin == "—" {
        Style::default().fg(theme::TEXT_DIM)
    } else {
        v
    };
    let origin_max = (inner_width as usize).saturating_sub(LABEL_WIDTH + 2);
    lines.push(Line::from(vec![
        label_block(t("Origin:", lang), l),
        Span::styled(truncate_visual(&origin, origin_max), origin_style),
    ]));

    // Related = originating_ailogs + originating_spec, surfaced through the
    // same MetaSelection::Related mechanism docs use, so Enter still follows
    // the link.
    let mut related: Vec<String> = Vec::new();
    if let Some(ailogs) = &fm.originating_ailogs {
        related.extend(ailogs.iter().cloned());
    }
    if let Some(spec) = &fm.originating_spec {
        related.push(spec.clone());
    }
    if !related.is_empty() {
        push_related_block(&mut lines, &related, lang, app, inner_width);
    }

    lines
}

fn push_related_block(
    lines: &mut Vec<Line<'static>>,
    related: &[String],
    lang: &str,
    app: &App,
    inner_width: u16,
) {
    let l = Style::default().fg(theme::TEXT_DIM);
    let sep_width = inner_width.saturating_sub(2) as usize;
    lines.push(Line::from(Span::styled(
        format!(" {}", "─".repeat(sep_width)),
        Style::default().fg(theme::SUBTLE),
    )));

    let hint = match app.meta_selection {
        Some(MetaSelection::Related(_)) => t(" (Enter: follow)", lang),
        _ => "",
    };
    lines.push(Line::from(vec![
        Span::styled(format!(" {}", t("Related:", lang)), l),
        Span::styled(hint.to_string(), Style::default().fg(theme::TEXT_DIM)),
    ]));

    let max_link_width = inner_width.saturating_sub(4) as usize;
    for (i, rel) in related.iter().enumerate() {
        let is_selected = app.meta_selection == Some(MetaSelection::Related(i));
        let marker = if is_selected { " ▸ " } else { "   " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Rgb(220, 224, 242))
                .bg(Color::Rgb(45, 50, 80))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::UNDERLINED)
        };
        let display = truncate_visual(rel, max_link_width);
        lines.push(Line::from(vec![
            Span::styled(marker, l),
            Span::styled(display, style),
        ]));
    }
}

fn status_style(status: &DocStatus) -> (&'static str, Color) {
    match status {
        DocStatus::Draft => ("○", theme::YELLOW),
        DocStatus::Accepted => ("■", theme::GREEN),
        DocStatus::Deprecated => ("✗", theme::RED),
        DocStatus::Superseded => ("◌", theme::TEXT_DIM),
        DocStatus::Unknown => ("?", theme::TEXT_DIM),
    }
}

fn charter_status_style(status: &CharterStatus) -> (&'static str, Color) {
    match status {
        CharterStatus::Declared => ("○", theme::YELLOW),
        CharterStatus::InProgress => ("◐", theme::ACCENT),
        CharterStatus::Closed => ("■", theme::GREEN),
    }
}

fn effort_style(effort: &EffortEstimate) -> (Color, &'static str) {
    match effort {
        EffortEstimate::Xs => (theme::SUBTLE, "XS"),
        EffortEstimate::S => (theme::GREEN, "S"),
        EffortEstimate::M => (theme::YELLOW, "M"),
        EffortEstimate::L => (theme::RED, "L"),
    }
}

fn confidence_bar(level: &ConfidenceLevel) -> (usize, usize, Color, &'static str) {
    match level {
        ConfidenceLevel::High => (8, 10, theme::GREEN, "high"),
        ConfidenceLevel::Medium => (5, 10, theme::YELLOW, "medium"),
        ConfidenceLevel::Low => (2, 10, theme::RED, "low"),
        ConfidenceLevel::Unknown => (0, 10, theme::TEXT_DIM, "unknown"),
    }
}

fn risk_bar(level: &RiskLevel) -> (usize, usize, Color, &'static str) {
    match level {
        RiskLevel::Low => (2, 10, theme::GREEN, "low"),
        RiskLevel::Medium => (5, 10, theme::YELLOW, "medium"),
        RiskLevel::High => (7, 10, theme::RED, "high"),
        RiskLevel::Critical => (10, 10, theme::RED, "critical"),
        RiskLevel::Unknown => (0, 10, theme::TEXT_DIM, "unknown"),
    }
}
