//! Escape untrusted scalar text without selecting terminal or colour policy.
use std::{borrow::Cow, fmt::Write as _};

/// Escape terminal controls, Unicode line separators and bidirectional controls.
///
/// LF, CR and tab become visible `\n`, `\r` and `\t`; other unsafe characters
/// become `\u{hex}`. Printable Unicode and literal backslashes remain unchanged.
/// This is a presentation, not an identity-preserving or reversible encoding.
/// It performs no terminal detection, output, truncation or size admission.
/// Text needing no escapes is borrowed; call `.into_owned()` when independent
/// ownership is required. Changed text uses ordinary String allocation semantics.
#[must_use]
pub fn terminal_text(value: &str) -> Cow<'_, str> {
    let Some(start) = value.find(needs_escape) else {
        return Cow::Borrowed(value);
    };
    let mut escaped = String::with_capacity(value.len());
    escaped.push_str(&value[..start]);
    for character in value[start..].chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if needs_escape(character) => {
                write!(escaped, "\\u{{{:x}}}", u32::from(character))
                    .expect("String writes cannot fail");
            }
            character => escaped.push(character),
        }
    }
    Cow::Owned(escaped)
}

fn needs_escape(character: char) -> bool {
    character.is_control()
        || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}'
        | '\u{2028}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_cannot_create_terminal_commands_or_report_lines() {
        assert_eq!(
            terminal_text("a\n\r\t\0\x1b[31m\u{009b}\u{2028}\u{202e}\u{2069}"),
            "a\\n\\r\\t\\u{0}\\u{1b}[31m\\u{9b}\\u{2028}\\u{202e}\\u{2069}"
        );
        assert_eq!(
            terminal_text("界 e\u{301} 👩‍💻\u{202e}x"),
            "界 e\u{301} 👩‍💻\\u{202e}x"
        );
    }

    #[test]
    fn printable_unicode_spaces_and_literal_backslashes_are_preserved() {
        let text = "  界 e\u{301} 👩‍💻 \\n  ";
        let escaped = terminal_text(text);
        assert_eq!(escaped, text);
        assert!(matches!(escaped, Cow::Borrowed(_)));
        assert!(matches!(terminal_text(""), Cow::Borrowed("")));
    }
}
