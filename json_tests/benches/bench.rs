#![cfg_attr(feature = "nightly-testing", feature(plugin))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]

#![cfg_attr(not(feature = "with-syntex"), feature(custom_attribute, custom_derive, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(serde_macros, indoc))]

#![feature(test)]

extern crate num_traits;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate test;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/bench.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("bench.rs.in");
