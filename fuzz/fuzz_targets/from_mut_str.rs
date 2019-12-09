#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json;

fuzz_target!(|data: String| {
    let mut data = data;
    let _ = serde_json::from_mut_str::<serde_json::Value>(&mut data);
});
