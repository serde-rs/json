#### To run tests

```sh
(cd deps && cargo clean && cargo update && cargo build)
cargo test
```

#### To update goldens after running ui tests

```sh
ui/update-references.sh
```
