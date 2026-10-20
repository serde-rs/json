use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Runtime-agnostic async byte writer.
///
/// The signature is identical to `futures::io::AsyncWrite` so a single
/// one-line forwarding impl bridges any runtime's I/O trait to this one.
pub trait AsyncWrite {
    /// Attempt to write bytes from `buf` into the writer.
    ///
    /// Returns the number of bytes written, or `Poll::Pending` if the writer
    /// is not ready. Must not return `Ok(0)` unless `buf` is empty.
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>>;

    /// Flush any buffered data to the underlying sink.
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>>;

    /// Close the writer, flushing and releasing any resources.
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>>;
}

// ------------------------------------------------------------------
// WriteAll future: drains a byte slice into an AsyncWrite
// ------------------------------------------------------------------

struct WriteAll<'a, W> {
    writer: W,
    buf: &'a [u8],
}

impl<'a, W: AsyncWrite + Unpin> Future for WriteAll<'a, W> {
    type Output = std::io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        while !me.buf.is_empty() {
            match Pin::new(&mut me.writer).poll_write(cx, me.buf) {
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "write zero",
                    )))
                }
                Poll::Ready(Ok(n)) => me.buf = &me.buf[n..],
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }
}

pub(super) async fn write_all<W: AsyncWrite + Unpin>(
    writer: W,
    buf: &[u8],
) -> std::io::Result<()> {
    WriteAll { writer, buf }.await
}
