use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::tui::app::{ActivePanel, App};
use crate::tui::i18n_strings::t;
use crate::tui::markdown::markdown_to_lines;
use crate::tui::theme;

pub struct DocViewer<'a> {
    app: &'a mut App,
}

impl<'a> DocViewer<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let is_active = self.app.active_panel == ActivePanel::Document;
        let border_style = if is_active {
            Style::default().fg(theme::BORDER_ACTIVE)
        } else {
            Style::default().fg(theme::SUBTLE)
        };

        let lang = self.app.language.as_str();
        let title = match &self.app.current_doc {
            Some(doc) => format!(" {} ", doc.filename),
            None => format!(" {} ", t("Document", lang)),
        };

        let block = Block::default()
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Center)
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

        if self.app.current_doc.is_none() {
            let fallback_info = if self.app.is_fallback {
                Some(self.app.project_root.display().to_string())
            } else {
                None
            };
            let welcome = render_welcome(self.app.index.total_docs, fallback_info, lang);
            let paragraph = Paragraph::new(welcome);
            paragraph.render(inner, buf);
            return;
        }

        let doc = self.app.current_doc.as_ref().unwrap();

        // Reserve 1 column on the right for the scrollbar so it doesn't
        // overlap the document text. Without this, the uncovered track
        // leaks the underlying text through where there is no thumb.
        let reserve_scrollbar = inner.width >= 2;
        let body_area = if reserve_scrollbar {
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width - 1,
                height: inner.height,
            }
        } else {
            inner
        };
        let scrollbar_area = if reserve_scrollbar {
            Rect {
                x: inner.x + inner.width - 1,
                y: inner.y,
                width: 1,
                height: inner.height,
            }
        } else {
            inner
        };

        // Render markdown body only (metadata is in separate panel)
        let body_width = body_area.width.max(1) as usize;
        let mut all_lines = vec![Line::from(""); 2];
        all_lines.extend(markdown_to_lines(&doc.body, body_width));

        // Estimate total lines accounting for wrapping, measured in visual
        // columns (CJK and other double-wide chars count as 2).
        let wrapped_count: usize = all_lines
            .iter()
            .map(|line| {
                let line_width: usize = line
                    .spans
                    .iter()
                    .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                    .sum();
                if line_width == 0 {
                    1
                } else {
                    line_width.div_ceil(body_width)
                }
            })
            .sum();
        self.app.doc_total_lines = wrapped_count;

        let text = Text::from(all_lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((self.app.doc_scroll, 0));
        paragraph.render(body_area, buf);

        // Render scrollbar in the reserved column.
        if self.app.doc_total_lines > body_area.height as usize {
            let mut scrollbar_state = ScrollbarState::new(self.app.doc_total_lines)
                .viewport_content_length(body_area.height as usize)
                .position(self.app.doc_scroll as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }
    }
}

fn render_welcome(
    total_docs: usize,
    fallback_path: Option<String>,
    lang: &str,
) -> Vec<Line<'static>> {
    let title = Style::default()
        .fg(theme::ACCENT)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(theme::SUBTLE);
    let key = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let text = Style::default().fg(theme::TEXT);

    let mut lines = vec![
        Line::from(""),
        Line::from(""),
        // Brand name stays English in every locale.
        Line::from(Span::styled("  StrayMark Explorer", title)),
        Line::from(Span::styled(
            format!(
                "  {}",
                t("Documentation Governance for AI-Assisted Development", lang)
            ),
            dim,
        )),
    ];

    if let Some(ref path) = fallback_path {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  → {}", t("Using repo root: ", lang)),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(path.clone(), text),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", t("Total documents: ", lang)), dim),
        Span::styled(
            total_docs.to_string(),
            text.add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", t("Quick start", lang)),
        title,
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("    ↑↓ ", key),
        Span::styled(t("Navigate groups in the left panel", lang).to_string(), text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Enter ", key),
        Span::styled(
            t("Expand a group and open a document", lang).to_string(),
            text,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   Tab  ", key),
        Span::styled(t("Next panel / ", lang).to_string(), text),
        Span::styled("Shift+Tab  ", key),
        Span::styled(t("Previous panel", lang).to_string(), text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("     /  ", key),
        Span::styled(
            t("Search by filename, title, tags, or date", lang).to_string(),
            text,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("     f  ", key),
        Span::styled(t("Toggle document fullscreen", lang).to_string(), text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("     ?  ", key),
        Span::styled(t("Show all keyboard shortcuts", lang).to_string(), text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("     q  ", key),
        Span::styled(t("Quit", lang).to_string(), text),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ─────────────────────────────────────────────",
        dim,
    )));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", t("Developed by ", lang)), dim),
        // Company name stays in its canonical form.
        Span::styled("Strange Days Tech, S.A.S.", text),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "https://strangedays.tech",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
        ),
    ]));

    lines
}
