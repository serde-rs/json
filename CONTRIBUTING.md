# Contributing to Serde

Serde welcomes contribution from everyone in the form of suggestions, bug
reports, pull requests, and feedback. This document gives some guidance if you
are thinking of helping us.

Please reach out here in a GitHub issue or in the #serde IRC channel on
[`irc.mozilla.org`] if we can do anything to help you contribute.

[`irc.mozilla.org`]: https://wiki.mozilla.org/IRC

## Submitting bug reports and feature requests

Serde development is spread across lots of repositories. In general, prefer to
open issues against the main [serde-rs/serde] repository unless the topic is
clearly specific to JSON.

[serde-rs/serde]: https://github.com/serde-rs/serde

When reporting a bug or asking for help, please include enough details so that
the people helping you can reproduce the behavior you are seeing. For some tips
on how to approach this, read about how to produce a [Minimal, Complete, and
Verifiable example].

[Minimal, Complete, and Verifiable example]: https://stackoverflow.com/help/mcve

When making a feature request, please make it clear what problem you intend to
solve with the feature, any ideas for how Serde could support solving that
problem, any possible alternatives, and any disadvantages.

## Running the test suite

We encourage you to check that the test suite passes locally before submitting a
pull request with your changes. If anything does not pass, typically it will be
easier to iterate and fix it locally than waiting for the CI servers to run
tests for you.

The test suite requires a nightly compiler.

##### In the [`tests/deps`] directory

```sh
# This is a prerequisite for running the full test suite
cargo clean && cargo update && cargo build
```

##### In the top level serde-json directory

```sh
# Run the full test suite, including doc test and compile-tests
cargo test
```

[`tests/deps`]: https://github.com/serde-rs/json/tree/master/tests/deps

## Conduct

In all Serde-related forums, we follow the [Rust Code of Conduct]. For
escalation or moderation issues please contact Erick (erick.tryzelaar@gmail.com)
instead of the Rust moderation team.

[Rust Code of Conduct]: https://www.rust-lang.org/conduct.html
