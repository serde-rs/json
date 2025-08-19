use core::marker::PhantomData;

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::read::Position;
#[cfg(feature = "raw_value")]
use crate::{de::StrRead, read::PositionedRead, value::RawValue};

pub const TOKEN: &str = "$serde_json::private::Positioned";

/// A value that is saved together with its position in the input.
pub struct Positioned<T> {
    /// The position in the input.
    pub position: Position,
    /// The actual deserialized value.
    pub value: T,
}

impl<'de, T> Deserialize<'de> for Positioned<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PosVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for PosVisitor<T> {
            type Value = Positioned<T>;
            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                write!(formatter, "positioned value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(Positioned {
                    position: seq.next_element()?.unwrap(),
                    value: seq.next_element()?.unwrap(),
                })
            }
        }

        deserializer.deserialize_tuple_struct(TOKEN, 2, PosVisitor(PhantomData))
    }
}

impl<T: Serialize> Serialize for Positioned<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(all(feature = "raw_value", feature = "std"))]
impl Positioned<Box<RawValue>> {
    /// Read from a positioned RawValue.
    pub fn read(&self) -> PositionedRead<StrRead<'_>> {
        PositionedRead::new(self.position.clone(), StrRead::new(self.value.get()))
    }
}

#[cfg(feature = "raw_value")]
impl<'a> Positioned<&'a RawValue> {
    /// Read from a positioned RawValue.
    pub fn read(&self) -> PositionedRead<StrRead<'a>> {
        PositionedRead::new(self.position.clone(), StrRead::new(self.value.get()))
    }
}
