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
use serde::ser;

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

    /// Invalid length
    Length(usize),

    /// Incorrect type from value
    Type(de::Type),

    /// Catchall for syntax error messages
    Syntax(String),
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
            ErrorCode::KeyMustBeAString => "key must be a string".fmt(f),
            ErrorCode::LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape".fmt(f),
            ErrorCode::UnknownField(ref field) => write!(f, "unknown field \"{}\"", field),
            ErrorCode::MissingField(ref field) => write!(f, "missing field \"{}\"", field),
            ErrorCode::TrailingCharacters => "trailing characters".fmt(f),
            ErrorCode::UnexpectedEndOfHexEscape => "unexpected end of hex escape".fmt(f),
            ErrorCode::Length(ref len) => write!(f, "incorrect value length {}", len),
            ErrorCode::Type(ref ty) => write!(f, "incorrect value type: {:?}", ty),
            ErrorCode::Syntax(ref msg) => write!(f, "syntax error: {:?}", msg),
        }
    }
}

/// This type represents all possible errors that can occur when serializing or deserializing a
/// value into JSON.
#[derive(Debug)]
pub enum Error {
    /// The JSON value had some syntatic error.
    SyntaxError(ErrorCode, usize, usize),

    /// Some IO error occurred when serializing or deserializing a value.
    IoError(io::Error),

    /// Some UTF8 error occurred while serializing or deserializing a value.
    FromUtf8Error(FromUtf8Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SyntaxError(..) => "syntax error",
            Error::IoError(ref error) => error::Error::description(error),
            Error::FromUtf8Error(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref error) => Some(error),
            Error::FromUtf8Error(ref error) => Some(error),
            _ => None,
        }
    }

}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::SyntaxError(ref code, line, col) => {
                write!(fmt, "{:?} at line {} column {}", code, line, col)
            }
            Error::IoError(ref error) => fmt::Display::fmt(error, fmt),
            Error::FromUtf8Error(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Error {
        Error::FromUtf8Error(error)
    }
}

impl From<de::value::Error> for Error {
    fn from(error: de::value::Error) -> Error {
        match error {
            de::value::Error::Syntax(_) => {
                Error::SyntaxError(ErrorCode::ExpectedSomeValue, 0, 0)
            }
            de::value::Error::EndOfStream => {
                de::Error::end_of_stream()
            }
            de::value::Error::UnknownField(field) => {
                Error::SyntaxError(ErrorCode::UnknownField(field), 0, 0)
            }
            de::value::Error::MissingField(field) => {
                Error::SyntaxError(ErrorCode::MissingField(field), 0, 0)
            }
            de::value::Error::Length(len) => {
                Error::SyntaxError(ErrorCode::Length(len), 0, 0)
            }
            de::value::Error::Type(ty) => {
                Error::SyntaxError(ErrorCode::Type(ty), 0, 0)
            }
        }
    }
}

impl de::Error for Error {
    fn syntax(_: &str) -> Error {
        Error::SyntaxError(ErrorCode::ExpectedSomeValue, 0, 0)
    }

    fn end_of_stream() -> Error {
        Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 0, 0)
    }

    fn unknown_field(field: &str) -> Error {
        Error::SyntaxError(ErrorCode::UnknownField(String::from(field)), 0, 0)
    }

    fn missing_field(field: &'static str) -> Error {
        Error::SyntaxError(ErrorCode::MissingField(field), 0, 0)
    }
}

impl ser::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn syntax(msg: &str) -> Self {
        Error::SyntaxError(ErrorCode::Syntax(String::from(msg)), 0, 0)
    }
}

/// Helper alias for `Result` objects that return a JSON `Error`.
pub type Result<T> = result::Result<T, Error>;
