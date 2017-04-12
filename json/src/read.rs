use std::{char, cmp, io, str};

use serde::iter::LineColIterator;

use super::error::{Error, ErrorCode, Result};

/// Trait used by the deserializer for iterating over input. This is manually
/// "specialized" for iterating over &[u8]. Once feature(specialization) is
/// stable we can use actual specialization.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `serde_json`.
pub trait Read: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> io::Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> io::Result<Option<u8>>;

    /// Only valid after a call to peek(). Discards the peeked byte.
    #[doc(hidden)]
    fn discard(&mut self);

    /// Position of the most recent call to next().
    ///
    /// The most recent call was probably next() and not peek(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually peek() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn position(&self) -> Position;

    /// Position of the most recent call to peek().
    ///
    /// The most recent call was probably peek() and not next(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually next() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn peek_position(&self) -> Position;

    /// Assumes the previous byte was a quotation mark. Parses a JSON-escaped
    /// string until the next quotation mark using the given scratch space if
    /// necessary. The scratch space is initially empty.
    #[doc(hidden)]
    fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<&'s str>;

    /// Assumes the previous byte was a quotation mark. Parses a JSON-escaped
    /// string until the next quotation mark using the given scratch space if
    /// necessary. The scratch space is initially empty.
    ///
    /// This function returns the raw bytes in the string with escape sequences
    /// expanded but without performing unicode validation.
    #[doc(hidden)]
    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s [u8]>;
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// JSON input source that reads from an iterator of bytes.
pub struct IteratorRead<Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
{
    iter: LineColIterator<Iter>,
    /// Temporary storage of peeked byte.
    ch: Option<u8>,
}

/// JSON input source that reads from a std::io input stream.
pub struct IoRead<R>
    where R: io::Read
{
    delegate: IteratorRead<io::Bytes<R>>,
}

/// JSON input source that reads from a slice of bytes.
//
// This is more efficient than other iterators because peek() can be read-only
// and we can compute line/col position only if an error happens.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    /// Index of the *next* byte that will be returned by next() or peek().
    index: usize,
}

/// JSON input source that reads from a UTF-8 string.
//
// Able to elide UTF-8 checks by assuming that the input is valid UTF-8.
pub struct StrRead<'a> {
    delegate: SliceRead<'a>,
}

// Prevent users from implementing the Read trait.
mod private {
    pub trait Sealed {}
}

//////////////////////////////////////////////////////////////////////////////

impl<Iter> IteratorRead<Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
{
    /// Create a JSON input source to read from an iterator of bytes.
    pub fn new(iter: Iter) -> Self {
        IteratorRead {
            iter: LineColIterator::new(iter),
            ch: None,
        }
    }
}

impl<Iter> private::Sealed for IteratorRead<Iter>
    where Iter: Iterator<Item = io::Result<u8>> {}

impl<Iter> IteratorRead<Iter>
    where Iter: Iterator<Item = io::Result<u8>>
{
    fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F
    ) -> Result<T>
        where T: 's,
              F: FnOnce(&'s Self, &'s [u8]) -> Result<T>,
    {
        loop {
            let ch = try!(next_or_eof(self));
            if !ESCAPE[ch as usize] {
                scratch.push(ch);
                continue;
            }
            match ch {
                b'"' => {
                    return result(self, scratch);
                }
                b'\\' => {
                    try!(parse_escape(self, scratch));
                }
                _ => {
                    if validate {
                        return error(self, ErrorCode::InvalidUnicodeCodePoint);
                    }
                    scratch.push(ch);
                }
            }
        }
    }
}

impl<Iter> Read for IteratorRead<Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
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

    fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s str> {
        self.parse_str_bytes(scratch, true, as_str)
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s [u8]> {
        self.parse_str_bytes(scratch, false, |_, bytes| Ok(bytes))
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<R> IoRead<R>
    where R: io::Read
{
    /// Create a JSON input source to read from a std::io input stream.
    pub fn new(reader: R) -> Self {
        IoRead {
            delegate: IteratorRead::new(reader.bytes()),
        }
    }
}

impl<R> private::Sealed for IoRead<R>
    where R: io::Read {}

impl<R> Read for IoRead<R>
    where R: io::Read
{
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        self.delegate.next()
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        self.delegate.peek()
    }

    #[inline]
    fn discard(&mut self) {
        self.delegate.discard();
    }

    #[inline]
    fn position(&self) -> Position {
        self.delegate.position()
    }

    #[inline]
    fn peek_position(&self) -> Position {
        self.delegate.peek_position()
    }

    #[inline]
    fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s str> {
        self.delegate.parse_str(scratch)
    }

    #[inline]
    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s [u8]> {
        self.delegate.parse_str_raw(scratch)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> SliceRead<'a> {
    /// Create a JSON input source to read from a slice of bytes.
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice: slice,
            index: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.index
    }

    fn position_of_index(&self, i: usize) -> Position {
        let mut pos = Position {
            line: 1,
            column: 0,
        };
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

    /// The big optimization here over IteratorRead is that if the string
    /// contains no backslash escape sequences, the returned &str is a slice of
    /// the raw JSON data so we avoid copying into the scratch space.
    fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F
    ) -> Result<T>
        where T: 's,
              F: FnOnce(&'s Self, &'s [u8]) -> Result<T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        let mut start = self.index;

        loop {
            while self.index < self.slice.len() &&
                  !ESCAPE[self.slice[self.index] as usize] {
                self.index += 1;
            }
            if self.index == self.slice.len() {
                return error(self, ErrorCode::EofWhileParsingString);
            }
            match self.slice[self.index] {
                b'"' => {
                    let string = if scratch.is_empty() {
                        // Fast path: return a slice of the raw JSON without any
                        // copying.
                        &self.slice[start..self.index]
                    } else {
                        scratch.extend_from_slice(&self.slice[start .. self.index]);
                        // "as &[u8]" is required for rustc 1.8.0
                        scratch as &[u8]
                    };
                    self.index += 1;
                    return result(self, string);
                }
                b'\\' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    try!(parse_escape(self, scratch));
                    start = self.index;
                }
                _ => {
                    if validate {
                        return error(self, ErrorCode::InvalidUnicodeCodePoint);
                    }
                    self.index += 1;
                }
            }
        }
    }
}

impl<'a> private::Sealed for SliceRead<'a> {}

impl<'a> Read for SliceRead<'a> {
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        // `Ok(self.slice.get(self.index).map(|ch| { self.index += 1; *ch }))`
        // is about 10% slower.
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
        // `Ok(self.slice.get(self.index).map(|ch| *ch))` is about 10% slower
        // for some reason.
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

    fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s str> {
        self.parse_str_bytes(scratch, true, as_str)
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s [u8]> {
        self.parse_str_bytes(scratch, false, |_, bytes| Ok(bytes))
    }

}

//////////////////////////////////////////////////////////////////////////////

impl<'a> StrRead<'a> {
    /// Create a JSON input source to read from a UTF-8 string.
    pub fn new(s: &'a str) -> Self {
        StrRead {
            delegate: SliceRead::new(s.as_bytes()),
        }
    }
}

impl<'a> private::Sealed for StrRead<'a> {}

impl<'a> Read for StrRead<'a> {
    #[inline]
    fn next(&mut self) -> io::Result<Option<u8>> {
        self.delegate.next()
    }

    #[inline]
    fn peek(&mut self) -> io::Result<Option<u8>> {
        self.delegate.peek()
    }

    #[inline]
    fn discard(&mut self) {
        self.delegate.discard();
    }

    fn position(&self) -> Position {
        self.delegate.position()
    }

    fn peek_position(&self) -> Position {
        self.delegate.peek_position()
    }

    fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s str> {
        self.delegate.parse_str_bytes(scratch, true, |_, bytes| {
            // The input is assumed to be valid UTF-8 and the \u-escapes are
            // checked along the way, so don't need to check here.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>
    ) -> Result<&'s [u8]> {
        self.delegate.parse_str_raw(scratch)
    }
}

//////////////////////////////////////////////////////////////////////////////

const CT: bool = true; // control character \x00...\x1F
const QU: bool = true; // quote \x22
const BS: bool = true; // backslash \x5C
const O: bool = false; // allow unescaped

// Lookup table of bytes that must be escaped. A value of true at index i means
// that byte i requires an escape sequence in the input.
#[cfg_attr(rustfmt, rustfmt_skip)]
static ESCAPE: [bool; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
    CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
     O,  O, QU,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 2
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 3
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 4
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, BS,  O,  O,  O, // 5
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 6
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 7
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 8
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 9
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // A
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // B
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // C
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // D
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // E
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // F
];

fn next_or_eof<R: Read>(read: &mut R) -> Result<u8> {
    match try!(read.next()) {
        Some(b) => Ok(b),
        None => error(read, ErrorCode::EofWhileParsingString),
    }
}

fn error<R: Read, T>(read: &R, reason: ErrorCode) -> Result<T> {
    let pos = read.position();
    Err(Error::syntax(reason, pos.line, pos.column))
}

fn as_str<'s, R: Read>(read: &R, slice: &'s [u8]) -> Result<&'s str> {
    str::from_utf8(slice)
        .or_else(|_| error(read, ErrorCode::InvalidUnicodeCodePoint))
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn parse_escape<R: Read>(read: &mut R, scratch: &mut Vec<u8>) -> Result<()> {
    let ch = try!(next_or_eof(read));

    match ch {
        b'"' => scratch.push(b'"'),
        b'\\' => scratch.push(b'\\'),
        b'/' => scratch.push(b'/'),
        b'b' => scratch.push(b'\x08'),
        b'f' => scratch.push(b'\x0c'),
        b'n' => scratch.push(b'\n'),
        b'r' => scratch.push(b'\r'),
        b't' => scratch.push(b'\t'),
        b'u' => {
            let c =
                match try!(decode_hex_escape(read)) {
                    0xDC00...0xDFFF => {
                        return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                    }

                    // Non-BMP characters are encoded as a sequence of
                    // two hex escapes, representing UTF-16 surrogates.
                    n1 @ 0xD800...0xDBFF => {
                        if try!(next_or_eof(read)) != b'\\' {
                            return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                        }
                        if try!(next_or_eof(read)) != b'u' {
                            return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                        }

                        let n2 = try!(decode_hex_escape(read));

                        if n2 < 0xDC00 || n2 > 0xDFFF {
                            return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                        }

                        let n = (((n1 - 0xD800) as u32) << 10 |
                                 (n2 - 0xDC00) as u32) +
                                0x1_0000;

                        match char::from_u32(n as u32) {
                            Some(c) => c,
                            None => {
                                return error(read, ErrorCode::InvalidUnicodeCodePoint);
                            }
                        }
                    }

                    n => {
                        match char::from_u32(n as u32) {
                            Some(c) => c,
                            None => {
                                return error(read, ErrorCode::InvalidUnicodeCodePoint);
                            }
                        }
                    }
                };

            // FIXME: this allocation is required in order to be compatible with stable
            // rust, which doesn't support encoding a `char` into a stack buffer.
            let mut buf = String::new();
            buf.push(c);
            scratch.extend(buf.bytes());
        }
        _ => {
            return error(read, ErrorCode::InvalidEscape);
        }
    }

    Ok(())
}

fn decode_hex_escape<R: Read>(read: &mut R) -> Result<u16> {
    let mut n = 0;
    for _ in 0..4 {
        n = match try!(next_or_eof(read)) {
            c @ b'0'...b'9' => n * 16_u16 + ((c as u16) - (b'0' as u16)),
            b'a' | b'A' => n * 16_u16 + 10_u16,
            b'b' | b'B' => n * 16_u16 + 11_u16,
            b'c' | b'C' => n * 16_u16 + 12_u16,
            b'd' | b'D' => n * 16_u16 + 13_u16,
            b'e' | b'E' => n * 16_u16 + 14_u16,
            b'f' | b'F' => n * 16_u16 + 15_u16,
            _ => {
                return error(read, ErrorCode::InvalidEscape);
            }
        };
    }
    Ok(n)
}
