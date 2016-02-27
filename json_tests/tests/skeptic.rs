#![cfg(feature = "nightly-testing")]
#![allow(toplevel_ref_arg)]
#![allow(useless_format)]

include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
