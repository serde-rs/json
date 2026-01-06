use crate::{read, Value};
use serde::{
    de::{value::BorrowedStrDeserializer, DeserializeSeed, IntoDeserializer, MapAccess},
    Deserialize, Deserializer,
};
use std::ops::Range;

pub(crate) const NAME: &str = "$__sciformats_serde_span_private_name";
pub(crate) const START_FIELD: &str = "$__sciformats_serde_span_private_start_field";
pub(crate) const END_FIELD: &str = "$__sciformats_serde_span_private_end_field";

/// Check if the given name and fields correspond to a Span.
pub(crate) fn is_span(name: &str, fields: &[&str]) -> bool {
    name == NAME && [START_FIELD, END_FIELD] == fields
}

/// A span representing a range of bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// The range of bytes.
    pub span: Range<u64>,
}

impl<'de> Deserialize<'de> for Span {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = SpanVisitor;
        deserializer.deserialize_struct(NAME, &[START_FIELD, END_FIELD], visitor)
    }
}

struct SpanVisitor;

impl<'de> serde::de::Visitor<'de> for SpanVisitor {
    type Value = Span;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a span")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Span, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut start: Option<u64> = None;
        let mut end: Option<u64> = None;

        while let Some(key) = tri!(access.next_key()) {
            match key {
                START_FIELD => {
                    if start.is_some() {
                        return Err(serde::de::Error::duplicate_field(START_FIELD));
                    }
                    start = Some(tri!(access.next_value()));
                }
                END_FIELD => {
                    if end.is_some() {
                        return Err(serde::de::Error::duplicate_field(END_FIELD));
                    }
                    end = Some(tri!(access.next_value()));
                }
                field => {
                    return Err(serde::de::Error::unknown_field(
                        field,
                        &[START_FIELD, END_FIELD],
                    ));
                }
            }
        }
        match (start, end) {
            (Some(start), Some(end)) => Ok(Span { span: start..end }),
            (None, _) => Err(serde::de::Error::missing_field(START_FIELD)),
            (_, None) => Err(serde::de::Error::missing_field(END_FIELD)),
        }
    }
}

pub(crate) enum SpanDeserializer<'d, R> {
    Start { de: &'d mut crate::Deserializer<R> },
    End { de: &'d mut crate::Deserializer<R> },
    Done,
}

impl<'d, R> SpanDeserializer<'d, R> {
    pub fn new(de: &'d mut crate::Deserializer<R>) -> Self {
        Self::Start { de }
    }
}

impl<'d, 'de: 'd, R> MapAccess<'de> for SpanDeserializer<'d, R>
where
    R: read::Read<'de>,
{
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let key = match self {
            Self::Start { .. } => START_FIELD,
            Self::End { .. } => END_FIELD,
            Self::Done => return Ok(None),
        };

        seed.deserialize(BorrowedStrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self {
            // It's not possible extract de in pattern match directly, hence use std::mem::replace.
            Self::Start { .. } => {
                if let Self::Start { de } = std::mem::replace(self, Self::Done) {
                    let start = de.byte_offset();
                    *self = Self::End { de };
                    seed.deserialize(start.into_deserializer())
                } else {
                    unreachable!()
                }
            }
            Self::End { de } => {
                // Read and ignore value.
                tri!(Value::deserialize(&mut **de));
                let end = de.byte_offset();
                *self = Self::Done;
                seed.deserialize(end.into_deserializer())
            }
            Self::Done => panic!("unexpected call to next_value_seed() after Done"),
        }
    }
}
