//! Fallible capacity growth within caller-selected element allowances.

use std::collections::TryReserveError;

#[cfg(test)]
mod tests;

/// Reserve storage for up to `additional` more elements within `limit`.
///
/// Returns the number of elements the caller may append. Existing elements are
/// unchanged; a vector already at or above the limit admits no additional items.
/// Capacity grows geometrically, without requesting capacity beyond the limit.
/// An allocator may provide more capacity than requested, and existing capacity
/// is never reduced. The limit counts elements, not their size in bytes.
///
/// This does not append, truncate, or select an overflow policy. Callers must
/// append no more than the returned count and retain their own rejection or
/// truncation behavior.
///
/// # Errors
/// Returns the original allocation or capacity-overflow failure.
pub fn reserve_bounded<T>(
    items: &mut Vec<T>,
    additional: usize,
    limit: usize,
) -> Result<usize, TryReserveError> {
    let admitted = additional.min(limit.saturating_sub(items.len()));
    let required = items.len() + admitted;
    if required > items.capacity() {
        let capacity = items.capacity().saturating_mul(2).max(required).min(limit);
        items.try_reserve_exact(capacity - items.len())?;
    }
    Ok(admitted)
}
