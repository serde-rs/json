use alloc::string::String;
use alloc::vec::Vec;

#[cfg(not(feature = "zeroize"))]
pub(crate) type Buffer = Vec<u8>;
#[cfg(feature = "zeroize")]
pub(crate) type Buffer = zeroize::Zeroizing<Vec<u8>>;

pub(crate) fn buffer() -> Buffer {
    #[cfg(not(feature = "zeroize"))]
    {
        Vec::new()
    }
    #[cfg(feature = "zeroize")]
    {
        zeroize::Zeroizing::new(Vec::new())
    }
}

#[cfg(all(feature = "std", feature = "raw_value"))]
pub(crate) fn into_vec(bytes: Buffer) -> Vec<u8> {
    #[cfg(not(feature = "zeroize"))]
    {
        bytes
    }
    #[cfg(feature = "zeroize")]
    {
        let mut bytes = bytes;
        core::mem::take(&mut *bytes)
    }
}

#[cfg(not(feature = "zeroize"))]
pub(crate) type Text = String;
#[cfg(feature = "zeroize")]
pub(crate) type Text = MessageText;

#[cfg(feature = "raw_value")]
pub(crate) fn display_text(value: impl core::fmt::Display) -> Text {
    #[cfg(not(feature = "zeroize"))]
    {
        use alloc::string::ToString;
        value.to_string()
    }
    #[cfg(feature = "zeroize")]
    {
        MessageText::from_display(value)
    }
}

#[cfg(any(feature = "arbitrary_precision", feature = "raw_value"))]
pub(crate) fn into_string(text: Text) -> String {
    #[cfg(not(feature = "zeroize"))]
    {
        text
    }
    #[cfg(feature = "zeroize")]
    {
        let mut text = text;
        core::mem::take(&mut text.0)
    }
}

#[cfg(feature = "raw_value")]
pub(crate) fn boxed_str(text: String) -> alloc::boxed::Box<str> {
    #[cfg(feature = "zeroize")]
    if text.capacity() != text.len() {
        // Shrinking a String into a Box may otherwise release an unwiped
        // allocation. Copy into the final owner, then wipe the old owner.
        let text = zeroize::Zeroizing::new(text);
        return alloc::boxed::Box::from(text.as_str());
    }
    text.into_boxed_str()
}

#[cfg(any(feature = "arbitrary_precision", feature = "raw_value"))]
pub(crate) fn deserialize_text<'de, S>(seed: S, text: Text) -> crate::Result<S::Value>
where
    S: serde::de::DeserializeSeed<'de>,
{
    #[cfg(not(feature = "zeroize"))]
    {
        use serde::de::IntoDeserializer;
        seed.deserialize(text.into_deserializer())
    }
    #[cfg(feature = "zeroize")]
    {
        seed.deserialize(text::Deserializer(text))
    }
}

#[cfg(all(
    feature = "zeroize",
    any(feature = "arbitrary_precision", feature = "raw_value")
))]
mod text;

#[cfg(not(feature = "zeroize"))]
pub(crate) type Output = Vec<u8>;

#[cfg(feature = "zeroize")]
pub(crate) struct Output(Buffer);

pub(crate) fn output() -> Output {
    #[cfg(not(feature = "zeroize"))]
    {
        Vec::with_capacity(128)
    }
    #[cfg(feature = "zeroize")]
    {
        Output(zeroize::Zeroizing::new(Vec::with_capacity(128)))
    }
}

pub(crate) fn finish_output(output: Output) -> Vec<u8> {
    #[cfg(not(feature = "zeroize"))]
    {
        output
    }
    #[cfg(feature = "zeroize")]
    {
        let mut output = output;
        core::mem::take(&mut *output.0)
    }
}

#[cfg(feature = "zeroize")]
impl crate::io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> crate::io::Result<usize> {
        extend_from_slice(&mut self.0, bytes);
        Ok(bytes.len())
    }

    fn write_all(&mut self, bytes: &[u8]) -> crate::io::Result<()> {
        extend_from_slice(&mut self.0, bytes);
        Ok(())
    }

    fn flush(&mut self) -> crate::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

#[inline]
pub(crate) fn clear(bytes: &mut Vec<u8>) {
    #[cfg(feature = "zeroize")]
    bytes.zeroize();
    #[cfg(not(feature = "zeroize"))]
    bytes.clear();
}

#[inline]
pub(crate) fn reserve(bytes: &mut Vec<u8>, additional: usize) {
    #[cfg(not(feature = "zeroize"))]
    bytes.reserve(additional);
    #[cfg(feature = "zeroize")]
    if additional > bytes.capacity() - bytes.len() {
        let mut replacement =
            Vec::with_capacity(grown_capacity(bytes.len(), bytes.capacity(), additional));
        replacement.extend_from_slice(bytes);
        bytes.zeroize();
        *bytes = replacement;
    }
}

#[cfg(feature = "zeroize")]
fn grown_capacity(len: usize, capacity: usize, additional: usize) -> usize {
    // Grow geometrically without letting realloc release live secret bytes.
    const AMORTIZED_GROWTH_FACTOR: usize = 2;
    let required = len.checked_add(additional).expect("capacity overflow");
    required.max(capacity.saturating_mul(AMORTIZED_GROWTH_FACTOR))
}

#[inline]
pub(crate) fn push(bytes: &mut Vec<u8>, byte: u8) {
    #[cfg(feature = "zeroize")]
    reserve(bytes, 1);
    bytes.push(byte);
}

#[inline]
pub(crate) fn extend_from_slice(bytes: &mut Vec<u8>, more: &[u8]) {
    #[cfg(feature = "zeroize")]
    reserve(bytes, more.len());
    bytes.extend_from_slice(more);
}

#[inline]
pub(crate) fn extend<I: IntoIterator<Item = u8>>(bytes: &mut Vec<u8>, more: I) {
    #[cfg(not(feature = "zeroize"))]
    bytes.extend(more);
    #[cfg(feature = "zeroize")]
    for byte in more {
        push(bytes, byte);
    }
}

#[cfg(feature = "zeroize")]
pub(crate) struct MessageText(alloc::string::String);

#[cfg(feature = "zeroize")]
impl MessageText {
    pub(crate) fn new() -> Self {
        Self(String::new())
    }

    #[cfg(feature = "arbitrary_precision")]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    pub(crate) fn push(&mut self, ch: char) {
        use core::fmt::Write;
        self.write_str(ch.encode_utf8(&mut [0; 4])).unwrap();
    }

    pub(crate) fn from_display(value: impl core::fmt::Display) -> Self {
        use core::fmt::Write;
        let mut text = Self(alloc::string::String::new());
        write!(&mut text, "{}", value).expect("a Display implementation returned an error");
        text
    }

    pub(crate) fn as_mut_string(&mut self) -> &mut alloc::string::String {
        &mut self.0
    }
}

#[cfg(feature = "zeroize")]
impl From<String> for MessageText {
    fn from(text: String) -> Self {
        Self(text)
    }
}

#[cfg(feature = "zeroize")]
impl core::fmt::Write for MessageText {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        if text.len() > self.0.capacity() - self.0.len() {
            let mut replacement = alloc::string::String::with_capacity(grown_capacity(
                self.0.len(),
                self.0.capacity(),
                text.len(),
            ));
            replacement.push_str(&self.0);
            self.0.zeroize();
            self.0 = replacement;
        }
        self.0.push_str(text);
        Ok(())
    }
}

#[cfg(feature = "zeroize")]
impl core::ops::Deref for MessageText {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "zeroize")]
impl core::fmt::Debug for MessageText {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, formatter)
    }
}

#[cfg(feature = "zeroize")]
impl Drop for MessageText {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(all(test, feature = "zeroize"))]
mod tests {
    use super::{extend_from_slice, MessageText};
    use crate::custody_tests::{watch, wiped};
    use alloc::vec::Vec;
    use core::fmt::Write as _;

    const PRIVATE_FRAGMENT: &str = "scratch-private-fragment";

    #[test]
    fn initialized_scratch_prefix_wipes_before_growth_and_after_clear() {
        let mut bytes = Vec::from(PRIVATE_FRAGMENT.as_bytes());
        watch(&bytes);
        // Capacity determines only the requested growth pressure. Owner identity comes from
        // the live initialized slice, and the allocator observer never reads spare capacity.
        let suffix = PRIVATE_FRAGMENT.repeat(bytes.capacity() + 1);
        extend_from_slice(&mut bytes, suffix.as_bytes());
        assert_eq!(wiped(), Some(true));
        assert!(bytes.starts_with(PRIVATE_FRAGMENT.as_bytes()));
        watch(&bytes);
        super::clear(&mut bytes);
        drop(bytes);
        assert_eq!(wiped(), Some(true));
    }

    #[test]
    fn error_formatter_wipes_its_initialized_text_before_growth() {
        let mut text = MessageText::from_display(PRIVATE_FRAGMENT);
        watch(text.as_bytes());
        let suffix = PRIVATE_FRAGMENT.repeat(text.0.capacity() + 1);
        text.write_str(&suffix).unwrap();
        assert_eq!(wiped(), Some(true));
        assert!(text.starts_with(PRIVATE_FRAGMENT));
        watch(text.as_bytes());
        drop(text);
        assert_eq!(wiped(), Some(true));
    }

    #[test]
    fn formatter_failure_unwinds_through_the_initialized_text_owner() {
        struct RefusingDisplay;
        impl core::fmt::Display for RefusingDisplay {
            fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                Err(core::fmt::Error)
            }
        }
        let text = MessageText::from_display(PRIVATE_FRAGMENT);
        watch(text.as_bytes());
        let outcome = std::panic::catch_unwind(move || {
            let mut text = text;
            core::fmt::write(&mut text, format_args!("{}", RefusingDisplay))
                .expect("the fixture formatter refuses its output");
        });
        assert!(outcome.is_err());
        assert_eq!(wiped(), Some(true));
    }

    #[test]
    fn output_growth_wipes_the_old_allocation_and_drop_wipes_partial_output() {
        use crate::io::Write;
        let mut output = super::output();
        output.write_all(PRIVATE_FRAGMENT.as_bytes()).unwrap();
        watch(&output.0);
        let suffix = PRIVATE_FRAGMENT.repeat(output.0.capacity() + 1);
        output.write_all(suffix.as_bytes()).unwrap();
        assert_eq!(wiped(), Some(true));
        assert!(output.0.starts_with(PRIVATE_FRAGMENT.as_bytes()));
        watch(&output.0);
        drop(output);
        assert_eq!(wiped(), Some(true));
    }

    #[test]
    fn serializer_error_leaves_partial_output_under_its_wiping_owner() {
        struct Failing;
        impl serde::Serialize for Failing {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::{Error, SerializeSeq};
                let mut seq = tri!(serializer.serialize_seq(Some(2)));
                tri!(seq.serialize_element(PRIVATE_FRAGMENT));
                Err(S::Error::custom("fixture failure after writing an element"))
            }
        }
        let mut output = super::output();
        assert!(crate::ser::to_writer(&mut output, &Failing).is_err());
        assert!(output
            .0
            .windows(PRIVATE_FRAGMENT.len())
            .any(|part| part == PRIVATE_FRAGMENT.as_bytes()));
        watch(&output.0);
        drop(output);
        assert_eq!(wiped(), Some(true));
        assert!(crate::to_vec(&Failing).is_err());
        assert!(crate::to_vec_pretty(&Failing).is_err());
    }

    #[test]
    #[cfg(feature = "raw_value")]
    fn boxing_a_string_wipes_excess_capacity_without_wiping_the_result() {
        let mut text = alloc::string::String::with_capacity(PRIVATE_FRAGMENT.len() * 2);
        text.push_str(PRIVATE_FRAGMENT);
        watch(text.as_bytes());
        let boxed = super::boxed_str(text);
        assert_eq!(wiped(), Some(true));
        assert_eq!(&*boxed, PRIVATE_FRAGMENT);
        watch(boxed.as_bytes());
        drop(boxed);
        assert_eq!(wiped(), Some(false));
    }
}
