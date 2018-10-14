use std::env;
use std::process::Command;
use std::str::{self, FromStr};

fn main() {
    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    if minor < 26 {
        panic!("serde_json requires rustc 1.26+");
    }
}

fn rustc_minor_version() -> Option<u32> {
    // Logic borrowed from https://github.com/serde-rs/serde/blob/master/serde/build.rs
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    // Temporary workaround to support the old 1.26-dev compiler on docs.rs.
    if version.contains("0eb87c9bf") {
        return Some(25);
    }

    let mut pieces = version.split('.');

    if pieces.next() != Some("rustc 1") {
        return None;
    }

    let next = match pieces.next() {
        Some(next) => next,
        None => return None,
    };

    u32::from_str(next).ok()
}
