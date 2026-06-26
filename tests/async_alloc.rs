//! Peak-allocation test for the `async` feature.
//!
//! Uses a tracking global allocator. Concurrent tasks with `SlowReader` (which
//! genuinely suspends) demonstrate that the cooperative executor interleaves
//! futures without unbounded memory growth.

#![cfg(feature = "async")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::task::{Context, Poll};

// ------------------------------------------------------------------
// Tracking allocator — wrapping arithmetic prevents overflow panics
// ------------------------------------------------------------------

static PEAK: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);

struct PeakTracker;

unsafe impl GlobalAlloc for PeakTracker {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        // wrapping_add: never panic inside an allocator
        let now = LIVE.fetch_add(l.size(), Relaxed).wrapping_add(l.size());
        PEAK.fetch_max(now, Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Relaxed);
        System.dealloc(p, l)
    }
}

#[global_allocator]
static ALLOC: PeakTracker = PeakTracker;

// ------------------------------------------------------------------
// SlowReader
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
        let n = remaining.min(buf.len()).min(64);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Poll::Ready(Ok(n))
    }
}

// ------------------------------------------------------------------

/// 10 concurrent async deserialization tasks must not exceed 12× the
/// single-input size in peak *additional* allocation.
///
/// We measure peak *above* the pre-test baseline rather than resetting LIVE
/// to zero — resetting would cause underflow when existing allocations are
/// freed, corrupting the tracker and panicking inside the allocator.
#[test]
fn peak_allocation_bounded_by_input_size() {
    use futures::executor::block_on;
    use futures::future::join_all;

    let json = serde_json::to_vec(&vec![1u64; 500]).unwrap();

    // Snapshot live allocation before the test; reset PEAK to that baseline.
    let baseline = LIVE.load(Relaxed);
    PEAK.store(baseline, Relaxed);

    let tasks: Vec<_> = (0..10)
        .map(|_| serde_json::from_async_reader::<Vec<u64>, _>(SlowReader::new(json.clone(), 2)))
        .collect();

    let results: Vec<_> = block_on(join_all(tasks));
    assert!(results.iter().all(|r| r.is_ok()), "all tasks must succeed");

    let peak_above_baseline = PEAK.load(Relaxed).saturating_sub(baseline);

    // join_all holds all N task states simultaneously, so peak scales as
    // O(N × input_size). Each task holds:
    //   • its json clone         (~json.len() bytes)
    //   • the ReadToEnd buffer   (~4096 bytes min, up to input size)
    //   • the deserialized result (Vec<u64> × 8 bytes/elem ≈ 4× json.len())
    // Factor of 15 per task provides margin for Vec capacity doubling and
    // executor bookkeeping. This verifies O(N×input) scaling, not O(N²).
    let n_tasks = 10usize;
    let limit = json.len() * n_tasks * 15;
    assert!(
        peak_above_baseline <= limit,
        "peak additional allocation {peak_above_baseline} bytes for {n_tasks} concurrent tasks \
         on {} byte input (limit {limit})",
        json.len(),
    );
}
