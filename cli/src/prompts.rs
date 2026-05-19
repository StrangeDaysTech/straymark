//! Common interactive prompts for `charter close` and `approve`.
//!
//! Wraps `dialoguer` with a consistent theme and adds a few specialized
//! prompts (multiline, optional-string, comma-separated array) that are not
//! direct dialoguer primitives. Detects non-TTY context and bails early so
//! scripted invocation fails fast instead of hanging.
//!
//! All prompts return `anyhow::Result<T>` so callers can `?` propagate Ctrl-C
//! and IO errors uniformly.

use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::io::IsTerminal;

/// Reject early if stdin is not a TTY. Call this once at the start of any
/// command that intends to drive the user through prompts.
pub fn require_interactive() -> Result<()> {
    if !std::io::stdin().is_terminal() {
        bail!(
            "interactive prompts require a TTY. Re-run with --from-template --non-interactive \
             to copy the YAML skeleton and edit it manually."
        );
    }
    Ok(())
}

/// Prompt for a free-text string. `default` is shown as the pre-filled value.
/// Empty input is allowed when `allow_empty` is true; otherwise the prompt
/// re-asks until something non-empty is entered.
pub fn prompt_string(label: &str, default: Option<&str>, allow_empty: bool) -> Result<String> {
    let theme = ColorfulTheme::default();
    let mut builder = Input::<String>::with_theme(&theme).with_prompt(label);
    if let Some(d) = default {
        builder = builder.default(d.to_string());
    }
    builder = builder.allow_empty(allow_empty);
    Ok(builder.interact_text()?)
}

/// Prompt for a non-negative integer. Rejects negative numbers and non-digits.
pub fn prompt_u32(label: &str, default: u32) -> Result<u32> {
    let raw: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .default(default.to_string())
        .validate_with(|input: &String| -> Result<(), String> {
            input
                .trim()
                .parse::<u32>()
                .map(|_| ())
                .map_err(|e| format!("not a non-negative integer: {e}"))
        })
        .interact_text()?;
    Ok(raw.trim().parse::<u32>().expect("validated above"))
}

/// Prompt for a yes/no answer. `default` pre-selects the response.
pub fn prompt_bool(label: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .default(default)
        .interact()?)
}

/// Prompt for one of a fixed set of options, returning the selected option as
/// an owned String. `default_index` pre-selects an option (0-based).
pub fn prompt_enum(label: &str, options: &[&str], default_index: usize) -> Result<String> {
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .items(options)
        .default(default_index.min(options.len().saturating_sub(1)))
        .interact()?;
    Ok(options[idx].to_string())
}

/// Prompt for a comma-separated list of strings. Empty input returns an empty
/// Vec. Trailing commas and inter-element whitespace are tolerated.
pub fn prompt_string_array(label: &str) -> Result<Vec<String>> {
    let raw: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{label} (comma-separated, blank for none)"))
        .allow_empty(true)
        .interact_text()?;
    Ok(raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Prompt for a multi-line block. Reads lines from stdin until a line
/// containing only `.` or EOF (Ctrl-D) is seen. Returns the joined lines
/// without the terminator.
pub fn prompt_multiline(label: &str) -> Result<String> {
    use std::io::{self, BufRead, Write};
    print!("{label} (end with a single '.' on a line, or Ctrl-D):\n");
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let mut buf = String::new();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim() == "." {
            break;
        }
        buf.push_str(&line);
        buf.push('\n');
    }
    // Trim a trailing newline if any but preserve internal newlines.
    Ok(buf.trim_end_matches('\n').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // We cannot test interactive prompts without a TTY. Test the few
    // pure helpers and the array-splitting logic via a private helper.
    fn split_csv(raw: &str) -> Vec<String> {
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    #[test]
    fn split_csv_handles_whitespace_and_empties() {
        assert_eq!(split_csv(""), Vec::<String>::new());
        assert_eq!(split_csv("a"), vec!["a"]);
        assert_eq!(split_csv("a, b , c"), vec!["a", "b", "c"]);
        assert_eq!(split_csv(",a,,b,"), vec!["a", "b"]);
    }

    #[test]
    fn require_interactive_errors_in_non_tty() {
        // Skip when stdin is a TTY (e.g. `cargo test` run from an interactive
        // shell). Under `cargo test` in CI or piped invocations stdin is not a
        // TTY and the assertion below exercises the real code path.
        if std::io::stdin().is_terminal() {
            return;
        }
        let result = require_interactive();
        assert!(result.is_err());
    }
}
