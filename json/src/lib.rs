//! JSON and serialization
//!
//! # What is JSON?
//!
//! JSON (JavaScript Object Notation) is a way to write data in JavaScript.  Like XML, it allows to
//! encode structured data in a text format that can be easily read by humans.  Its simple syntax
//! and native compatibility with JavaScript have made it a widely used format.
//!
//! Data types that can be encoded are JavaScript types (see the `serde_json:Value` enum for more
//! details):
//!
//! * `Bool`: equivalent to rust's `bool`
//! * `I64`: equivalent to rust's `i64`
//! * `U64`: equivalent to rust's `u64`
//! * `F64`: equivalent to rust's `f64`
//! * `String`: equivalent to rust's `String`
//! * `Array`: equivalent to rust's `Vec<T>`, but also allowing objects of different types in the
//!    same array
//! * `Object`: equivalent to rust's `BTreeMap<String, serde_json::Value>`; set the
//!    `preserve_order` feature to use `LinkedHashMap<String, serde_json::Value>` instead
//! * `Null`
//!
//! An object is a series of string keys mapping to values, in `"key": value` format.  Arrays are
//! enclosed in square brackets ([ ... ]) and objects in curly brackets ({ ... }).  A simple JSON
//! document encoding a person, his/her age, address and phone numbers could look like
//!
//! ```ignore
//! {
//!     "FirstName": "John",
//!     "LastName": "Doe",
//!     "Age": 43,
//!     "Address": {
//!         "Street": "Downing Street 10",
//!         "City": "London",
//!         "Country": "Great Britain"
//!     },
//!     "PhoneNumbers": [
//!         "+44 1234567",
//!         "+44 2345678"
//!     ]
//! }
//! ```
//!
//! If we assume that `FirstName` is optional and all other fields are mandatory, the above JSON could
//! correspond to the following Rust structs:
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! struct Data {
//!     #[serde(rename="FirstName")] // to comply with Rust coding standards
//!     first_name: Option<String>,
//!     LastName: String,
//!     Age: u32,
//!     Address: Address,
//!     PhoneNumbers: Vec<String>,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct Address {
//!     Street: String,
//!     City: String,
//!     Country: String,
//! }
//! ```
//!
//! # Type-based Serialization and Deserialization
//!
//! Serde provides a mechanism for low boilerplate serialization & deserialization of values to and
//! from JSON via the serialization API.  To be able to serialize a piece of data, it must implement
//! the `serde::Serialize` trait.  To be able to deserialize a piece of data, it must implement the
//! `serde::Deserialize` trait.  Serde provides provides an annotation to automatically generate
//! the code for these traits: `#[derive(Serialize, Deserialize)]`.
//!
//! The JSON API also provides an enum `serde_json::Value` and a method `to_value` to serialize
//! objects.  A `serde_json::Value` value can be serialized as a string or buffer using the
//! functions described above.  You can also use the `json::Serializer` object, which implements the
//! `Serializer` trait.
//!
//! # Examples of use
//!
//! ## Parsing a `str` to `Value` and reading the result
//!
//! ```rust
//! extern crate serde_json;
//!
//! use serde_json::Value;
//!
//! fn main() {
//!     let data: Value = serde_json::from_str("{\"foo\": 13, \"bar\": \"baz\"}").unwrap();
//!     println!("data: {:?}", data);
//!     // data: {"bar":"baz","foo":13}
//!     println!("object? {}", data.is_object());
//!     // object? true
//!
//!     let obj = data.as_object().unwrap();
//!     let foo = obj.get("foo").unwrap();
//!
//!     println!("array? {:?}", foo.as_array());
//!     // array? None
//!     println!("u64? {:?}", foo.as_u64());
//!     // u64? Some(13u64)
//!
//!     for (key, value) in obj.iter() {
//!         println!("{}: {}", key, match *value {
//!             Value::U64(v) => format!("{} (u64)", v),
//!             Value::String(ref v) => format!("{} (string)", v),
//!             _ => format!("other")
//!         });
//!     }
//!     // bar: baz (string)
//!     // foo: 13 (u64)
//! }
//! ```

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", deny(clippy, clippy_pedantic))]
// Because of "JavaScript"... fixed in Manishearth/rust-clippy#1071
#![cfg_attr(feature = "clippy", allow(doc_markdown))]
// Whitelisted clippy_pedantic lints
#![cfg_attr(feature = "clippy", allow(
// integer and float ser/de requires these sorts of casts
    cast_possible_truncation,
    cast_possible_wrap,
    cast_precision_loss,
    cast_sign_loss,
// string ser/de uses indexing and slicing
    indexing_slicing,
// things are often more readable this way
    shadow_reuse,
    shadow_unrelated,
    single_match_else,
    stutter,
// not practical
    missing_docs_in_private_items,
))]

#![deny(missing_docs)]

extern crate num_traits;
extern crate core;
#[macro_use]
extern crate serde;
extern crate itoa;
extern crate dtoa;
#[cfg(feature = "preserve_order")]
extern crate linked_hash_map;

pub use self::de::{Deserializer, StreamDeserializer, from_iter, from_reader,
                   from_slice, from_str};
pub use self::error::{Error, ErrorCode, Result};
pub use self::ser::{Serializer, escape_str, to_string, to_string_pretty,
                    to_vec, to_vec_pretty, to_writer, to_writer_pretty};
pub use self::value::{Map, Value, from_value, to_value};

pub mod builder;
pub mod de;
pub mod error;
pub mod ser;
pub mod value;

mod read;
