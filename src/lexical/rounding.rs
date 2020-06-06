//! Defines rounding schemes for floating-point numbers.

use crate::lib::mem;
use super::float::ExtendedFloat;
use super::num::*;
use super::shift::*;

// MASKS

/// Calculate a scalar factor of 2 above the halfway point.
#[inline]
fn nth_bit(n: u64) -> u64
{
    let bits: u64 = mem::size_of::<u64>() as u64 * 8;
    debug_assert!(n < bits, "nth_bit() overflow in shl.");

    1 << n
}

/// Generate a bitwise mask for the lower `n` bits.
#[inline]
pub(crate) fn lower_n_mask(n: u64) -> u64
{
    let bits: u64 = mem::size_of::<u64>() as u64 * 8;
    debug_assert!(n <= bits, "lower_n_mask() overflow in shl.");

    match n == bits {
        true  => u64::max_value(),
        false => (1 << n) - 1,
    }
}

/// Calculate the halfway point for the lower `n` bits.
#[inline]
pub(crate) fn lower_n_halfway(n: u64) -> u64
{
    let bits: u64 = mem::size_of::<u64>() as u64 * 8;
    debug_assert!(n <= bits, "lower_n_halfway() overflow in shl.");

    match n == 0 {
        true  => 0,
        false => nth_bit(n - 1),
    }
}

/// Calculate a bitwise mask with `n` 1 bits starting at the `bit` position.
#[inline]
fn internal_n_mask(bit: u64, n: u64) -> u64 {
    let bits: u64 = mem::size_of::<u64>() as u64 * 8;
    debug_assert!(bit <= bits, "internal_n_halfway() overflow in shl.");
    debug_assert!(n <= bits, "internal_n_halfway() overflow in shl.");
    debug_assert!(bit >= n, "internal_n_halfway() overflow in sub.");

    lower_n_mask(bit) ^ lower_n_mask(bit - n)
}

// NEAREST ROUNDING

// Shift right N-bytes and round to the nearest.
//
// Return if we are above halfway and if we are halfway.
#[inline]
pub(crate) fn round_nearest(fp: &mut ExtendedFloat, shift: i32)
    -> (bool, bool)
{
    // Extract the truncated bits using mask.
    // Calculate if the value of the truncated bits are either above
    // the mid-way point, or equal to it.
    //
    // For example, for 4 truncated bytes, the mask would be b1111
    // and the midway point would be b1000.
    let mask: u64 = lower_n_mask(shift as u64);
    let halfway: u64 = lower_n_halfway(shift as u64);

    let truncated_bits = fp.mant & mask;
    let is_above = truncated_bits > halfway;
    let is_halfway = truncated_bits == halfway;

    // Bit shift so the leading bit is in the hidden bit.
    overflowing_shr(fp, shift);

    (is_above, is_halfway)
}

// Tie rounded floating point to event.
#[inline]
pub(crate) fn tie_even(fp: &mut ExtendedFloat, is_above: bool, is_halfway: bool)
{
    // Extract the last bit after shifting (and determine if it is odd).
    let is_odd = fp.mant & 1 == 1;

    // Calculate if we need to roundup.
    // We need to roundup if we are above halfway, or if we are odd
    // and at half-way (need to tie-to-even).
    if is_above || (is_odd && is_halfway) {
        fp.mant += 1;
    }
}

// Shift right N-bytes and round nearest, tie-to-even.
//
// Floating-point arithmetic uses round to nearest, ties to even,
// which rounds to the nearest value, if the value is halfway in between,
// round to an even value.
#[inline]
pub(crate) fn round_nearest_tie_even(fp: &mut ExtendedFloat, shift: i32) {
    let (is_above, is_halfway) = round_nearest(fp, shift);
    tie_even(fp, is_above, is_halfway);
}

// DIRECTED ROUNDING

// Shift right N-bytes and round towards a direction.
//
// Return if we have any truncated bytes.
#[inline]
fn round_toward(fp: &mut ExtendedFloat, shift: i32) -> bool
{
    let mask: u64 = lower_n_mask(shift as u64);
    let truncated_bits = fp.mant & mask;

    // Bit shift so the leading bit is in the hidden bit.
    overflowing_shr(fp, shift);

    truncated_bits != 0
}

// Round down.
#[inline]
fn downard(_: &mut ExtendedFloat, _: bool)
{}

// Shift right N-bytes and round toward zero.
//
// Floating-point arithmetic defines round toward zero, which rounds
// towards positive zero.
#[inline]
pub(crate) fn round_downward(fp: &mut ExtendedFloat, shift: i32)
{
    // Bit shift so the leading bit is in the hidden bit.
    // No rounding schemes, so we just ignore everything else.
    let is_truncated = round_toward(fp, shift);
    downard(fp, is_truncated);
}

// ROUND TO FLOAT

// Shift the ExtendedFloat fraction to the fraction bits in a native float.
//
// Floating-point arithmetic uses round to nearest, ties to even,
// which rounds to the nearest value, if the value is halfway in between,
// round to an even value.
#[inline]
fn round_to_float<F, Algorithm>(fp: &mut ExtendedFloat, algorithm: Algorithm)
    where F: Float,
          Algorithm: FnOnce(&mut ExtendedFloat, i32)
{
    // Calculate the difference to allow a single calculation
    // rather than a loop, to minimize the number of ops required.
    // This does underflow detection.
    let final_exp = fp.exp + F::DEFAULT_SHIFT;
    if final_exp < F::DENORMAL_EXPONENT {
        // We would end up with a denormal exponent, try to round to more
        // digits. Only shift right if we can avoid zeroing out the value,
        // which requires the exponent diff to be < M::BITS. The value
        // is already normalized, so we shouldn't have any issue zeroing
        // out the value.
        let diff = F::DENORMAL_EXPONENT - fp.exp;
        if diff <= u64::FULL {
            // We can avoid underflow, can get a valid representation.
            algorithm(fp, diff);
        } else {
            // Certain underflow, assign literal 0s.
            fp.mant = 0;
            fp.exp = 0;
        }
    } else {
        algorithm(fp, F::DEFAULT_SHIFT);
    }

    if fp.mant & F::CARRY_MASK == F::CARRY_MASK {
        // Roundup carried over to 1 past the hidden bit.
        shr(fp, 1);
    }
}

// AVOID OVERFLOW/UNDERFLOW

// Avoid overflow for large values, shift left as needed.
//
// Shift until a 1-bit is in the hidden bit, if the mantissa is not 0.
#[inline]
fn avoid_overflow<F>(fp: &mut ExtendedFloat)
    where F: Float
{
    // Calculate the difference to allow a single calculation
    // rather than a loop, minimizing the number of ops required.
    if fp.exp >= F::MAX_EXPONENT {
        let diff = fp.exp - F::MAX_EXPONENT;
        if diff <= F::MANTISSA_SIZE {
            // Our overflow mask needs to start at the hidden bit, or at
            // `F::MANTISSA_SIZE+1`, and needs to have `diff+1` bits set,
            // to see if our value overflows.
            let bit = (F::MANTISSA_SIZE + 1).as_u64();
            let n = (diff + 1).as_u64();
            let mask = internal_n_mask(bit, n);
            if (fp.mant & mask) == 0 {
                // If we have no 1-bit in the hidden-bit position,
                // which is index 0, we need to shift 1.
                let shift = diff + 1;
                shl(fp, shift);
            }
        }
    }
}

// ROUND TO NATIVE

// Round an extended-precision float to a native float representation.
#[inline]
pub(crate) fn round_to_native<F, Algorithm>(fp: &mut ExtendedFloat, algorithm: Algorithm)
    where F: Float,
          Algorithm: FnOnce(&mut ExtendedFloat, i32)
{
    // Shift all the way left, to ensure a consistent representation.
    // The following right-shifts do not work for a non-normalized number.
    fp.normalize();

    // Round so the fraction is in a native mantissa representation,
    // and avoid overflow/underflow.
    round_to_float::<F, _>(fp, algorithm);
    avoid_overflow::<F>(fp);
}

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    // MASKS

    #[test]
    fn lower_n_mask_test() {
        assert_eq!(lower_n_mask(0u64), 0b0);
        assert_eq!(lower_n_mask(1u64), 0b1);
        assert_eq!(lower_n_mask(2u64), 0b11);
        assert_eq!(lower_n_mask(10u64), 0b1111111111);
        assert_eq!(lower_n_mask(32u64), 0b11111111111111111111111111111111);
    }

    #[test]
    fn lower_n_halfway_test() {
        assert_eq!(lower_n_halfway(0u64), 0b0);
        assert_eq!(lower_n_halfway(1u64), 0b1);
        assert_eq!(lower_n_halfway(2u64), 0b10);
        assert_eq!(lower_n_halfway(10u64), 0b1000000000);
        assert_eq!(lower_n_halfway(32u64), 0b10000000000000000000000000000000);
    }

    #[test]
    fn nth_bit_test() {
        assert_eq!(nth_bit(0u64), 0b1);
        assert_eq!(nth_bit(1u64), 0b10);
        assert_eq!(nth_bit(2u64), 0b100);
        assert_eq!(nth_bit(10u64), 0b10000000000);
        assert_eq!(nth_bit(31u64), 0b10000000000000000000000000000000);
    }

    #[test]
    fn internal_n_mask_test() {
        assert_eq!(internal_n_mask(1u64, 0u64), 0b0);
        assert_eq!(internal_n_mask(1u64, 1u64), 0b1);
        assert_eq!(internal_n_mask(2u64, 1u64), 0b10);
        assert_eq!(internal_n_mask(4u64, 2u64), 0b1100);
        assert_eq!(internal_n_mask(10u64, 2u64), 0b1100000000);
        assert_eq!(internal_n_mask(10u64, 4u64), 0b1111000000);
        assert_eq!(internal_n_mask(32u64, 4u64), 0b11110000000000000000000000000000);
    }

    // NEAREST ROUNDING

    #[test]
    fn round_nearest_test() {
        // Check exactly halfway (b'1100000')
        let mut fp = ExtendedFloat { mant: 0x60, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, 6);
        assert!(!above);
        assert!(halfway);
        assert_eq!(fp.mant, 1);

        // Check above halfway (b'1100001')
        let mut fp = ExtendedFloat { mant: 0x61, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, 6);
        assert!(above);
        assert!(!halfway);
        assert_eq!(fp.mant, 1);

        // Check below halfway (b'1011111')
        let mut fp = ExtendedFloat { mant: 0x5F, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, 6);
        assert!(!above);
        assert!(!halfway);
        assert_eq!(fp.mant, 1);
    }

    // DIRECTED ROUNDING

    #[test]
    fn round_downward_test() {
        // b0000000
        let mut fp = ExtendedFloat { mant: 0x00, exp: 0 };
        round_downward(&mut fp, 6);
        assert_eq!(fp.mant, 0);

        // b1000000
        let mut fp = ExtendedFloat { mant: 0x40, exp: 0 };
        round_downward(&mut fp, 6);
        assert_eq!(fp.mant, 1);

        // b1100000
        let mut fp = ExtendedFloat { mant: 0x60, exp: 0 };
        round_downward(&mut fp, 6);
        assert_eq!(fp.mant, 1);

        // b1110000
        let mut fp = ExtendedFloat { mant: 0x70, exp: 0 };
        round_downward(&mut fp, 6);
        assert_eq!(fp.mant, 1);
    }

    #[test]
    fn round_nearest_tie_even_test() {
        // Check round-up, halfway
        let mut fp = ExtendedFloat { mant: 0x60, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 2);

        // Check round-down, halfway
        let mut fp = ExtendedFloat { mant: 0x20, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 0);

        // Check round-up, above halfway
        let mut fp = ExtendedFloat { mant: 0x61, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 2);

        let mut fp = ExtendedFloat { mant: 0x21, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 1);

        // Check round-down, below halfway
        let mut fp = ExtendedFloat { mant: 0x5F, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 1);

        let mut fp = ExtendedFloat { mant: 0x1F, exp: 0 };
        round_nearest_tie_even(&mut fp, 6);
        assert_eq!(fp.mant, 0);
    }

    // HIGH-LEVEL

    #[test]
    fn round_to_float_test() {
        // Denormal
        let mut fp = ExtendedFloat { mant: 1<<63, exp: f64::DENORMAL_EXPONENT - 15 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<48);
        assert_eq!(fp.exp, f64::DENORMAL_EXPONENT);

        // Halfway, round-down (b'1000000000000000000000000000000000000000000000000000010000000000')
        let mut fp = ExtendedFloat { mant: 0x8000000000000400, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<52);
        assert_eq!(fp.exp, -52);

        // Halfway, round-up (b'1000000000000000000000000000000000000000000000000000110000000000')
        let mut fp = ExtendedFloat { mant: 0x8000000000000C00, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Above halfway
        let mut fp = ExtendedFloat { mant: 0x8000000000000401, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52)+1);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat { mant: 0x8000000000000C01, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Below halfway
        let mut fp = ExtendedFloat { mant: 0x80000000000003FF, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<52);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat { mant: 0x8000000000000BFF, exp: -63 };
        round_to_float::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 1);
        assert_eq!(fp.exp, -52);
    }

    #[test]
    fn avoid_overflow_test() {
        // Avoid overflow, fails by 1
        let mut fp = ExtendedFloat { mant: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 5 };
        avoid_overflow::<f64>(&mut fp);
        assert_eq!(fp.mant, 0xFFFFFFFFFFFF);
        assert_eq!(fp.exp, f64::MAX_EXPONENT+5);

        // Avoid overflow, succeeds
        let mut fp = ExtendedFloat { mant: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 4 };
        avoid_overflow::<f64>(&mut fp);
        assert_eq!(fp.mant, 0x1FFFFFFFFFFFE0);
        assert_eq!(fp.exp, f64::MAX_EXPONENT-1);
    }

    #[test]
    fn round_to_native_test() {
        // Overflow
        let mut fp = ExtendedFloat { mant: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 4 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 0x1FFFFFFFFFFFE0);
        assert_eq!(fp.exp, f64::MAX_EXPONENT-1);

        // Need denormal
        let mut fp = ExtendedFloat { mant: 1, exp: f64::DENORMAL_EXPONENT +48 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<48);
        assert_eq!(fp.exp, f64::DENORMAL_EXPONENT);

        // Halfway, round-down (b'10000000000000000000000000000000000000000000000000000100000')
        let mut fp = ExtendedFloat { mant: 0x400000000000020, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<52);
        assert_eq!(fp.exp, -52);

        // Halfway, round-up (b'10000000000000000000000000000000000000000000000000001100000')
        let mut fp = ExtendedFloat { mant: 0x400000000000060, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Above halfway
        let mut fp = ExtendedFloat { mant: 0x400000000000021, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52)+1);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat { mant: 0x400000000000061, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Below halfway
        let mut fp = ExtendedFloat { mant: 0x40000000000001F, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1<<52);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat { mant: 0x40000000000005F, exp: -58 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, (1<<52) + 1);
        assert_eq!(fp.exp, -52);

        // Underflow
        // Adapted from failures in strtod.
        let mut fp = ExtendedFloat { exp: -1139, mant: 18446744073709550712 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 0);
        assert_eq!(fp.exp, 0);

        let mut fp = ExtendedFloat { exp: -1139, mant: 18446744073709551460 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 0);
        assert_eq!(fp.exp, 0);

        let mut fp = ExtendedFloat { exp: -1138, mant: 9223372036854776103 };
        round_to_native::<f64, _>(&mut fp, round_nearest_tie_even);
        assert_eq!(fp.mant, 1);
        assert_eq!(fp.exp, -1074);
    }
}
