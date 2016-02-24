//! JSON Serialization
//!
//! This module provides for JSON serialization with the type `Serializer`.

use std::io;
use std::num::FpCategory;

use serde::ser;
use super::error::{Error, ErrorCode, Result};

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

impl<W, F> ser::Serializer for Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    type Error = Error;

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
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<()> {
        write!(&mut self.writer, "{}", value).map_err(From::from)
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
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.writer.write_all(b"[]"));
        self.formatter.close(&mut self.writer, b'}')
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
    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        match visitor.len() {
            Some(len) if len == 0 => {
                self.writer.write_all(b"[]").map_err(From::from)
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'['));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b']').map_err(From::from)
            }
        }

    }

    #[inline]
    fn serialize_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.serialize_seq(visitor));
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
    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        match visitor.len() {
            Some(len) if len == 0 => {
                self.writer.write_all(b"{}").map_err(From::from)
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'{'));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b'}')
            }
        }
    }

    #[inline]
    fn serialize_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(self.serialize_str(variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.serialize_map(visitor));

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

    #[inline]
    fn format() -> &'static str {
        "json"
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

    fn serialize_bool(&mut self, _value: bool) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i64(&mut self, _value: i64) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u64(&mut self, _value: u64) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_f64(&mut self, _value: f64) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_unit(&mut self) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_none(&mut self) -> Result<()> {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_some<V>(&mut self, _value: V) -> Result<()>
        where V: ser::Serialize
    {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq<V>(&mut self, _visitor: V) -> Result<()>
        where V: ser::SeqVisitor,
    {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_elt<T>(&mut self, _value: T) -> Result<()>
        where T: ser::Serialize,
    {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map<V>(&mut self, _visitor: V) -> Result<()>
        where V: ser::MapVisitor,
    {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_elt<K, V>(&mut self, _key: K, _value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        Err(Error::SyntaxError(ErrorCode::KeyMustBeAString, 0, 0))
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

/// Serializes and escapes a `&[u8]` into a JSON string.
#[inline]
pub fn escape_bytes<W>(wr: &mut W, bytes: &[u8]) -> Result<()>
    where W: io::Write
{
    try!(wr.write_all(b"\""));

    let mut start = 0;

    for (i, byte) in bytes.iter().enumerate() {
        let escaped = match *byte {
            b'"' => b"\\\"",
            b'\\' => b"\\\\",
            b'\x08' => b"\\b",
            b'\x0c' => b"\\f",
            b'\n' => b"\\n",
            b'\r' => b"\\r",
            b'\t' => b"\\t",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write_all(&bytes[start..i]));
        }

        try!(wr.write_all(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    try!(wr.write_all(b"\""));
    Ok(())
}

/// Serializes and escapes a `&str` into a JSON string.
#[inline]
pub fn escape_str<W>(wr: &mut W, value: &str) -> Result<()>
    where W: io::Write
{
    escape_bytes(wr, value.as_bytes())
}

#[inline]
fn escape_char<W>(wr: &mut W, value: char) -> Result<()>
    where W: io::Write
{
    // FIXME: this allocation is required in order to be compatible with stable
    // rust, which doesn't support encoding a `char` into a stack buffer.
    let mut s = String::new();
    s.push(value);
    escape_bytes(wr, s.as_bytes())
}

fn fmt_f32_or_null<W>(wr: &mut W, value: f32) -> Result<()>
    where W: io::Write
{
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => {
            try!(wr.write_all(b"null"))
        }
        _ => {
            try!(write!(wr, "{:?}", value))
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
            try!(write!(wr, "{:?}", value))
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
    let string = try!(String::from_utf8(vec));
    Ok(string)
}

/// Encode the specified struct into a json `String` buffer.
#[inline]
pub fn to_string_pretty<T>(value: &T) -> Result<String>
    where T: ser::Serialize
{
    let vec = try!(to_vec_pretty(value));
    let string = try!(String::from_utf8(vec));
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
