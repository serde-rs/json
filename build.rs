fn main() {
    // Decide ideal limb width for arithmetic in the float parser. Refer to
    // src/lexical/math.rs for where this has an effect.
    let limb_width_64 = cfg!(any(
        target_arch = "aarch64",
        target_arch = "mips64",
        target_arch = "powerpc64",
        target_arch = "x86_64"
    ));
    if limb_width_64 {
        println!("cargo:rustc-cfg=limb_width_64");
    } else {
        println!("cargo:rustc-cfg=limb_width_32");
    }
}
