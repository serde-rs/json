//! Compare the mantissa to the halfway representation of the float.
//!
//! Compares the actual significant digits of the mantissa to the
//! theoretical digits from `b+h`, scaled into the proper range.

use crate::lib::{cmp, mem};
use super::bignum::*;
use super::digit::*;
use super::exponent::*;
use super::float::*;
use super::math::*;
use super::num::*;
use super::rounding::*;

// MANTISSA

/// Parse the full mantissa into a big integer.
///
/// Max digits is the maximum number of digits plus one.
fn parse_mantissa<'a, F, Iter>(mut iter: Iter) -> Bigint
    where F: Float,
          Iter: Iterator<Item=&'a u8> + Clone
{
    // Main loop
    let small_powers = POW10_LIMB;
    let step = small_powers.len() - 2;
    let max_digits = F::MAX_DIGITS - 1;
    let mut counter = 0;
    let mut value: Limb = 0;
    let mut i: usize = 0;
    let mut result = Bigint::default();

    // Iteratively process all the data in the mantissa.
    while let Some(&digit) = iter.next() {
        // We've parsed the max digits using small values, add to bignum
        if counter == step {
            result.imul_small(small_powers[counter]);
            result.iadd_small(value);
            counter = 0;
            value = 0;
        }

        value *= 10;
        value += as_limb(to_digit(digit).unwrap());

        i += 1;
        counter += 1;
        if i == max_digits {
            break;
        }
    }

    // We will always have a remainder, as long as we entered the loop
    // once, or counter % step is 0.
    if counter != 0 {
        result.imul_small(small_powers[counter]);
        result.iadd_small(value);
    }

    // If we have any remaining digits after the last value, we need
    // to add a 1 after the rest of the array, it doesn't matter where,
    // just move it up. This is good for the worst-possible float
    // representation. We also need to return an index.
    // Since we already trimmed trailing zeros, we know there has
    // to be a non-zero digit if there are any left.
    let is_consumed = iter.count() == 0;
    if !is_consumed {
        result.imul_small(10);
        result.iadd_small(1);
    }

    result
}

// FLOAT OPS

/// Calculate `b` from a a representation of `b` as a float.
#[inline]
pub(super) fn b_extended<F: Float>(f: F) -> ExtendedFloat {
    ExtendedFloat::from_float(f)
}

/// Calculate `b+h` from a a representation of `b` as a float.
#[inline]
pub(super) fn bh_extended<F: Float>(f: F) -> ExtendedFloat {
    // None of these can overflow.
    let b = b_extended(f);
    ExtendedFloat {
        mant: (b.mant << 1) + 1,
        exp: b.exp - 1
    }
}

// ROUNDING

/// Custom round-nearest, tie-event algorithm for bhcomp.
#[inline]
fn round_nearest_tie_even(fp: &mut ExtendedFloat, shift: i32, is_truncated: bool) {
    let (mut is_above, mut is_halfway) = round_nearest(fp, shift);
    if is_halfway && is_truncated {
        is_above = true;
        is_halfway = false;
    }
    tie_even(fp, is_above, is_halfway);
}

// BHCOMP

/// Calculate the mantissa for a big integer with a positive exponent.
fn large_atof<'a, F, Iter>(iter: Iter, exponent: i32) -> F
    where F: Float,
          Iter: Iterator<Item=&'a u8> + Clone
{
    let bits = mem::size_of::<u64>() * 8;

    // Simple, we just need to multiply by the power of the radix.
    // Now, we can calculate the mantissa and the exponent from this.
    // The binary exponent is the binary exponent for the mantissa
    // shifted to the hidden bit.
    let mut bigmant = parse_mantissa::<F, _>(iter);
    bigmant.imul_pow10(exponent.as_u32());

    // Get the exact representation of the float from the big integer.
    let (mant, is_truncated) = bigmant.hi64();
    let exp = bigmant.bit_length().as_i32() - bits.as_i32();
    let mut fp = ExtendedFloat { mant: mant, exp: exp };
    fp.round_to_native::<F, _>(| fp, shift | round_nearest_tie_even(fp, shift, is_truncated));
    into_float(fp)
}

/// Calculate the mantissa for a big integer with a negative exponent.
///
/// This invokes the comparison with `b+h`.
fn small_atof<'a, F, Iter>(iter: Iter, exponent: i32, f: F) -> F
    where F: Float,
          Iter: Iterator<Item=&'a u8> + Clone
{
    // Get the significant digits and radix exponent for the real digits.
    let mut real_digits = parse_mantissa::<F, _>(iter);
    let real_exp = exponent;
    debug_assert!(real_exp < 0);

    // Get the significant digits and the binary exponent for `b+h`.
    let theor = bh_extended(f);
    let mut theor_digits = Bigint::from_u64(theor.mant);
    let theor_exp = theor.exp;

    // We need to scale the real digits and `b+h` digits to be the same
    // order. We currently have `real_exp`, in `radix`, that needs to be
    // shifted to `theor_digits` (since it is negative), and `theor_exp`
    // to either `theor_digits` or `real_digits` as a power of 2 (since it
    // may be positive or negative). Try to remove as many powers of 2
    // as possible. All values are relative to `theor_digits`, that is,
    // reflect the power you need to multiply `theor_digits` by.

    // Can remove a power-of-two, since the radix is 10.
    // Both are on opposite-sides of equation, can factor out a
    // power of two.
    //
    // Example: 10^-10, 2^-10   -> ( 0, 10, 0)
    // Example: 10^-10, 2^-15   -> (-5, 10, 0)
    // Example: 10^-10, 2^-5    -> ( 5, 10, 0)
    // Example: 10^-10, 2^5 -> (15, 10, 0)
    let binary_exp = theor_exp - real_exp;
    let halfradix_exp = -real_exp;
    let radix_exp = 0;

    // Carry out our multiplication.
    if halfradix_exp != 0 {
        theor_digits.imul_pow5(halfradix_exp.as_u32());
    }
    if radix_exp != 0 {
        theor_digits.imul_pow10(radix_exp.as_u32());
    }
    if binary_exp > 0 {
        theor_digits.imul_pow2(binary_exp.as_u32());
    } else if binary_exp < 0 {
        real_digits.imul_pow2((-binary_exp).as_u32());
    }

    // Compare real digits to theoretical digits and round the float.
    match real_digits.compare(&theor_digits) {
        cmp::Ordering::Greater  => f.next_positive(),
        cmp::Ordering::Less     => f,
        cmp::Ordering::Equal    => f.round_positive_even(),
    }
}

/// Calculate the exact value of the float.
///
/// Notes:
///     The digits iterator must not have any trailing zeros (true for
///     `FloatState2`).
///     sci_exponent and digits.size_hint() must not overflow i32.
pub(crate) fn bhcomp<'a, F, Iter1, Iter2>(
    b: F,
    integer: Iter1,
    fraction: Iter2,
    exponent: i32
) -> F
    where F: Float,
          Iter1: Iterator<Item=&'a u8> + Clone,
          Iter2: Iterator<Item=&'a u8> + Clone
{
    // Calculate the number of integer digits and use that to determine
    // where the significant digits start in the fraction.
    let integer_digits = integer.clone().count();
    let fraction_digits = fraction.clone().count();
    let digits_start = match integer_digits {
        0 => fraction.clone().take_while(|&x| x == &b'0').count(),
        _ => 0
    };
    let sci_exp = scientific_exponent(exponent, integer_digits, digits_start);
    let count = F::MAX_DIGITS.min(integer_digits + fraction_digits - digits_start);
    let scaled_exponent = sci_exp + 1 - count.as_i32();

    // We have a finite conversions number of digits for base10.
    // We need to then limit the iterator over that number of digits.
    // Skip all leading zeros (can occur if fraction is empty).
    let iter = integer.chain(fraction).skip(digits_start);

    if scaled_exponent >= 0 {
        large_atof(iter, scaled_exponent)
    } else {
        small_atof(iter, scaled_exponent, b)
    }
}
