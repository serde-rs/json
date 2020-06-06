// The code in this module is derived from the `lexical` crate by @Alexhuszagh
// which the author condensed into this minimal subset for use in serde_json.
// For the serde_json use case we care more about reliably round tripping all
// possible floating point values than about parsing any arbitrarily long string
// of digits with perfect accuracy, as the latter would take a high cost in
// compile time and performance.
//
// Dual licensed as MIT and Apache 2.0 just like the rest of serde_json, but
// copyright Alexander Huszagh.

//! Fast, minimal float-parsing algorithm.

// MODULES
mod algorithm;
mod bhcomp;
mod bignum;
mod cached;
mod cached_float80;
mod digit;
mod errors;
mod exponent;
mod float;
mod large_powers;
mod math;
mod num;
mod parse;
mod rounding;
mod shift;
mod slice;
mod small_powers;

#[cfg(limb_width_32)]
mod large_powers32;

#[cfg(limb_width_64)]
mod large_powers64;

// API
pub use self::num::Float;
pub use self::parse::parse_float;
