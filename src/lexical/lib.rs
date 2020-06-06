//! Fast, minimal float-parsing algorithm.

// FEATURES

// Require intrinsics in a no_std context.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(no_alloc), not(feature = "std")))]
extern crate alloc;

/// Facade around the core features for name mangling.
pub(crate) mod lib {
#[cfg(feature = "std")]
pub(crate) use std::*;

#[cfg(not(feature = "std"))]
pub(crate) use core::*;

#[cfg(all(not(no_alloc), feature = "std"))]
pub(crate) use std::vec::Vec;

#[cfg(all(not(no_alloc), not(feature = "std")))]
pub(crate) use ::alloc::vec::Vec;
}

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
pub use self::parse::parse_float;
pub use self::num::Float;

