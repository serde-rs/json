//! Tests feature unification
//! Run via:
//!
//! ```sh
//! cargo run --example test
//! cargo run --example test --features alloc --no-default-features
//! ```


use serde_json::io::{Write, Result, Error, ErrorKind};

struct X {
}

impl Write for X {
	fn write(&mut self, _buf: &[u8]) -> Result<usize> {
		Ok(0)
	}

	fn flush(&mut self) -> Result<()> {
		Err(Error::new(ErrorKind::Other, "flush not implemented"))
	}
}

fn main() {
	let _x = &mut X{} as &mut dyn Write;
}
