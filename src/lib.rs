// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Serde JSON
//!
//! JSON is a ubiquitous open-standard format that uses human-readable text to
//! transmit data objects consisting of key-value pairs.
//!
//! ```json,ignore
//! {
//!   "name": "John Doe",
//!   "age": 43,
//!   "address": {
//!     "street": "10 Downing Street",
//!     "city": "London"
//!   },
//!   "phones": [
//!     "+44 1234567",
//!     "+44 2345678"
//!   ]
//! }
//! ```
//!
//! There are three common ways that you might find yourself needing to work
//! with JSON data in Rust.
//!
//!  - **As text data.** An unprocessed string of JSON data that you receive on
//!    an HTTP endpoint, read from a file, or prepare to send to a remote
//!    server.
//!  - **As an untyped or loosely typed representation.** Maybe you want to
//!    check that some JSON data is valid before passing it on, but without
//!    knowing the structure of what it contains. Or you want to do very basic
//!    manipulations like insert a key in a particular spot.
//!  - **As a strongly typed Rust data structure.** When you expect all or most
//!    of your data to conform to a particular structure and want to get real
//!    work done without JSON's loosey-goosey nature tripping you up.
//!
//! Serde JSON provides efficient, flexible, safe ways of converting data
//! between each of these representations.
//!
//! # Operating on untyped JSON values
//!
//! Any valid JSON data can be manipulated in the following recursive enum
//! representation. This data structure is [`serde_json::Value`][value].
//!
//! ```rust
//! # use serde_json::{Number, Map};
//! #
//! # #[allow(dead_code)]
//! enum Value {
//!     Null,
//!     Bool(bool),
//!     Number(Number),
//!     String(String),
//!     Array(Vec<Value>),
//!     Object(Map<String, Value>),
//! }
//! ```
//!
//! A string of JSON data can be parsed into a `serde_json::Value` by the
//! [`serde_json::from_str`][from_str] function. There is also
//! [`from_slice`][from_slice] for parsing from a byte slice &[u8] and
//! [`from_reader`][from_reader] for parsing from any `io::Read` like a File or
//! a TCP stream.
//!
//! ```rust
//! extern crate serde_json;
//!
//! use serde_json::{Value, Error};
//!
//! fn untyped_example() -> Result<(), Error> {
//!     // Some JSON input data as a &str. Maybe this comes from the user.
//!     let data = r#"{
//!                     "name": "John Doe",
//!                     "age": 43,
//!                     "phones": [
//!                       "+44 1234567",
//!                       "+44 2345678"
//!                     ]
//!                   }"#;
//!
//!     // Parse the string of data into serde_json::Value.
//!     let v: Value = serde_json::from_str(data)?;
//!
//!     // Access parts of the data by indexing with square brackets.
//!     println!("Please call {} at the number {}", v["name"], v["phones"][0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     untyped_example().unwrap();
//! # }
//! ```
//!
//! The `Value` representation is sufficient for very basic tasks but can be
//! tedious to work with for anything more significant. Error handling is
//! verbose to implement correctly, for example imagine trying to detect the
//! presence of unrecognized fields in the input data. The compiler is powerless
//! to help you when you make a mistake, for example imagine typoing `v["name"]`
//! as `v["nmae"]` in one of the dozens of places it is used in your code.
//!
//! # Parsing JSON as strongly typed data structures
//!
//! Serde provides a powerful way of mapping JSON data into Rust data structures
//! largely automatically.
//!
//! ```rust
//! extern crate serde;
//! extern crate serde_json;
//!
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use serde_json::Error;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: u8,
//!     phones: Vec<String>,
//! }
//!
//! fn typed_example() -> Result<(), Error> {
//!     // Some JSON input data as a &str. Maybe this comes from the user.
//!     let data = r#"{
//!                     "name": "John Doe",
//!                     "age": 43,
//!                     "phones": [
//!                       "+44 1234567",
//!                       "+44 2345678"
//!                     ]
//!                   }"#;
//!
//!     // Parse the string of data into a Person object. This is exactly the
//!     // same function as the one that produced serde_json::Value above, but
//!     // now we are asking it for a Person as output.
//!     let p: Person = serde_json::from_str(data)?;
//!
//!     // Do things just like with any other Rust data structure.
//!     println!("Please call {} at the number {}", p.name, p.phones[0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     typed_example().unwrap();
//! # }
//! ```
//!
//! This is the same `serde_json::from_str` function as before, but this time we
//! assign the return value to a variable of type `Person` so Serde will
//! automatically interpret the input data as a `Person` and produce informative
//! error messages if the layout does not conform to what a `Person` is expected
//! to look like.
//!
//! Any type that implements Serde's `Deserialize` trait can be deserialized
//! this way. This includes built-in Rust standard library types like `Vec<T>`
//! and `HashMap<K, V>`, as well as any structs or enums annotated with
//! `#[derive(Deserialize)]`.
//!
//! Once we have `p` of type `Person`, our IDE and the Rust compiler can help us
//! use it correctly like they do for any other Rust code. The IDE can
//! autocomplete field names to prevent typos, which was impossible in the
//! `serde_json::Value` representation. And the Rust compiler can check that
//! when we write `p.phones[0]`, then `p.phones` is guaranteed to be a
//! `Vec<String>` so indexing into it makes sense and produces a `String`.
//!
//! # Constructing JSON values
//!
//! Serde JSON provides a [`json!` macro][macro] to build `serde_json::Value`
//! objects with very natural JSON syntax. In order to use this macro,
//! `serde_json` needs to be imported with the `#[macro_use]` attribute.
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_json;
//!
//! fn main() {
//!     // The type of `john` is `serde_json::Value`
//!     let john = json!({
//!       "name": "John Doe",
//!       "age": 43,
//!       "phones": [
//!         "+44 1234567",
//!         "+44 2345678"
//!       ]
//!     });
//!
//!     println!("first phone number: {}", john["phones"][0]);
//!
//!     // Convert to a string of JSON and print it out
//!     println!("{}", john.to_string());
//! }
//! ```
//!
//! The `Value::to_string()` function converts a `serde_json::Value` into a
//! `String` of JSON text.
//!
//! One neat thing about the `json!` macro is that variables and expressions can
//! be interpolated directly into the JSON value as you are building it. Serde
//! will check at compile time that the value you are interpolating is able to
//! be represented as JSON.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate serde_json;
//! #
//! # fn random_phone() -> u16 { 0 }
//! #
//! # fn main() {
//! let full_name = "John Doe";
//! let age_last_year = 42;
//!
//! // The type of `john` is `serde_json::Value`
//! let john = json!({
//!   "name": full_name,
//!   "age": age_last_year + 1,
//!   "phones": [
//!     format!("+44 {}", random_phone())
//!   ]
//! });
//! #     let _ = john;
//! # }
//! ```
//!
//! This is amazingly convenient but we have the problem we had before with
//! `Value` which is that the IDE and Rust compiler cannot help us if we get it
//! wrong. Serde JSON provides a better way of serializing strongly-typed data
//! structures into JSON text.
//!
//! # Creating JSON by serializing data structures
//!
//! A data structure can be converted to a JSON string by
//! [`serde_json::to_string`][to_string]. There is also
//! [`serde_json::to_vec`][to_vec] which serializes to a `Vec<u8>` and
//! [`serde_json::to_writer`][to_writer] which serializes to any `io::Write`
//! such as a File or a TCP stream.
//!
//! ```rust
//! extern crate serde;
//! extern crate serde_json;
//!
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use serde_json::Error;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Address {
//!     street: String,
//!     city: String,
//! }
//!
//! fn print_an_address() -> Result<(), Error> {
//!     // Some data structure.
//!     let address = Address {
//!         street: "10 Downing Street".to_owned(),
//!         city: "London".to_owned(),
//!     };
//!
//!     // Serialize it to a JSON string.
//!     let j = serde_json::to_string(&address)?;
//!
//!     // Print, write to a file, or send to an HTTP server.
//!     println!("{}", j);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     print_an_address().unwrap();
//! # }
//! ```
//!
//! Any type that implements Serde's `Serialize` trait can be serialized this
//! way. This includes built-in Rust standard library types like `Vec<T>` and
//! `HashMap<K, V>`, as well as any structs or enums annotated with
//! `#[derive(Serialize)]`.
//!
//! [value]: https://docs.serde.rs/serde_json/value/enum.Value.html
//! [from_str]: https://docs.serde.rs/serde_json/de/fn.from_str.html
//! [from_slice]: https://docs.serde.rs/serde_json/de/fn.from_slice.html
//! [from_reader]: https://docs.serde.rs/serde_json/de/fn.from_reader.html
//! [to_string]: https://docs.serde.rs/serde_json/ser/fn.to_string.html
//! [to_vec]: https://docs.serde.rs/serde_json/ser/fn.to_vec.html
//! [to_writer]: https://docs.serde.rs/serde_json/ser/fn.to_writer.html
//! [macro]: https://docs.serde.rs/serde_json/macro.json.html

#![doc(html_root_url = "https://docs.rs/serde_json/1.0.3")]
#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
// Because of "JavaScript"... fixed in Manishearth/rust-clippy#1071
#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
// Whitelisted clippy_pedantic lints
#![cfg_attr(feature = "cargo-clippy", allow(
// Deserializer::from_str, into_iter
    should_implement_trait,
// integer and float ser/de requires these sorts of casts
    cast_possible_truncation,
    cast_possible_wrap,
    cast_precision_loss,
    cast_sign_loss,
// string ser/de uses indexing and slicing
    indexing_slicing,
// things are often more readable this way
    cast_lossless,
    shadow_reuse,
    shadow_unrelated,
    single_match_else,
    stutter,
    use_self,
// not practical
    missing_docs_in_private_items,
))]

#![deny(missing_docs)]

extern crate num_traits;
#[macro_use]
extern crate serde;
extern crate itoa;
extern crate dtoa;
#[cfg(feature = "preserve_order")]
extern crate linked_hash_map;

#[doc(inline)]
pub use self::de::{Deserializer, StreamDeserializer, from_reader, from_slice, from_str};
#[doc(inline)]
pub use self::error::{Error, Result};
#[doc(inline)]
pub use self::ser::{Serializer, to_string, to_string_pretty, to_vec, to_vec_pretty, to_writer,
                    to_writer_pretty};
#[doc(inline)]
pub use self::value::{Map, Number, Value, from_value, to_value};

#[macro_use]
mod macros;

pub mod de;
pub mod error;
pub mod map;
pub mod ser;
pub mod value;

mod iter;
mod number;
mod read;
