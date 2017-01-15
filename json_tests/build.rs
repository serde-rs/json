extern crate skeptic;

#[cfg(feature = "with-syntex")]
mod with_syntex {
    extern crate serde_codegen;
    extern crate indoc;

    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();

        let src = Path::new("tests/test.rs.in");
        let dst = Path::new(&out_dir).join("test.rs");

        serde_codegen::expand(&src, &dst).unwrap();
        indoc::expand(&dst, &dst).unwrap();
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
