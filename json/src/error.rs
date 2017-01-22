//! JSON Errors
//!
//! This module is centered around the `Error` and `ErrorCode` types, which represents all possible
//! `serde_json` errors.

use std::error;
use std::fmt::{self, Display};
use std::io;
use std::result;

use serde::de;
use serde::ser;

/// The errors that can arise while parsing a JSON stream.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode {
    /// Catchall for syntax error messages
    Message(String),

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

    /// Number is bigger than the maximum value of its type.
    NumberOutOfRange,

    /// Invalid unicode code point.
    InvalidUnicodeCodePoint,

    /// Object key is not a string.
    KeyMustBeAString,

    /// Lone leading surrogate in hex escape.
    LoneLeadingSurrogateInHexEscape,

    /// JSON has non-whitespace trailing characters after the value.
    TrailingCharacters,

    /// Unexpected end of hex excape.
    UnexpectedEndOfHexEscape,

    /// Encountered nesting of JSON maps and arrays more than 128 layers deep.
    RecursionLimitExceeded,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Message(ref msg) => write!(f, "{}", msg),
            ErrorCode::EOFWhileParsingList => "EOF while parsing a list".fmt(f),
            ErrorCode::EOFWhileParsingObject => {
                "EOF while parsing an object".fmt(f)
            }
            ErrorCode::EOFWhileParsingString => {
                "EOF while parsing a string".fmt(f)
            }
            ErrorCode::EOFWhileParsingValue => {
                "EOF while parsing a value".fmt(f)
            }
            ErrorCode::ExpectedColon => "expected `:`".fmt(f),
            ErrorCode::ExpectedListCommaOrEnd => "expected `,` or `]`".fmt(f),
            ErrorCode::ExpectedObjectCommaOrEnd => "expected `,` or `}`".fmt(f),
            ErrorCode::ExpectedSomeIdent => "expected ident".fmt(f),
            ErrorCode::ExpectedSomeValue => "expected value".fmt(f),
            ErrorCode::InvalidEscape => "invalid escape".fmt(f),
            ErrorCode::InvalidNumber => "invalid number".fmt(f),
            ErrorCode::NumberOutOfRange => "number out of range".fmt(f),
            ErrorCode::InvalidUnicodeCodePoint => {
                "invalid unicode code point".fmt(f)
            }
            ErrorCode::KeyMustBeAString => "key must be a string".fmt(f),
            ErrorCode::LoneLeadingSurrogateInHexEscape => {
                "lone leading surrogate in hex escape".fmt(f)
            }
            ErrorCode::TrailingCharacters => "trailing characters".fmt(f),
            ErrorCode::UnexpectedEndOfHexEscape => {
                "unexpected end of hex escape".fmt(f)
            }
            ErrorCode::RecursionLimitExceeded => {
                "recursion limit exceeded".fmt(f)
            }
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
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Syntax(..) => "syntax error",
            Error::Io(ref error) => error::Error::description(error),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref error) => Some(error),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Syntax(ref code, line, col) => {
                if line == 0 && col == 0 {
                    write!(fmt, "{}", code)
                } else {
                    write!(fmt, "{} at line {} column {}", code, line, col)
                }
            }
            Error::Io(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<de::value::Error> for Error {
    fn from(error: de::value::Error) -> Error {
        Error::Syntax(ErrorCode::Message(error.to_string()), 0, 0)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Syntax(ErrorCode::Message(msg.to_string()), 0, 0)
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Syntax(ErrorCode::Message(msg.to_string()), 0, 0)
    }
}

/// Helper alias for `Result` objects that return a JSON `Error`.
pub type Result<T> = result::Result<T, Error>;
