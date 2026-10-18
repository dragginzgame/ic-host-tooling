//! Shared input-label validation errors for Cargo grouping and artifact batches.

use std::collections::HashMap;

/// Invalid labels, rejected before the operation performs work.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchLabelError {
    /// An input label was empty.
    Empty {
        /// Zero-based position of the empty label.
        index: usize,
    },
    /// Two inputs used the same label.
    Duplicate {
        /// Repeated label.
        label: String,
        /// Position of the first occurrence.
        first_index: usize,
        /// Position of the repeated occurrence.
        duplicate_index: usize,
    },
}

pub(crate) fn validate_labels<'a>(
    labels: impl IntoIterator<Item = &'a str>,
) -> Result<(), BatchLabelError> {
    let mut seen = HashMap::new();
    for (index, label) in labels.into_iter().enumerate() {
        if label.is_empty() {
            return Err(BatchLabelError::Empty { index });
        }
        if let Some(first_index) = seen.insert(label, index) {
            return Err(BatchLabelError::Duplicate {
                label: label.to_owned(),
                first_index,
                duplicate_index: index,
            });
        }
    }
    Ok(())
}

impl std::fmt::Display for BatchLabelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty { index } => write!(formatter, "label at index {index} is empty"),
            Self::Duplicate {
                label,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "label {label:?} at index {duplicate_index} duplicates index {first_index}"
            ),
        }
    }
}

impl std::error::Error for BatchLabelError {}
