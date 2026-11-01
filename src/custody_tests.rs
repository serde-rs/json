//! Parser-owned initialized bytes are observed before their allocation is released.
use alloc::format;
use alloc::string::{String, ToString};
use serde::de::Visitor;
use serde::Deserializer;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::thread_local;

thread_local! {
    static WATCH: Cell<Option<(*mut u8, usize)>> = const { Cell::new(None) };
    static WIPED: Cell<Option<bool>> = const { Cell::new(None) };
}

struct InspectBeforeFree;

// SAFETY: allocations are forwarded unchanged to System. Only the initialized string prefix
// registered by this thread is observed, while its matching allocation still belongs to it.
unsafe impl GlobalAlloc for InspectBeforeFree {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the allocator caller supplies a valid layout.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let _ = WATCH.try_with(|watch| {
            if let Some((_, len)) = watch.get().filter(|(watched, _)| *watched == pointer) {
                watch.set(None);
                let wiped = len <= layout.size()
                    && (0..len).all(|index| {
                        // SAFETY: these bytes were initialized String contents when registered,
                        // and the matching live allocation has not yet reached System.dealloc.
                        // No spare capacity, allocation padding, or freed memory is read.
                        unsafe { pointer.add(index).read_volatile() == 0 }
                    });
                let _ = WIPED.try_with(|result| result.set(Some(wiped)));
            }
        });
        // SAFETY: pointer and layout match the allocator caller's live allocation.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: InspectBeforeFree = InspectBeforeFree;

pub(crate) fn watch(bytes: &[u8]) {
    // Every caller registers an initialized prefix beginning at the base of a
    // live Vec/String/Box allocation. No allocation is inferred from capacity,
    // and no registration is allowed to survive its matching deallocation.
    assert!(!bytes.is_empty());
    WATCH.with(|watched| {
        assert!(
            watched.get().is_none(),
            "the prior initialized owner must retire first"
        );
        watched.set(Some((bytes.as_ptr().cast_mut(), bytes.len())));
    });
    WIPED.with(|result| result.set(None));
}

pub(crate) fn reset_observation() {
    WATCH.with(|watched| assert!(watched.get().is_none()));
    WIPED.with(|result| result.set(None));
}

#[test]
fn initialized_prefix_witness_distinguishes_cleartext_and_wiped_owners() {
    let plain = "typed-private-control".to_string();
    watch(plain.as_bytes());
    drop(plain);
    assert_eq!(WIPED.with(Cell::get), Some(false));
    let wiped = zeroize::Zeroizing::new("typed-private-control".to_string());
    watch(wiped.as_bytes());
    drop(wiped);
    assert_eq!(WIPED.with(Cell::get), Some(true));
}

pub(crate) fn wiped() -> Option<bool> {
    WIPED.with(Cell::get)
}

struct ScratchSeed;
impl<'de> serde::de::DeserializeSeed<'de> for ScratchSeed {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        struct ScratchVisitor;
        impl<'de> Visitor<'de> for ScratchVisitor {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an escaped JSON string")
            }
            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<(), E> {
                // serde_json's escaped slice reader calls visit_str with its live scratch.
                watch(text.as_bytes());
                Ok(())
            }
            fn visit_borrowed_str<E: serde::de::Error>(self, _: &'de str) -> Result<(), E> {
                Err(E::custom(
                    "fixture must exercise parser scratch, not borrowed input",
                ))
            }
        }
        deserializer.deserialize_str(ScratchVisitor)
    }
}

#[test]
fn serde_json_parser_wipes_escaped_string_scratch_on_successful_drop() {
    use serde::de::DeserializeSeed as _;
    let input = br#""parser-private-credential\u0021""#;
    {
        let mut parser = crate::Deserializer::from_slice(input);
        ScratchSeed.deserialize(&mut parser).unwrap();
        assert_eq!(WIPED.with(Cell::get), None);
    }
    assert_eq!(WIPED.with(Cell::get), Some(true));
}

struct ScratchSequence;
impl<'de> serde::de::DeserializeSeed<'de> for ScratchSequence {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        struct ScratchSequenceVisitor;
        impl<'de> Visitor<'de> for ScratchSequenceVisitor {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("the scratch lifetime fixture sequence")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<(), A::Error> {
                tri!(tri!(sequence.next_element_seed(ScratchSeed))
                    .ok_or_else(|| serde::de::Error::custom("first fixture element is missing")));
                // The second escaped string must be read, rather than IgnoredAny's skip path.
                let _next = tri!(sequence.next_element::<String>());
                Ok(())
            }
        }
        deserializer.deserialize_seq(ScratchSequenceVisitor)
    }
}

#[test]
fn serde_json_parser_wipes_prior_scratch_on_malformed_later_input() {
    use serde::de::DeserializeSeed as _;
    let input = br#"["parser-private-credential\u0021",]"#;
    {
        let mut parser = crate::Deserializer::from_slice(input);
        assert!(ScratchSequence.deserialize(&mut parser).is_err());
    }
    assert_eq!(WIPED.with(Cell::get), Some(true));
}

#[test]
fn serde_json_parser_wipes_prior_scratch_before_growth_for_a_later_string() {
    use serde::de::DeserializeSeed as _;
    // A later string grows the same scratch; its first escape comes after the long fragment.
    // The length follows the first string, rather than a production budget or policy limit.
    const FIRST_FRAGMENT: &str = "parser-private-credential";
    let later_fragment = FIRST_FRAGMENT.repeat(FIRST_FRAGMENT.len());
    let input = format!(r#"["{FIRST_FRAGMENT}\u0021","{later_fragment}\u0021"]"#);
    {
        let mut parser = crate::Deserializer::from_str(&input);
        ScratchSequence.deserialize(&mut parser).unwrap();
        assert_eq!(WIPED.with(Cell::get), Some(true));
    }
}

#[test]
fn parser_drop_cancels_an_unfinished_stream_and_wipes_scratch() {
    use serde::de::DeserializeSeed as _;
    let input = br#""parser-private-credential\u0021" "unfinished"#;
    let mut parser = crate::Deserializer::from_slice(input);
    ScratchSeed.deserialize(&mut parser).unwrap();
    assert_eq!(wiped(), None);
    drop(parser);
    assert_eq!(wiped(), Some(true));
}

#[test]
#[cfg(feature = "std")]
fn reader_parser_wipes_initialized_scratch_after_a_later_parse_error() {
    use serde::de::DeserializeSeed as _;
    let input = br#"["reader-private-credential\u0021",]"#;
    let mut parser = crate::Deserializer::from_reader(input.as_slice());
    assert!(ScratchSequence.deserialize(&mut parser).is_err());
    drop(parser);
    assert_eq!(wiped(), Some(true));
}

#[test]
fn moving_a_parser_into_its_stream_retains_scratch_custody() {
    struct Watched;
    impl<'de> serde::Deserialize<'de> for Watched {
        fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            serde::de::DeserializeSeed::deserialize(ScratchSeed, de).map(|()| Watched)
        }
    }
    let input = br#""stream-private-credential\u0021" "unfinished"#;
    let mut stream = crate::Deserializer::from_slice(input).into_iter::<Watched>();
    assert!(stream.next().unwrap().is_ok());
    assert_eq!(wiped(), None);
    drop(stream);
    assert_eq!(wiped(), Some(true));
}

#[test]
fn visitor_error_keeps_scratch_owned_until_parser_drop() {
    struct Refused;
    impl<'de> serde::Deserialize<'de> for Refused {
        fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct RefusingVisitor;
            impl Visitor<'_> for RefusingVisitor {
                type Value = Refused;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("the fixture's refused string")
                }
                fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Refused, E> {
                    watch(text.as_bytes());
                    Err(E::custom("the fixture visitor refused its string"))
                }
            }
            de.deserialize_str(RefusingVisitor)
        }
    }
    let mut parser = crate::Deserializer::from_slice(br#""refused-private-credential\u0021""#);
    assert!(<Refused as serde::Deserialize>::deserialize(&mut parser).is_err());
    assert_eq!(wiped(), None);
    drop(parser);
    assert_eq!(wiped(), Some(true));
}

#[test]
fn unwinding_a_visitor_releases_the_parser_scratch() {
    struct Panicking;
    impl<'de> serde::Deserialize<'de> for Panicking {
        fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct PanickingVisitor;
            impl Visitor<'_> for PanickingVisitor {
                type Value = Panicking;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a string")
                }
                fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Panicking, E> {
                    watch(text.as_bytes());
                    panic!("fixture visitor panic")
                }
            }
            de.deserialize_str(PanickingVisitor)
        }
    }
    let result = std::panic::catch_unwind(|| {
        let _ = crate::from_str::<Panicking>(r#""private\u0021""#);
    });
    assert!(result.is_err());
    assert_eq!(wiped(), Some(true));
}

#[test]
fn returned_strings_values_and_serialized_output_keep_normal_ownership() {
    let text: String = crate::from_str(r#""returned-private\u0021""#).unwrap();
    watch(text.as_bytes());
    drop(text);
    assert_eq!(wiped(), Some(false));

    let value: crate::Value = crate::from_str(r#""value-private\u0021""#).unwrap();
    watch(value.as_str().unwrap().as_bytes());
    drop(value);
    assert_eq!(wiped(), Some(false));

    let output = crate::to_vec(&"output-private").unwrap();
    watch(&output);
    drop(output);
    assert_eq!(wiped(), Some(false));
}

#[test]
fn caller_supplied_writer_keeps_its_own_growth_behavior() {
    let mut writer = alloc::vec::Vec::with_capacity(7);
    writer.extend_from_slice(b"private");
    watch(&writer);
    crate::ser::to_writer(&mut writer, &"caller-private".repeat(32)).unwrap();
    assert_eq!(wiped(), Some(false));
    assert!(writer.starts_with(b"private\"caller-private"));
}

#[test]
fn partial_output_is_the_visitors_responsibility() {
    struct Partial;
    impl<'de> serde::Deserialize<'de> for Partial {
        fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct PartialVisitor;
            impl<'de> Visitor<'de> for PartialVisitor {
                type Value = Partial;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a sequence")
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut seq: A,
                ) -> Result<Partial, A::Error> {
                    let value = tri!(seq.next_element::<String>()).unwrap();
                    watch(value.as_bytes());
                    // A completed element already belongs to this visitor.
                    // Its normal drop on the later parse failure is a negative control.
                    tri!(seq.next_element::<String>());
                    Ok(Partial)
                }
            }
            de.deserialize_seq(PartialVisitor)
        }
    }
    assert!(crate::from_str::<Partial>(r#"["partial-private\u0021",]"#).is_err());
    assert_eq!(wiped(), Some(false));
}
