//! JSON Deserialization
//!
//! This module provides for JSON deserialization with the type `Deserializer`.

use std::{i32, u64};
use std::io;
use std::marker::PhantomData;

use serde::de;

use super::error::{Error, ErrorCode, Result};

use read::{self, Read};

//////////////////////////////////////////////////////////////////////////////

/// A structure that deserializes JSON into Rust values.
pub struct Deserializer<Iter>(DeserializerImpl<read::IteratorRead<Iter>>)
    where Iter: Iterator<Item = io::Result<u8>>;

impl<Iter> Deserializer<Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
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
    where Iter: Iterator<Item = io::Result<u8>>,
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
    fn deserialize_newtype_struct<V>(
        &mut self,
        name: &'static str,
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.0.deserialize_newtype_struct(name, visitor)
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V>(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        self.0.deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit seq seq_fixed_size bytes map unit_struct tuple_struct struct
        struct_field tuple ignored_any
    }
}

//////////////////////////////////////////////////////////////////////////////

struct DeserializerImpl<R: Read> {
    read: R,
    str_buf: Vec<u8>,
    remaining_depth: u8,
}

macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        $a >= $c / 10 && ($a > $c / 10 || $b > $c % 10)
    }
}

impl<R: Read> DeserializerImpl<R> {
    fn new(read: R) -> Self {
        DeserializerImpl {
            read: read,
            str_buf: Vec::with_capacity(128),
            remaining_depth: 128,
        }
    }

    fn end(&mut self) -> Result<()> {
        if try!(self.parse_whitespace()) { // true if eof
            Ok(())
        } else {
            Err(self.peek_error(ErrorCode::TrailingCharacters))
        }
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

    /// Consume whitespace until the next non-whitespace character.
    ///
    /// Return `Ok(true)` if EOF was encountered in the process and `Ok(false)` otherwise.
    fn parse_whitespace(&mut self) -> Result<bool> {
        loop {
            match try!(self.peek()) {
                Some(b) => match b {
                    b' ' | b'\n' | b'\t' | b'\r' => {
                        self.eat_char();
                    }
                    _ => {
                        return Ok(false);
                    }
                },
                None => return Ok(true),
            }
        }
    }

    fn parse_value<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        if try!(self.parse_whitespace()) { // true if eof
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
            b'0'...b'9' => self.parse_integer(true, visitor),
            b'"' => {
                self.eat_char();
                self.str_buf.clear();
                let s = try!(self.read.parse_str(&mut self.str_buf));
                visitor.visit_str(s)
            }
            b'[' => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err(self.peek_error(stack_overflow()));
                }

                self.eat_char();
                let ret = visitor.visit_seq(SeqVisitor::new(self));

                self.remaining_depth += 1;

                ret
            }
            b'{' => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err(self.peek_error(stack_overflow()));
                }

                self.eat_char();
                let ret = visitor.visit_map(MapVisitor::new(self));

                self.remaining_depth += 1;

                ret
            }
            _ => Err(self.peek_error(ErrorCode::ExpectedSomeValue)),
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
                    b'0'...b'9' => {
                        Err(self.peek_error(ErrorCode::InvalidNumber))
                    }
                    _ => self.parse_number(pos, 0, visitor),
                }
            }
            c @ b'1'...b'9' => {
                let mut res = (c - b'0') as u64;

                loop {
                    match try!(self.peek_or_null()) {
                        c @ b'0'...b'9' => {
                            self.eat_char();
                            let digit = (c - b'0') as u64;

                            // We need to be careful with overflow. If we can, try to keep the
                            // number as a `u64` until we grow too large. At that point, switch to
                            // parsing the value as a `f64`.
                            if overflow!(res * 10 + digit, u64::MAX) {
                                return self.parse_long_integer(pos,
                                                               res,
                                                               1, // res * 10^1
                                                               visitor);
                            }

                            res = res * 10 + digit;
                        }
                        _ => {
                            return self.parse_number(pos, res, visitor);
                        }
                    }
                }
            }
            _ => Err(self.error(ErrorCode::InvalidNumber)),
        }
    }

    fn parse_long_integer<V>(
        &mut self,
        pos: bool,
        significand: u64,
        mut exponent: i32,
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        loop {
            match try!(self.peek_or_null()) {
                b'0'...b'9' => {
                    self.eat_char();
                    // This could overflow... if your integer is gigabytes long.
                    // Ignore that possibility.
                    exponent += 1;
                }
                b'.' => {
                    return self.parse_decimal(pos, significand, exponent, visitor);
                }
                b'e' | b'E' => {
                    return self.parse_exponent(pos, significand, exponent, visitor);
                }
                _ => {
                    return self.visit_f64_from_parts(pos,
                                                     significand,
                                                     exponent,
                                                     visitor);
                }
            }
        }
    }

    fn parse_number<V>(
        &mut self,
        pos: bool,
        significand: u64,
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        match try!(self.peek_or_null()) {
            b'.' => self.parse_decimal(pos, significand, 0, visitor),
            b'e' | b'E' => self.parse_exponent(pos, significand, 0, visitor),
            _ => {
                if pos {
                    visitor.visit_u64(significand)
                } else {
                    let neg = (significand as i64).wrapping_neg();

                    // Convert into a float if we underflow.
                    if neg > 0 {
                        visitor.visit_f64(-(significand as f64))
                    } else {
                        visitor.visit_i64(neg)
                    }
                }
            }
        }
    }

    fn parse_decimal<V>(
        &mut self,
        pos: bool,
        mut significand: u64,
        mut exponent: i32,
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let mut at_least_one_digit = false;
        while let c @ b'0'...b'9' = try!(self.peek_or_null()) {
            self.eat_char();
            let digit = (c - b'0') as u64;
            at_least_one_digit = true;

            if overflow!(significand * 10 + digit, u64::MAX) {
                // The next multiply/add would overflow, so just ignore all
                // further digits.
                while let b'0'...b'9' = try!(self.peek_or_null()) {
                    self.eat_char();
                }
                break;
            }

            significand = significand * 10 + digit;
            exponent -= 1;
        }

        if !at_least_one_digit {
            return Err(self.peek_error(ErrorCode::InvalidNumber));
        }

        match try!(self.peek_or_null()) {
            b'e' | b'E' => {
                self.parse_exponent(pos, significand, exponent, visitor)
            }
            _ => self.visit_f64_from_parts(pos, significand, exponent, visitor),
        }
    }

    fn parse_exponent<V>(
        &mut self,
        pos: bool,
        significand: u64,
        starting_exp: i32,
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        self.eat_char();

        let pos_exp = match try!(self.peek_or_null()) {
            b'+' => {
                self.eat_char();
                true
            }
            b'-' => {
                self.eat_char();
                false
            }
            _ => true,
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match try!(self.next_char_or_null()) {
            c @ b'0'...b'9' => (c - b'0') as i32,
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        };

        while let c @ b'0'...b'9' = try!(self.peek_or_null()) {
            self.eat_char();
            let digit = (c - b'0') as i32;

            if overflow!(exp * 10 + digit, i32::MAX) {
                return self.parse_exponent_overflow(pos,
                                                    significand,
                                                    pos_exp,
                                                    visitor);
            }

            exp = exp * 10 + digit;
        }

        let final_exp = if pos_exp {
            starting_exp.saturating_add(exp)
        } else {
            starting_exp.saturating_sub(exp)
        };

        self.visit_f64_from_parts(pos, significand, final_exp, visitor)
    }

    // This cold code should not be inlined into the middle of the hot
    // exponent-parsing loop above.
    #[cold]
    #[inline(never)]
    fn parse_exponent_overflow<V>(
        &mut self,
        pos: bool,
        significand: u64,
        pos_exp: bool,
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        // Error instead of +/- infinity.
        if significand != 0 && pos_exp {
            return Err(self.error(ErrorCode::NumberOutOfRange));
        }

        while let b'0'...b'9' = try!(self.peek_or_null()) {
            self.eat_char();
        }
        visitor.visit_f64(if pos {
            0.0
        } else {
            -0.0
        })
    }

    fn visit_f64_from_parts<V>(
        &mut self,
        pos: bool,
        significand: u64,
        mut exponent: i32,
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        let mut f = significand as f64;
        loop {
            match POW10.get(exponent.abs() as usize) {
                Some(&pow) => {
                    if exponent >= 0 {
                        f *= pow;
                        if f.is_infinite() {
                            return Err(self.error(ErrorCode::NumberOutOfRange));
                        }
                    } else {
                        f /= pow;
                    }
                    break;
                }
                None => {
                    if f == 0.0 {
                        break;
                    }
                    if exponent >= 0 {
                        return Err(self.error(ErrorCode::NumberOutOfRange));
                    }
                    f /= 1e308;
                    exponent += 308;
                }
            }
        }
        visitor.visit_f64(if pos {
            f
        } else {
            -f
        })
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

fn stack_overflow() -> ErrorCode {
    ErrorCode::Custom("recursion limit exceeded".into())
}

static POW10: [f64; 309] =
    [1e000, 1e001, 1e002, 1e003, 1e004, 1e005, 1e006, 1e007, 1e008, 1e009,
     1e010, 1e011, 1e012, 1e013, 1e014, 1e015, 1e016, 1e017, 1e018, 1e019,
     1e020, 1e021, 1e022, 1e023, 1e024, 1e025, 1e026, 1e027, 1e028, 1e029,
     1e030, 1e031, 1e032, 1e033, 1e034, 1e035, 1e036, 1e037, 1e038, 1e039,
     1e040, 1e041, 1e042, 1e043, 1e044, 1e045, 1e046, 1e047, 1e048, 1e049,
     1e050, 1e051, 1e052, 1e053, 1e054, 1e055, 1e056, 1e057, 1e058, 1e059,
     1e060, 1e061, 1e062, 1e063, 1e064, 1e065, 1e066, 1e067, 1e068, 1e069,
     1e070, 1e071, 1e072, 1e073, 1e074, 1e075, 1e076, 1e077, 1e078, 1e079,
     1e080, 1e081, 1e082, 1e083, 1e084, 1e085, 1e086, 1e087, 1e088, 1e089,
     1e090, 1e091, 1e092, 1e093, 1e094, 1e095, 1e096, 1e097, 1e098, 1e099,
     1e100, 1e101, 1e102, 1e103, 1e104, 1e105, 1e106, 1e107, 1e108, 1e109,
     1e110, 1e111, 1e112, 1e113, 1e114, 1e115, 1e116, 1e117, 1e118, 1e119,
     1e120, 1e121, 1e122, 1e123, 1e124, 1e125, 1e126, 1e127, 1e128, 1e129,
     1e130, 1e131, 1e132, 1e133, 1e134, 1e135, 1e136, 1e137, 1e138, 1e139,
     1e140, 1e141, 1e142, 1e143, 1e144, 1e145, 1e146, 1e147, 1e148, 1e149,
     1e150, 1e151, 1e152, 1e153, 1e154, 1e155, 1e156, 1e157, 1e158, 1e159,
     1e160, 1e161, 1e162, 1e163, 1e164, 1e165, 1e166, 1e167, 1e168, 1e169,
     1e170, 1e171, 1e172, 1e173, 1e174, 1e175, 1e176, 1e177, 1e178, 1e179,
     1e180, 1e181, 1e182, 1e183, 1e184, 1e185, 1e186, 1e187, 1e188, 1e189,
     1e190, 1e191, 1e192, 1e193, 1e194, 1e195, 1e196, 1e197, 1e198, 1e199,
     1e200, 1e201, 1e202, 1e203, 1e204, 1e205, 1e206, 1e207, 1e208, 1e209,
     1e210, 1e211, 1e212, 1e213, 1e214, 1e215, 1e216, 1e217, 1e218, 1e219,
     1e220, 1e221, 1e222, 1e223, 1e224, 1e225, 1e226, 1e227, 1e228, 1e229,
     1e230, 1e231, 1e232, 1e233, 1e234, 1e235, 1e236, 1e237, 1e238, 1e239,
     1e240, 1e241, 1e242, 1e243, 1e244, 1e245, 1e246, 1e247, 1e248, 1e249,
     1e250, 1e251, 1e252, 1e253, 1e254, 1e255, 1e256, 1e257, 1e258, 1e259,
     1e260, 1e261, 1e262, 1e263, 1e264, 1e265, 1e266, 1e267, 1e268, 1e269,
     1e270, 1e271, 1e272, 1e273, 1e274, 1e275, 1e276, 1e277, 1e278, 1e279,
     1e280, 1e281, 1e282, 1e283, 1e284, 1e285, 1e286, 1e287, 1e288, 1e289,
     1e290, 1e291, 1e292, 1e293, 1e294, 1e295, 1e296, 1e297, 1e298, 1e299,
     1e300, 1e301, 1e302, 1e303, 1e304, 1e305, 1e306, 1e307, 1e308];

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
            _ => visitor.visit_some(self),
        }
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V>(
        &mut self,
        _name: &str,
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V>(
        &mut self,
        _name: &str,
        _variants: &'static [&'static str],
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::EnumVisitor,
    {
        try!(self.parse_whitespace());

        match try!(self.peek_or_null()) {
            b'{' => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err(self.peek_error(stack_overflow()));
                }

                self.eat_char();
                let value = try!(visitor.visit(VariantVisitor::new(self)));

                self.remaining_depth += 1;

                try!(self.parse_whitespace());

                match try!(self.next_char_or_null()) {
                    b'}' => Ok(value),
                    _ => Err(self.error(ErrorCode::ExpectedSomeValue)),
                }
            }
            b'"' => visitor.visit(KeyOnlyVariantVisitor::new(self)),
            _ => Err(self.peek_error(ErrorCode::ExpectedSomeValue)),
        }
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit seq seq_fixed_size bytes map unit_struct tuple_struct struct
        struct_field tuple ignored_any
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
                    return Err(self.de
                        .peek_error(ErrorCode::ExpectedListCommaOrEnd));
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
            Some(b']') => Ok(()),
            Some(_) => Err(self.de.error(ErrorCode::TrailingCharacters)),
            None => Err(self.de.error(ErrorCode::EOFWhileParsingList)),
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
                    return Err(self.de
                        .peek_error(ErrorCode::ExpectedObjectCommaOrEnd));
                }
            }
            None => {
                return Err(self.de
                    .peek_error(ErrorCode::EOFWhileParsingObject));
            }
        }

        match try!(self.de.peek()) {
            Some(b'"') => Ok(Some(try!(de::Deserialize::deserialize(self.de)))),
            Some(_) => Err(self.de.peek_error(ErrorCode::KeyMustBeAString)),
            None => Err(self.de.peek_error(ErrorCode::EOFWhileParsingValue)),
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
            Some(b'}') => Ok(()),
            Some(_) => Err(self.de.error(ErrorCode::TrailingCharacters)),
            None => Err(self.de.error(ErrorCode::EOFWhileParsingObject)),
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {
        use std;

        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = de::value::Error;

            fn deserialize<V>(
                &mut self,
                _visitor: V
            ) -> std::result::Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                let &mut MissingFieldDeserializer(field) = self;
                Err(de::value::Error::MissingField(field))
            }

            fn deserialize_option<V>(
                &mut self,
                mut visitor: V
            ) -> std::result::Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }

            forward_to_deserialize! {
                bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
                string unit seq seq_fixed_size bytes map unit_struct
                newtype_struct tuple_struct struct struct_field tuple enum
                ignored_any
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
        where V: de::Deserialize,
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

    fn visit_tuple<V>(&mut self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(
        &mut self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
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
        where V: de::Deserialize,
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

    fn visit_tuple<V>(&mut self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(
        &mut self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        de::Deserializer::deserialize(self.de, visitor)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Iterator that deserializes a stream into multiple JSON values.
pub struct StreamDeserializer<T, Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
          T: de::Deserialize,
{
    deser: DeserializerImpl<read::IteratorRead<Iter>>,
    _marker: PhantomData<T>,
}

impl<T, Iter> StreamDeserializer<T, Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
          T: de::Deserialize,
{
    /// Returns an `Iterator` of decoded JSON values from an iterator over
    /// `Iterator<Item=io::Result<u8>>`.
    pub fn new(iter: Iter) -> StreamDeserializer<T, Iter> {
        StreamDeserializer {
            deser: DeserializerImpl::new(read::IteratorRead::new(iter)),
            _marker: PhantomData,
        }
    }
}

impl<T, Iter> Iterator for StreamDeserializer<T, Iter>
    where Iter: Iterator<Item = io::Result<u8>>,
          T: de::Deserialize,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        // skip whitespaces, if any
        // this helps with trailing whitespaces, since whitespaces between
        // values are handled for us.
        match self.deser.parse_whitespace() {
            Ok(true) => None, // eof
            Ok(false) => {
                match de::Deserialize::deserialize(&mut self.deser) {
                    Ok(v) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn from_trait<R, T>(read: R) -> Result<T>
    where R: Read,
          T: de::Deserialize,
{
    let mut de = DeserializerImpl::new(read);
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

/// Decodes a json value from an iterator over an iterator
/// `Iterator<Item=io::Result<u8>>`.
pub fn from_iter<I, T>(iter: I) -> Result<T>
    where I: Iterator<Item = io::Result<u8>>,
          T: de::Deserialize,
{
    from_trait(read::IteratorRead::new(iter))
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
    where T: de::Deserialize,
{
    from_trait(read::SliceRead::new(v))
}

/// Decodes a json value from a `&str`.
pub fn from_str<T>(s: &str) -> Result<T>
    where T: de::Deserialize,
{
    from_trait(read::StrRead::new(s))
}
