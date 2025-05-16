use crate::de::{Deserializer, Read};
use serde::de::value::BorrowedStrDeserializer;
use serde::de::IntoDeserializer as _;

pub(crate) enum SpannedDeserializer<'d, R> {
    Start {
        value_deserializer: &'d mut Deserializer<R>,
    },
    Value {
        value_deserializer: &'d mut Deserializer<R>,
    },
    End {
        end_pos: usize,
    },
    Done,
}

impl<'d, R> SpannedDeserializer<'d, R> {
    pub fn new(value_deserializer: &'d mut Deserializer<R>) -> Self {
        Self::Start { value_deserializer }
    }
}

impl<'d, 'de, R> serde::de::MapAccess<'de> for SpannedDeserializer<'d, R>
where
    R: Read<'de>,
{
    type Error = <&'d mut Deserializer<R> as serde::de::Deserializer<'de>>::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let key = match self {
            Self::Start { .. } => serde_spanned::__unstable::START_FIELD,
            Self::End { .. } => serde_spanned::__unstable::END_FIELD,
            Self::Value { .. } => serde_spanned::__unstable::VALUE_FIELD,
            Self::Done => return Ok(None),
        };

        seed.deserialize(BorrowedStrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self {
            Self::Start { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::Start { value_deserializer } = prev else {
                    unreachable!()
                };

                let start = value_deserializer.byte_offset();
                *self = Self::Value { value_deserializer };
                seed.deserialize(start.into_deserializer())
            }

            Self::Value { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::Value { value_deserializer } = prev else {
                    unreachable!()
                };

                let val = seed.deserialize(&mut *value_deserializer);
                *self = Self::End {
                    end_pos: value_deserializer.byte_offset(),
                };
                val
            }

            Self::End { .. } => {
                let prev = std::mem::replace(self, Self::Done);
                let Self::End { end_pos } = prev else {
                    unreachable!()
                };
                seed.deserialize(end_pos.into_deserializer())
            }

            Self::Done => {
                panic!("should not get here");
            }
        }
    }
}
