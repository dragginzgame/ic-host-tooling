//! Exact two-place decimal text without token, unit or output policy.

use std::fmt;

#[cfg(test)]
mod tests;

/// Input is not an unsigned ASCII decimal with an optional fractional part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidDecimal;

impl fmt::Display for InvalidDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected unsigned decimal text")
    }
}

impl std::error::Error for InvalidDecimal {}

/// Divide decimal text by `10^exponent` and render exactly two decimal places.
///
/// Uses exact round-half-up arithmetic on text, including values larger than
/// `u128`. Input accepts leading zeroes and an optional decimal point; signs,
/// whitespace and separators refuse. At least one whole digit is required;
/// a trailing point is accepted. No numeric conversion or unit selection.
///
/// # Errors
/// Returns [`InvalidDecimal`] for invalid syntax.
pub fn fixed_decimal_text(value: &str, exponent: usize) -> Result<String, InvalidDecimal> {
    render(value, exponent, false)
}

/// As [`fixed_decimal_text`], trimming trailing fractional zeroes and the point.
///
/// # Errors
/// Returns [`InvalidDecimal`] for invalid syntax.
pub fn compact_decimal_text(value: &str, exponent: usize) -> Result<String, InvalidDecimal> {
    render(value, exponent, true)
}

fn render(value: &str, exponent: usize, compact: bool) -> Result<String, InvalidDecimal> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.bytes().all(|digit| digit.is_ascii_digit())
        || !fraction.bytes().all(|digit| digit.is_ascii_digit())
    {
        return Err(InvalidDecimal);
    }
    let whole = whole.trim_start_matches('0');
    let (integer, integer_fraction) = if whole.len() > exponent {
        whole.split_at(whole.len() - exponent)
    } else {
        ("0", whole)
    };
    let mut digits = std::iter::repeat_n(b'0', exponent.saturating_sub(whole.len()))
        .chain(integer_fraction.bytes())
        .chain(fraction.bytes());
    let tenths = digits.next().unwrap_or(b'0') - b'0';
    let hundredth = digits.next().unwrap_or(b'0') - b'0';
    let mut hundredths = tenths * 10 + hundredth;
    if digits.next().unwrap_or(b'0') >= b'5' {
        hundredths += 1;
    }
    let mut integer = integer.to_string();
    if hundredths == 100 {
        increment(&mut integer);
        hundredths = 0;
    }
    if !compact || hundredths != 0 {
        use std::fmt::Write as _;
        write!(integer, ".{hundredths:02}").expect("writing to a String cannot fail");
        if compact && hundredths.is_multiple_of(10) {
            integer.pop();
        }
    }
    Ok(integer)
}

fn increment(value: &mut String) {
    let mut digits = value.as_bytes().to_vec();
    for digit in digits.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            *value = String::from_utf8(digits).expect("decimal digits remain UTF-8");
            return;
        }
        *digit = b'0';
    }
    digits.insert(0, b'1');
    *value = String::from_utf8(digits).expect("decimal digits remain UTF-8");
}
