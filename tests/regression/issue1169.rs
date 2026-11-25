use serde_json::to_writer;
use std::error::Error as _;
use std::io::{self, Write};

struct BrokenPipeWriter;

impl Write for BrokenPipeWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
    }
}

#[test]
fn test() {
    // Writing JSON to a writer that fails produces an error of category Io
    // whose source must be the underlying io::Error itself, so that callers
    // can inspect its ErrorKind (e.g. to filter out broken pipe errors).
    let err = to_writer(BrokenPipeWriter, "x").unwrap_err();
    let source = err
        .source()
        .expect("io error must expose the underlying io::Error as its source");
    let io_err = source
        .downcast_ref::<io::Error>()
        .expect("source must be the io::Error");
    assert_eq!(io_err.kind(), io::ErrorKind::BrokenPipe);
}
