use alloc::vec::Vec;
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Runtime-agnostic async byte reader.
///
/// The signature is identical to `futures::io::AsyncRead` so a single
/// one-line forwarding impl bridges any runtime's I/O trait to this one.
pub trait AsyncRead {
    /// Attempt to read bytes into `buf`, returning the number of bytes read.
    ///
    /// Returns `Poll::Pending` if no bytes are available yet; the waker is
    /// called when progress can be made. Returns `Ok(0)` to signal EOF.
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>>;
}

// ------------------------------------------------------------------
// ReadToEnd future: drains an AsyncRead into a Vec<u8>
// ------------------------------------------------------------------

struct ReadToEnd<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: AsyncRead + Unpin> Future for ReadToEnd<R> {
    type Output = std::io::Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        loop {
            let len = me.buf.len();
            me.buf.resize(len + 4096, 0);
            match Pin::new(&mut me.reader).poll_read(cx, &mut me.buf[len..]) {
                Poll::Ready(Ok(0)) => {
                    me.buf.truncate(len);
                    return Poll::Ready(Ok(mem::take(&mut me.buf)));
                }
                Poll::Ready(Ok(n)) => me.buf.truncate(len + n),
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    me.buf.truncate(len);
                    return Poll::Pending;
                }
            }
        }
    }
}

pub(super) async fn read_to_end<R: AsyncRead + Unpin>(reader: R) -> std::io::Result<Vec<u8>> {
    ReadToEnd {
        reader,
        buf: Vec::new(),
    }
    .await
}
