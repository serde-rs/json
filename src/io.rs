#[cfg(not(feature = "std"))]
use core::slice;

#[cfg(feature = "std")]
pub use std::io::{Result, Write, Read, Error, Bytes, ErrorKind};

#[cfg(not(feature = "std"))]
pub type Error = &'static str;

#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, Error>;

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
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (*self).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (*self).flush()
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl Write for &mut serde::export::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {Ok(())}
}

#[cfg(not(feature = "std"))]
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes {
            inner: self,
        }
    }
}

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