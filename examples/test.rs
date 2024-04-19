//! Tests feature unification
//! Run via:
//!
//! ```sh
//! cargo run --example test --features use-core2
//! cargo run --example test --features alloc,use-core2 --no-default-features
//! ```


use serde_json::alloc_io::{Write, Result, Error, ErrorKind};


struct Buffer {}

impl Write for Buffer {
	fn write(&mut self, _bytes: &[u8]) -> Result<usize> { panic!() }
	fn flush(&mut self) -> Result<()> {
		Err(Error::new(ErrorKind::Other, "flush not implemented"))
	}
}

impl core2::io::Write for Buffer {
	fn write(&mut self, _bytes: &[u8]) -> core2::io::Result<usize> { panic!() }
	fn flush(&mut self) -> core2::io::Result<()> { panic!() }
}

fn main() {
	println!("Hello, world!");
	let _x = &mut Buffer {} as &mut dyn Write;
}
