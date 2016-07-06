//! JSON Deserialization
//!
//! This module provides for JSON deserialization with the type `Deserializer`.

use std::i32;
use std::io;
use std::str;
use std::marker::PhantomData;

use serde::de;

use super::error::{Error, ErrorCode, Result};

use read::{self, Read};

//////////////////////////////////////////////////////////////////////////////

/// A structure that deserializes JSON into Rust values.
pub struct Deserializer<Iter>(DeserializerImpl<read::IteratorRead<Iter>>)
    where Iter: Iterator<Item=io::Result<u8>>;

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    /// Creates the JSON parser from an `std::iter::Iterator`.
    #[inline]
    pub fn new(rdr: Iter) -> Self {
        Deserializer(DeserializerImpl::new(read::IteratorRead::new(rdr)))
    }

    /// The `Deserializer::end` method should be called after a value has been fully deserialized.
    /// This allows the `Deserializer` to validate that the input stream is at the end or that it
    /// only has trailing whitespace.
    #[inline]
    pub fn end(&mut self) -> Result<()> {
        self.0.end()
    }
}

impl<Iter> de::Deserializer for Deserializer<Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
{
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.0.deserialize(visitor)
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn deserialize_option<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.0.deserialize_option(visitor)
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V>(&mut self,
                                     name: &'static str,
                                     visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.0.deserialize_newtype_struct(name, visitor)
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V>(&mut self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        self.0.deserialize_enum(name, variants, visitor)
    }
}

//////////////////////////////////////////////////////////////////////////////

struct DeserializerImpl<R: Read> {
    read: R,
    str_buf: Vec<u8>,
}

macro_rules! try_or_invalid {
    ($self_:expr, $e:expr) => {
        match $e {
            Some(v) => v,
            None => { return Err($self_.error(ErrorCode::InvalidNumber)); }
        }
    }
}

impl<R: Read> DeserializerImpl<R> {
    fn new(read: R) -> Self {
        DeserializerImpl {
            read: read,
            str_buf: Vec::with_capacity(128),
        }
    }

    fn end(&mut self) -> Result<()> {
        try!(self.parse_whitespace());
        if try!(self.eof()) {
            Ok(())
        } else {
            Err(self.peek_error(ErrorCode::TrailingCharacters))
        }
    }

    fn eof(&mut self) -> Result<bool> {
        Ok(try!(self.peek()).is_none())
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        self.read.peek().map_err(Error::Io)
    }

    fn peek_or_null(&mut self) -> Result<u8> {
        Ok(try!(self.peek()).unwrap_or(b'\x00'))
    }

    fn eat_char(&mut self) {
        self.read.discard();
    }

    fn next_char(&mut self) -> Result<Option<u8>> {
        self.read.next().map_err(Error::Io)
    }

    fn next_char_or_null(&mut self) -> Result<u8> {
        Ok(try!(self.next_char()).unwrap_or(b'\x00'))
    }

    /// Error caused by a byte from next_char().
    fn error(&mut self, reason: ErrorCode) -> Error {
        let pos = self.read.position();
        Error::Syntax(reason, pos.line, pos.column)
    }

    /// Error caused by a byte from peek().
    fn peek_error(&mut self, reason: ErrorCode) -> Error {
        let pos = self.read.peek_position();
        Error::Syntax(reason, pos.line, pos.column)
    }

    fn parse_whitespace(&mut self) -> Result<()> {
        loop {
            match try!(self.peek_or_null()) {
                b' ' | b'\n' | b'\t' | b'\r' => {
                    self.eat_char();
                }
                _ => { return Ok(()); }
            }
        }
    }

    fn parse_value<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        try!(self.parse_whitespace());

        if try!(self.eof()) {
            return Err(self.peek_error(ErrorCode::EOFWhileParsingValue));
        }

        let value = match try!(self.peek_or_null()) {
            b'n' => {
                self.eat_char();
                try!(self.parse_ident(b"ull"));
                visitor.visit_unit()
            }
            b't' => {
                self.eat_char();
                try!(self.parse_ident(b"rue"));
                visitor.visit_bool(true)
            }
            b'f' => {
                self.eat_char();
                try!(self.parse_ident(b"alse"));
                visitor.visit_bool(false)
            }
            b'-' => {
                self.eat_char();
                self.parse_integer(false, visitor)
            }
            b'0' ... b'9' => {
                self.parse_integer(true, visitor)
            }
            b'"' => {
                self.eat_char();
                self.str_buf.clear();
                let s = try!(self.read.parse_str(&mut self.str_buf));
                visitor.visit_str(s)
            }
            b'[' => {
                self.eat_char();
                visitor.visit_seq(SeqVisitor::new(self))
            }
            b'{' => {
                self.eat_char();
                visitor.visit_map(MapVisitor::new(self))
            }
            _ => {
                Err(self.peek_error(ErrorCode::ExpectedSomeValue))
            }
        };

        match value {
            Ok(value) => Ok(value),
            // The de::Error and From<de::value::Error> impls both create errors
            // with unknown line and column. Fill in the position here by
            // looking at the current index in the input. There is no way to
            // tell whether this should call `error` or `peek_error` so pick the
            // one that seems correct more often. Worst case, the position is
            // off by one character.
            Err(Error::Syntax(code, 0, 0)) => Err(self.error(code)),
            Err(err) => Err(err),
        }
    }

    fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for c in ident {
            if Some(*c) != try!(self.next_char()) {
                return Err(self.error(ErrorCode::ExpectedSomeIdent));
            }
        }

        Ok(())
    }

    fn parse_integer<V>(&mut self, pos: bool, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match try!(self.next_char_or_null()) {
            b'0' => {
                // There can be only one leading '0'.
                match try!(self.peek_or_null()) {
                    b'0' ... b'9' => {
                        Err(self.peek_error(ErrorCode::InvalidNumber))
                    }
                    _ => {
                        self.parse_number(pos, 0, visitor)
                    }
                }
            },
            c @ b'1' ... b'9' => {
                let mut res: u64 = (c as u64) - ('0' as u64);

                loop {
                    match try!(self.peek_or_null()) {
                        c @ b'0' ... b'9' => {
                            self.eat_char();

                            let digit = (c as u64) - ('0' as u64);

                            // We need to be careful with overflow. If we can, try to keep the
                            // number as a `u64` until we grow too large. At that point, switch to
                            // parsing the value as a `f64`.
                            match res.checked_mul(10).and_then(|val| val.checked_add(digit)) {
                                Some(res_) => { res = res_; }
                                None => {
                                    return self.parse_float(
                                        pos,
                                        (res as f64) * 10.0 + (digit as f64),
                                        visitor);
                                }
                            }
                        }
                        _ => {
                            return self.parse_number(pos, res, visitor);
                        }
                    }
                }
            }
            _ => {
                Err(self.error(ErrorCode::InvalidNumber))
            }
        }
    }

    fn parse_float<V>(&mut self,
                      pos: bool,
                      mut res: f64,
                      mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        loop {
            match try!(self.next_char_or_null()) {
                c @ b'0' ... b'9' => {
                    let digit = (c as u64) - ('0' as u64);

                    res *= 10.0;
                    res += digit as f64;
                }
                _ => {
                    match try!(self.peek_or_null()) {
                        b'.' => {
                            return self.parse_decimal(pos, res, visitor);
                        }
                        b'e' | b'E' => {
                            return self.parse_exponent(pos, res, visitor);
                        }
                        _ => {
                            if !pos {
                                res = -res;
                            }

                            return visitor.visit_f64(res);
                        }
                    }
                }
            }
        }
    }

    fn parse_number<V>(&mut self,
                       pos: bool,
                       res: u64,
                       mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        match try!(self.peek_or_null()) {
            b'.' => {
                self.parse_decimal(pos, res as f64, visitor)
            }
            b'e' | b'E' => {
                self.parse_exponent(pos, res as f64, visitor)
            }
            _ => {
                if pos {
                    visitor.visit_u64(res)
                } else {
                    let res_i64 = (res as i64).wrapping_neg();

                    // Convert into a float if we underflow.
                    if res_i64 > 0 {
                        visitor.visit_f64(-(res as f64))
                    } else {
                        visitor.visit_i64(res_i64)
                    }
                }
            }
        }
    }

    fn parse_decimal<V>(&mut self,
                        pos: bool,
                        mut res: f64,
                        mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let mut dec = 0.1;

        // Make sure a digit follows the decimal place.
        match try!(self.next_char_or_null()) {
            c @ b'0' ... b'9' => {
                res += (((c as u64) - (b'0' as u64)) as f64) * dec;
            }
             _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        }

        loop {
            match try!(self.peek_or_null()) {
                c @ b'0' ... b'9' => {
                    self.eat_char();

                    dec /= 10.0;
                    res += (((c as u64) - (b'0' as u64)) as f64) * dec;
                }
                _ => { break; }
            }
        }

        match try!(self.peek_or_null()) {
            b'e' | b'E' => {
                self.parse_exponent(pos, res, visitor)
            }
            _ => {
                if pos {
                    visitor.visit_f64(res)
                } else {
                    visitor.visit_f64(-res)
                }
            }
        }

    }

    fn parse_exponent<V>(&mut self,
                         pos: bool,
                         mut res: f64,
                         mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let pos_exp = match try!(self.peek_or_null()) {
            b'+' => { self.eat_char(); true }
            b'-' => { self.eat_char(); false }
            _ => { true }
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match try!(self.next_char_or_null()) {
            c @ b'0' ... b'9' => { (c as u64) - (b'0' as u64) }
            _ => { return Err(self.error(ErrorCode::InvalidNumber)); }
        };

        loop {
            match try!(self.peek_or_null()) {
                c @ b'0' ... b'9' => {
                    self.eat_char();

                    exp = try_or_invalid!(self, exp.checked_mul(10));
                    exp = try_or_invalid!(self, exp.checked_add((c as u64) - (b'0' as u64)));
                }
                _ => { break; }
            }
        }

        let exp = if exp <= i32::MAX as u64 {
            10_f64.powi(exp as i32)
        } else {
            return Err(self.peek_error(ErrorCode::InvalidNumber));
        };

        if pos_exp {
            res *= exp;
        } else {
            res /= exp;
        }

        if pos {
            visitor.visit_f64(res)
        } else {
            visitor.visit_f64(-res)
        }
    }

    fn parse_object_colon(&mut self) -> Result<()> {
        try!(self.parse_whitespace());

        match try!(self.peek()) {
            Some(b':') => {
                self.eat_char();
                Ok(())
            }
            Some(_) => Err(self.peek_error(ErrorCode::ExpectedColon)),
            None => Err(self.peek_error(ErrorCode::EOFWhileParsingObject)),
        }
    }
}

impl<R: Read> de::Deserializer for DeserializerImpl<R> {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.parse_value(visitor)
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        try!(self.parse_whitespace());

        match try!(self.peek_or_null()) {
            b'n' => {
                self.eat_char();
                try!(self.parse_ident(b"ull"));
                visitor.visit_none()
            }
            _ => {
                visitor.visit_some(self)
            }
        }
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V>(&mut self,
                               _name: &str,
                               mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V>(&mut self,
                     _name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        try!(self.parse_whitespace());

        match try!(self.peek_or_null()) {
            b'{' => {
                self.eat_char();
                try!(self.parse_whitespace());

                let value = {
                    try!(visitor.visit(VariantVisitor::new(self)))
                };

                try!(self.parse_whitespace());

                match try!(self.next_char_or_null()) {
                    b'}' => {
                        Ok(value)
                    }
                    _ => {
                        Err(self.error(ErrorCode::ExpectedSomeValue))
                    }
                }
            }
            b'"' => {
                visitor.visit(KeyOnlyVariantVisitor::new(self))
            }
            _ => {
                Err(self.peek_error(ErrorCode::ExpectedSomeValue))
            }
        }
    }
}

struct SeqVisitor<'a, R: Read + 'a> {
    de: &'a mut DeserializerImpl<R>,
    first: bool,
}

impl<'a, R: Read + 'a> SeqVisitor<'a, R> {
    fn new(de: &'a mut DeserializerImpl<R>) -> Self {
        SeqVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, R: Read + 'a> de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize,
    {
        try!(self.de.parse_whitespace());

        match try!(self.de.peek()) {
            Some(b']') => {
                return Ok(None);
            }
            Some(b',') if !self.first => {
                self.de.eat_char();
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(self.de.peek_error(ErrorCode::ExpectedListCommaOrEnd));
                }
            }
            None => {
                return Err(self.de.peek_error(ErrorCode::EOFWhileParsingList));
            }
        }

        let value = try!(de::Deserialize::deserialize(self.de));
        Ok(Some(value))
    }

    fn end(&mut self) -> Result<()> {
        try!(self.de.parse_whitespace());

        match try!(self.de.next_char()) {
            Some(b']') => { Ok(()) }
            Some(_) => {
                Err(self.de.error(ErrorCode::TrailingCharacters))
            }
            None => {
                Err(self.de.error(ErrorCode::EOFWhileParsingList))
            }
        }
    }
}

struct MapVisitor<'a, R: Read + 'a> {
    de: &'a mut DeserializerImpl<R>,
    first: bool,
}

impl<'a, R: Read + 'a> MapVisitor<'a, R> {
    fn new(de: &'a mut DeserializerImpl<R>) -> Self {
        MapVisitor {
            de: de,
            first: true,
        }
    }
}

impl<'a, R: Read + 'a> de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize,
    {
        try!(self.de.parse_whitespace());

        match try!(self.de.peek()) {
            Some(b'}') => {
                return Ok(None);
            }
            Some(b',') if !self.first => {
                self.de.eat_char();
                try!(self.de.parse_whitespace());
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(self.de.peek_error(ErrorCode::ExpectedObjectCommaOrEnd));
                }
            }
            None => {
                return Err(self.de.peek_error(ErrorCode::EOFWhileParsingObject));
            }
        }

        match try!(self.de.peek()) {
            Some(b'"') => {
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            Some(_) => {
                Err(self.de.peek_error(ErrorCode::KeyMustBeAString))
            }
            None => {
                Err(self.de.peek_error(ErrorCode::EOFWhileParsingValue))
            }
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        try!(self.de.parse_object_colon());

        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<()> {
        try!(self.de.parse_whitespace());

        match try!(self.de.next_char()) {
            Some(b'}') => { Ok(()) }
            Some(_) => {
                Err(self.de.error(ErrorCode::TrailingCharacters))
            }
            None => {
                Err(self.de.error(ErrorCode::EOFWhileParsingObject))
            }
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {
        use std;

        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = de::value::Error;

            fn deserialize<V>(&mut self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                let &mut MissingFieldDeserializer(field) = self;
                Err(de::value::Error::MissingField(field))
            }

            fn deserialize_option<V>(&mut self,
                                     mut visitor: V) -> std::result::Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        let mut de = MissingFieldDeserializer(field);
        Ok(try!(de::Deserialize::deserialize(&mut de)))
    }
}

struct VariantVisitor<'a, R: Read + 'a> {
    de: &'a mut DeserializerImpl<R>,
}

impl<'a, R: Read + 'a> VariantVisitor<'a, R> {
    fn new(de: &'a mut DeserializerImpl<R>) -> Self {
        VariantVisitor {
            de: de,
        }
    }
}

impl<'a, R: Read + 'a> de::VariantVisitor for VariantVisitor<'a, R> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V>
        where V: de::Deserialize
    {
        let val = try!(de::Deserialize::deserialize(self.de));
        try!(self.de.parse_object_colon());
        Ok(val)
    }

    fn visit_unit(&mut self) -> Result<()> {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_newtype<T>(&mut self) -> Result<T>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }
}

struct KeyOnlyVariantVisitor<'a, R: Read + 'a> {
    de: &'a mut DeserializerImpl<R>,
}

impl<'a, R: Read + 'a> KeyOnlyVariantVisitor<'a, R> {
    fn new(de: &'a mut DeserializerImpl<R>) -> Self {
        KeyOnlyVariantVisitor {
            de: de,
        }
    }
}

impl<'a, R: Read + 'a> de::VariantVisitor for KeyOnlyVariantVisitor<'a, R> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V>
        where V: de::Deserialize
    {
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn visit_unit(&mut self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype<T>(&mut self) -> Result<T>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(self.de)
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Iterator that deserializes a stream into multiple JSON values.
pub struct StreamDeserializer<T, Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize
{
    deser: DeserializerImpl<read::IteratorRead<Iter>>,
    _marker: PhantomData<T>,
}

impl <T, Iter> StreamDeserializer<T, Iter>
    where Iter:Iterator<Item=io::Result<u8>>,
          T: de::Deserialize
{
    /// Returns an `Iterator` of decoded JSON values from an iterator over
    /// `Iterator<Item=io::Result<u8>>`.
    pub fn new(iter: Iter) -> StreamDeserializer<T, Iter> {
        StreamDeserializer {
            deser: DeserializerImpl::new(read::IteratorRead::new(iter)),
            _marker: PhantomData
        }
    }
}

impl <T, Iter> Iterator for StreamDeserializer<T, Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        // skip whitespaces, if any
        // this helps with trailing whitespaces, since whitespaces between
        // values are handled for us.
        if let Err(e) = self.deser.parse_whitespace() {
            return Some(Err(e))
        };

        match self.deser.eof() {
            Ok(true) => None,
            Ok(false) => match de::Deserialize::deserialize(&mut self.deser) {
                Ok(v) => Some(Ok(v)),
                Err(e) => Some(Err(e))
            },
            Err(e) => Some(Err(e))
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Decodes a json value from an iterator over an iterator
/// `Iterator<Item=io::Result<u8>>`.
pub fn from_iter<I, T>(iter: I) -> Result<T>
    where I: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize,
{
    let mut de = DeserializerImpl::new(read::IteratorRead::new(iter));
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes a json value from a `std::io::Read`.
pub fn from_reader<R, T>(rdr: R) -> Result<T>
    where R: io::Read,
          T: de::Deserialize,
{
    from_iter(rdr.bytes())
}

/// Decodes a json value from a byte slice `&[u8]`.
pub fn from_slice<T>(v: &[u8]) -> Result<T>
    where T: de::Deserialize
{
    let mut de = DeserializerImpl::new(read::SliceRead::new(v));
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes a json value from a `&str`.
pub fn from_str<T>(s: &str) -> Result<T>
    where T: de::Deserialize
{
    from_slice(s.as_bytes())
}
