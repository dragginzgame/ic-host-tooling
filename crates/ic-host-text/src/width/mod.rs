//! Pure clipping to a caller-selected terminal column allowance.
use unicode_width::UnicodeWidthChar;

use crate::escape::terminal_text;

/// Escape controls and fit one line within `columns`, adding `…` when clipped.
///
/// Zero columns return an empty string. Width is the sum of Unicode scalar
/// widths, conservatively counting joined emoji separately. Clipping preserves
/// UTF-8 boundaries, but does not promise grapheme-cluster boundaries. This is
/// display text, not an identity-preserving encoding or an ANSI parser.
/// Callers own input byte limits, terminal dimensions, output and redraw policy.
#[must_use]
pub fn fit_line(line: &str, columns: usize) -> String {
    if columns == 0 {
        return String::new();
    }
    let line = terminal_text(line);
    let total = display_columns(&line);
    if total <= columns {
        return line.into_owned();
    }
    let mut width = 0;
    let mut text = String::new();
    for ch in line.chars() {
        let next = width + ch.width().unwrap_or(0);
        if next >= columns {
            break;
        }
        text.push(ch);
        width = next;
    }
    text.push('…');
    text
}

/// Count display columns in already escaped text using scalar widths.
pub(crate) fn display_columns(text: &str) -> usize {
    text.chars().map(|ch| ch.width().unwrap_or(0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_width_and_small_allowances() {
        for (text, columns, expected) in [
            ("", 0, ""),
            ("", usize::MAX, ""),
            ("abc", 0, ""),
            ("abc", 1, "…"),
            ("abc", 2, "a…"),
            ("abc", 3, "abc"),
            ("abc", usize::MAX, "abc"),
            ("界界", 3, "界…"),
            ("e\u{301}x", 2, "e\u{301}x"),
        ] {
            assert_eq!(fit_line(text, columns), expected);
        }
    }

    #[test]
    fn controls_are_visible_before_width_admission() {
        assert_eq!(fit_line("a\nb", 4), "a\\nb");
        assert_eq!(fit_line("a\nb", 3), "a\\…");
        assert_eq!(fit_line("\x1b[31m", 40), "\\u{1b}[31m");
    }

    #[test]
    fn unicode_output_obeys_each_selected_allowance() {
        for input in ["界界界", "e\u{301}e\u{301}xx", "👩‍💻 ready", "\n\t\u{202e}"] {
            for columns in 0..32 {
                let fitted = fit_line(input, columns);
                assert!(
                    fitted
                        .chars()
                        .map(|ch| ch.width().unwrap_or(0))
                        .sum::<usize>()
                        <= columns
                );
                assert!(!fitted.chars().any(char::is_control));
            }
        }
    }
}
