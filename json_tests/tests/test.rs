#![cfg_attr(feature = "nightly-testing", feature(plugin))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]

#![cfg_attr(not(feature = "with-syntex"), feature(plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(indoc))]

#![cfg_attr(feature = "trace-macros", feature(trace_macros))]
#[cfg(feature = "trace-macros")]
trace_macros!(true);

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
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
