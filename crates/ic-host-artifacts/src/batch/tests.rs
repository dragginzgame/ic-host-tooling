use super::{BatchLabelError, validate_labels};

#[test]
fn exact_labels_preserve_empty_batch_whitespace_and_case() {
    assert_eq!(validate_labels([]), Ok(()));
    assert_eq!(validate_labels(["a", "A", " a", " "]), Ok(()));
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
