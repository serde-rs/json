//! JSON Serialization
//!
//! This module provides for JSON serialization with the type `Serializer`.

use std::io;
use std::num::FpCategory;

use serde::ser;
use super::error::{Error, ErrorCode, Result};

use itoa;
use dtoa;
use num_traits;

/// A structure for serializing Rust values into JSON.
pub struct Serializer<W, F = CompactFormatter> {
    writer: W,
    formatter: F,
}

impl<W> Serializer<W>
    where W: io::Write,
{
    /// Creates a new JSON serializer.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
    where W: io::Write,
{
    /// Creates a new JSON pretty print serializer.
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    /// Creates a new JSON visitor whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }

    #[inline]
    fn write_integer<V>(&mut self, value: V) -> Result<()>
        where V: itoa::Integer
    {
        try!(self.formatter.write_integer(&mut self.writer, value));
        Ok(())
    }

    #[inline]
    fn write_floating<V>(&mut self, value: V) -> Result<()>
        where V: dtoa::Floating + num_traits::Float
    {
        match value.classify() {
            FpCategory::Nan | FpCategory::Infinite => {
                self.formatter.write_null(&mut self.writer)
            },
            _ => {
                self.formatter.write_floating(&mut self.writer, value)
            },
        }
    }
}

#[doc(hidden)]
#[derive(Eq, PartialEq)]
pub enum State {
    Empty,
    First,
    Rest,
}

impl<W, F> ser::Serializer for Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    type Error = Error;

    type SeqState = State;
    type TupleState = State;
    type TupleStructState = State;
    type TupleVariantState = State;
    type MapState = State;
    type StructState = State;
    type StructVariantState = State;

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        self.formatter.write_bool(&mut self.writer, value)
    }

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<()> {
        self.write_integer(value)
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> Result<()> {
        self.write_floating(value)
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<()> {
        self.write_floating(value)
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<()> {
        format_escaped_char(&mut self.writer, &mut self.formatter, value).map_err(From::from)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        format_escaped_str(&mut self.writer, &mut self.formatter, value).map_err(From::from)
    }

    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<()> {
        let mut state = try!(self.serialize_seq(Some(value.len())));
        for byte in value {
            try!(self.serialize_seq_elt(&mut state, byte));
        }
        self.serialize_seq_end(state)
    }

    #[inline]
    fn serialize_unit(&mut self) -> Result<()> {
        self.formatter.write_null(&mut self.writer)
    }

    #[inline]
    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    /// Serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(
        &mut self,
        _name: &'static str,
        value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        try!(self.formatter.begin_object(&mut self.writer));
        try!(self.formatter.begin_object_key(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.end_object_key(&mut self.writer));
        try!(self.formatter.begin_object_value(&mut self.writer));
        try!(value.serialize(self));
        try!(self.formatter.end_object_value(&mut self.writer));
        try!(self.formatter.end_object(&mut self.writer));
        Ok(())
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(&mut self, value: T) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(&mut self, len: Option<usize>) -> Result<State> {
        if len == Some(0) {
            try!(self.formatter.begin_array(&mut self.writer));
            try!(self.formatter.end_array(&mut self.writer));
            Ok(State::Empty)
        } else {
            try!(self.formatter.begin_array(&mut self.writer));
            Ok(State::First)
        }
    }

    #[inline]
    fn serialize_seq_elt<T: ser::Serialize>(
        &mut self,
        state: &mut State,
        value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        try!(self.formatter.begin_array_value(&mut self.writer, *state == State::First));
        *state = State::Rest;
        try!(value.serialize(self));
        try!(self.formatter.end_array_value(&mut self.writer));

        Ok(())
    }

    #[inline]
    fn serialize_seq_end(&mut self, state: State) -> Result<()> {
        match state {
            State::Empty => Ok(()),
            _ => self.formatter.end_array(&mut self.writer),
        }
    }

    #[inline]
    fn serialize_seq_fixed_size(&mut self, size: usize) -> Result<State> {
        self.serialize_seq(Some(size))
    }

    #[inline]
    fn serialize_tuple(&mut self, len: usize) -> Result<State> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_elt<T: ser::Serialize>(
        &mut self,
        state: &mut State,
        value: T
    ) -> Result<()> {
        self.serialize_seq_elt(state, value)
    }

    #[inline]
    fn serialize_tuple_end(&mut self, state: State) -> Result<()> {
        self.serialize_seq_end(state)
    }

    #[inline]
    fn serialize_tuple_struct(
        &mut self,
        _name: &'static str,
        len: usize
    ) -> Result<State> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct_elt<T: ser::Serialize>(
        &mut self,
        state: &mut State,
        value: T
    ) -> Result<()> {
        self.serialize_seq_elt(state, value)
    }

    #[inline]
    fn serialize_tuple_struct_end(&mut self, state: State) -> Result<()> {
        self.serialize_seq_end(state)
    }

    #[inline]
    fn serialize_tuple_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        len: usize
    ) -> Result<State> {
        try!(self.formatter.begin_object(&mut self.writer));
        try!(self.formatter.begin_object_key(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.end_object_key(&mut self.writer));
        try!(self.formatter.begin_object_value(&mut self.writer));
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant_elt<T: ser::Serialize>(
        &mut self,
        state: &mut State,
        value: T
    ) -> Result<()> {
        self.serialize_seq_elt(state, value)
    }

    #[inline]
    fn serialize_tuple_variant_end(&mut self, state: State) -> Result<()> {
        try!(self.serialize_seq_end(state));
        try!(self.formatter.end_object_value(&mut self.writer));
        try!(self.formatter.end_object(&mut self.writer));
        Ok(())
    }

    #[inline]
    fn serialize_map(&mut self, len: Option<usize>) -> Result<State> {
        if len == Some(0) {
            try!(self.formatter.begin_object(&mut self.writer));
            try!(self.formatter.end_object(&mut self.writer));
            Ok(State::Empty)
        } else {
            try!(self.formatter.begin_object(&mut self.writer));
            Ok(State::First)
        }
    }

    #[inline]
    fn serialize_map_key<T: ser::Serialize>(
        &mut self,
        state: &mut State,
        key: T,
    ) -> Result<()> {
        try!(self.formatter.begin_object_key(&mut self.writer, *state == State::First));
        *state = State::Rest;

        try!(key.serialize(&mut MapKeySerializer {
            ser: self,
        }));

        try!(self.formatter.end_object_key(&mut self.writer));
        Ok(())
    }

    #[inline]
    fn serialize_map_value<T: ser::Serialize>(
        &mut self,
        _: &mut State,
        value: T,
    ) -> Result<()> {
        try!(self.formatter.begin_object_value(&mut self.writer));
        try!(value.serialize(self));
        try!(self.formatter.end_object_value(&mut self.writer));
        Ok(())
    }

    #[inline]
    fn serialize_map_end(&mut self, state: State) -> Result<()> {
        match state {
            State::Empty => Ok(()),
            _ => self.formatter.end_object(&mut self.writer),
        }
    }

    #[inline]
    fn serialize_struct(
        &mut self,
        _name: &'static str,
        len: usize
    ) -> Result<State> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_elt<V: ser::Serialize>(
        &mut self,
        state: &mut State,
        key: &'static str,
        value: V
    ) -> Result<()> {
        try!(self.serialize_map_key(state, key));
        self.serialize_map_value(state, value)
    }

    #[inline]
    fn serialize_struct_end(&mut self, state: State) -> Result<()> {
        self.serialize_map_end(state)
    }

    #[inline]
    fn serialize_struct_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        len: usize
    ) -> Result<State> {
        try!(self.formatter.begin_object(&mut self.writer));
        try!(self.formatter.begin_object_key(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.end_object_key(&mut self.writer));
        try!(self.formatter.begin_object_value(&mut self.writer));
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant_elt<V: ser::Serialize>(
        &mut self,
        state: &mut State,
        key: &'static str,
        value: V
    ) -> Result<()> {
        self.serialize_struct_elt(state, key, value)
    }

    #[inline]
    fn serialize_struct_variant_end(&mut self, state: State) -> Result<()> {
        try!(self.serialize_struct_end(state));
        try!(self.formatter.end_object_value(&mut self.writer));
        try!(self.formatter.end_object(&mut self.writer));
        Ok(())
    }
}

struct MapKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ser::Serializer for MapKeySerializer<'a, W, F>
    where W: io::Write,
          F: Formatter,
{
    type Error = Error;

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        self.ser.serialize_str(value)
    }

    #[inline]
    fn serialize_unit_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str
    ) -> Result<()> {
        self.ser.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        &mut self,
        _name: &'static str,
        value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    type SeqState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_bool(&mut self, _value: bool) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_isize(&mut self, _value: isize) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i8(&mut self, _value: i8) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i16(&mut self, _value: i16) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i32(&mut self, _value: i32) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i64(&mut self, _value: i64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_usize(&mut self, _value: usize) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u8(&mut self, _value: u8) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u16(&mut self, _value: u16) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u32(&mut self, _value: u32) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u64(&mut self, _value: u64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_f32(&mut self, _value: f32) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_f64(&mut self, _value: f64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_char(&mut self, _value: char) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_bytes(&mut self, _value: &[u8]) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_unit(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_newtype_variant<T>(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_none(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_some<T>(&mut self, _value: T) -> Result<()>
        where T: ser::Serialize,
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq(&mut self, _len: Option<usize>) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_elt<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _value: T
    ) -> Result<()>
        where T: ser::Serialize,
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_fixed_size(&mut self, _size: usize) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple(&mut self, _len: usize) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_elt<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _value: T
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_struct(
        &mut self,
        _name: &'static str,
        _len: usize
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_struct_elt<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _value: T
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_struct_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _len: usize
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_variant_elt<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _value: T
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_tuple_variant_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map(&mut self, _len: Option<usize>) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_key<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _key: T,
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_value<T: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _value: T,
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct(
        &mut self,
        _name: &'static str,
        _len: usize
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct_elt<V: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _key: &'static str,
        _value: V
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _len: usize
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct_variant_elt<V: ser::Serialize>(
        &mut self,
        _state: &mut (),
        _key: &'static str,
        _value: V
    ) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_struct_variant_end(&mut self, _state: ()) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }
}

/// Represents a character escape code in a type-safe manner.
pub enum CharEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    /// An escaped solidus `/`
    Solidus,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped tab character (usually escaped as `\t`)
    Tab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\u00XX` where `XX` are two hex characters)
    AsciiControl(u8),
}

impl CharEscape {
    #[inline]
    fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
        match escape {
            self::BB => CharEscape::Backspace,
            self::TT => CharEscape::Tab,
            self::NN => CharEscape::LineFeed,
            self::FF => CharEscape::FormFeed,
            self::RR => CharEscape::CarriageReturn,
            self::QU => CharEscape::Quote,
            self::BS => CharEscape::ReverseSolidus,
            self::U => CharEscape::AsciiControl(byte),
            _ => unreachable!(),
        }
    }
}

/// This trait abstracts away serializing the JSON control characters, which allows the user to
/// optionally pretty print the JSON output.
pub trait Formatter {
    /// Writes a `null` value to the specified writer.
    #[inline]
    fn write_null<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"null").map_err(From::from)
    }

    /// Writes a `true` or `false` value to the specified writer.
    #[inline]
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> Result<()>
        where W: io::Write
    {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        writer.write_all(s).map_err(From::from)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_integer<W, I>(&mut self, writer: &mut W, value: I) -> Result<()>
        where W: io::Write,
              I: itoa::Integer
    {
        itoa::write(writer, value).map_err(From::from)
    }

    /// Writes a floating point value like `-31.26e+12` to the
    /// specified writer.
    #[inline]
    fn write_floating<W, F>(&mut self, writer: &mut W, value: F) -> Result<()>
        where W: io::Write,
              F: dtoa::Floating
    {
        dtoa::write(writer, value).map_err(From::from)
    }

    /// Called before each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn begin_string<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"\"").map_err(From::from)
    }

    /// Called after each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn end_string<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"\"").map_err(From::from)
    }

    /// Writes a string fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &[u8]) -> Result<()>
        where W: io::Write
    {
        writer.write_all(fragment).map_err(From::from)
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_char_escape<W>(&mut self, writer: &mut W, char_escape: CharEscape) -> Result<()>
        where W: io::Write
    {
        use self::CharEscape::*;

        let s = match char_escape {
            Quote => b"\\\"",
            ReverseSolidus => b"\\\\",
            Solidus => b"\\/",
            Backspace => b"\\b",
            FormFeed => b"\\f",
            LineFeed => b"\\n",
            CarriageReturn => b"\\r",
            Tab => b"\\t",
            AsciiControl(byte) => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[b'\\', b'u', b'0', b'0',
                              HEX_DIGITS[(byte >> 4) as usize],
                              HEX_DIGITS[(byte & 0xF) as usize]];
                return writer.write_all(bytes).map_err(From::from);
            }
        };

        writer.write_all(s).map_err(From::from)
    }

    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"[").map_err(From::from)
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"]").map_err(From::from)
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",").map_err(From::from)
        }
    }

    /// Called after every array value.
    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> Result<()>
        where W: io::Write
    {
        Ok(())
    }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"{").map_err(From::from)
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"}").map_err(From::from)
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",").map_err(From::from)
        }
    }

    /// Called after every object key.  A `:` should be written to the
    /// specified writer by either this method or
    /// `begin_object_value`.
    #[inline]
    fn end_object_key<W>(&mut self, _writer: &mut W) -> Result<()>
        where W: io::Write
    {
        Ok(())
    }

    /// Called before every object value.  A `:` should be written to
    /// the specified writer by either this method or
    /// `end_object_key`.
    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b":").map_err(From::from)
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> Result<()>
        where W: io::Write
    {
        Ok(())
    }
}

/// This structure compacts a JSON value with no extra whitespace.
#[derive(Clone, Debug)]
pub struct CompactFormatter;

impl Formatter for CompactFormatter {}

/// This structure pretty prints a JSON value to make it human readable.
#[derive(Clone, Debug)]
pub struct PrettyFormatter<'a> {
    current_indent: usize,
    has_value: bool,
    indent: &'a [u8],
}

impl<'a> PrettyFormatter<'a> {
    /// Construct a pretty printer formatter that defaults to using two spaces for indentation.
    pub fn new() -> Self {
        PrettyFormatter::with_indent(b"  ")
    }

    /// Construct a pretty printer formatter that uses the `indent` string for indentation.
    pub fn with_indent(indent: &'a [u8]) -> Self {
        PrettyFormatter {
            current_indent: 0,
            has_value: false,
            indent: indent,
        }
    }
}

impl<'a> Default for PrettyFormatter<'a> {
    fn default() -> Self {
        PrettyFormatter::new()
    }
}

impl<'a> Formatter for PrettyFormatter<'a> {
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"[").map_err(From::from)
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.current_indent -= 1;

        if self.has_value {
            try!(writer.write(b"\n"));
            try!(indent(writer, self.current_indent, self.indent));
        }

        writer.write_all(b"]").map_err(From::from)
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write
    {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b",\n"));
        }
        try!(indent(writer, self.current_indent, self.indent));
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"{").map_err(From::from)
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.current_indent -= 1;

        if self.has_value {
            try!(writer.write(b"\n"));
            try!(indent(writer, self.current_indent, self.indent));
        }

        writer.write_all(b"}").map_err(From::from)
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write
    {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b",\n"));
        }
        indent(writer, self.current_indent, self.indent)
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b": ").map_err(From::from)
    }

    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> Result<()>
        where W: io::Write
    {
        self.has_value = true;
        Ok(())
    }
}

/// Serializes and escapes a `&str` into a JSON string.
pub fn escape_str<W>(wr: &mut W, value: &str) -> Result<()>
    where W: io::Write
{
    format_escaped_str(wr, &mut CompactFormatter, value)
}

fn format_escaped_str<W, F>(writer: &mut W, formatter: &mut F, value: &str) -> Result<()>
    where W: io::Write,
          F: Formatter
{
    let bytes = value.as_bytes();

    try!(formatter.begin_string(writer));

    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            try!(formatter.write_string_fragment(writer, &bytes[start..i]));
        }

        let char_escape = CharEscape::from_escape_table(escape, byte);
        try!(formatter.write_char_escape(writer, char_escape));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(formatter.write_string_fragment(writer, &bytes[start..]));
    }

    try!(formatter.end_string(writer));
    Ok(())
}

const BB: u8 = b'b';  // \x08
const TT: u8 = b't';  // \x09
const NN: u8 = b'n';  // \x0A
const FF: u8 = b'f';  // \x0C
const RR: u8 = b'r';  // \x0D
const QU: u8 = b'"';  // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u';   // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[cfg_attr(rustfmt, rustfmt_skip)]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];

#[inline]
fn format_escaped_char<W, F>(wr: &mut W, formatter: &mut F, value: char) -> Result<()>
    where W: io::Write,
          F: Formatter,
{
    // FIXME: this allocation is required in order to be compatible with stable
    // rust, which doesn't support encoding a `char` into a stack buffer.
    let mut s = String::new();
    s.push(value);
    format_escaped_str(wr, formatter, &s)
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<W: ?Sized, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer_pretty<W: ?Sized, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::pretty(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
    where T: ser::Serialize,
{
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let mut writer = Vec::with_capacity(128);
    try!(to_writer(&mut writer, value));
    Ok(writer)
}

/// Encode the specified struct into a json `[u8]` buffer.
#[inline]
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>>
    where T: ser::Serialize,
{
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let mut writer = Vec::with_capacity(128);
    try!(to_writer_pretty(&mut writer, value));
    Ok(writer)
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
    where T: ser::Serialize,
{
    let vec = try!(to_vec(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string_pretty<T>(value: &T) -> Result<String>
    where T: ser::Serialize,
{
    let vec = try!(to_vec_pretty(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> Result<()>
    where W: io::Write,
{
    for _ in 0..n {
        try!(wr.write_all(s));
    }

    Ok(())
}
