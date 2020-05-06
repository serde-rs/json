//! Represent the structure the `Stream` parser is in

use crate::de::{Deserializer, ErrorCode, Read, Result};

/// Type of the Currently parsed element
///
/// This is a recursive trait, where P is the parent type.
///
/// Example `Array<Map<Map<Root>>>` would be an array inside two nested map :
///```json
/// {
///     "a": {
///         "b": [
///             // We are here
///         ]
///     }
/// }
///```
pub trait ParseType<P> {
    /// Wrap the type `P`, and retrun a new `ParserType`
    fn new(p: P) -> Self;

    /// Unwrap self, and return the inner value
    fn unwrap(self) -> P;

    /// Parse the separator for this kind of element
    ///
    /// Can be `,` or `]` for `Array`, `,` or `}` for `Map`, or just space for `Root`
    fn parse_separator<'de, R: Read<'de>>(
        &mut self,
        deserializer: &mut Deserializer<R>,
    ) -> Result<Option<u8>>;

    /// Enter in an array, parse the '['
    fn enter_array<'de, R: Read<'de>>(&mut self, deserializer: &mut Deserializer<R>) -> Result<()> {
        match deserializer.parse_whitespace()? {
            Some(b'[') => Ok(deserializer.eat_char()),
            Some(_) => Err(deserializer.peek_invalid_type(&"array")),
            None => Err(deserializer.peek_error(ErrorCode::EofWhileParsingValue)),
        }
    }

    /// Enter in the map, parse the `{`
    fn enter_map<'de, R: Read<'de>>(&mut self, deserializer: &mut Deserializer<R>) -> Result<()> {
        match deserializer.parse_whitespace()? {
            Some(b'{') => Ok(deserializer.eat_char()),
            Some(_) => Err(deserializer.peek_invalid_type(&"map")),
            None => Err(deserializer.peek_error(ErrorCode::EofWhileParsingValue)),
        }
    }
}

/// Root a the Stream, don't have a parent (use ()). Separator in only empty space, allowing to parse `1 2 3 4`
pub struct Root;
impl ParseType<()> for Root {
    fn new(_: ()) -> Self {
        Root {}
    }
    fn unwrap(self) {}
    fn parse_separator<'de, R: Read<'de>>(
        &mut self,
        deserializer: &mut Deserializer<R>,
    ) -> Result<Option<u8>> {
        deserializer.parse_whitespace()
    }
}

/// Represent an array structure. Separator is comma `,` and end is `]`. It will parse `1, 2, 3, 4`
pub struct Array<P> {
    parent: P,
    first: bool,
}

impl<P> ParseType<P> for Array<P> {
    fn new(parent: P) -> Self {
        Array {
            parent,
            first: true,
        }
    }
    fn unwrap(self) -> P {
        self.parent
    }

    fn parse_separator<'de, R: Read<'de>>(
        &mut self,
        deserializer: &mut Deserializer<R>,
    ) -> Result<Option<u8>> {
        match tri!(deserializer.parse_whitespace()) {
            Some(b']') => Err(deserializer.peek_error(ErrorCode::ExpectedSomeValue)),
            Some(b',') if !self.first => {
                deserializer.eat_char();
                deserializer.parse_whitespace()
            }
            Some(b) if self.first => {
                self.first = false;
                Ok(Some(b))
            }
            Some(_) => Err(deserializer.peek_error(ErrorCode::ExpectedListCommaOrEnd)),
            None => Ok(None),
        }
    }
}

/// Represent an object structure. Separator is comma `,` and end is `}`. Expect key-value elements. It will parse `"key": 1, "key2": 2`
pub struct Map<P> {
    parent: P,
    first: bool,
}

impl<P> ParseType<P> for Map<P> {
    fn new(parent: P) -> Self {
        Map {
            parent,
            first: true,
        }
    }
    fn unwrap(self) -> P {
        self.parent
    }

    fn parse_separator<'de, R: Read<'de>>(
        &mut self,
        deserializer: &mut Deserializer<R>,
    ) -> Result<Option<u8>> {
        match tri!(deserializer.parse_whitespace()) {
            Some(b'}') => Err(deserializer.peek_error(ErrorCode::ExpectedSomeValue)),
            Some(b',') if !self.first => {
                deserializer.eat_char();
                deserializer.parse_whitespace()
            }
            Some(b) if self.first => {
                self.first = false;
                Ok(Some(b))
            }
            Some(_) => Err(deserializer.peek_error(ErrorCode::ExpectedObjectCommaOrEnd)),
            None => Ok(None),
        }
    }
}
