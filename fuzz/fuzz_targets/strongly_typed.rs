#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_value, to_string};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
enum Enum {
    A(u32),
    B(i32, i32),
    C { x: i32, y: i32 },
    U,
}

#[derive(Serialize, Deserialize)]
struct WrappedInt(i32);

#[derive(Serialize, Deserialize)]
struct Numbers {
    a: Option<i8>,
    b: Option<u8>,
    c: Option<i16>,
    d: Option<u16>,
    e: Option<i32>,
    f: Option<u32>,
    g: Option<i64>,
    h: Option<u64>,
    i: Option<i128>,
    j: Option<u128>,
    k: Option<f32>,
    l: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct CommonTypes {
    a: Option<char>,
    b: Option<String>,
    c: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct Arrays {
    a: Option<Vec<u8>>,
    b: Option<[u8; 3]>,
}

#[derive(Serialize, Deserialize)]
struct Maps {
    a: Option<HashMap<String, i32>>,
    b: Option<HashMap<char, i32>>,
    c: Option<HashMap<i8, i32>>,
    d: Option<HashMap<i16, i32>>,
    e: Option<HashMap<i32, i32>>,
    f: Option<HashMap<i64, i32>>,
    g: Option<HashMap<i128, i32>>,
    h: Option<HashMap<u8, i32>>,
    i: Option<HashMap<u16, i32>>,
    j: Option<HashMap<u32, i32>>,
    k: Option<HashMap<u64, i32>>,
    l: Option<HashMap<u128, i32>>,
    o: Option<HashMap<bool, i32>>,
}

#[derive(Serialize, Deserialize)]
struct Others {
    a: Option<(f32, char, i8)>,
    b: Option<Enum>,
    c: Option<std::time::Duration>,
    d: Option<std::time::SystemTime>,
    e: Option<WrappedInt>,
}

#[derive(Deserialize, Serialize)]
struct Data {
    a: Option<Numbers>,
    b: Option<Maps>,
    c: Option<CommonTypes>,
    d: Option<Arrays>,
    e: Option<Others>,
}

fuzz_target!(|data: &[u8]| {
    if let Ok(d) = from_slice::<Data>(data) {
        let _ = to_string(&d);
    }

    if let Ok(value) = from_slice(data) {
        let _ = from_value::<Data>(value);
    }
});
