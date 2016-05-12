#![cfg_attr(feature = "nightly-testing", feature(plugin))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]

#![cfg_attr(not(feature = "with-syntex"), feature(custom_attribute, custom_derive, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(serde_macros, indoc))]

extern crate serde;
extern crate serde_json;
extern crate skeptic;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/test.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("test.rs.in");

#[cfg(feature = "nightly-testing")]
mod skeptic_tests {
    #![cfg_attr(feature = "nightly-testing", allow(toplevel_ref_arg))]
    #![cfg_attr(feature = "nightly-testing", allow(useless_format))]

    include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
}
