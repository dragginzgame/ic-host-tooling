//! Cargo flag composition; callers retain flag selection and build policy.

#[cfg(test)]
mod tests;

/// Append arguments to explicitly supplied inherited Rust flags.
///
/// `CARGO_ENCODED_RUSTFLAGS` takes precedence over `RUSTFLAGS`, including when
/// explicitly empty. Ordinary flags are split on whitespace, matching Cargo;
/// additions retain their argument boundaries through unit-separator encoding.
/// Callers resolve their command environment and treat non-Unicode values as
/// unavailable before passing them here. No ambient environment is read.
///
/// Returns `None` when `extra_flags` is empty:
/// leave the command's inherited flags unchanged in that case. Otherwise set
/// the returned string as `CARGO_ENCODED_RUSTFLAGS` on the Cargo command. The
/// caller chooses additions, target-specific flags and the command environment.
#[must_use]
pub fn combined_encoded_rustflags(
    encoded: Option<&str>,
    ordinary: Option<&str>,
    extra_flags: &[String],
) -> Option<String> {
    if extra_flags.is_empty() {
        return None;
    }

    let mut combined = encoded.map_or_else(
        || {
            let mut encoded = String::new();
            for flag in ordinary.unwrap_or_default().split_whitespace() {
                if !encoded.is_empty() {
                    encoded.push('\x1f');
                }
                encoded.push_str(flag);
            }
            encoded
        },
        str::to_owned,
    );
    for flag in extra_flags {
        if !combined.is_empty() {
            combined.push('\x1f');
        }
        combined.push_str(flag);
    }

    Some(combined)
}
