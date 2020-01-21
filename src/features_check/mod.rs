//! Shows a user-friendly compiler error on incompatible selected features.

#[allow(unused_macros)]
macro_rules! hide_from_rustfmt {
    ($mod:item) => {
        $mod
    };
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
hide_from_rustfmt! {
    mod error;
}
