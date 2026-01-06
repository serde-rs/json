#![no_main]

use libfuzzer_sys::fuzz_target;
use sciformats_serde_json::{from_slice, Value};

fuzz_target!(|data: &[u8]| {
    _ = from_slice::<Value>(data);
});
