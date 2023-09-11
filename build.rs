use std::env;
use std::process::Command;
use std::str::{self, FromStr};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Decide ideal limb width for arithmetic in the float parser. Refer to
    // src/lexical/math.rs for where this has an effect.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_arch.as_str() {
        "aarch64" | "mips64" | "powerpc64" | "x86_64" => {
            println!("cargo:rustc-cfg=limb_width_64");
        }
        _ => {
            println!("cargo:rustc-cfg=limb_width_32");
        }
    }

    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let next = pieces.next()?;
    u32::from_str(next).ok()
}
