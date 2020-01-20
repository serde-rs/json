//! A tiny, `no_std`-friendly facade around `std::io`.
//! Reexports types from `std` when available; otherwise reimplements and
//! provides some of the core logic.
//!
//! The main reason that `std::io` hasn't found itself reexported as part of
//! the `core` crate is the `std::io::{Read, Write}` traits' reliance on
//! `std::io::Error`, which may contain internally a heap-allocated `Box<Error>`
//! and/or now relying on OS-specific `std::backtrace::Backtrace`.
//!
//! Because of this, we simply redefine those traits as if the error type is
//! simply a `&'static str` and reimplement those traits for `core` primitives
//! or `alloc` types, e.g. `Vec<T>`.
#[cfg(not(feature = "std"))]
use lib::*;

#[cfg(feature = "std")]
pub use std::io::ErrorKind;

#[cfg(feature = "std")]
pub use std::io::Error;
#[cfg(not(feature = "std"))]
pub type Error = &'static str;

#[cfg(feature = "std")]
pub use std::io::Result;
#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
pub use std::io::Write;
#[cfg(not(feature = "std"))]
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err("failed to write whole buffer"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()>;
}

#[cfg(not(feature = "std"))]
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

#[cfg(not(feature = "std"))]
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

#[cfg(feature = "std")]
pub use std::io::Read;
#[cfg(not(feature = "std"))]
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes { inner: self }
    }
}

#[cfg(feature = "std")]
pub use std::io::Bytes;
#[cfg(not(feature = "std"))]
pub struct Bytes<R> {
    inner: R,
}

#[cfg(not(feature = "std"))]
impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        let mut byte = 0;
        match self.inner.read(slice::from_mut(&mut byte)) {
            Ok(0) => None,
            Ok(..) => Some(Ok(byte)),
            Err(e) => Some(Err(e)),
        }
    }
}
