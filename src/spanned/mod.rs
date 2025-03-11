#![cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
//! TODO: document

pub(crate) use de::SpannedDeserializer;
#[doc(inline)]
pub use de::{from_reader_spanned, from_slice_spanned, from_str_spanned};
pub use spanned::Spanned;
pub(crate) use spanned::{
    is_spanned, END_COL_FIELD, END_LINE_FIELD, END_OFFSET_FIELD, START_COL_FIELD, START_LINE_FIELD,
    START_OFFSET_FIELD, VALUE_FIELD,
};
pub use value::SpannedValue;

mod de;
mod spanned;
pub mod value;

/// TODO: document
#[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
#[derive(Debug, Copy, Clone)]
pub struct Span {
    /// TODO: document
    pub start: SpanPosition,
    /// Non-inclusive end position
    pub end: SpanPosition,
}

impl Span {
    /// TODO: document
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub const fn new(start: SpanPosition, end: SpanPosition) -> Self {
        Self { start, end }
    }

    /// TODO: document
    pub const fn default() -> Self {
        Self {
            start: SpanPosition::default(),
            end: SpanPosition::default(),
        }
    }

    /// Byte range
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn byte_span(&self) -> core::ops::Range<usize> {
        self.start.byte_offset..self.end.byte_offset
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::default()
    }
}

/// TODO: document
#[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
#[derive(Debug, Copy, Clone)]
pub struct SpanPosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Character number in the line (1-indexed)
    pub column: usize,
    /// Offset (global) in the document's byte stream
    pub byte_offset: usize,
}

impl SpanPosition {
    /// TODO: document
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub const fn new(line: usize, col: usize, offset: usize) -> Self {
        Self {
            line,
            column: col,
            byte_offset: offset,
        }
    }

    /// TODO: document
    pub const fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            byte_offset: 0,
        }
    }
}

impl Default for SpanPosition {
    fn default() -> Self {
        Self::default()
    }
}
