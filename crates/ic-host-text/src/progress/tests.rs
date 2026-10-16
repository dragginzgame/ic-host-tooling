use super::ProgressLine;

#[test]
fn shorter_updates_blank_only_the_previous_columns() {
    let mut line = ProgressLine::new();
    assert_eq!(line.clear(), "");
    assert_eq!(line.update("working"), "\rworking");
    assert_eq!(line.update("done"), "\rdone   ");
    assert_eq!(line.clear(), "\r    \r");
    assert_eq!(line.clear(), "");
}

#[test]
fn finish_and_empty_updates_reset_the_outstanding_width() {
    let mut line = ProgressLine::default();
    assert_eq!(line.update("long"), "\rlong");
    assert_eq!(line.finish("ok"), "\rok  \n");
    assert_eq!(line.clear(), "");
    assert_eq!(line.update("x"), "\rx");
    assert_eq!(line.update(""), "\r ");
    assert_eq!(line.clear(), "");
    assert_eq!(line.finish(""), "\r\n");
}

#[test]
fn width_uses_unicode_columns_instead_of_scalar_counts() {
    let mut line = ProgressLine::new();
    assert_eq!(line.update("界界"), "\r界界");
    assert_eq!(line.update("e\u{301}"), "\re\u{301}   ");
    assert_eq!(line.clear(), "\r \r");
    assert_eq!(line.update("👩‍💻"), "\r👩‍💻");
    assert_eq!(line.finish("x"), "\rx   \n");
    assert_eq!(line.clear(), "");
}

#[test]
fn input_controls_cannot_inject_progress_frames() {
    let mut line = ProgressLine::new();
    assert_eq!(line.update("\x1b[31m\n\r\t"), "\r\\u{1b}[31m\\n\\r\\t");
    assert_eq!(line.finish("safe"), format!("\rsafe{}\n", " ".repeat(12)));
    assert_eq!(line.clear(), "");
}
