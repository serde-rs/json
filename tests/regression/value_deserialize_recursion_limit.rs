// Miri can't spawn and inspect a real child process (it needs
// std::env::current_exe(), which requires readlink, unsupported under
// Miri's isolation), and this whole file exists only for that child-process
// test, so it follows the same convention as test_deserialize_from_stream
// in tests/test.rs for the same underlying reason.
#![cfg(not(miri))]

// Regression test for a missing recursion-depth guard in `Deserialize for
// Value` (src/value/de.rs) when it is driven by a `serde::Deserializer`
// other than serde_json's own JSON-text one.
//
// serde_json's text parser has always bounded its own recursion (see
// `check_recursion!` in src/de.rs), but `ValueVisitor::visit_seq`/
// `visit_map` had no limit of their own: any other format (bincode,
// rmp-serde, ciborium, postcard, a hand-rolled `Deserializer`, ...) feeding
// a sufficiently deeply nested value into `Value::deserialize` overflowed
// the stack and aborted the whole process instead of returning an `Err`.
//
// Actually overflowing the stack on purpose, in-process, would take this
// entire test binary down with it (SIGABRT / exit code 134 on Linux), so
// the deep deserialization below runs in a re-exec'd child process and
// only its exit status is checked here. This keeps the test meaningful: if
// the guard is ever removed again, this test observes the child crashing
// and fails cleanly, rather than crashing the whole suite.

use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use std::env;
use std::process::Command;

const RUN_CHILD_VAR: &str = "SERDE_JSON_RECURSION_LIMIT_TEST_CHILD";
const DEPTH: u64 = 1_000_000;

// A minimal hand-rolled `Deserializer` with no notion of a recursion limit
// of its own, standing in for a non-JSON format. It generates nested
// single-element sequences on the fly rather than allocating them upfront,
// so a crash here can only be caused by unbounded *recursive calls*, not by
// first building a huge value in memory.
struct UnboundedNestedSeq {
    remaining: u64,
}

impl<'de> Deserializer<'de> for UnboundedNestedSeq {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.remaining == 0 {
            return visitor.visit_u64(0);
        }
        visitor.visit_seq(OneElement {
            remaining: self.remaining,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct OneElement {
    remaining: u64,
}

impl<'de> SeqAccess<'de> for OneElement {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        let remaining = self.remaining - 1;
        self.remaining = 0;
        seed.deserialize(UnboundedNestedSeq { remaining }).map(Some)
    }
}

#[test]
fn test_deeply_nested_value_deserialize_does_not_crash() {
    if env::var_os(RUN_CHILD_VAR).is_some() {
        // Re-exec'd child: do the actual deep deserialization here, where
        // a crash only takes down this disposable child process.
        let de = UnboundedNestedSeq { remaining: DEPTH };
        let value: Result<serde_json::Value, _> = serde::Deserialize::deserialize(de);
        match value {
            Ok(_) => std::process::exit(2),
            Err(err) => {
                assert!(err.to_string().contains("recursion limit exceeded"));
                std::process::exit(0);
            }
        }
    }

    let exe = env::current_exe().expect("failed to get current test executable path");
    let status = Command::new(exe)
        .arg("test_deeply_nested_value_deserialize_does_not_crash")
        .env(RUN_CHILD_VAR, "1")
        .status()
        .expect("failed to spawn child test process");

    assert!(
        status.success(),
        "deeply nested Value::deserialize() crashed the process (status: {status:?}) \
         instead of returning an error; the recursion-depth guard in \
         src/value/de.rs may have been removed or bypassed",
    );
}
