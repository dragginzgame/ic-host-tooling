//! Unicode display-width alignment over escaped caller-selected cells.
use crate::escape::terminal_text;
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// Caller-selected alignment of one complete column.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alignment {
    /// Pad after the cell.
    Left,
    /// Pad before the cell.
    Right,
}

/// Explicit formatting choices; no ambient terminal policy or defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Options {
    /// Spaces between columns.
    pub column_gap: usize,
    /// Whether a dashed line follows the headers.
    pub header_separator: bool,
}

/// Render headers and rows after escaping every cell, including headers.
///
/// Widths use Unicode terminal display columns, not byte or character counts.
/// Rows have the same column count by type. Header/data lines omit trailing
/// whitespace; the result has no final newline. Zero columns return empty text.
/// Caller owns input/output size, row construction, headers, column choices,
/// truncation and all output, terminal detection, colour and progress behavior.
/// Storage allocation follows ordinary String/Vec allocation semantics.
#[must_use]
pub fn render<const N: usize>(
    headers: &[impl AsRef<str>; N],
    rows: &[[impl AsRef<str>; N]],
    alignments: &[Alignment; N],
    options: Options,
) -> String {
    if N == 0 {
        return String::new();
    }
    let headers = headers.each_ref().map(|cell| terminal_text(cell.as_ref()));
    let rows = rows
        .iter()
        .map(|row| row.each_ref().map(|cell| terminal_text(cell.as_ref())))
        .collect::<Vec<_>>();
    let widths = std::array::from_fn(|index| {
        rows.iter().fold(
            UnicodeWidthStr::width(headers[index].as_ref()),
            |width, row| width.max(UnicodeWidthStr::width(row[index].as_ref())),
        )
    });
    let gap = " ".repeat(options.column_gap);
    let mut output = String::new();
    render_row(&headers, &widths, alignments, &gap, &mut output);
    if options.header_separator {
        output.push('\n');
        for (index, width) in widths.iter().enumerate() {
            if index != 0 {
                output.push_str(&gap);
            }
            output.extend(std::iter::repeat_n('-', *width));
        }
    }
    for row in &rows {
        output.push('\n');
        render_row(row, &widths, alignments, &gap, &mut output);
    }
    output
}

fn render_row<const N: usize>(
    row: &[Cow<'_, str>; N],
    widths: &[usize; N],
    alignments: &[Alignment; N],
    gap: &str,
    output: &mut String,
) {
    let start = output.len();
    for (index, ((value, width), alignment)) in row.iter().zip(widths).zip(alignments).enumerate() {
        if index != 0 {
            output.push_str(gap);
        }
        let padding = width.saturating_sub(UnicodeWidthStr::width(value.as_ref()));
        if *alignment == Alignment::Right {
            output.extend(std::iter::repeat_n(' ', padding));
        }
        output.push_str(value);
        if *alignment == Alignment::Left {
            output.extend(std::iter::repeat_n(' ', padding));
        }
    }
    // Trim only this row so empty cells cannot erase earlier line separators.
    output.truncate(start + output[start..].trim_end().len());
}

#[cfg(test)]
mod tests {
    use super::*;
    const OPTIONS: Options = Options {
        column_gap: 2,
        header_separator: true,
    };

    #[test]
    fn wide_and_combining_cells_align_in_display_columns() {
        assert_eq!(
            render(
                &["NAME", "N"],
                &[["界", "7"], ["e\u{301}", "123"]],
                &[Alignment::Left, Alignment::Right],
                OPTIONS
            ),
            "NAME    N\n----  ---\n界      7\ne\u{301}     123"
        );
    }

    #[test]
    fn escaped_headers_and_cells_are_measured_and_stay_on_their_rows() {
        let text = render(
            &["H\n"],
            &[["x\x1b[2J\t\u{2029}"]],
            &[Alignment::Left],
            OPTIONS,
        );
        assert_eq!(text.lines().count(), 3);
        assert!(text.starts_with("H\\n\n"));
        assert!(text.ends_with("x\\u{1b}[2J\\t\\u{2029}"));
        assert!(!text.contains('\x1b'));
    }

    #[test]
    fn whitespace_cells_preserve_empty_rows_and_separator_spacing() {
        assert_eq!(
            render(
                &["  ", "B "],
                &[["", ""], ["x ", "\u{2003}"], ["界", "7"]],
                &[Alignment::Left, Alignment::Right],
                OPTIONS,
            ),
            "    B\n--  --\n\nx\n界   7"
        );
        assert_eq!(
            render(&["H"], &[[""], [""]], &[Alignment::Left], OPTIONS),
            "H\n-\n\n"
        );
        assert_eq!(
            render(&["", ""], &[["", ""]], &[Alignment::Left; 2], OPTIONS),
            "\n  \n"
        );
    }

    #[test]
    fn explicit_options_and_empty_tables_have_no_hidden_defaults() {
        assert_eq!(
            render(
                &["A", "B"],
                &[["1", "2"]],
                &[Alignment::Left; 2],
                Options {
                    column_gap: 0,
                    header_separator: false
                }
            ),
            "AB\n12"
        );
        assert_eq!(
            render(&["A"], &[] as &[[&str; 1]], &[Alignment::Left], OPTIONS),
            "A\n-"
        );
        assert_eq!(
            render(&[] as &[&str; 0], &[] as &[[&str; 0]], &[], OPTIONS),
            ""
        );
    }
}
