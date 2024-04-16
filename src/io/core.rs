//! Reimplements core logic and types from `std::io` in an `alloc`-friendly
//! fashion.

use alloc::vec::Vec;
use core::fmt::{self, Display};
use core::result;

/// see [`std::io::ErrorKind`]
pub enum ErrorKind {
    /// see [`std::io::ErrorKind::Other`]
    Other,
}

/// see [`std::io::Error`]
// I/O errors can never occur in no-std mode. All our no-std I/O implementations
// are infallible.
pub struct Error {
    _priv: (),
}

impl Display for Error {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}

impl Error {
    /// see [`std::io::Error::new`]
    pub fn new(_kind: ErrorKind, _error: &'static str) -> Error {
        Error(())
    }
}

/// see [`std::io::Result`]
pub type Result<T> = result::Result<T, Error>;

/// see [`std::io::Write`]
pub trait Write {
    /// see [`std::io::Write::write`]
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// see [`std::io::Write::write_all`]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        // All our Write impls in no_std mode always write the whole buffer in
        // one call infallibly.
        let result = self.write(buf);
        debug_assert!(result.is_ok());
        debug_assert_eq!(result.unwrap_or(0), buf.len());
        Ok(())
    }

    /// see [`std::io::Write::flush`]
    fn flush(&mut self) -> Result<()>;
}

impl<W: Write> Write for &mut W {
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
