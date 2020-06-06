//! Traits to accept generic slices.

use crate::lib::{ops, Vec};

// RSLICE INDEX

/// A trait for reversed-indexing operations.
pub trait RSliceIndex<T: ?Sized> {
    /// Output type for the index.
    type Output: ?Sized;

    /// Get reference to element or subslice, panic if out-of-bounds.
    fn rindex(self, slc: &T) -> &Self::Output;
}

impl<T> RSliceIndex<[T]> for usize {
    type Output = T;

    #[inline]
    fn rindex(self, slc: &[T]) -> &T {
        let len = slc.len();
        &(*slc)[len - self - 1]
    }
}

/// REVERSE VIEW

/// Reverse, immutable view of a sequence.
pub struct ReverseView<'a, T: 'a> {
    inner: &'a [T],
}

impl<'a, T> ops::Index<usize> for ReverseView<'a, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        self.inner.rindex(index)
    }
}

// SLICE

/// Trait for generic slices.
pub trait Slice<T> {
    // AS SLICE

    /// Get slice of immutable elements.
    fn as_slice(&self) -> &[T];

    /// Get the length of the collection.
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self.as_slice())
    }

    // RINDEX

    /// Get reference to element or subslice.
    fn rindex<I: RSliceIndex<[T]>>(&self, index: I) -> &I::Output;

    // RVIEW

    /// Create a reverse view of the vector for indexing.
    #[inline]
    fn rview(&self) -> ReverseView<T> {
        ReverseView {
            inner: self.as_slice(),
        }
    }
}

impl<T> Slice<T> for [T] {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    fn rindex<I: RSliceIndex<[T]>>(&self, index: I) -> &I::Output {
        index.rindex(self)
    }
}

impl<T> Slice<T> for Vec<T> {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    fn rindex<I: RSliceIndex<[T]>>(&self, index: I) -> &I::Output {
        index.rindex(self.as_slice())
    }
}
