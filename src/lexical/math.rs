//! Building-blocks for arbitrary-precision math.
//!
//! These algorithms assume little-endian order for the large integer
//! buffers, so for a `vec![0, 1, 2, 3]`, `3` is the most significant limb,
//! and `0` is the least significant limb.

use crate::lib::{cmp, iter, mem, ptr};
use crate::num::*;
use crate::large_powers;
use crate::slice::*;
use crate::small_powers::*;

// ALIASES
// -------

//  Type for a single limb of the big integer.
//
//  A limb is analogous to a digit in base10, except, it stores 32-bit
//  or 64-bit numbers instead.
//
//  This should be all-known 64-bit platforms supported by Rust.
//      https://forge.rust-lang.org/platform-support.html
//
//  Platforms where native 128-bit multiplication is explicitly supported:
//      - x86_64 (Supported via `MUL`).
//      - mips64 (Supported via `DMULTU`, which `HI` and `LO` can be read-from).
//
//  Platforms where native 64-bit multiplication is supported and
//  you can extract hi-lo for 64-bit multiplications.
//      aarch64 (Requires `UMULH` and `MUL` to capture high and low bits).
//      powerpc64 (Requires `MULHDU` and `MULLD` to capture high and low bits).
//
//  Platforms where native 128-bit multiplication is not supported,
//  requiring software emulation.
//      sparc64 (`UMUL` only supported double-word arguments).

// 32-BIT LIMB
#[cfg(limb_width_32)]
pub type Limb = u32;

#[cfg(limb_width_32)]
pub const POW5_LIMB: &[Limb] = &POW5_32;

#[cfg(limb_width_32)]
pub const POW10_LIMB: &[Limb] = &POW10_32;

#[cfg(limb_width_32)]
type Wide = u64;

// 64-BIT LIMB
#[cfg(limb_width_64)]
pub type Limb = u64;

#[cfg(limb_width_64)]
pub const POW5_LIMB: &[Limb] = &POW5_64;

#[cfg(limb_width_64)]
pub const POW10_LIMB: &[Limb] = &POW10_64;

#[cfg(limb_width_64)]
type Wide = u128;

// Maximum denominator is 767 mantissa digits + 324 exponent,
// or 1091 digits, or approximately 3600 bits (round up to 4k).
#[cfg(all(no_alloc, limb_width_32))]
pub(crate) type LimbVecType = arrayvec::ArrayVec<[Limb; 128]>;

#[cfg(all(no_alloc, limb_width_64))]
pub(crate) type LimbVecType = arrayvec::ArrayVec<[Limb; 64]>;

#[cfg(not(no_alloc))]
pub(crate) type LimbVecType = crate::lib::Vec<Limb>;

/// Cast to limb type.
#[inline(always)]
pub(crate) fn as_limb<T: Integer>(t: T) -> Limb {
    Limb::as_cast(t)
}

/// Cast to wide type.
#[inline(always)]
fn as_wide<T: Integer>(t: T) -> Wide {
    Wide::as_cast(t)
}

// SPLIT
// -----

/// Split u64 into limbs, in little-endian order.
#[inline]
#[cfg(limb_width_32)]
fn split_u64(x: u64) -> [Limb; 2] {
    [as_limb(x), as_limb(x >> 32)]
}

/// Split u64 into limbs, in little-endian order.
#[inline]
#[cfg(limb_width_64)]
fn split_u64(x: u64) -> [Limb; 1] {
    [as_limb(x)]
}

// HI64
// ----

// NONZERO

/// Check if any of the remaining bits are non-zero.
#[inline]
pub fn nonzero<T: Integer>(x: &[T], rindex: usize) -> bool {
    let len = x.len();
    let slc = &x[..len-rindex];
    slc.iter().rev().any(|&x| x != T::ZERO)
}

/// Shift 64-bit integer to high 64-bits.
#[inline]
fn u64_to_hi64_1(r0: u64) -> (u64, bool) {
    debug_assert!(r0 != 0);
    let ls = r0.leading_zeros();
    (r0 << ls, false)
}

/// Shift 2 64-bit integers to high 64-bits.
#[inline]
fn u64_to_hi64_2(r0: u64, r1: u64) -> (u64, bool) {
    debug_assert!(r0 != 0);
    let ls = r0.leading_zeros();
    let rs = 64 - ls;
    let v = match ls {
        0 => r0,
        _ => (r0 << ls) | (r1 >> rs),
    };
    let n = r1 << ls != 0;
    (v, n)
}

/// Trait to export the high 64-bits from a little-endian slice.
trait Hi64<T>: Slice<T> {
    /// Get the hi64 bits from a 1-limb slice.
    fn hi64_1(&self) -> (u64, bool);

    /// Get the hi64 bits from a 2-limb slice.
    fn hi64_2(&self) -> (u64, bool);

    /// Get the hi64 bits from a 3-limb slice.
    fn hi64_3(&self) -> (u64, bool);

    /// Get the hi64 bits from a 4-limb slice.
    fn hi64_4(&self) -> (u64, bool);

    /// Get the hi64 bits from a 5-limb slice.
    fn hi64_5(&self) -> (u64, bool);

    /// High-level exporter to extract the high 64 bits from a little-endian slice.
    #[inline]
    fn hi64(&self) -> (u64, bool) {
        match self.len() {
            0 => (0, false),
            1 => self.hi64_1(),
            2 => self.hi64_2(),
            3 => self.hi64_3(),
            4 => self.hi64_4(),
            _ => self.hi64_5(),
        }
    }
}

impl Hi64<u32> for [u32] {
    #[inline]
    fn hi64_1(&self) -> (u64, bool) {
        debug_assert!(self.len() == 1);
        let rview = self.rview();
        let r0 = rview[0].as_u64();
        u64_to_hi64_1(r0)
    }

    #[inline]
    fn hi64_2(&self) -> (u64, bool) {
        debug_assert!(self.len() == 2);
        let rview = self.rview();
        let r0 = rview[0].as_u64() << 32;
        let r1 = rview[1].as_u64();
        u64_to_hi64_1(r0 | r1)
    }

    #[inline]
    fn hi64_3(&self) -> (u64, bool) {
        debug_assert!(self.len() >= 3);
        let rview = self.rview();
        let r0 = rview[0].as_u64();
        let r1 = rview[1].as_u64() << 32;
        let r2 = rview[2].as_u64();
        let (v, n) = u64_to_hi64_2(r0, r1 | r2);
        (v, n || nonzero(self, 3))
    }

    #[inline]
    fn hi64_4(&self) -> (u64, bool) {
        self.hi64_3()
    }

    #[inline]
    fn hi64_5(&self) -> (u64, bool) {
        self.hi64_3()
    }
}

impl Hi64<u64> for [u64] {
    #[inline]
    fn hi64_1(&self) -> (u64, bool) {
        debug_assert!(self.len() == 1);
        let rview = self.rview();
        let r0 = rview[0];
        u64_to_hi64_1(r0)
    }

    #[inline]
    fn hi64_2(&self) -> (u64, bool) {
        debug_assert!(self.len() >= 2);
        let rview = self.rview();
        let r0 = rview[0];
        let r1 = rview[1];
        let (v, n) = u64_to_hi64_2(r0, r1);
        (v, n || nonzero(self, 2))
    }

    #[inline]
    fn hi64_3(&self) -> (u64, bool) {
        self.hi64_2()
    }

    #[inline]
    fn hi64_4(&self) -> (u64, bool) {
        self.hi64_2()
    }

    #[inline]
    fn hi64_5(&self) -> (u64, bool) {
        self.hi64_2()
    }
}

// SEQUENCES
// ---------

/// Insert multiple elements at position `index`.
///
/// Shifts all elements before index to the back of the iterator.
/// It uses size hints to try to minimize the number of moves,
/// however, it does not rely on them. We cannot internally allocate, so
/// if we overstep the lower_size_bound, we have to do extensive
/// moves to shift each item back incrementally.
///
/// This implementation is adapted from [`smallvec`], which has a battle-tested
/// implementation that has been revised for at least a security advisory
/// warning. Smallvec is similarly licensed under an MIT/Apache dual license.
///
/// [`smallvec`]: https://github.com/servo/rust-smallvec
fn insert_many<Iter>(vec: &mut LimbVecType, index: usize, iterable: Iter)
    where Iter: iter::IntoIterator<Item=Limb>
{
    let iter = iterable.into_iter();
    if index == vec.len() {
        return vec.extend(iter);
    }

    let (lower_size_bound, _) = iter.size_hint();
    assert!(lower_size_bound <= isize::max_value() as usize);   // Ensure offset is indexable
    assert!(index + lower_size_bound >= index);                 // Protect against overflow

    unsafe {
        let old_len = vec.len();
        assert!(index <= old_len);
        let mut ptr = vec.as_mut_ptr().add(index);

        // Move the trailing elements.
        ptr::copy(ptr, ptr.add(lower_size_bound), old_len - index);

        // In case the iterator panics, don't double-drop the items we just copied above.
        vec.set_len(index);

        let mut num_added = 0;
        for element in iter {
            let mut cur = ptr.add(num_added);
            if num_added >= lower_size_bound {
                // Iterator provided more elements than the hint.  Move trailing items again.
                reserve(vec, 1);
                ptr = vec.as_mut_ptr().add(index);
                cur = ptr.add(num_added);
                ptr::copy(cur, cur.add(1), old_len - index);
            }
            ptr::write(cur, element);
            num_added += 1;
        }
        if num_added < lower_size_bound {
            // Iterator provided fewer elements than the hint
            ptr::copy(ptr.add(lower_size_bound), ptr.add(num_added), old_len - index);
        }

        vec.set_len(old_len + num_added);
    }
}

/// Resize arrayvec to size.
#[inline]
#[cfg(no_alloc)]
fn resize(vec: &mut LimbVecType, len: usize, value: Limb) {
    assert!(len <= vec.capacity());
    let old_len = vec.len();
    if len > old_len {
        vec.extend(iter::repeat(value).take(len - old_len));
    } else {
        vec.truncate(len);
    }
}

/// Resize vec to size.
#[inline]
#[cfg(not(no_alloc))]
fn resize(vec: &mut LimbVecType, len: usize, value: Limb) {
    vec.resize(len, value)
}

/// Reserve arrayvec capacity.
#[inline]
#[cfg(no_alloc)]
pub(crate) fn reserve(vec: &mut LimbVecType, capacity: usize) {
    assert!(vec.len() + capacity <= vec.capacity());
}

/// Reserve vec capacity.
#[inline]
#[cfg(not(no_alloc))]
pub(crate) fn reserve(vec: &mut LimbVecType, capacity: usize) {
    vec.reserve(capacity)
}

/// Reserve exact arrayvec capacity.
#[inline]
#[cfg(no_alloc)]
fn reserve_exact(vec: &mut LimbVecType, capacity: usize) {
    assert!(vec.len() + capacity <= vec.capacity());
}

/// Reserve exact vec capacity.
#[inline]
#[cfg(not(no_alloc))]
fn reserve_exact(vec: &mut LimbVecType, capacity: usize) {
    vec.reserve_exact(capacity)
}

// SCALAR
// ------

// Scalar-to-scalar operations, for building-blocks for arbitrary-precision
// operations.

mod scalar {
use super::*;

// ADDITION

/// Add two small integers and return the resulting value and if overflow happens.
#[inline]
pub fn add(x: Limb, y: Limb) -> (Limb, bool) {
    x.overflowing_add(y)
}

/// AddAssign two small integers and return if overflow happens.
#[inline]
pub fn iadd(x: &mut Limb, y: Limb) -> bool {
    let t = add(*x, y);
    *x = t.0;
    t.1
}

// SUBTRACTION

/// Subtract two small integers and return the resulting value and if overflow happens.
#[inline]
pub fn sub(x: Limb, y: Limb) -> (Limb, bool) {
    x.overflowing_sub(y)
}

/// SubAssign two small integers and return if overflow happens.
#[inline]
pub fn isub(x: &mut Limb, y: Limb) -> bool {
    let t = sub(*x, y);
    *x = t.0;
    t.1
}

// MULTIPLICATION

/// Multiply two small integers (with carry) (and return the overflow contribution).
///
/// Returns the (low, high) components.
#[inline]
pub fn mul(x: Limb, y: Limb, carry: Limb) -> (Limb, Limb) {
    // Cannot overflow, as long as wide is 2x as wide. This is because
    // the following is always true:
    // `Wide::max_value() - (Narrow::max_value() * Narrow::max_value()) >= Narrow::max_value()`
    let z: Wide = as_wide(x) * as_wide(y) + as_wide(carry);
    let bits = mem::size_of::<Limb>() * 8;
    (as_limb(z), as_limb(z >> bits))
}

/// Multiply two small integers (with carry) (and return if overflow happens).
#[inline]
pub fn imul(x: &mut Limb, y: Limb, carry: Limb) -> Limb {
    let t = mul(*x, y, carry);
    *x = t.0;
    t.1
}

}   // scalar

// SMALL
// -----

// Large-to-small operations, to modify a big integer from a native scalar.

mod small {
use super::*;

// MULTIPLICATIION

/// ADDITION

/// Implied AddAssign implementation for adding a small integer to bigint.
///
/// Allows us to choose a start-index in x to store, to allow incrementing
/// from a non-zero start.
#[inline]
pub fn iadd_impl(x: &mut LimbVecType, y: Limb, xstart: usize)
{
    if x.len() <= xstart {
        x.push(y);
    } else {
        // Initial add
        let mut carry = scalar::iadd(&mut x[xstart], y);

        // Increment until overflow stops occurring.
        let mut size = xstart + 1;
        while carry && size < x.len() {
            carry = scalar::iadd(&mut x[size], 1);
            size += 1;
        }

        // If we overflowed the buffer entirely, need to add 1 to the end
        // of the buffer.
        if carry {
            x.push(1);
        }
    }
}

/// AddAssign small integer to bigint.
#[inline]
pub fn iadd(x: &mut LimbVecType, y: Limb) {
    iadd_impl(x, y, 0);
}

// SUBTRACTION

/// SubAssign small integer to bigint.
/// Does not do overflowing subtraction.
#[inline]
pub fn isub_impl(x: &mut LimbVecType, y: Limb, xstart: usize) {
    debug_assert!(x.len() > xstart && (x[xstart] >= y || x.len() > xstart+1));

    // Initial subtraction
    let mut carry = scalar::isub(&mut x[xstart], y);

    // Increment until overflow stops occurring.
    let mut size = xstart + 1;
    while carry && size < x.len() {
        carry = scalar::isub(&mut x[size], 1);
        size += 1;
    }
    normalize(x);
}

// MULTIPLICATION

/// MulAssign small integer to bigint.
#[inline]
pub fn imul(x: &mut LimbVecType, y: Limb) {
    // Multiply iteratively over all elements, adding the carry each time.
    let mut carry: Limb = 0;
    for xi in x.iter_mut() {
        carry = scalar::imul(xi, y, carry);
    }

    // Overflow of value, add to end.
    if carry != 0 {
        x.push(carry);
    }
}

/// Mul small integer to bigint.
#[inline]
pub fn mul(x: &[Limb], y: Limb) -> LimbVecType {
    let mut z = LimbVecType::default();
    z.extend(x.iter().cloned());
    imul(&mut z, y);
    z
}

/// MulAssign by a power.
///
/// Theoretically...
///
/// Use an exponentiation by squaring method, since it reduces the time
/// complexity of the multiplication to ~`O(log(n))` for the squaring,
/// and `O(n*m)` for the result. Since `m` is typically a lower-order
/// factor, this significantly reduces the number of multiplications
/// we need to do. Iteratively multiplying by small powers follows
/// the nth triangular number series, which scales as `O(p^2)`, but
/// where `p` is `n+m`. In short, it scales very poorly.
///
/// Practically....
///
/// Exponentiation by Squaring:
///     running 2 tests
///     test bigcomp_f32_lexical ... bench:       1,018 ns/iter (+/- 78)
///     test bigcomp_f64_lexical ... bench:       3,639 ns/iter (+/- 1,007)
///
/// Exponentiation by Iterative Small Powers:
///     running 2 tests
///     test bigcomp_f32_lexical ... bench:         518 ns/iter (+/- 31)
///     test bigcomp_f64_lexical ... bench:         583 ns/iter (+/- 47)
///
/// Exponentiation by Iterative Large Powers (of 2):
///     running 2 tests
///     test bigcomp_f32_lexical ... bench:         671 ns/iter (+/- 31)
///     test bigcomp_f64_lexical ... bench:       1,394 ns/iter (+/- 47)
///
/// Even using worst-case scenarios, exponentiation by squaring is
/// significantly slower for our workloads. Just multiply by small powers,
/// in simple cases, and use precalculated large powers in other cases.
pub fn imul_pow5(x: &mut LimbVecType, n: u32) {
    use super::large::KARATSUBA_CUTOFF;

    let small_powers = POW5_LIMB;
    let large_powers = large_powers::POW5;

    if n == 0 {
        // No exponent, just return.
        // The 0-index of the large powers is `2^0`, which is 1, so we want
        // to make sure we don't take that path with a literal 0.
        return;
    }

    // We want to use the asymptotically faster algorithm if we're going
    // to be using Karabatsu multiplication sometime during the result,
    // otherwise, just use exponentiation by squaring.
    let bit_length = 32 - n.leading_zeros().as_usize();
    debug_assert!(bit_length != 0 && bit_length <= large_powers.len());
    if x.len() + large_powers[bit_length-1].len() < 2 * KARATSUBA_CUTOFF {
        // We can use iterative small powers to make this faster for the
        // easy cases.

        // Multiply by the largest small power until n < step.
        let step = small_powers.len() - 1;
        let power = small_powers[step];
        let mut n = n.as_usize();
        while n >= step {
            imul(x, power);
            n -= step;
        }

        // Multiply by the remainder.
        imul(x, small_powers[n]);
    } else {
        // In theory, this code should be asymptotically a lot faster,
        // in practice, our small::imul seems to be the limiting step,
        // and large imul is slow as well.

        // Multiply by higher order powers.
        let mut idx: usize = 0;
        let mut bit: usize = 1;
        let mut n = n.as_usize();
        while n != 0 {
            if n & bit != 0 {
                debug_assert!(idx < large_powers.len());
                large::imul(x, large_powers[idx]);
                n ^= bit;
            }
            idx += 1;
            bit <<= 1;
        }
    }
}

// BIT LENGTH

/// Get number of leading zero bits in the storage.
#[inline]
pub fn leading_zeros(x: &[Limb]) -> usize {
    if x.is_empty() {
        0
    } else {
        x.rindex(0).leading_zeros().as_usize()
    }
}

/// Calculate the bit-length of the big-integer.
#[inline]
pub fn bit_length(x: &[Limb]) -> usize {
    let bits = mem::size_of::<Limb>() * 8;
    // Avoid overflowing, calculate via total number of bits
    // minus leading zero bits.
    let nlz = leading_zeros(x);
    bits.checked_mul(x.len())
        .map(|v| v - nlz)
        .unwrap_or(usize::max_value())
}

// SHL

/// Shift-left bits inside a buffer.
///
/// Assumes `n < Limb::BITS`, IE, internally shifting bits.
#[inline]
pub fn ishl_bits(x: &mut LimbVecType, n: usize) {
    // Need to shift by the number of `bits % Limb::BITS)`.
    let bits = mem::size_of::<Limb>() * 8;
    debug_assert!(n < bits);
    if n == 0 {
        return;
    }

    // Internally, for each item, we shift left by n, and add the previous
    // right shifted limb-bits.
    // For example, we transform (for u8) shifted left 2, to:
    //      b10100100 b01000010
    //      b10 b10010001 b00001000
    let rshift = bits - n;
    let lshift = n;
    let mut prev: Limb = 0;
    for xi in x.iter_mut() {
        let tmp = *xi;
        *xi <<= lshift;
        *xi |= prev >> rshift;
        prev = tmp;
    }

    // Always push the carry, even if it creates a non-normal result.
    let carry = prev >> rshift;
    if carry != 0 {
        x.push(carry);
    }
}

/// Shift-left `n` digits inside a buffer.
///
/// Assumes `n` is not 0.
#[inline]
pub fn ishl_limbs(x: &mut LimbVecType, n: usize) {
    debug_assert!(n != 0);
    if !x.is_empty() {
        insert_many(x, 0, iter::repeat(0).take(n));
    }
}

/// Shift-left buffer by n bits.
#[inline]
pub fn ishl(x: &mut LimbVecType, n: usize) {
    let bits = mem::size_of::<Limb>() * 8;
    // Need to pad with zeros for the number of `bits / Limb::BITS`,
    // and shift-left with carry for `bits % Limb::BITS`.
    let rem = n % bits;
    let div = n / bits;
    ishl_bits(x, rem);
    if div != 0 {
        ishl_limbs(x, div);
    }
}

// NORMALIZE

/// Normalize the container by popping any leading zeros.
#[inline]
pub fn normalize(x: &mut LimbVecType) {
    // Remove leading zero if we cause underflow. Since we're dividing
    // by a small power, we have at max 1 int removed.
    while !x.is_empty() && *x.rindex(0) == 0 {
        x.pop();
    }
}

}   // small

// LARGE
// -----

// Large-to-large operations, to modify a big integer from a native scalar.

mod large {
use super::*;

// RELATIVE OPERATORS

/// Compare `x` to `y`, in little-endian order.
#[inline]
pub fn compare(x: &[Limb], y: &[Limb]) -> cmp::Ordering {
    if x.len() > y.len() {
        return cmp::Ordering::Greater;
    } else if x.len() < y.len() {
        return cmp::Ordering::Less;
    } else {
        let iter = x.iter().rev().zip(y.iter().rev());
        for (&xi, &yi) in iter {
            if xi > yi {
                return cmp::Ordering::Greater;
            } else if xi < yi {
                return cmp::Ordering::Less;
            }
        }
        // Equal case.
        return cmp::Ordering::Equal;
    }
}

/// Check if x is less than y.
#[inline]
pub fn less(x: &[Limb], y: &[Limb]) -> bool {
    compare(x, y) == cmp::Ordering::Less
}

/// Check if x is greater than or equal to y.
#[inline]
pub fn greater_equal(x: &[Limb], y: &[Limb]) -> bool {
    !less(x, y)
}

// ADDITION

/// Implied AddAssign implementation for bigints.
///
/// Allows us to choose a start-index in x to store, so we can avoid
/// padding the buffer with zeros when not needed, optimized for vectors.
pub fn iadd_impl(x: &mut LimbVecType, y: &[Limb], xstart: usize) {
    // The effective x buffer is from `xstart..x.len()`, so we need to treat
    // that as the current range. If the effective y buffer is longer, need
    // to resize to that, + the start index.
    if y.len() > x.len() - xstart {
        resize(x, y.len() + xstart, 0);
    }

    // Iteratively add elements from y to x.
    let mut carry = false;
    for (xi, yi) in (&mut x[xstart..]).iter_mut().zip(y.iter()) {
        // Only one op of the two can overflow, since we added at max
        // Limb::max_value() + Limb::max_value(). Add the previous carry,
        // and store the current carry for the next.
        let mut tmp = scalar::iadd(xi, *yi);
        if carry {
            tmp |= scalar::iadd(xi, 1);
        }
        carry = tmp;
    }

    // Overflow from the previous bit.
    if carry {
        small::iadd_impl(x, 1, y.len()+xstart);
    }
}

/// AddAssign bigint to bigint.
#[inline]
pub fn iadd(x: &mut LimbVecType, y: &[Limb]) {
    iadd_impl(x, y, 0)
}

/// Add bigint to bigint.
#[inline]
pub fn add(x: &[Limb], y: &[Limb]) -> LimbVecType {
    let mut z = LimbVecType::default();
    z.extend(x.iter().cloned());
    iadd(&mut z, y);
    z
}

// SUBTRACTION

/// SubAssign bigint to bigint.
pub fn isub(x: &mut LimbVecType, y: &[Limb])
{
    // Basic underflow checks.
    debug_assert!(greater_equal(x, y));

    // Iteratively add elements from y to x.
    let mut carry = false;
    for (xi, yi) in x.iter_mut().zip(y.iter()) {
        // Only one op of the two can overflow, since we added at max
        // Limb::max_value() + Limb::max_value(). Add the previous carry,
        // and store the current carry for the next.
        let mut tmp = scalar::isub(xi, *yi);
        if carry {
            tmp |= scalar::isub(xi, 1);
        }
        carry = tmp;
    }

    if carry {
        small::isub_impl(x, 1, y.len());
    } else {
        small::normalize(x);
    }
}

// MULTIPLICATION

/// Number of digits to bottom-out to asymptotically slow algorithms.
///
/// Karatsuba tends to out-perform long-multiplication at ~320-640 bits,
/// so we go halfway, while Newton division tends to out-perform
/// Algorithm D at ~1024 bits. We can toggle this for optimal performance.
pub const KARATSUBA_CUTOFF: usize = 32;

/// Grade-school multiplication algorithm.
///
/// Slow, naive algorithm, using limb-bit bases and just shifting left for
/// each iteration. This could be optimized with numerous other algorithms,
/// but it's extremely simple, and works in O(n*m) time, which is fine
/// by me. Each iteration, of which there are `m` iterations, requires
/// `n` multiplications, and `n` additions, or grade-school multiplication.
fn long_mul(x: &[Limb], y: &[Limb]) -> LimbVecType {
    // Using the immutable value, multiply by all the scalars in y, using
    // the algorithm defined above. Use a single buffer to avoid
    // frequent reallocations. Handle the first case to avoid a redundant
    // addition, since we know y.len() >= 1.
    let mut z: LimbVecType = small::mul(x, y[0]);
    resize(&mut z, x.len() + y.len(), 0);

    // Handle the iterative cases.
    for (i, &yi) in y[1..].iter().enumerate() {
        let zi: LimbVecType = small::mul(x, yi);
        iadd_impl(&mut z, &zi, i+1);
    }

    small::normalize(&mut z);

    z
}

/// Split two buffers into halfway, into (lo, hi).
#[inline]
pub fn karatsuba_split<'a>(z: &'a [Limb], m: usize) -> (&'a [Limb], &'a [Limb]) {
    (&z[..m], &z[m..])
}

/// Karatsuba multiplication algorithm with roughly equal input sizes.
///
/// Assumes `y.len() >= x.len()`.
fn karatsuba_mul(x: &[Limb], y: &[Limb]) -> LimbVecType {
    if y.len() <= KARATSUBA_CUTOFF {
        // Bottom-out to long division for small cases.
        long_mul(x, y)
    } else if x.len() < y.len() / 2 {
        karatsuba_uneven_mul(x, y)
    } else {
        // Do our 3 multiplications.
        let m = y.len() / 2;
        let (xl, xh) = karatsuba_split(x, m);
        let (yl, yh) = karatsuba_split(y, m);
        let sumx = add(xl, xh);
        let sumy = add(yl, yh);
        let z0 = karatsuba_mul(xl, yl);
        let mut z1 = karatsuba_mul(&sumx, &sumy);
        let z2 = karatsuba_mul(xh, yh);
        // Properly scale z1, which is `z1 - z2 - zo`.
        isub(&mut z1, &z2);
        isub(&mut z1, &z0);

        // Create our result, which is equal to, in little-endian order:
        // [z0, z1 - z2 - z0, z2]
        //  z1 must be shifted m digits (2^(32m)) over.
        //  z2 must be shifted 2*m digits (2^(64m)) over.
        let mut result = LimbVecType::default();
        let len = z0.len().max(m + z1.len()).max(2*m + z2.len());
        reserve_exact(&mut result, len);
        result.extend(z0.iter().cloned());
        iadd_impl(&mut result, &z1, m);
        iadd_impl(&mut result, &z2, 2*m);

        result
    }
}

/// Karatsuba multiplication algorithm where y is substantially larger than x.
///
/// Assumes `y.len() >= x.len()`.
fn karatsuba_uneven_mul(x: &[Limb], mut y: &[Limb]) -> LimbVecType {
    let mut result = LimbVecType::default();
    resize(&mut result, x.len() + y.len(), 0);

    // This effectively is like grade-school multiplication between
    // two numbers, except we're using splits on `y`, and the intermediate
    // step is a Karatsuba multiplication.
    let mut start = 0;
    while y.len() != 0 {
        let m = x.len().min(y.len());
        let (yl, yh) = karatsuba_split(y, m);
        let prod = karatsuba_mul(x, yl);
        iadd_impl(&mut result, &prod, start);
        y = yh;
        start += m;
    }
    small::normalize(&mut result);

    result
}

/// Forwarder to the proper Karatsuba algorithm.
#[inline]
fn karatsuba_mul_fwd(x: &[Limb], y: &[Limb]) -> LimbVecType {
    if x.len() < y.len() {
        karatsuba_mul(x, y)
    } else {
        karatsuba_mul(y, x)
    }
}

/// MulAssign bigint to bigint.
#[inline]
pub fn imul(x: &mut LimbVecType, y: &[Limb])
{
    if y.len() == 1 {
        small::imul(x, y[0]);
    } else {
        // We're not really in a condition where using Karatsuba
        // multiplication makes sense, so we're just going to use long
        // division. ~20% speedup compared to:
        //      *x = karatsuba_mul_fwd(x, y);
        *x = karatsuba_mul_fwd(x, y);
    }
}

}   // large

// TRAITS
// ------

/// Traits for shared operations for big integers.
///
/// None of these are implemented using normal traits, since these
/// are very expensive operations, and we want to deliberately
/// and explicitly use these functions.
pub(crate) trait Math: Clone + Sized + Default {
    // DATA

    /// Get access to the underlying data
    fn data<'a>(&'a self) -> &'a LimbVecType;

    /// Get access to the underlying data
    fn data_mut<'a>(&'a mut self) -> &'a mut LimbVecType;

    // RELATIVE OPERATIONS

    /// Compare self to y.
    #[inline]
    fn compare(&self, y: &Self) -> cmp::Ordering {
        large::compare(self.data(), y.data())
    }

    // PROPERTIES

    /// Get the high 64-bits from the bigint and if there are remaining bits.
    #[inline]
    fn hi64(&self) -> (u64, bool) {
        self.data().as_slice().hi64()
    }

    /// Calculate the bit-length of the big-integer.
    /// Returns usize::max_value() if the value overflows,
    /// IE, if `self.data().len() > usize::max_value() / 8`.
    #[inline]
    fn bit_length(&self) -> usize {
        small::bit_length(self.data())
    }

    // INTEGER CONVERSIONS

    /// Create new big integer from u64.
    #[inline]
    fn from_u64(x: u64) -> Self {
        let mut v = Self::default();
        let slc = split_u64(x);
        v.data_mut().extend(slc.iter().cloned());
        v.normalize();
        v
    }

    // NORMALIZE

    /// Normalize the integer, so any leading zero values are removed.
    #[inline]
    fn normalize(&mut self) {
        small::normalize(self.data_mut());
    }

    // ADDITION

    /// AddAssign small integer.
    #[inline]
    fn iadd_small(&mut self, y: Limb) {
        small::iadd(self.data_mut(), y);
    }

    // MULTIPLICATION

    /// MulAssign small integer.
    #[inline]
    fn imul_small(&mut self, y: Limb) {
        small::imul(self.data_mut(), y);
    }

    /// Multiply by a power of 2.
    #[inline]
    fn imul_pow2(&mut self, n: u32) {
        self.ishl(n.as_usize())
    }

    /// Multiply by a power of 5.
    #[inline]
    fn imul_pow5(&mut self, n: u32) {
        small::imul_pow5(self.data_mut(), n)
    }

    /// MulAssign by a power of 10.
    #[inline]
    fn imul_pow10(&mut self, n: u32) {
        self.imul_pow5(n);
        self.imul_pow2(n);
    }

    // SHIFTS

    /// Shift-left the entire buffer n bits.
    #[inline]
    fn ishl(&mut self, n: usize) {
        small::ishl(self.data_mut(), n);
    }
}

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Bigint {
        data: LimbVecType,
    }

    impl Math for Bigint {
        #[inline]
        fn data<'a>(&'a self) -> &'a LimbVecType {
            &self.data
        }

        #[inline]
        fn data_mut<'a>(&'a mut self) -> &'a mut LimbVecType {
            &mut self.data
        }
    }

    #[cfg(limb_width_32)]
    pub(crate) fn from_u32(x: &[u32]) -> LimbVecType {
        x.iter().cloned().collect()
    }

    #[cfg(limb_width_64)]
    pub(crate) fn from_u32(x: &[u32]) -> LimbVecType {
        let mut v = LimbVecType::default();
        for xi in x.chunks(2) {
            match xi.len() {
                1 => v.push(xi[0] as u64),
                2 => v.push(((xi[1] as u64) << 32) | (xi[0] as u64)),
                _ => unreachable!(),
            }
        }

        v
    }

    #[test]
    fn compare_test() {
        // Simple
        let x = Bigint { data: from_u32(&[1]) };
        let y = Bigint { data: from_u32(&[2]) };
        assert_eq!(x.compare(&y), cmp::Ordering::Less);
        assert_eq!(x.compare(&x), cmp::Ordering::Equal);
        assert_eq!(y.compare(&x), cmp::Ordering::Greater);

        // Check asymmetric
        let x = Bigint { data: from_u32(&[5, 1]) };
        let y = Bigint { data: from_u32(&[2]) };
        assert_eq!(x.compare(&y), cmp::Ordering::Greater);
        assert_eq!(x.compare(&x), cmp::Ordering::Equal);
        assert_eq!(y.compare(&x), cmp::Ordering::Less);

        // Check when we use reverse ordering properly.
        let x = Bigint { data: from_u32(&[5, 1, 9]) };
        let y = Bigint { data: from_u32(&[6, 2, 8]) };
        assert_eq!(x.compare(&y), cmp::Ordering::Greater);
        assert_eq!(x.compare(&x), cmp::Ordering::Equal);
        assert_eq!(y.compare(&x), cmp::Ordering::Less);

        // Complex scenario, check it properly uses reverse ordering.
        let x = Bigint { data: from_u32(&[0, 1, 9]) };
        let y = Bigint { data: from_u32(&[4294967295, 0, 9]) };
        assert_eq!(x.compare(&y), cmp::Ordering::Greater);
        assert_eq!(x.compare(&x), cmp::Ordering::Equal);
        assert_eq!(y.compare(&x), cmp::Ordering::Less);
    }

    #[test]
    fn hi64_test() {
        assert_eq!(Bigint::from_u64(0xA).hi64(), (0xA000000000000000, false));
        assert_eq!(Bigint::from_u64(0xAB).hi64(), (0xAB00000000000000, false));
        assert_eq!(Bigint::from_u64(0xAB00000000).hi64(), (0xAB00000000000000, false));
        assert_eq!(Bigint::from_u64(0xA23456789A).hi64(), (0xA23456789A000000, false));
    }

    #[test]
    fn bit_length_test() {
        let x = Bigint { data: from_u32(&[0, 0, 0, 1]) };
        assert_eq!(x.bit_length(), 97);

        let x = Bigint { data: from_u32(&[0, 0, 0, 3]) };
        assert_eq!(x.bit_length(), 98);

        let x = Bigint { data: from_u32(&[1<<31]) };
        assert_eq!(x.bit_length(), 32);
    }

    #[test]
    fn iadd_small_test() {
        // Overflow check (single)
        // This should set all the internal data values to 0, the top
        // value to (1<<31), and the bottom value to (4>>1).
        // This is because the max_value + 1 leads to all 0s, we set the
        // topmost bit to 1.
        let mut x = Bigint { data: from_u32(&[4294967295]) };
        x.iadd_small(5);
        assert_eq!(x.data, from_u32(&[4, 1]));

        // No overflow, single value
        let mut x = Bigint { data: from_u32(&[5]) };
        x.iadd_small(7);
        assert_eq!(x.data, from_u32(&[12]));

        // Single carry, internal overflow
        let mut x = Bigint::from_u64(0x80000000FFFFFFFF);
        x.iadd_small(7);
        assert_eq!(x.data, from_u32(&[6, 0x80000001]));

        // Double carry, overflow
        let mut x = Bigint::from_u64(0xFFFFFFFFFFFFFFFF);
        x.iadd_small(7);
        assert_eq!(x.data, from_u32(&[6, 0, 1]));
    }

    #[test]
    fn imul_small_test() {
        // No overflow check, 1-int.
        let mut x = Bigint { data: from_u32(&[5]) };
        x.imul_small(7);
        assert_eq!(x.data, from_u32(&[35]));

        // No overflow check, 2-ints.
        let mut x = Bigint::from_u64(0x4000000040000);
        x.imul_small(5);
        assert_eq!(x.data, from_u32(&[0x00140000, 0x140000]));

        // Overflow, 1 carry.
        let mut x = Bigint { data: from_u32(&[0x33333334]) };
        x.imul_small(5);
        assert_eq!(x.data, from_u32(&[4, 1]));

        // Overflow, 1 carry, internal.
        let mut x = Bigint::from_u64(0x133333334);
        x.imul_small(5);
        assert_eq!(x.data, from_u32(&[4, 6]));

        // Overflow, 2 carries.
        let mut x = Bigint::from_u64(0x3333333333333334);
        x.imul_small(5);
        assert_eq!(x.data, from_u32(&[4, 0, 1]));
    }

    #[test]
    fn shl_test() {
        // Pattern generated via `''.join(["1" +"0"*i for i in range(20)])`
        let mut big = Bigint { data: from_u32(&[0xD2210408]) };
        big.ishl(5);
        assert_eq!(big.data, from_u32(&[0x44208100, 0x1A]));
        big.ishl(32);
        assert_eq!(big.data, from_u32(&[0, 0x44208100, 0x1A]));
        big.ishl(27);
        assert_eq!(big.data, from_u32(&[0, 0, 0xD2210408]));

        // 96-bits of previous pattern
        let mut big = Bigint { data: from_u32(&[0x20020010, 0x8040100, 0xD2210408]) };
        big.ishl(5);
        assert_eq!(big.data, from_u32(&[0x400200, 0x802004, 0x44208101, 0x1A]));
        big.ishl(32);
        assert_eq!(big.data, from_u32(&[0, 0x400200, 0x802004, 0x44208101, 0x1A]));
        big.ishl(27);
        assert_eq!(big.data, from_u32(&[0, 0, 0x20020010, 0x8040100, 0xD2210408]));
    }
}
