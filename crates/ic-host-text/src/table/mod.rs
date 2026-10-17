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
    let headers = escaped_cells(headers);
    let rows = rows
        .iter()
        .map(|row| escaped_cells(row))
        .collect::<Vec<_>>();
    let widths = column_widths(&headers, &rows);
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

fn escaped_cells(cells: &[impl AsRef<str>]) -> Vec<Cow<'_, str>> {
    cells
        .iter()
        .map(|cell| terminal_text(cell.as_ref()))
        .collect()
}

fn column_widths(headers: &[Cow<'_, str>], rows: &[Vec<Cow<'_, str>>]) -> Vec<usize> {
    let mut widths = headers
        .iter()
        .map(|cell| UnicodeWidthStr::width(cell.as_ref()))
        .collect::<Vec<_>>();
    for row in rows {
        widths.resize(widths.len().max(row.len()), 0);
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(UnicodeWidthStr::width(cell.as_ref()));
        }
    }
    widths
}

fn render_row(
    row: &[Cow<'_, str>],
    widths: &[usize],
    alignments: &[Alignment],
    gap: &str,
    output: &mut String,
) {
    let start = output.len();
    append_cells(row, widths, alignments, gap, output);
    // Trim only this row so empty cells cannot erase earlier line separators.
    output.truncate(start + output[start..].trim_end().len());
}

fn append_cells(
    row: &[Cow<'_, str>],
    widths: &[usize],
    alignments: &[Alignment],
    gap: &str,
    output: &mut String,
) {
    for (index, (width, alignment)) in widths.iter().zip(alignments).enumerate() {
        if index != 0 {
            output.push_str(gap);
        }
        let value = row.get(index).map_or("", |cell| cell.as_ref());
        let padding = width.saturating_sub(UnicodeWidthStr::width(value));
        if *alignment == Alignment::Right {
            output.extend(std::iter::repeat_n(' ', padding));
        }
        output.push_str(value);
        if *alignment == Alignment::Left {
            output.extend(std::iter::repeat_n(' ', padding));
        }
    }
}

/// Render a dynamic-column table with ASCII borders and left-aligned cells.
///
/// All cells are terminal-escaped before measuring Unicode display columns.
/// Missing cells are empty; extra cells create unnamed columns. No columns
/// returns empty text. Header-only output retains its separator. No final newline
/// is added. Callers own SQL/value semantics, row construction, budgets and output.
#[must_use]
pub fn render_bordered(headers: &[impl AsRef<str>], rows: &[Vec<impl AsRef<str>>]) -> String {
    let headers = escaped_cells(headers);
    let rows = rows
        .iter()
        .map(|row| escaped_cells(row))
        .collect::<Vec<_>>();
    let widths = column_widths(&headers, &rows);
    if widths.is_empty() {
        return String::new();
    }
    let mut separator = String::from("+");
    for width in &widths {
        separator.extend(std::iter::repeat_n('-', width.saturating_add(2)));
        separator.push('+');
    }
    let alignments = vec![Alignment::Left; widths.len()];
    let mut output = separator.clone();
    let mut append_bordered_row = |row: &[Cow<'_, str>]| {
        output.push_str("\n| ");
        append_cells(row, &widths, &alignments, " | ", &mut output);
        output.push_str(" |");
    };
    append_bordered_row(&headers);
    output.push('\n');
    output.push_str(&separator);
    for row in &rows {
        output.push_str("\n| ");
        append_cells(row, &widths, &alignments, " | ", &mut output);
        output.push_str(" |");
    }
    if !rows.is_empty() {
        output.push('\n');
        output.push_str(&separator);
    }
    output
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
    #[test]
    fn bordered_dynamic_columns_preserve_ascii_layout_and_ragged_cells() {
        assert_eq!(
            render_bordered(&["A", "B"], &[vec!["x", "123"], vec!["long"]]),
            "+------+-----+\n| A    | B   |\n+------+-----+\n| x    | 123 |\n| long |     |\n+------+-----+"
        );
        assert_eq!(
            render_bordered(&["A"], &[vec!["x", "y"]]),
            "+---+---+\n| A |   |\n+---+---+\n| x | y |\n+---+---+"
        );
        assert_eq!(
            render_bordered(&["A"], &Vec::<Vec<&str>>::new()),
            "+---+\n| A |\n+---+"
        );
        assert_eq!(
            render_bordered(&[] as &[&str], &Vec::<Vec<&str>>::new()),
            ""
        );
        assert_eq!(render_bordered(&[] as &[&str], &[vec![] as Vec<&str>]), "");
    }

    #[test]
    fn bordered_tables_escape_before_unicode_alignment() {
        let result = render_bordered(
            &["界", "h\n"],
            &[
                vec!["e\u{301}", "\x1b]52;c;x\x07"],
                vec!["x\ny", "\u{202e}\u{0085}\u{2028}"],
            ],
        );
        let lines = result.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 6);
        assert!(!result.contains('\x1b'));
        assert!(!result.contains('\u{202e}'));
        assert!(result.contains("e\u{301}"));
        assert!(result.contains("x\\ny"));
        assert!(result.contains("h\\n"));
        let width = UnicodeWidthStr::width(lines[0]);
        for line in lines {
            assert_eq!(UnicodeWidthStr::width(line), width);
        }
        // The new dynamic layout and the maintained fixed-column API share
        // escaping and width calculations; plain formatting remains unchanged.
        assert_eq!(
            render(
                &["界", "B"],
                &[["e\u{301}", "x"]],
                &[Alignment::Left; 2],
                Options {
                    column_gap: 1,
                    header_separator: false
                }
            ),
            "界 B\ne\u{301}  x"
        );
    }
}
