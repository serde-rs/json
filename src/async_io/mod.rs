//! Async JSON (de)serialization — feature `async`.
//!
//! # Design
//!
//! serde_json's deserializer and serializer are synchronous by design.
//! The only correct integration point for async I/O is to buffer the byte
//! stream to memory, then hand off to the existing sync code path:
//!
//! - **Read path**: drain `AsyncRead` → `Vec<u8>` → `from_slice`
//! - **Write path**: `to_vec` → drain `Vec<u8>` → `AsyncWrite`
//!
//! Memory cost is bounded by the size of a single JSON value, not the stream.
//!
//! # Runtime agnosticism
//!
//! [`AsyncRead`] and [`AsyncWrite`] are self-defined here with signatures
//! identical to `futures::io::{AsyncRead, AsyncWrite}`.  A single forwarding
//! impl bridges any runtime's I/O traits:
//!
//! ```ignore
//! impl serde_json::AsyncRead for Compat<tokio::io::DuplexStream> {
//!     fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
//!         -> Poll<io::Result<usize>>
//!     {
//!         // tokio's AsyncRead → serde_json::AsyncRead
//!         use tokio::io::AsyncRead as _;
//!         Pin::new(&mut self.0).poll_read(cx, &mut ReadBuf::new(buf))
//!             .map_ok(|_| buf.len())
//!     }
//! }
//! ```
//!
//! # MSRV
//!
//! This feature requires Rust **1.75** (async fn in trait).
//! The surrounding crate MSRV remains 1.71.

mod de;
mod read;
mod ser;
mod write;

pub use de::{from_async_reader, AsyncDeserializer};
pub use read::AsyncRead;
pub use ser::{to_async_writer, AsyncSerializer};
pub use write::AsyncWrite;
