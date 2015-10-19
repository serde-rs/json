//! JSON Errors
//!
//! This module is centered around the `Error` and `ErrorCode` types, which represents all possible
//! `serde_json` errors.

use std::error;
use std::fmt;
use std::io;
use std::result;
use std::string::FromUtf8Error;

use serde::de;

/// The errors that can arise while parsing a JSON stream.
#[derive(Clone, PartialEq)]
pub enum ErrorCode {
    /// EOF while parsing a list.
    EOFWhileParsingList,

    /// EOF while parsing an object.
    EOFWhileParsingObject,

    /// EOF while parsing a string.
    EOFWhileParsingString,

    /// EOF while parsing a JSON value.
    EOFWhileParsingValue,

    /// Expected this character to be a `':'`.
    ExpectedColon,

    /// Expected this character to be either a `','` or a `]`.
    ExpectedListCommaOrEnd,

    /// Expected this character to be either a `','` or a `}`.
    ExpectedObjectCommaOrEnd,

    /// Expected to parse either a `true`, `false`, or a `null`.
    ExpectedSomeIdent,

    /// Expected this character to start a JSON value.
    ExpectedSomeValue,

    /// Invalid hex escape code.
    InvalidEscape,

    /// Invalid number.
    InvalidNumber,

    /// Invalid unicode code point.
    InvalidUnicodeCodePoint,

    /// Invalid value type.
    InvalidType(de::Type),

    /// Invalid length.
    InvalidLength(usize),

    /// Object key is not a string.
    KeyMustBeAString,

    /// Lone leading surrogate in hex escape.
    LoneLeadingSurrogateInHexEscape,

    /// Unknown field in struct.
    UnknownField(String),

    /// Struct is missing a field.
    MissingField(&'static str),

    /// JSON has non-whitespace trailing characters after the value.
    TrailingCharacters,

    /// Unexpected end of hex excape.
    UnexpectedEndOfHexEscape,

    /// Custom error message.
    Custom(String),
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;

        match *self {
            ErrorCode::EOFWhileParsingList => "EOF while parsing a list".fmt(f),
            ErrorCode::EOFWhileParsingObject => "EOF while parsing an object".fmt(f),
            ErrorCode::EOFWhileParsingString => "EOF while parsing a string".fmt(f),
            ErrorCode::EOFWhileParsingValue => "EOF while parsing a value".fmt(f),
            ErrorCode::ExpectedColon => "expected `:`".fmt(f),
            ErrorCode::ExpectedListCommaOrEnd => "expected `,` or `]`".fmt(f),
            ErrorCode::ExpectedObjectCommaOrEnd => "expected `,` or `}`".fmt(f),
            ErrorCode::ExpectedSomeIdent => "expected ident".fmt(f),
            ErrorCode::ExpectedSomeValue => "expected value".fmt(f),
            ErrorCode::InvalidEscape => "invalid escape".fmt(f),
            ErrorCode::InvalidNumber => "invalid number".fmt(f),
            ErrorCode::InvalidUnicodeCodePoint => "invalid unicode code point".fmt(f),
            ErrorCode::InvalidType(ref _type) => write!(f, "invalid type \"{:?}\"", _type),
            ErrorCode::InvalidLength(ref length) => write!(f, "invalid length \"{}\"", length),
            ErrorCode::KeyMustBeAString => "key must be a string".fmt(f),
            ErrorCode::LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape".fmt(f),
            ErrorCode::UnknownField(ref field) => write!(f, "unknown field \"{}\"", field),
            ErrorCode::MissingField(ref field) => write!(f, "missing field \"{}\"", field),
            ErrorCode::TrailingCharacters => "trailing characters".fmt(f),
            ErrorCode::UnexpectedEndOfHexEscape => "unexpected end of hex escape".fmt(f),
            ErrorCode::Custom(ref msg) => write!(f, "custom message: \"{}\"", msg),
        }
    }
}

/// This type represents all possible errors that can occur when serializing or deserializing a
/// value into JSON.
#[derive(Debug)]
pub enum Error {
    /// The JSON value had some syntatic error.
    Syntax(ErrorCode, usize, usize),

    /// Some IO error occurred when serializing or deserializing a value.
    Io(io::Error),

    /// Some UTF8 error occurred while serializing or deserializing a value.
    FromUtf8(FromUtf8Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Syntax(..) => "syntax error",
            Error::Io(ref error) => error::Error::description(error),
            Error::FromUtf8(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref error) => Some(error),
            Error::FromUtf8(ref error) => Some(error),
            _ => None,
        }
    }

}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Syntax(ref code, line, col) => {
                write!(fmt, "{:?} at line {} column {}", code, line, col)
            }
            Error::Io(ref error) => fmt::Display::fmt(error, fmt),
            Error::FromUtf8(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::FromUtf8(error)
    }
}

impl From<de::value::Error> for Error {
    fn from(error: de::value::Error) -> Self {
        match error {
            de::value::Error::Syntax(msg) => {
                Error::Syntax(ErrorCode::Custom(msg.into()), 0, 0)
            }
            de::value::Error::Type(type_) => {
                Error::Syntax(ErrorCode::InvalidType(type_), 0, 0)
            }
            de::value::Error::Length(length) => {
                Error::Syntax(ErrorCode::InvalidLength(length), 0, 0)
            }
            de::value::Error::EndOfStream => {
                de::Error::end_of_stream()
            }
            de::value::Error::UnknownField(field) => {
                Error::Syntax(ErrorCode::UnknownField(String::from(field)), 0, 0)
            }
            de::value::Error::MissingField(field) => {
                Error::Syntax(ErrorCode::MissingField(field), 0, 0)
            }
        }
    }
}

impl de::Error for Error {
    fn syntax(msg: &str) -> Self {
        Error::Syntax(ErrorCode::Custom(msg.into()), 0, 0)
    }

    fn length_mismatch(length: usize) -> Self {
        Error::Syntax(ErrorCode::InvalidLength(length), 0, 0)
    }

    fn type_mismatch(type_: de::Type) -> Self {
        Error::Syntax(ErrorCode::InvalidType(type_), 0, 0)
    }

    fn end_of_stream() -> Self {
        Error::Syntax(ErrorCode::EOFWhileParsingValue, 0, 0)
    }

    fn unknown_field(field: &str) -> Self {
        Error::Syntax(ErrorCode::UnknownField(String::from(field)), 0, 0)
    }

    fn missing_field(field: &'static str) -> Self {
        Error::Syntax(ErrorCode::MissingField(field), 0, 0)
    }
}

/// Helper alias for `Result` objects that return a JSON `Error`.
pub type Result<T> = result::Result<T, Error>;
