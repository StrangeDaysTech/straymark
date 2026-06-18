use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::i18n_strings::t;
use crate::tui::theme;

pub struct HelpPopup<'a> {
    language: &'a str,
}

impl<'a> HelpPopup<'a> {
    pub fn new(language: &'a str) -> Self {
        Self { language }
    }
}

impl Widget for HelpPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup = centered_rect(65, 80, area);
        let lang = self.language;

        Clear.render(popup, buf);

        let title = format!(" {} ", t("Keyboard Shortcuts", lang));
        let block = Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(theme::BORDER_TYPE)
            .border_style(Style::default().fg(theme::ACCENT))
            .style(Style::default().bg(theme::SURFACE));

        let key_style = Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(theme::TEXT);
        let section_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(theme::TEXT_DIM);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("Navigation panel", lang)),
                section_style,
            )),
            help_line("  j / ↓       ", t("Move selection down", lang), key_style, desc_style),
            help_line("  k / ↑       ", t("Move selection up", lang), key_style, desc_style),
            help_line("  Enter       ", t("Expand group / Open document", lang), key_style, desc_style),
            help_line("  Esc         ", t("Collapse group / Clear search", lang), key_style, desc_style),
            help_line("  1-8         ", t("Jump to group by number", lang), key_style, desc_style),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("Metadata panel", lang)),
                section_style,
            )),
            help_line("  j / ↓       ", t("Move between related links", lang), key_style, desc_style),
            help_line("  k / ↑       ", t("Move between related links", lang), key_style, desc_style),
            help_line("  Enter       ", t("Follow selected related link", lang), key_style, desc_style),
            help_line("  Esc         ", t("Back to Navigation", lang), key_style, desc_style),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("Document panel", lang)),
                section_style,
            )),
            help_line("  j / ↓       ", t("Scroll down", lang), key_style, desc_style),
            help_line("  k / ↑       ", t("Scroll up", lang), key_style, desc_style),
            help_line("  g / G       ", t("Top / Bottom of document", lang), key_style, desc_style),
            help_line("  Ctrl+d / u  ", t("Half page down / up", lang), key_style, desc_style),
            help_line("  PgDn / PgUp ", t("Page down / up", lang), key_style, desc_style),
            help_line("  n / N       ", t("Next / Previous document", lang), key_style, desc_style),
            help_line("  f           ", t("Toggle fullscreen", lang), key_style, desc_style),
            help_line("  Esc         ", t("Exit fullscreen / Back to Nav", lang), key_style, desc_style),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("General", lang)),
                section_style,
            )),
            help_line("  Tab         ", t("Next panel: Nav → Meta → Doc", lang), key_style, desc_style),
            help_line("  Shift+Tab   ", t("Prev panel: Doc → Meta → Nav", lang), key_style, desc_style),
            help_line("  /           ", t("Search by name, title, tags, date", lang), key_style, desc_style),
            help_line("  s           ", t("Cycle sort order", lang), key_style, desc_style),
            help_line("  r           ", t("Refresh document index", lang), key_style, desc_style),
            help_line("  L           ", t("Cycle display language", lang), key_style, desc_style),
            help_line("  q           ", t("Quit", lang), key_style, desc_style),
            help_line("  Ctrl+C      ", t("Force quit", lang), key_style, desc_style),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", t("Press any key to close", lang)),
                dim,
            )),
        ];

        let paragraph = Paragraph::new(lines).block(block);
        paragraph.render(popup, buf);
    }
}

fn help_line<'a>(key: &'a str, desc: &'a str, key_style: Style, desc_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, key_style),
        Span::styled(desc, desc_style),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
