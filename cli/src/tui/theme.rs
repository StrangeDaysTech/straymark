use ratatui::style::Color;
use ratatui::widgets::BorderType;

// Terminal-adaptive palette. The TUI defers to the user's terminal theme:
// it uses only `Color::Reset` (the terminal's default fg/bg) and the 16
// standard ANSI colors, which the terminal remaps per its active theme.
// We never hard-code RGB, so `explore` stays legible on BOTH light and dark
// terminals — the same principle the rest of the CLI already follows.
// Emphasis/selection that needs a "highlight box" uses reverse video
// (Modifier::REVERSED) at the call site instead of a fixed background.

// Background: defer to the terminal's default background.
pub const BG: Color = Color::Reset;
// Surface (panels): also the terminal background — no forced fill.
pub const SURFACE: Color = Color::Reset;
// Text: the terminal's default foreground.
pub const TEXT: Color = Color::Reset;
// Dimmed text: secondary info (counts, dates, badges, hints).
pub const TEXT_DIM: Color = Color::DarkGray;
// Subtle: borders, separators when inactive.
pub const SUBTLE: Color = Color::DarkGray;
// Subgroup labels.
pub const SUBGROUP: Color = Color::Yellow;
// User dir labels.
pub const USER_DIR: Color = Color::Magenta;

// Accent: panel titles, expand/collapse icons, shortcut keys, bullets.
pub const ACCENT: Color = Color::Cyan;
// Active panel border.
pub const BORDER_ACTIVE: Color = Color::Cyan;

// Semantic colors.
pub const GREEN: Color = Color::Green;
pub const YELLOW: Color = Color::Yellow;
pub const RED: Color = Color::Red;

// Border type: rounded corners
pub const BORDER_TYPE: BorderType = BorderType::Rounded;
