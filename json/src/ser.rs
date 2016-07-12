//! JSON Serialization
//!
//! This module provides for JSON serialization with the type `Serializer`.

use std::io;
use std::num::FpCategory;

use serde::ser;
use super::error::{Error, ErrorCode, Result};

use itoa;
use dtoa;

/// A structure for serializing Rust values into JSON.
pub struct Serializer<W, F=CompactFormatter> {
    writer: W,
    formatter: F,

    /// `first` is used to signify if we should print a comma when we are walking through a
    /// sequence.
    first: bool,
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
            first: false,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[doc(hidden)]
pub enum MapSerializer {
    Map,
    Enum,
    Empty,
}

impl ser::MapSerializer for MapSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, K, V>(&mut self, serializer: &mut S, key: K, value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
              S: ser::Serializer<Error = Error> {
        serializer.serialize_map_elt(key, value)
    }

    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<()> where S: ser::Serializer<Error = Error> {
        match self {
            MapSerializer::Map => serializer.serialize_map_end(),
            MapSerializer::Empty => Ok(()),
            MapSerializer::Enum => serializer.serialize_struct_variant_end(),
        }
    }
}

#[doc(hidden)]
pub enum SeqSerializer {
    Seq,
    Enum,
    Empty,
}

impl ser::SeqSerializer for SeqSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, T>(&mut self, serializer: &mut S, value: T) -> Result<()>
        where T: ser::Serialize, S: ser::Serializer<Error = Error> {
        serializer.serialize_seq_elt(value)
    }

    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<()> where S: ser::Serializer<Error = Error> {
        match self {
            SeqSerializer::Seq => serializer.serialize_seq_end(),
            SeqSerializer::Empty => Ok(()),
            SeqSerializer::Enum => serializer.serialize_tuple_variant_end(),
        }
    }
}

impl<W, F> ser::Serializer for Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    type Error = Error;
    type SeqSerializer = SeqSerializer;
    type MapSerializer = MapSerializer;

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        if value {
            self.writer.write_all(b"true").map_err(From::from)
        } else {
            self.writer.write_all(b"false").map_err(From::from)
        }
    }

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<()> {
        itoa::write(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> Result<()> {
        fmt_f32_or_null(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<()> {
        fmt_f64_or_null(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<()> {
        escape_char(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        escape_str(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> Result<()>
        where V: ser::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(&mut self) -> Result<()> {
        self.writer.write_all(b"null").map_err(From::from)
    }

    /// Override `visit_newtype_struct` to serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(&mut self,
                               _name: &'static str,
                               value: T) -> Result<()>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<()> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<()>
        where T: ser::Serialize,
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(value.serialize(self));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_seq<'a>(&'a mut self, len: Option<usize>) -> Result<ser::SeqHelper<'a, Self>>
    {
        if let Some(0) = len {
            try!(self.writer.write_all(b"[]"));
            Ok(ser::SeqHelper::new(self, SeqSerializer::Empty))
        } else {
            try!(self.formatter.open(&mut self.writer, b'['));

            self.first = true;

            Ok(ser::SeqHelper::new(self, SeqSerializer::Seq))
        }
    }

    #[inline]
    fn serialize_seq_end(&mut self) -> Result<()> {
        self.formatter.close(&mut self.writer, b']')
    }

    #[inline]
    fn serialize_tuple_variant<'a>(&'a mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              len: usize) -> Result<ser::SeqHelper<'a, Self>>
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        if len == 0 {
            try!(self.writer.write_all(b"[]"));
            Ok(ser::SeqHelper::new(self, SeqSerializer::Empty))
        } else {
            try!(self.formatter.open(&mut self.writer, b'['));

            self.first = true;

            Ok(ser::SeqHelper::new(self, SeqSerializer::Enum))
        }
    }

    #[inline]
    fn serialize_tuple_variant_end(&mut self) -> Result<()> {
        try!(self.formatter.close(&mut self.writer, b']'));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<()>
        where T: ser::Serialize,
    {
        try!(self.formatter.comma(&mut self.writer, self.first));
        try!(value.serialize(self));

        self.first = false;

        Ok(())
    }

    #[inline]
    fn serialize_map<'a>(&'a mut self, len: Option<usize>) -> Result<ser::MapHelper<'a, Self>>
    {
        if let Some(0) = len {
            try!(self.writer.write_all(b"{}"));
            Ok(ser::MapHelper::new(self, MapSerializer::Empty))
        } else {
            try!(self.formatter.open(&mut self.writer, b'{'));

            self.first = true;

            Ok(ser::MapHelper::new(self, MapSerializer::Map))
        }
    }

    #[inline]
    fn serialize_struct_variant<'a>(&'a mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               len: usize) -> Result<ser::MapHelper<'a, Self>>
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        if len == 0 {
            try!(self.writer.write_all(b"{}"));
            Ok(ser::MapHelper::new(self, MapSerializer::Empty))
        } else {
            try!(self.formatter.open(&mut self.writer, b'{'));

            self.first = true;

            Ok(ser::MapHelper::new(self, MapSerializer::Enum))
        }
    }

    fn serialize_struct_variant_end(&mut self) -> Result<()> {
        try!(self.formatter.close(&mut self.writer, b'}'));
        self.formatter.close(&mut self.writer, b'}')
    }

    fn serialize_map_end(&mut self) -> Result<()> {
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(self.formatter.comma(&mut self.writer, self.first));

        try!(key.serialize(&mut MapKeySerializer { ser: self }));
        try!(self.formatter.colon(&mut self.writer));
        try!(value.serialize(self));

        self.first = false;

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
    type SeqSerializer = SeqSerializer;
    type MapSerializer = MapSerializer;

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        self.ser.serialize_str(value)
    }

    fn serialize_bool(&mut self, _value: bool) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i64(&mut self, _value: i64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u64(&mut self, _value: u64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_f64(&mut self, _value: f64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_unit(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_none(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_some<V>(&mut self, _value: V) -> Result<()>
        where V: ser::Serialize
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq<'b>(&'b mut self, _len: Option<usize>) -> Result<ser::SeqHelper<'b, Self>>
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_elt<T>(&mut self, _value: T) -> Result<()>
        where T: ser::Serialize,
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_end(&mut self) -> Result<()>
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map<'b>(&'b mut self, _len: Option<usize>) -> Result<ser::MapHelper<'b, Self>>
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_elt<K, V>(&mut self, _key: K, _value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_end(&mut self) -> Result<()>
    {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }
}

/// This trait abstracts away serializing the JSON control characters, which allows the user to
/// optionally pretty print the JSON output.
pub trait Formatter {
    /// Called when serializing a '{' or '['.
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ','.
    fn comma<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ':'.
    fn colon<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;

    /// Called when serializing a '}' or ']'.
    fn close<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write;
}

/// This structure compacts a JSON value with no extra whitespace.
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write,
    {
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",").map_err(From::from)
        }
    }

    fn colon<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write,
    {
        writer.write_all(b":").map_err(From::from)
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write,
    {
        writer.write_all(&[ch]).map_err(From::from)
    }
}

/// This structure pretty prints a JSON value to make it human readable.
pub struct PrettyFormatter<'a> {
    current_indent: usize,
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
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write,
    {
        self.current_indent += 1;
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write,
    {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b",\n"));
        }

        indent(writer, self.current_indent, self.indent)
    }

    fn colon<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write,
    {
        writer.write_all(b": ").map_err(From::from)
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write,
    {
        self.current_indent -= 1;
        try!(writer.write(b"\n"));
        try!(indent(writer, self.current_indent, self.indent));

        writer.write_all(&[ch]).map_err(From::from)
    }
}

/// Serializes and escapes a `&str` into a JSON string.
pub fn escape_str<W>(wr: &mut W, value: &str) -> Result<()>
    where W: io::Write
{
    let bytes = value.as_bytes();

    try!(wr.write_all(b"\""));

    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            try!(wr.write_all(&bytes[start..i]));
        }

        if escape == b'u' {
            static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
            try!(wr.write_all(&[
                b'\\', b'u', b'0', b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ]));
        } else {
            try!(wr.write_all(&[b'\\', escape]));
        }

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    try!(wr.write_all(b"\""));
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
fn escape_char<W>(wr: &mut W, value: char) -> Result<()>
    where W: io::Write
{
    // FIXME: this allocation is required in order to be compatible with stable
    // rust, which doesn't support encoding a `char` into a stack buffer.
    let mut s = String::new();
    s.push(value);
    escape_str(wr, &s)
}

fn fmt_f32_or_null<W>(wr: &mut W, value: f32) -> Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => {
            try!(wr.write_all(b"null"))
        }
        _ => {
            try!(dtoa::write(wr, value))
        }
    }

    Ok(())
}

fn fmt_f64_or_null<W>(wr: &mut W, value: f64) -> Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => {
            try!(wr.write_all(b"null"))
        }
        _ => {
            try!(dtoa::write(wr, value))
        }
    }

    Ok(())
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Encode the specified struct into a json `[u8]` writer.
#[inline]
pub fn to_writer_pretty<W, T>(writer: &mut W, value: &T) -> Result<()>
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
    where T: ser::Serialize
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
    where T: ser::Serialize
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
    for _ in 0 .. n {
        try!(wr.write_all(s));
    }

    Ok(())
}
