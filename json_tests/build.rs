extern crate skeptic;

#[cfg(feature = "with-syntex")]
mod with_syntex {
    extern crate serde_codegen;
    extern crate indoc;

    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();

        for &(src, dst) in &[
            ("tests/test.rs.in", "test.rs"),
            ("benches/bench.rs.in", "bench.rs"),
        ] {
            let src = Path::new(src);
            let dst = Path::new(&out_dir).join(dst);

            serde_codegen::expand(&src, &dst).unwrap();
            indoc::expand(&dst, &dst).unwrap();
        }
    }
}

#[cfg(not(feature = "with-syntex"))]
mod with_syntex {
    pub fn main() {}
}

pub fn main() {
    with_syntex::main();

    skeptic::generate_doc_tests(&["../README.md"]);
}
