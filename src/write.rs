#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(not(feature = "std"))]
use core::str;
use error::{Error, ErrorCode, Result};
use ser;
#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::str;
#[cfg(feature = "std")]
use std::io;

pub trait Write: private::Sealed {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error::syntax(
                        ErrorCode::Message(
                            "failed to write whole buffer".to_string().into_boxed_str(),
                        ),
                        0,
                        0,
                    ))
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn as_io_write(&mut self) -> ImplIoWrite
    where
        Self: Sized,
    {
        ImplIoWrite(self)
    }
}

#[doc(hidden)]
#[cfg(feature = "std")]
pub struct ImplIoWrite<'v>(&'v mut Write);

#[cfg(feature = "std")]
impl<'v> io::Write for ImplIoWrite<'v> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .write(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "io error"))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0
            .flush()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "io error"))
    }
}

mod private {
    pub trait Sealed {}
}

pub struct WriterFormatter<'a, 'b: 'a> {
    pub inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> private::Sealed for WriterFormatter<'a, 'b> {}

impl<'a, 'b> Write for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        fn error<E>(_: E) -> Error {
            // Error value does not matter because fmt::Display impl below just
            // maps it to fmt::Error
            Error::syntax(
                ErrorCode::Message("fmt error".to_string().into_boxed_str()),
                0,
                0,
            )
        }
        let s = try!(str::from_utf8(buf).map_err(error));
        try!(self.inner.write_str(s).map_err(error));
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// extra type needed so we can implement Write for Vec
pub struct VecWriter<'v>(pub &'v mut Vec<u8>);

impl<'v> private::Sealed for VecWriter<'v> {}

impl<'v> Write for VecWriter<'v> {
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct Adapter<'ser, W: 'ser, F: 'ser> {
    pub writer: &'ser mut W,
    pub formatter: &'ser mut F,
    pub error: Option<Error>,
}

impl<'ser, W, F> fmt::Write for Adapter<'ser, W, F>
where
    W: Write,
    F: ser::Formatter,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        assert!(self.error.is_none());
        match ser::format_escaped_str_contents(self.writer, self.formatter, s) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.error = Some(err);
                Err(fmt::Error)
            }
        }
    }
}

#[cfg(feature = "std")]
pub struct IoWrite<'i>(pub &'i mut io::Write);

#[cfg(feature = "std")]
impl<'i> private::Sealed for IoWrite<'i> {}

#[cfg(feature = "std")]
impl<'i> Write for IoWrite<'i> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf).map_err(Error::io)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush().map_err(Error::io)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.0.write_all(buf).map_err(Error::io)
    }
}
