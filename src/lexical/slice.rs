//! Traits to accept generic slices.

use crate::lib::{ops, Vec};

/// REVERSE VIEW

/// Reverse, immutable view of a sequence.
pub struct ReverseView<'a, T: 'a> {
    inner: &'a [T],
}

impl<'a, T> ops::Index<usize> for ReverseView<'a, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.inner[self.inner.len() - index - 1]
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
}

impl<T> Slice<T> for Vec<T> {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self
    }
}
