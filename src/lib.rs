extern crate serde;
extern crate serde_json;

use std::io;

use serde::de::DeserializeOwned;
use serde_json::{Error, Result};

/// Like [`from_reader`] but eagerly reads the content of the reader to a string
/// and delegates to `from_str`.
///
/// [`from_reader`]: https://docs.serde.rs/serde_json/fn.from_reader.html
pub fn from_reader_eager<R, T>(mut reader: R) -> Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    let mut s = String::new();
    if let Err(io_err) = reader.read_to_string(&mut s) {
        // Error::io is private to serde_json. Do not use.
        return Err(Error::io(io_err));
    }
    serde_json::from_str(&s)
}
