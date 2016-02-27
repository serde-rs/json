Serde JSON Serialization Library
================================

[![Build status](https://api.travis-ci.org/serde-rs/json.png)](https://travis-ci.org/serde-rs/json)
[![Coverage Status](https://coveralls.io/repos/serde-rs/json/badge.svg?branch=master&service=github)](https://coveralls.io/github/serde-rs/json?branch=master)
[![Latest Version](https://img.shields.io/crates/v/serde_json.svg)](https://crates.io/crates/serde\_json)

[Documentation](https://serde-rs.github.io/json/serde_json)

This crate is a Rust library for parsing and generating the
[JSON](http://json.org) (JavaScript Object Notation) file format. It is built
upon [Serde](https://github.com/serde-rs/serde), a high performance generic
serialization framework.

Installation
============

This crate works with Cargo and can be found on
[crates.io](https://crates.io/crates/serde_json) with a `Cargo.toml` like:

```toml
[dependencies]
serde = "*"
serde_json = "*"
```

Using Serde JSON
================

`serde_json` is very simple to use out of the box:

```rust
extern crate serde;
extern crate serde_json;

use std::collections::BTreeMap;

fn main() {
    let mut map = BTreeMap::new();
    map.insert("x".to_string(), 1.0);
    map.insert("y".to_string(), 2.0);

    let s = serde_json::to_string(&map).unwrap();
    assert_eq!(s, "{\"x\":1,\"y\":2}");

    let deserialized_map: BTreeMap<String, f64> = serde_json::from_str(&s).unwrap();
    assert_eq!(map, deserialized_map);
}
```

It also can be used with Serde's automatic serialization library,
`serde_macros`. First add this to `Cargo.toml`:

```toml
[dependencies]
...
serde = "*"
serde_macros = "*"
...
```

Then run:

```rust
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let point = Point { x: 1.0, y: 2.0 };

    let s = serde_json::to_string(&point).unwrap();
    assert_eq!(s, "{\"x\":1,\"y\":2}");

    let deserialized_point: Point = serde_json::from_str(&s).unwrap();
    assert_eq!(point, deserialized_point);
}
```
