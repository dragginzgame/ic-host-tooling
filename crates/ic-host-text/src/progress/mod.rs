//! Pure single-line progress frames; callers own terminal admission and output.

use crate::{escape::terminal_text, width::display_columns};

///
/// `ProgressLine`
///
/// Tracks the last rendered width and constructs carriage-return progress frames.
/// Input controls are escaped before width accounting. Callers write returned
/// frames verbatim and own terminal detection, input limits, clipping to prevent
/// wrapping, output failures, flushing and when to finish or clear the line.
/// Width follows the table/clipping scalar policy, not grapheme-cluster layout.
/// No output or automatic cleanup occurs when this value is dropped.
///

#[derive(Debug, Default)]
pub struct ProgressLine {
    last_width: usize,
}

impl ProgressLine {
    /// Start with no outstanding progress text.
    #[must_use]
    pub const fn new() -> Self {
        Self { last_width: 0 }
    }

    /// Replace the current text, blanking columns left by a longer previous frame.
    #[must_use]
    pub fn update(&mut self, text: &str) -> String {
        self.render(text, false)
    }

    /// Render the final text and newline, then forget the outstanding width.
    #[must_use]
    pub fn finish(&mut self, text: &str) -> String {
        self.render(text, true)
    }

    /// Blank outstanding text and return to its starting column.
    /// Returns an empty string when no columns need clearing.
    #[must_use]
    pub fn clear(&mut self) -> String {
        if self.last_width == 0 {
            return String::new();
        }
        let frame = format!("\r{}\r", " ".repeat(self.last_width));
        self.last_width = 0;
        frame
    }

    fn render(&mut self, text: &str, newline: bool) -> String {
        let text = terminal_text(text);
        let width = display_columns(&text);
        let padding = " ".repeat(self.last_width.saturating_sub(width));
        let mut frame = format!("\r{text}{padding}");
        if newline {
            frame.push('\n');
        }
        self.last_width = if newline { 0 } else { width };
        frame
    }
}

#[cfg(test)]
mod tests;
