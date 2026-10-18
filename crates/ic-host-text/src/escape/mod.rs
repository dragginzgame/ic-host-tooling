//! Escape untrusted scalar text without selecting terminal or colour policy.
use std::borrow::Cow;

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
    escaped.extend(escaped_chars(&value[start..]));
    Cow::Owned(escaped)
}

// Yield the presentation encoding without allocating a complete escaped copy.
pub(crate) fn escaped_chars(value: &str) -> impl Iterator<Item = char> + '_ {
    let mut characters = value.chars();
    let mut short = None;
    let mut unicode: Option<std::char::EscapeUnicode> = None;
    std::iter::from_fn(move || {
        if let Some(character) = short.take() {
            return Some(character);
        }
        if let Some(escaped) = &mut unicode
            && let Some(character) = escaped.next()
        {
            return Some(character);
        }
        unicode = None;
        let character = characters.next()?;
        match character {
            '\n' | '\r' | '\t' => {
                short = Some(match character {
                    '\n' => 'n',
                    '\r' => 'r',
                    _ => 't',
                });
                Some('\\')
            }
            character if needs_escape(character) => {
                let mut escaped = character.escape_unicode();
                let first = escaped.next();
                unicode = Some(escaped);
                first
            }
            character => Some(character),
        }
    })
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
