#![cfg(feature = "async")]

//! Async I/O integration tests.
//!
//! Every test uses readers/writers that genuinely return `Poll::Pending` to
//! verify the futures correctly save and restore state across suspension
//! boundaries. In-memory tests that never yield are labeled "correctness
//! regression" and kept minimal.

use futures::executor::block_on;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{from_async_reader, to_async_writer, AsyncDeserializer, AsyncSerializer};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

// ------------------------------------------------------------------
// Compat bridge: futures::io traits → serde_json traits
// ------------------------------------------------------------------

struct Compat<T>(T);

impl<T: futures::io::AsyncRead + Unpin> serde_json::AsyncRead for Compat<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<T: futures::io::AsyncWrite + Unpin> serde_json::AsyncWrite for Compat<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

// ------------------------------------------------------------------
// SlowReader: yields `yields_per_chunk` times before each data chunk.
//
// Simulates a network socket that periodically has no bytes ready.
// The future under test must save `buf` state across every `Pending`
// return and resume from the correct offset.
// ------------------------------------------------------------------

struct SlowReader {
    data: Vec<u8>,
    pos: usize,
    yields_per_chunk: usize,
    pending_left: usize,
}

impl SlowReader {
    fn new(data: Vec<u8>, yields_per_chunk: usize) -> Self {
        Self {
            data,
            pos: 0,
            yields_per_chunk,
            pending_left: yields_per_chunk,
        }
    }
}

impl serde_json::AsyncRead for SlowReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.pending_left > 0 {
            self.pending_left -= 1;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending_left = self.yields_per_chunk;
        let remaining = self.data.len() - self.pos;
        if remaining == 0 {
            return Poll::Ready(Ok(0));
        }
        // Return small chunks to exercise the read-loop buffer growth.
        let n = remaining.min(buf.len()).min(32);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Poll::Ready(Ok(n))
    }
}

// ------------------------------------------------------------------
// SlowWriter: yields once before accepting each write batch.
//
// Simulates a socket whose send buffer alternates between full and ready.
// WriteAll must track how many bytes it has already drained on each resume.
// ------------------------------------------------------------------

struct SlowWriter {
    sink: Arc<Mutex<Vec<u8>>>,
    pending: bool,
}

impl SlowWriter {
    fn new(sink: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { sink, pending: true }
    }
}

impl serde_json::AsyncWrite for SlowWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        self.sink.lock().unwrap().extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

// ------------------------------------------------------------------
// Test data
// ------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// ------------------------------------------------------------------
// Correctness regression (readers never yield — tests the happy path only)
// ------------------------------------------------------------------

/// Verify the async wrapper does not corrupt data when the reader never yields.
#[test]
fn roundtrip_correctness_regression() {
    let p = Point { x: 1.0, y: 2.0 };
    let json = serde_json::to_vec(&p).unwrap();
    let got: Point = block_on(from_async_reader(Compat(json.as_slice()))).unwrap();
    assert_eq!(p, got);
}

// ------------------------------------------------------------------
// Async-specific tests (readers/writers yield Pending)
// ------------------------------------------------------------------

/// `ReadToEnd` must save and restore its partially-filled buffer across every
/// `Poll::Pending` boundary. The reader here yields 4 times before each 32-byte
/// chunk, so the future is suspended and resumed many times per deserialization.
#[test]
fn reader_suspends_and_resumes_across_pending() {
    let p = Point { x: 3.0, y: 4.0 };
    let json = serde_json::to_vec(&p).unwrap();
    let reader = SlowReader::new(json, 4 /* yields before each chunk */);
    let got: Point = block_on(from_async_reader(reader)).unwrap();
    assert_eq!(p, got);
}

/// `WriteAll` must correctly track the drained slice position across every
/// `Poll::Pending` boundary. The writer here yields once before accepting each
/// write, so the future is suspended and resumed for every write call.
#[test]
fn writer_suspends_and_resumes_across_pending() {
    let p = Point { x: 5.0, y: 6.0 };
    let sink = Arc::new(Mutex::new(Vec::<u8>::new()));
    let writer = SlowWriter::new(Arc::clone(&sink));
    block_on(to_async_writer(writer, &p)).unwrap();
    let written = sink.lock().unwrap();
    let got: Point = serde_json::from_slice(&written).unwrap();
    assert_eq!(p, got);
}

/// Extension trait version of the write test.
#[test]
fn async_serializer_trait_suspends_and_resumes() {
    let p = Point { x: 7.0, y: 8.0 };
    let sink = Arc::new(Mutex::new(Vec::<u8>::new()));
    block_on(p.to_async_writer(SlowWriter::new(Arc::clone(&sink)))).unwrap();
    let written = sink.lock().unwrap();
    let got: Point = serde_json::from_slice(&written).unwrap();
    assert_eq!(p, got);
}

/// Extension trait version of the read test.
#[test]
fn async_deserializer_trait_suspends_and_resumes() {
    let p = Point { x: 9.0, y: 10.0 };
    let json = serde_json::to_vec(&p).unwrap();
    let got: Point = block_on(Point::from_async_reader(SlowReader::new(json, 2))).unwrap();
    assert_eq!(p, got);
}

/// A payload larger than the 4096-byte read chunk forces `ReadToEnd` to
/// grow its buffer across multiple poll cycles with a slow reader.
#[test]
fn large_payload_with_slow_reader() {
    // 0..1500 serializes to ~6 KB (1000-1499 are 4-digit numbers), safely > 4096.
    let v: Vec<u64> = (0..1500).collect();
    let json = serde_json::to_vec(&v).unwrap();
    assert!(json.len() > 4096, "test precondition: payload must exceed one read chunk");
    let reader = SlowReader::new(json, 2);
    let got: Vec<u64> = block_on(from_async_reader(reader)).unwrap();
    assert_eq!(v, got);
}

/// `join_all` cooperatively polls all futures: when one reader yields `Pending`,
/// the executor advances the next. Ten independent deserialization futures
/// interleave their `Pending` suspensions and all complete correctly.
#[test]
fn concurrent_readers_interleave_via_join_all() {
    let payloads: Vec<Vec<u8>> = (0u64..10)
        .map(|i| serde_json::to_vec(&Point { x: i as f64, y: i as f64 * 2.0 }).unwrap())
        .collect();

    let tasks: Vec<_> = payloads
        .iter()
        .map(|json| from_async_reader::<Point, _>(SlowReader::new(json.clone(), 3)))
        .collect();

    let results: Vec<serde_json::Result<Point>> = block_on(join_all(tasks));

    for (i, result) in results.into_iter().enumerate() {
        let got = result.unwrap();
        assert_eq!(got.x, i as f64);
        assert_eq!(got.y, i as f64 * 2.0);
    }
}

// ------------------------------------------------------------------
// Error-path tests
// ------------------------------------------------------------------

/// Invalid JSON produces a parse error, not a hang or panic.
/// The reader still yields Pending before returning the data.
#[test]
fn invalid_json_error_with_slow_reader() {
    let reader = SlowReader::new(b"not valid json".to_vec(), 2);
    let r: serde_json::Result<serde_json::Value> = block_on(from_async_reader(reader));
    assert!(r.is_err());
    assert!(!r.unwrap_err().is_io(), "should be a parse error, not an I/O error");
}

/// An I/O error from the reader is surfaced as `serde_json::Error::is_io`.
#[test]
fn io_error_bridges_correctly() {
    struct Broken;

    impl serde_json::AsyncRead for Broken {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "broken",
            )))
        }
    }

    let err = block_on(from_async_reader::<serde_json::Value, _>(Broken)).unwrap_err();
    assert!(err.is_io());
}

/// A write I/O error is surfaced as `serde_json::Error::is_io`.
#[test]
fn write_io_error_bridges_correctly() {
    struct BrokenWriter;

    impl serde_json::AsyncWrite for BrokenWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "broken",
            )))
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let p = Point { x: 1.0, y: 2.0 };
    let err = block_on(to_async_writer(BrokenWriter, &p)).unwrap_err();
    assert!(err.is_io());
}
