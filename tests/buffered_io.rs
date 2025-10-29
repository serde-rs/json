//! Tests for the `BufferedIoRead` implementation, focusing on
//! forcing buffer refills at critical boundaries.

use serde::Deserialize;
use serde_json::de::BufferedIoRead;
use serde_json::de::Deserializer;
use serde_json::value::Value;
use std::io::{self, Read};

/// A custom reader that wraps a byte slice and yields it in
/// fixed-size chunks. This is the key to testing buffer boundaries.
struct SlowReader<'a> {
    data: &'a [u8],
    chunk_size: usize,
}

impl<'a> SlowReader<'a> {
    fn new(data: &'a [u8], chunk_size: usize) -> Self {
        // Chunk size must be > 0
        assert!(chunk_size > 0, "chunk_size must be positive");
        SlowReader { data, chunk_size }
    }
}

impl<'a> Read for SlowReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Determine how much we can read:
        // min(buffer_len, chunk_size, data_remaining)
        let max_read = std::cmp::min(buf.len(), self.chunk_size);
        let bytes_to_read = std::cmp::min(max_read, self.data.len());

        if bytes_to_read == 0 {
            return Ok(0); // EOF
        }

        let (chunk, rest) = self.data.split_at(bytes_to_read);
        buf[..bytes_to_read].copy_from_slice(chunk);
        self.data = rest;

        Ok(bytes_to_read)
    }
}

/// Helper to run a test with a specific internal buffer size
/// and a specific I/O chunk size.
fn run_test<'de, T>(
    json: &'static [u8],
    io_chunk_size: usize,
    internal_buf_size: usize,
) -> serde_json::Result<T>
where
    T: Deserialize<'de>,
{
    let slow_reader = SlowReader::new(json, io_chunk_size);

    // We must use a dynamic buffer to set its size at runtime.
    let buffer: Vec<u8> = vec![0; internal_buf_size];
    let buffered_read = BufferedIoRead::new(slow_reader, buffer);

    let mut de = Deserializer::new(buffered_read);
    T::deserialize(&mut de)
}

#[test]
fn test_string_across_buffer_boundary() {
    // This JSON is 26 bytes.
    let json = br#"{"key": "long-value-str"}"#;

    // Test 1: Internal buffer boundary is in the middle of the string.
    // IO chunks of 5 bytes. Internal buffer of 16 bytes.
    // 1. Read: 5 bytes ("{"key")
    // 2. Read: 5 bytes (": "lon")
    // 3. Read: 5 bytes ("g-val") -> Buffer is at 15 bytes
    // 4. Read: 1 byte ("u") -> Buffer is full (16 bytes)
    //    `parse_str` will be called. It consumes to the quote.
    //    `next` will trigger refill.
    let res = run_test::<Value>(json, 5, 16).unwrap();
    assert_eq!(res["key"], "long-value-str");
}

#[test]
fn test_escape_sequence_at_boundary() {
    // The '\u' will land right at the buffer boundary.
    // `{"key": "val\u0041"}`
    // `{"key": "val` is 13 bytes.
    let json = br#"{"key": "val\u0041"}"#; // 21 bytes

    // Test 1: '\u' is split.
    // IO chunks of 10. Internal buffer of 13.
    // 1. Read: 10 bytes (`{"key": "v`)
    // 2. Read: 3 bytes (`al`) -> Buffer is full (13 bytes: `{"key": "val`)
    //    `parse_str` will read to `val`.
    //    `next` will be called, sees `\`.
    //    `decode_hex_escape` will call `next()` 4 times, forcing refills.
    let res = run_test::<Value>(json, 10, 13).unwrap();
    assert_eq!(res["key"], "valA");
}

#[test]
fn test_string_parsing_with_many_refills() {
    // A long string.
    let json = br#"{"key": "abcdefghijklmnopqrstuvwxyz"}"#;

    // Use a tiny internal buffer (8 bytes) and tiny IO chunks (3 bytes).
    // This forces `parse_str_bytes` to loop and refill many times.
    let res = run_test::<Value>(json, 3, 8).unwrap();
    assert_eq!(res["key"], "abcdefghijklmnopqrstuvwxyz");
}

#[test]
fn test_ignore_str_with_many_refills() {
    // `ignore_str` will be called on "z_key".
    let json = br#"{"key": "value", "z_key": "another-long-string-to-ignore"}"#;

    #[derive(Deserialize)]
    struct MyStruct {
        key: String,
    }

    // Force refills
    let res = run_test::<MyStruct>(json, 5, 16).unwrap();

    assert_eq!(res.key, "value");

    // The test passes if `ignore_str` correctly consumed the rest
    // and `deserialize` didn't EOF unexpectedly.
}

#[test]
fn test_error_at_boundary() {
    // An invalid UTF-8 sequence `\xFF` at a buffer boundary.
    // `{"key": "` is 9 bytes.
    let json = b"{\"key\": \"\xFF\"}";

    // Test: Force buffer to end right before the `\xFF`.
    // IO: 5 bytes. Internal Buffer: 9 bytes.
    // 1. Read: 5 bytes (`{"key`)
    // 2. Read: 4 bytes (`: "`) -> Buffer is full (9 bytes)
    //    `parse_str` starts.
    //    `next()` for `\xFF` will trigger refill.
    // 3. Read: 1 byte (`\xFF`)
    //    This byte is now in the buffer.
    let err = run_test::<Value>(json, 5, 9).unwrap_err();
    assert!(err.is_syntax(), "{err}");
    assert_eq!(err.line(), 1);
    assert_eq!(err.column(), 11); // Error is at the `\xFF`
}

/// RawValue

#[cfg(feature = "raw_value")]
#[derive(Deserialize)]
struct Wrapper {
    before: String,
    raw: Box<serde_json::value::RawValue>,
    after: u32,
}

#[cfg(feature = "raw_value")]
#[test]
fn test_raw_value_fits_in_one_buffer() {
    let json = br#"{
        "before": "value",
        "raw": { "a": 1, "b": [1, 2] },
        "after": 123
    }"#;

    // IO chunk and buffer are large. No refills needed.
    let res: Wrapper = run_test(json, 1024, 1024).unwrap();

    assert_eq!(res.before, "value");
    assert_eq!(res.after, 123);
    assert_eq!(res.raw.get(), r#"{ "a": 1, "b": [1, 2] }"#);
}

#[cfg(feature = "raw_value")]
#[test]
fn test_raw_value_spans_two_buffers() {
    let json = br#"{
        "before": "value",
        "raw": { "a": 1, "b": [1, 2] },
        "after": 123
    }"#; // "raw": { "a": 1, "b"

    // The "raw" value starts at byte 31.
    // Let's set the internal buffer to 40.
    // Let's set IO chunks to 25.

    // 1. Read: 25 bytes. (Buffer: `{"before": "value", "raw"`)
    // 2. Read: 15 bytes. (Buffer full at 40: `{"before": "value", "raw": { "a": 1,`)
    //
    // `begin_raw_buffering` is called. `raw_buffering_start_index` is set.
    // `end_raw_buffering` will be called, forcing a refill.
    // `refill` will copy ` { "a": 1,` into the owned `raw_buffer`.
    //
    // 3. Read: 25 bytes...
    // The test ensures the stitch-up of the `raw_buffer` in `refill`
    // and the final chunk in `end_raw_buffering` works.

    let res: Wrapper = run_test(json, 25, 40).unwrap();

    assert_eq!(res.before, "value");
    assert_eq!(res.after, 123);
    assert_eq!(res.raw.get(), r#"{ "a": 1, "b": [1, 2] }"#);
}

#[cfg(feature = "raw_value")]
#[test]
fn test_raw_value_spans_many_buffers() {
    let json = br#"{
        "before": "value",
        "raw": { "a": "---long string---", "b": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] },
        "after": 123
    }"#;

    // This test uses tiny buffers to force many refills *during*
    // the `raw_value` parsing.

    let res: Wrapper = run_test(json, 5, 16).unwrap(); // 5-byte IO, 16-byte internal

    assert_eq!(res.before, "value");
    assert_eq!(res.after, 123);
    assert_eq!(
        res.raw.get(),
        r#"{ "a": "---long string---", "b": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] }"#
    );
}
