use crate::spanned::{Span, SpanPosition};
use core::cmp::Ordering;
use core::marker::Copy;
use serde::de::Visitor;

pub(crate) const NAME: &str = "$serde_json::private::Spanned";
pub(crate) const START_LINE_FIELD: &str = "$serde_json::private::Spanned::line_start";
pub(crate) const START_COL_FIELD: &str = "$serde_json::private::Spanned::col_start";
pub(crate) const START_OFFSET_FIELD: &str = "$serde_json::private::Spanned::offset_start";
pub(crate) const VALUE_FIELD: &str = "$serde_json::private::Spanned::value";
pub(crate) const END_LINE_FIELD: &str = "$serde_json::private::Spanned::line_end";
pub(crate) const END_COL_FIELD: &str = "$serde_json::private::Spanned::col_end";
pub(crate) const END_OFFSET_FIELD: &str = "$serde_json::private::Spanned::offset_end";
const FIELDS: [&str; 7] = [
    START_LINE_FIELD,
    START_COL_FIELD,
    START_OFFSET_FIELD,
    VALUE_FIELD,
    END_LINE_FIELD,
    END_COL_FIELD,
    END_OFFSET_FIELD,
];
pub(crate) fn is_spanned(name: &'static str, fields: &'static [&'static str]) -> bool {
    name == NAME && fields == FIELDS
}

/// A spanned value, indicating the where it is defined in the source.
///
/// While this type implements [`serde::Deserialize`], it only works as intended with the
/// deserializers provided by this crate.
///
/// Traits like `PartialEq`, `Hash`, etc. are all forwarded to the inner value and ignore the span.
/// Likewise, is its [`serde::Serialize`] implementation opaque.
#[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
pub struct Spanned<T> {
    /// TODO: document
    pub(crate) span: Span,
    /// TODO: document
    pub(crate) value: T,
}

impl<T> Spanned<T> {
    /// Creates a new spanned value.
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub const fn new(span: Span, value: T) -> Self {
        Spanned { span, value }
    }

    /// Creates a new value with the default span.
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub const fn new_default_span(value: T) -> Self {
        let span = Span::default();
        Spanned { span, value }
    }

    /// Byte range
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn byte_span(&self) -> core::ops::Range<usize> {
        self.span.byte_span()
    }

    /// TODO: docoument
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Consumes the spanned value and returns the contained value.
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Returns a reference to the contained value.
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn get_ref(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the contained value.
    #[cfg_attr(docsrs, doc(cfg(feature = "spanned")))]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Spanned")
            .field("span", &self.span)
            .field("value", &self.value)
            .finish()
    }
}

impl<T: Clone> Clone for Spanned<T> {
    fn clone(&self) -> Self {
        Spanned {
            span: self.span,
            value: self.value.clone(),
        }
    }
}

impl<T: Copy> Copy for Spanned<T> {}

impl core::borrow::Borrow<str> for Spanned<alloc::string::String> {
    fn borrow(&self) -> &str {
        self.get_ref()
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        self.get_ref()
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: core::hash::Hash> core::hash::Hash for Spanned<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}
//
// impl<'de, T: serde::de::Deserialize<'de>> FromStr for Spanned<T> {
//     type Err = Error;
//     fn from_str(s: &'de str) -> Result<Spanned<T>, Error> {
//         crate::spanned::from_str_spanned(s)
//     }
// }

impl<T> From<T> for Spanned<T> {
    fn from(value: T) -> Self {
        Spanned {
            span: Default::default(),
            value,
        }
    }
}

// impl<T> core::borrow::Borrow<T> for Spanned<T> {
//     fn borrow(&self) -> &T {
//         <Spanned<T>>::as_ref(self)
//     }
// }
//
// impl<T> core::borrow::BorrowMut<T> for Spanned<T> {
//     fn borrow_mut(&mut self) -> &mut T {
//         <Spanned<T>>::as_mut(self)
//     }
// }

// impl<Borrowed, T> core::borrow::Borrow<Borrowed> for Spanned<T>
// where
//     T: core::borrow::Borrow<Borrowed>,
// {
//     fn borrow(&self) -> &Borrowed {
//         self.as_ref().borrow()
//     }
// }
//
// impl<Borrowed, T> core::borrow::BorrowMut<Borrowed> for Spanned<T>
// where
//     T: core::borrow::BorrowMut<Borrowed>,
// {
//     fn borrow_mut(&mut self) -> &mut Borrowed {
//         self.as_mut().borrow_mut()
//     }
// }

impl<'de, T> serde::de::Deserialize<'de> for Spanned<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Spanned<T>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        pub(crate) struct SpannedVisitor<T>(core::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for SpannedVisitor<T>
        where
            T: serde::de::Deserialize<'de>,
        {
            type Value = Spanned<T>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a spanned value")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Spanned<T>, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut start_line: Option<usize> = None;
                let mut start_col: Option<usize> = None;
                let mut start_offset: Option<usize> = None;
                let mut end_line: Option<usize> = None;
                let mut end_col: Option<usize> = None;
                let mut end_offset: Option<usize> = None;
                let mut value: Option<T> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        START_LINE_FIELD => {
                            if start_line.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_LINE_FIELD));
                            }
                            start_line = Some(map.next_value()?);
                        }
                        START_COL_FIELD => {
                            if start_col.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_COL_FIELD));
                            }
                            start_col = Some(map.next_value()?);
                        }
                        START_OFFSET_FIELD => {
                            if start_offset.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_OFFSET_FIELD));
                            }
                            start_offset = Some(map.next_value()?);
                        }
                        VALUE_FIELD => {
                            if value.is_some() {
                                return Err(serde::de::Error::duplicate_field(VALUE_FIELD));
                            }
                            value = Some(map.next_value()?);
                        }
                        END_LINE_FIELD => {
                            if end_line.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_LINE_FIELD));
                            }
                            end_line = Some(map.next_value()?);
                        }
                        END_COL_FIELD => {
                            if end_col.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_COL_FIELD));
                            }
                            end_col = Some(map.next_value()?);
                        }
                        END_OFFSET_FIELD => {
                            if end_offset.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_OFFSET_FIELD));
                            }
                            end_offset = Some(map.next_value()?);
                        }
                        field => {
                            return Err(serde::de::Error::unknown_field(
                                field,
                                &[
                                    START_LINE_FIELD,
                                    START_COL_FIELD,
                                    START_OFFSET_FIELD,
                                    VALUE_FIELD,
                                    END_LINE_FIELD,
                                    END_COL_FIELD,
                                    END_OFFSET_FIELD,
                                ],
                            ));
                        }
                    }
                }

                let Some(start_line) = start_line else {
                    return Err(serde::de::Error::missing_field(START_LINE_FIELD));
                };
                let Some(start_col) = start_col else {
                    return Err(serde::de::Error::missing_field(START_COL_FIELD));
                };
                let Some(start_offset) = start_offset else {
                    return Err(serde::de::Error::missing_field(START_OFFSET_FIELD));
                };
                let Some(value) = value else {
                    return Err(serde::de::Error::missing_field(VALUE_FIELD));
                };
                let Some(end_line) = end_line else {
                    return Err(serde::de::Error::missing_field(END_LINE_FIELD));
                };
                let Some(end_col) = end_col else {
                    return Err(serde::de::Error::missing_field(END_COL_FIELD));
                };
                let Some(end_offset) = end_offset else {
                    return Err(serde::de::Error::missing_field(END_OFFSET_FIELD));
                };
                Ok(Spanned {
                    span: Span::new(
                        SpanPosition::new(start_line, start_col, start_offset),
                        SpanPosition::new(end_line, end_col, end_offset),
                    ),
                    value,
                })
            }
        }

        let visitor = SpannedVisitor(core::marker::PhantomData::<T>);

        deserializer.deserialize_struct(NAME, &FIELDS, visitor)
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for Spanned<T>
where
    T: serde::de::DeserializeSeed<'de>,
{
    type Value = Spanned<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Spanned<T::Value>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        pub(crate) struct SpannedVisitor<T>(T);

        impl<'de, T> serde::de::Visitor<'de> for SpannedVisitor<Option<T>>
        where
            T: serde::de::DeserializeSeed<'de>,
        {
            type Value = Spanned<T::Value>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a spanned value")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Spanned<T::Value>, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut start_line: Option<usize> = None;
                let mut start_col: Option<usize> = None;
                let mut start_offset: Option<usize> = None;
                let mut end_line: Option<usize> = None;
                let mut end_col: Option<usize> = None;
                let mut end_offset: Option<usize> = None;
                let mut value: Option<T::Value> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        START_LINE_FIELD => {
                            if start_line.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_LINE_FIELD));
                            }
                            start_line = Some(map.next_value()?);
                        }
                        START_COL_FIELD => {
                            if start_col.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_COL_FIELD));
                            }
                            start_col = Some(map.next_value()?);
                        }
                        START_OFFSET_FIELD => {
                            if start_offset.is_some() {
                                return Err(serde::de::Error::duplicate_field(START_OFFSET_FIELD));
                            }
                            start_offset = Some(map.next_value()?);
                        }
                        VALUE_FIELD => {
                            if value.is_some() {
                                return Err(serde::de::Error::duplicate_field(VALUE_FIELD));
                            }
                            value = Some(map.next_value_seed(self.0.take().unwrap())?);
                        }
                        END_LINE_FIELD => {
                            if end_line.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_LINE_FIELD));
                            }
                            end_line = Some(map.next_value()?);
                        }
                        END_COL_FIELD => {
                            if end_col.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_COL_FIELD));
                            }
                            end_col = Some(map.next_value()?);
                        }
                        END_OFFSET_FIELD => {
                            if end_offset.is_some() {
                                return Err(serde::de::Error::duplicate_field(END_OFFSET_FIELD));
                            }
                            end_offset = Some(map.next_value()?);
                        }
                        field => {
                            return Err(serde::de::Error::unknown_field(
                                field,
                                &[
                                    START_LINE_FIELD,
                                    START_COL_FIELD,
                                    START_OFFSET_FIELD,
                                    VALUE_FIELD,
                                    END_LINE_FIELD,
                                    END_COL_FIELD,
                                    END_OFFSET_FIELD,
                                ],
                            ));
                        }
                    }
                }

                let Some(start_line) = start_line else {
                    return Err(serde::de::Error::missing_field(START_LINE_FIELD));
                };
                let Some(start_col) = start_col else {
                    return Err(serde::de::Error::missing_field(START_COL_FIELD));
                };
                let Some(start_offset) = start_offset else {
                    return Err(serde::de::Error::missing_field(START_OFFSET_FIELD));
                };
                let Some(value) = value else {
                    return Err(serde::de::Error::missing_field(VALUE_FIELD));
                };
                let Some(end_line) = end_line else {
                    return Err(serde::de::Error::missing_field(END_LINE_FIELD));
                };
                let Some(end_col) = end_col else {
                    return Err(serde::de::Error::missing_field(END_COL_FIELD));
                };
                let Some(end_offset) = end_offset else {
                    return Err(serde::de::Error::missing_field(END_OFFSET_FIELD));
                };
                Ok(Spanned {
                    span: Span::new(
                        SpanPosition::new(start_line, start_col, start_offset),
                        SpanPosition::new(end_line, end_col, end_offset),
                    ),
                    value,
                })
            }
        }

        let visitor = SpannedVisitor(Some(self.into_inner()));

        deserializer.deserialize_struct(NAME, &FIELDS, visitor)
    }
}

impl<'de, T> serde::Deserializer<'de> for Spanned<T>
where
    T: serde::Deserializer<'de>,
{
    type Error = T::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_bool(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_i8(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_i16(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_i32(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_i64(visitor)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_i128(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_u8(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_u16(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_u32(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_u64(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_u128(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_f32(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_f64(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_char(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_string(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_bytes(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_option(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner()
            .deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_map(visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_inner().deserialize_ignored_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        self.get_ref().is_human_readable()
    }
}

impl<'de, T> serde::Deserializer<'de> for &'de Spanned<T>
where
    &'de T: serde::Deserializer<'de>,
{
    type Error = <&'de T as serde::de::Deserializer<'de>>::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_bool(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_i8(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_i16(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_i32(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_i64(visitor)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_i128(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_u8(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_u16(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_u32(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_u64(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_u128(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_f32(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_f64(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_char(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_string(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_bytes(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_option(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_map(visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.get_ref().deserialize_ignored_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        self.get_ref().is_human_readable()
    }
}

impl<T> serde::ser::Serialize for Spanned<T>
where
    T: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.value.serialize(serializer)
    }
}

// TODO: remove - just for testing
#[cfg(test)]
mod test {
    use crate::spanned::*;
    use crate::*;

    fn show_spanned<T: core::fmt::Debug>(s: &Spanned<T>, source: &str) {
        use codespan_reporting::diagnostic::{Diagnostic, Label};
        use codespan_reporting::files::SimpleFiles;
        use codespan_reporting::term;
        use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

        let mut files = SimpleFiles::new();
        let file_id = files.add("input", source);
        let diagnostic = Diagnostic::note()
            .with_message(std::format!("Look, it's a {}", std::any::type_name::<T>()))
            .with_notes(std::vec![
                std::format!("{:?}", s.as_ref()),
                std::format!(
                    "From {}:{} ({}) to {}:{} ({})",
                    s.span().start.line,
                    s.span().start.column,
                    s.span().start.byte_offset,
                    s.span().end.line,
                    s.span().end.column,
                    s.span().end.byte_offset,
                )
            ])
            .with_labels(std::vec![Label::primary(file_id, s.byte_span())]);

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        let mut writer_lock = writer.lock();

        term::emit(&mut writer_lock, &config, &files, &diagnostic).unwrap()
    }

    fn show_recursive(v: &Spanned<SpannedValue>, source: &str) {
        show_spanned(v, source);
        match v.get_ref() {
            SpannedValue::Array(values) => values.iter().for_each(|v| show_recursive(v, source)),
            SpannedValue::Object(map) => map.iter().for_each(|(k, v)| {
                show_spanned(k, source);
                show_recursive(v, source)
            }),
            _ => {}
        }
    }

    #[test]
    fn foo() {
        // Some JSON input data as a &str. Maybe this comes from the user.
        let data = r#"
        {
            "name🔥": "John Doe",
            "age": 42,
            "phones":    [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

        let v: Spanned<SpannedValue> = from_str_spanned(data).unwrap();
        show_recursive(&v, data);
    }
}
