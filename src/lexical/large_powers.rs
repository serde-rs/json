// Adapted from https://github.com/Alexhuszagh/rust-lexical.

//! Precalculated large powers for limbs.

#[cfg(arithmetic32)]
pub(crate) use super::large_powers32::*;

#[cfg(arithmetic64)]
pub(crate) use super::large_powers64::*;
