use super::algorithm::*;
use super::digit::*;
use super::exponent::*;
use super::num::*;

// PARSERS
// -------

/// Parse the significant digits of the float.
///
/// * `integer`     - Slice containing the integer digits.
/// * `fraction`    - Slice containing the fraction digits.
fn parse_mantissa(integer: &[u8], fraction: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    // On overflow, calculate the number of truncated digits.
    let mut integer = integer.iter();
    while let Some(c) = integer.next() {
        value = match add_digit(value, to_digit(*c).unwrap()) {
            Some(v) => v,
            None => return (value, 1 + integer.count() + fraction.len()),
        };
    }
    let mut fraction = fraction.iter();
    while let Some(c) = fraction.next() {
        value = match add_digit(value, to_digit(*c).unwrap()) {
            Some(v) => v,
            None => return (value, 1 + fraction.count()),
        };
    }
    (value, 0)
}

/// Parse float from extracted float components.
///
/// * `integer`     - Slice containing the integer digits.
/// * `fraction`    - Slice containing the fraction digits.
/// * `exponent`    - Parsed, 32-bit exponent.
///
/// Precondition: The integer must not have leading zeros.
pub fn parse_float<F>(integer: &[u8], mut fraction: &[u8], exponent: i32) -> F
where
    F: Float,
{
    // Trim trailing zeroes from the fraction part.
    while fraction.last() == Some(&b'0') {
        fraction = &fraction[..fraction.len() - 1];
    }

    // Parse the mantissa and attempt the fast and moderate-path algorithms.
    let (mantissa, truncated) = parse_mantissa(integer, fraction);

    if mantissa == 0 {
        // Literal 0, return early. Value cannot be truncated since truncation
        // only occurs on overflow or underflow.
        return F::ZERO;
    }

    let mant_exp = mantissa_exponent(exponent, fraction.len(), truncated);

    // Try the fast path if no mantissa truncation.
    let is_truncated = truncated != 0;
    if !is_truncated {
        if let Some(float) = fast_path(mantissa, mant_exp) {
            return float;
        }
    }

    fallback_path(
        integer,
        fraction,
        mantissa,
        exponent,
        mant_exp,
        is_truncated,
    )
}
