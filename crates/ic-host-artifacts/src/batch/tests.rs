use super::{BatchLabelError, validate_labels};

#[test]
fn exact_labels_preserve_empty_batch_whitespace_and_case() {
    assert_eq!(validate_labels([]), Ok(()));
    assert_eq!(validate_labels(["a", "A", " a", " "]), Ok(()));
    assert_eq!(validate_labels(["é", "e\u{301}"]), Ok(()));
}

#[test]
fn validation_stops_at_the_first_invalid_entry() {
    let mut examined = 0;
    let labels = ["first", "first", "unused"]
        .into_iter()
        .inspect(|_| examined += 1);
    assert_eq!(
        validate_labels(labels),
        Err(BatchLabelError::Duplicate {
            label: "first".into(),
            first_index: 0,
            duplicate_index: 1,
        })
    );
    assert_eq!(examined, 2);
}

#[test]
fn first_invalid_entry_and_original_duplicate_position_are_retained() {
    assert_eq!(
        validate_labels(["a", "", "a"]),
        Err(BatchLabelError::Empty { index: 1 })
    );
    assert_eq!(
        validate_labels(["a", "b", "a", ""]),
        Err(BatchLabelError::Duplicate {
            label: "a".into(),
            first_index: 0,
            duplicate_index: 2,
        })
    );
    assert_eq!(
        validate_labels(["a", "b", "b"]),
        Err(BatchLabelError::Duplicate {
            label: "b".into(),
            first_index: 1,
            duplicate_index: 2,
        })
    );
}
