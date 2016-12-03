use std::{f64, i64, u64};
use test::Bencher;
use serde_json;

#[bench]
fn bench_deserializer_i64(b: &mut Bencher) {
    let s = serde_json::to_string(&i64::MIN).unwrap();
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: i64 = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_u64(b: &mut Bencher) {
    let s = serde_json::to_string(&u64::MAX).unwrap();
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: u64 = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_f64_epsilon(b: &mut Bencher) {
    let s = serde_json::to_string(&f64::EPSILON).unwrap();
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: f64 = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_f64_min(b: &mut Bencher) {
    let s = serde_json::to_string(&f64::MIN).unwrap();
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: f64 = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_f64_max(b: &mut Bencher) {
    let s = "1.7976931348623157e+308";
    let s = serde_json::to_string(&f64::MAX).unwrap();
    println!("{}", s);
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: f64 = serde_json::from_str(&s).unwrap();
    });
}

fn make_string(pattern: &str) -> String {
    let times = 1000;
    let mut s = String::with_capacity(pattern.len() * times + 2);

    s.push('"');

    for _ in 0..times {
        s.push_str(pattern);
    }

    s.push('"');

    s
}

#[bench]
fn bench_deserializer_string(b: &mut Bencher) {
    let s = make_string("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456790");
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: String = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_escapes(b: &mut Bencher) {
    let s = make_string(r"\b\f\n\r\t");
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: String = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_deserializer_unicode(b: &mut Bencher) {
    let s = make_string(r"\uD834\uDD1E");
    b.bytes = s.len() as u64;

    b.iter(|| {
        let _s: String = serde_json::from_str(&s).unwrap();
    });
}

#[bench]
fn bench_pointer(b: &mut Bencher) {
    use serde_json::{self, Value};

    let data: Value = serde_json::from_str(r#"{
        "foo": ["bar", "baz"],
        "": 0,
        "a/b": 1,
        "c%d": 2,
        "e^f": 3,
        "g|h": 4,
        "i\\j": 5,
        "k\"l": 6,
        " ": 7,
        "m~n": 8
    }"#).unwrap();

    b.iter(|| {
        let _ = data.pointer("").unwrap();
        let _ = data.pointer("/foo/0").unwrap();
        let _ = data.pointer("/unknown");
    });
}
