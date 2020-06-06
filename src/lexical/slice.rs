//! Traits to accept generic slices.

use crate::lib::ops;

// RSLICE INDEX

/// A trait for reversed-indexing operations.
pub trait RSliceIndex<T: ?Sized> {
    /// Output type for the index.
    type Output: ?Sized;

    /// Get reference to element or subslice.
    fn rget(self, slc: &T) -> Option<&Self::Output>;

    /// Get mutable reference to element or subslice.
    fn rget_mut(self, slc: &mut T) -> Option<&mut Self::Output>;

    /// Get reference to element or subslice without bounds checking.
    unsafe fn rget_unchecked(self, slc: &T) -> &Self::Output;

    /// Get mutable reference to element or subslice without bounds checking.
    unsafe fn rget_unchecked_mut(self, slc: &mut T) -> &mut Self::Output;

    /// Get reference to element or subslice, panic if out-of-bounds.
    fn rindex(self, slc: &T) -> &Self::Output;

    /// Get mutable reference to element or subslice, panic if out-of-bounds.
    fn rindex_mut(self, slc: &mut T) -> &mut Self::Output;
}

impl<T> RSliceIndex<[T]> for usize {
    type Output = T;

    #[inline]
    fn rget(self, slc: &[T]) -> Option<&T> {
        let len = slc.len();
        slc.get(len - self - 1)
    }

    #[inline]
    fn rget_mut(self, slc: &mut [T]) -> Option<&mut T> {
        let len = slc.len();
        slc.get_mut(len - self - 1)
    }

    #[inline]
    unsafe fn rget_unchecked(self, slc: &[T]) -> &T {
        let len = slc.len();
        slc.get_unchecked(len - self - 1)
    }

    #[inline]
    unsafe fn rget_unchecked_mut(self, slc: &mut [T]) -> &mut T {
        let len = slc.len();
        slc.get_unchecked_mut(len - self - 1)
    }

    #[inline]
    fn rindex(self, slc: &[T]) -> &T {
        let len = slc.len();
        &(*slc)[len - self - 1]
    }

    #[inline]
    fn rindex_mut(self, slc: &mut [T]) -> &mut T {
        let len = slc.len();
        &mut (*slc)[len - self - 1]
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
    fn rview<'a>(&'a self) -> ReverseView<'a, T> {
        ReverseView { inner: self.as_slice() }
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

#[cfg(no_alloc)]
impl<A: arrayvec::Array> Slice<A::Item> for arrayvec::ArrayVec<A> {
    #[inline]
    fn as_slice(&self) -> &[A::Item] {
        arrayvec::ArrayVec::as_slice(self)
    }

    #[inline]
    fn rindex<I: RSliceIndex<[A::Item]>>(&self, index: I) -> &I::Output {
        index.rindex(self.as_slice())
    }
}

#[cfg(not(no_alloc))]
impl<T> Slice<T> for crate::lib::Vec<T> {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    fn rindex<I: RSliceIndex<[T]>>(&self, index: I) -> &I::Output {
        index.rindex(self.as_slice())
    }
}
