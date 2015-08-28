use std::error;
use std::fmt;
use std::io;
use std::result;
use std::string::FromUtf8Error;

use serde::de;

/// The errors that can arise while parsing a JSON stream.
#[derive(Clone, PartialEq)]
pub enum ErrorCode {
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,

    ExpectedListCommaOrEnd,
    ExpectedObjectCommaOrEnd,
    ExpectedSomeIdent,
    ExpectedSomeValue,
    InvalidEscape,
    InvalidNumber,
    InvalidUnicodeCodePoint,
    KeyMustBeAString,
    LoneLeadingSurrogateInHexEscape,
    UnknownField(String),
    MissingField(&'static str),
    TrailingCharacters,
    UnexpectedEndOfHexEscape,
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;

        match *self {
            ErrorCode::EOFWhileParsingList => "EOF While parsing list".fmt(f),
            ErrorCode::EOFWhileParsingObject => "EOF While parsing object".fmt(f),
            ErrorCode::EOFWhileParsingString => "EOF While parsing string".fmt(f),
            ErrorCode::EOFWhileParsingValue => "EOF While parsing value".fmt(f),
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
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// msg, line, col
    SyntaxError(ErrorCode, usize, usize),
    IoError(io::Error),
    MissingFieldError(&'static str),
    FromUtf8Error(FromUtf8Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SyntaxError(..) => "syntax error",
            Error::IoError(ref error) => error::Error::description(error),
            Error::MissingFieldError(_) => "missing field",
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
            Error::MissingFieldError(ref field) => {
                write!(fmt, "missing field {}", field)
            }
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
            de::value::Error::SyntaxError => {
                Error::SyntaxError(ErrorCode::ExpectedSomeValue, 0, 0)
            }
            de::value::Error::EndOfStreamError => {
                de::Error::end_of_stream()
            }
            de::value::Error::UnknownFieldError(field) => {
                Error::SyntaxError(ErrorCode::UnknownField(field), 0, 0)
            }
            de::value::Error::MissingFieldError(field) => {
                de::Error::missing_field(field)
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
        Error::MissingFieldError(field)
    }
}

/// Helper alias for `Result` objects that return a JSON `Error`.
pub type Result<T> = result::Result<T, Error>;
