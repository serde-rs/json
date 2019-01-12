extern crate compiletest_rs as compiletest;

#[test]
fn ui() {
    compiletest::run_tests(&compiletest::Config {
        mode: compiletest::common::Mode::Ui,
        src_base: std::path::PathBuf::from("tests/ui"),
        target_rustcflags: Some(String::from("-L tests/deps/target/debug/deps")),
        ..Default::default()
    });
}
