//! Reimplements core logic and types from `std::io` in an `alloc`-friendly
//! fashion.

use alloc::vec::Vec;
use core::fmt::{self, Display};
use core::result;

/// Simple ErrorKind to mimic std::io::ErrorKind.
pub enum ErrorKind {
    /// A catch-all.
    Other,
}

/// I/O errors can never occur in no-std mode. All our no-std I/O implementations
/// are infallible.
pub struct Error;

impl Display for Error {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}

impl Error {
    /// Creates a new I/O error to mimic std::io::Error interface.
    /// In no-std mode, this is a no-op.
    pub fn new(_kind: ErrorKind, _error: &'static str) -> Error {
        Error
    }
}

/// Mimic std::io::Result.
pub type Result<T> = result::Result<T, Error>;

/// A minimal reimplementation of `std::io::Write`.
pub trait Write {
    /// Writes a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Writes an entire buffer into this writer.
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        // All our Write impls in no_std mode always write the whole buffer in
        // one call infallibly.
        let result = self.write(buf);
        debug_assert!(result.is_ok());
        debug_assert_eq!(result.unwrap_or(0), buf.len());
        Ok(())
    }

    /// Flushes this writer.
    fn flush(&mut self) -> Result<()>;
}

impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (*self).write(buf)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (*self).write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        (*self).flush()
    }
}

impl Write for Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
