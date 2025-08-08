use crate::de::Deserializer;
use crate::error::Error;
use crate::read;
use crate::read::Read;
use crate::spanned::SpanPosition;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::IntoDeserializer as _;

#[derive(Default)]
enum SpannedDeserializerState {
    #[default]
    StartLine,
    StartCol,
    StartOffset,
    Value,
    EndLine,
    EndCol,
    EndOffset,
    Done,
}

pub(crate) struct SpannedDeserializer<'de, 'p, R: Read<'de>> {
    phantom_data: core::marker::PhantomData<&'de ()>,
    state: SpannedDeserializerState,
    start: SpanPosition,
    end: SpanPosition,
    value_de: Option<&'p mut Deserializer<R>>,
}

impl<'de, 'p, R: Read<'de>> SpannedDeserializer<'de, 'p, R> {
    pub(crate) fn new(de: &'p mut Deserializer<R>) -> Self {
        let read_pos = de.position();
        Self {
            phantom_data: Default::default(),
            state: Default::default(),
            start: SpanPosition {
                line: read_pos.line,
                column: read_pos.column + 1,
                byte_offset: de.byte_offset(),
            },
            end: SpanPosition::default(),
            value_de: Some(de),
        }
    }
}

impl<'de, 'p, R: Read<'de>> serde::de::MapAccess<'de> for SpannedDeserializer<'de, 'p, R> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.state {
            SpannedDeserializerState::StartLine => seed
                .deserialize(BorrowedStrDeserializer::new(
                    crate::spanned::START_LINE_FIELD,
                ))
                .map(Some),
            SpannedDeserializerState::StartCol => seed
                .deserialize(BorrowedStrDeserializer::new(
                    crate::spanned::START_COL_FIELD,
                ))
                .map(Some),
            SpannedDeserializerState::StartOffset => seed
                .deserialize(BorrowedStrDeserializer::new(
                    crate::spanned::START_OFFSET_FIELD,
                ))
                .map(Some),
            SpannedDeserializerState::Value => seed
                .deserialize(BorrowedStrDeserializer::new(crate::spanned::VALUE_FIELD))
                .map(Some),
            SpannedDeserializerState::EndLine => seed
                .deserialize(BorrowedStrDeserializer::new(crate::spanned::END_LINE_FIELD))
                .map(Some),
            SpannedDeserializerState::EndCol => seed
                .deserialize(BorrowedStrDeserializer::new(crate::spanned::END_COL_FIELD))
                .map(Some),
            SpannedDeserializerState::EndOffset => seed
                .deserialize(BorrowedStrDeserializer::new(
                    crate::spanned::END_OFFSET_FIELD,
                ))
                .map(Some),
            SpannedDeserializerState::Done => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.state {
            SpannedDeserializerState::StartLine => {
                self.state = SpannedDeserializerState::StartCol;
                seed.deserialize(self.start.line.into_deserializer())
            }
            SpannedDeserializerState::StartCol => {
                self.state = SpannedDeserializerState::StartOffset;
                seed.deserialize(self.start.column.into_deserializer())
            }
            SpannedDeserializerState::StartOffset => {
                self.state = SpannedDeserializerState::Value;
                seed.deserialize(self.start.byte_offset.into_deserializer())
            }
            SpannedDeserializerState::Value => {
                self.state = SpannedDeserializerState::EndLine;
                let de = self.value_de.take().unwrap();
                let res = seed.deserialize(&mut *de);
                let read_pos = de.position();
                self.end = SpanPosition {
                    line: read_pos.line,
                    column: read_pos.column,
                    byte_offset: de.byte_offset(),
                };
                res
            }
            SpannedDeserializerState::EndLine => {
                self.state = SpannedDeserializerState::EndCol;
                seed.deserialize(self.end.line.into_deserializer())
            }
            SpannedDeserializerState::EndCol => {
                self.state = SpannedDeserializerState::EndOffset;
                seed.deserialize(self.end.column.into_deserializer())
            }
            SpannedDeserializerState::EndOffset => {
                self.state = SpannedDeserializerState::Done;
                seed.deserialize(self.end.byte_offset.into_deserializer())
            }
            SpannedDeserializerState::Done => {
                panic!("next_value_seed called before next_key_seed")
            }
        }
    }
}

#[cfg(feature = "spanned")]
fn from_trait_spanned<'de, R, T>(read: R) -> crate::Result<T>
where
    R: Read<'de>,
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new_spanned(read);
    let value = tri!(serde::de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    tri!(de.end());
    Ok(value)
}

/// TODO: document
#[cfg(all(feature = "std", feature = "spanned"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "spanned"))))]
pub fn from_reader_spanned<R, T>(rdr: R) -> crate::Result<T>
where
    R: crate::io::Read,
    T: serde::de::DeserializeOwned,
{
    from_trait_spanned(read::IoRead::new(rdr))
}

/// TODO: document
#[cfg(feature = "spanned")]
#[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
pub fn from_slice_spanned<'a, T>(v: &'a [u8]) -> crate::Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait_spanned(read::SliceRead::new(v))
}

/// TODO: document
#[cfg(feature = "spanned")]
#[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
pub fn from_str_spanned<'a, T>(s: &'a str) -> crate::Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait_spanned(read::StrRead::new(s))
}
