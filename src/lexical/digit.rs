//! Helpers to convert and add digits from characters.

// Convert u8 to digit.
#[inline]
pub(crate) fn to_digit(c: u8) -> Option<u32> {
    (c as char).to_digit(10)
}

// Add digit to mantissa.
#[inline]
pub(crate) fn add_digit(value: u64, digit: u32) -> Option<u64> {
    return value.checked_mul(10)?.checked_add(digit as u64);
}
