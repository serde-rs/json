# Serde JSON &emsp; [![Build Status](https://api.travis-ci.org/serde-rs/json.svg?branch=master)](https://travis-ci.org/serde-rs/json) [![Latest Version](https://img.shields.io/crates/v/serde_json.svg)](https://crates.io/crates/serde\_json)

**Serde is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

You may be looking for:

- [JSON API documentation](http://docs.serde.rs/serde_json/)
- [Serde API documentation](https://docs.serde.rs/serde/)
- [Detailed documentation about Serde](https://serde.rs/)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/codegen.html)
- [Release notes](https://github.com/serde-rs/json/releases)

## Serde JSON in action

```toml
[dependencies]
serde_json = "0.8"
```

Out of the box, Serde JSON is able to serialize and deserialize most Rust
standard library types as JSON (primitives, `String`, `HashMap`, tuples, `Vec`,
`Option`, etc). This works equally well on stable and nightly compilers and does
not require code generation or a build script.

Serde JSON can also serialize and deserialize structs and enums defined in your
program. See Serde's [codegen documentation](https://serde.rs/codegen.html) for
how to set this up.

```rust
#[macro_use]
extern crate serde_derive;

extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}
```

## Getting help

Serde developers live in the #serde channel on
[`irc.mozilla.org`](https://wiki.mozilla.org/IRC). The #rust channel is also a
good resource with generally faster response time but less specific knowledge
about Serde. If IRC is not your thing, we are happy to respond to [GitHub
issues](https://github.com/serde-rs/json/issues/new) as well.

## License

Serde JSON is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde JSON by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
