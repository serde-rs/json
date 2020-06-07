#![allow(
    clippy::comparison_chain,
    clippy::excessive_precision,
    clippy::float_cmp
)]

#[path = "../src/lexical/mod.rs"]
mod lexical;

mod lib {
    pub use std::vec::Vec;
    pub use std::{cmp, iter, mem, ops};
}

#[path = "lexical/algorithm.rs"]
mod algorithm;

#[path = "lexical/exponent.rs"]
mod exponent;

#[path = "lexical/float.rs"]
mod float;

#[path = "lexical/math.rs"]
mod math;

#[path = "lexical/num.rs"]
mod num;

#[path = "lexical/parse.rs"]
mod parse;

#[path = "lexical/rounding.rs"]
mod rounding;
