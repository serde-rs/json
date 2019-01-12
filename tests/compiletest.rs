extern crate compiletest_rs as compiletest;

#[test]
fn ui() {
    compiletest::run_tests(&compiletest::Config {
        mode: compiletest::common::Mode::Ui,
        src_base: std::path::PathBuf::from("tests/ui"),
        target_rustcflags: Some(String::from(
            "\
             --edition=2018 \
             -L tests/deps/target/debug/deps \
             -Z unstable-options \
             --extern serde_json \
             ",
        )),
        ..Default::default()
    });
}
