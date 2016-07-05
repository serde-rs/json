use std::{cmp, io};

use serde::iter::LineColIterator;

/// Trait used by the deserializer for iterating over input. This is manually
/// "specialized" for iterating over &[u8]. Once feature(specialization) is
/// stable we can use actual specialization.
pub trait Read {
    fn next(&mut self) -> io::Result<Option<u8>>;
    fn peek(&mut self) -> io::Result<Option<u8>>;

    /// Only valid after a call to peek(). Discards the peeked byte.
    fn discard(&mut self);

    /// Position of the most recent call to next().
    ///
    /// The most recent call was probably next() and not peek(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually peek() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    fn position(&self) -> Position;

    /// Position of the most recent call to peek().
    ///
    /// The most recent call was probably peek() and not next(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually next() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    fn peek_position(&self) -> Position;
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct IteratorRead<Iter> where Iter: Iterator<Item=io::Result<u8>> {
    iter: LineColIterator<Iter>,
    /// Temporary storage of peeked byte.
    ch: Option<u8>,
}

/// Specialization for Iter=&[u8]. This is more efficient than other iterators
/// because peek() can be read-only and we can compute line/col position only if
/// an error happens.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    /// Index of the *next* byte that will be returned by next() or peek().
    index: usize,
}

//////////////////////////////////////////////////////////////////////////////

impl<Iter> IteratorRead<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    pub fn new(iter: Iter) -> Self {
        IteratorRead {
            iter: LineColIterator::new(iter),
            ch: None,
        }
    }
}

impl<Iter> Read for IteratorRead<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        match self.ch.take() {
            Some(ch) => Ok(Some(ch)),
            None => {
                match self.iter.next() {
                    Some(Err(err)) => Err(err),
                    Some(Ok(ch)) => Ok(Some(ch)),
                    None => Ok(None),
                }
            }
        }
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        match self.ch {
            Some(ch) => Ok(Some(ch)),
            None => {
                match self.iter.next() {
                    Some(Err(err)) => Err(err),
                    Some(Ok(ch)) => {
                        self.ch = Some(ch);
                        Ok(self.ch)
                    }
                    None => Ok(None),
                }
            }
        }
    }

    #[inline]
    fn discard(&mut self) {
        self.ch = None;
    }

    fn position(&self) -> Position {
        Position {
            line: self.iter.line(),
            column: self.iter.col(),
        }
    }

    fn peek_position(&self) -> Position {
        // The LineColIterator updates its position during peek() so it has the
        // right one here.
        self.position()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> SliceRead<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice: slice,
            index: 0,
        }
    }

    fn position_of_index(&self, i: usize) -> Position {
        let mut pos = Position { line: 1, column: 0 };
        for ch in &self.slice[..i] {
            match *ch {
                b'\n' => {
                    pos.line += 1;
                    pos.column = 0;
                }
                _ => {
                    pos.column += 1;
                }
            }
        }
        pos
    }
}

impl<'a> Read for SliceRead<'a> {
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        })
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        })
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    fn position(&self) -> Position {
        self.position_of_index(self.index)
    }

    fn peek_position(&self) -> Position {
        // Cap it at slice.len() just in case the most recent call was next()
        // and it returned the last byte.
        self.position_of_index(cmp::min(self.slice.len(), self.index + 1))
    }
}
