//! Heap ownership for the float parser's binary arithmetic intermediates.
//!
//! The math interface continues to accept Vec references, while every owned
//! intermediate uses Buffer. Returning a Vec transfers it directly to another
//! guarded owner or to replace(), with no allocation or user code in between.

use super::math::Limb;
use alloc::vec::Vec;

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

#[cfg(not(feature = "zeroize"))]
pub(crate) type Buffer = Vec<Limb>;

#[cfg(feature = "zeroize")]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Buffer(Vec<Limb>);

#[cfg(feature = "zeroize")]
impl core::ops::Deref for Buffer {
    type Target = Vec<Limb>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "zeroize")]
impl core::ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "zeroize")]
impl Drop for Buffer {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[inline]
pub(crate) fn with_capacity(capacity: usize) -> Buffer {
    wrap(Vec::with_capacity(capacity))
}

#[inline]
pub(crate) fn wrap(limbs: Vec<Limb>) -> Buffer {
    #[cfg(not(feature = "zeroize"))]
    {
        limbs
    }
    #[cfg(feature = "zeroize")]
    {
        Buffer(limbs)
    }
}

#[inline]
pub(crate) fn into_vec(limbs: Buffer) -> Vec<Limb> {
    #[cfg(not(feature = "zeroize"))]
    {
        limbs
    }
    #[cfg(feature = "zeroize")]
    {
        let mut limbs = limbs;
        core::mem::take(&mut limbs.0)
    }
}

#[inline]
#[cfg(feature = "zeroize")]
pub(crate) fn reserve(limbs: &mut Vec<Limb>, additional: usize) {
    if additional > limbs.capacity() - limbs.len() {
        let required = limbs
            .len()
            .checked_add(additional)
            .expect("capacity overflow");
        let capacity = required.max(limbs.capacity().saturating_mul(2));
        let mut replacement = Vec::with_capacity(capacity);
        replacement.extend_from_slice(limbs);
        replace(limbs, replacement);
    }
}

#[inline]
pub(crate) fn reserve_exact(limbs: &mut Vec<Limb>, additional: usize) {
    #[cfg(not(feature = "zeroize"))]
    limbs.reserve_exact(additional);
    #[cfg(feature = "zeroize")]
    reserve(limbs, additional);
}

#[inline]
pub(crate) fn push(limbs: &mut Vec<Limb>, limb: Limb) {
    #[cfg(feature = "zeroize")]
    reserve(limbs, 1);
    limbs.push(limb);
}

#[inline]
pub(crate) fn extend_from_slice(limbs: &mut Vec<Limb>, more: &[Limb]) {
    #[cfg(feature = "zeroize")]
    reserve(limbs, more.len());
    limbs.extend_from_slice(more);
}

#[inline]
pub(crate) fn resize(limbs: &mut Vec<Limb>, len: usize) {
    #[cfg(feature = "zeroize")]
    if len > limbs.len() {
        reserve(limbs, len - limbs.len());
    } else {
        limbs[len..].zeroize();
    }
    limbs.resize(len, 0);
}

#[inline]
pub(crate) fn replace(limbs: &mut Vec<Limb>, replacement: Vec<Limb>) {
    #[cfg(feature = "zeroize")]
    limbs.zeroize();
    *limbs = replacement;
}
