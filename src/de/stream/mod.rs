use crate::de::{Deserializer, Read, Result};
use serde::Deserialize;

mod parse_type;
use parse_type::{Array, Map, ParseType, Root};

/// Iterator that deserializes a stream into multiple JSON values.
///
/// A stream deserializer can be created from any JSON deserializer using the
/// `Stream::new` method.
///
/// The data can consist of any JSON value. Values need to be a self-delineating value e.g.
/// arrays, objects, or strings, or be followed by whitespace or a self-delineating value.
///
/// ```
/// use serde_json::{Deserializer, Stream, Value};
/// use std::collections::HashMap;
///
/// fn main() {
///     let data = "{\"k\": 3}1\"cool\"\"stuff\" 3{}  [0, 1, 2]";
///
///     let stream = Stream::new(Deserializer::from_str(data));
///     let mut stream = stream.enter_map().unwrap();
///     assert_eq!(stream.next_value().unwrap(), ("k".to_string(), 3));
///     let mut stream = stream.end_map().unwrap();
///
///     assert_eq!(stream.next_value::<usize>().unwrap(), 1);
///     assert_eq!(stream.next_value::<&str>().unwrap(), "cool");
///     assert_eq!(stream.next_value::<&str>().unwrap(), "stuff");
///     assert_eq!(stream.next_value::<usize>().unwrap(), 3);
///     assert_eq!(stream.next_value::<HashMap<String, Value>>().unwrap(), HashMap::new());
///     assert_eq!(stream.next_value::<Vec<usize>>().unwrap(), vec![0, 1, 2]);
///
///     stream.end().unwrap();
/// }
/// ```
pub struct Stream<'de, R: Read<'de>, P> {
    deserializer: Deserializer<R>,
    parse_type: P,
    lifetime: std::marker::PhantomData<&'de ()>,
}

/// Iterator variant of `Stream`, created by `Stream::iter`
pub struct StreamIterator<'de, 'a, R: Read<'de>, P, T: Deserialize<'de>> {
    stream: &'a mut Stream<'de, R, P>,
    element_type: std::marker::PhantomData<T>,
}

impl<'de, R: Read<'de>, P> Stream<'de, R, P> {
    fn inner_new<PP>(src: Stream<'de, R, PP>) -> Self
    where
        P: ParseType<PP>,
    {
        Stream {
            deserializer: src.deserializer,
            parse_type: P::new(src.parse_type),
            lifetime: std::marker::PhantomData::default(),
        }
    }

    fn unwrap<PP>(self) -> Stream<'de, R, PP>
    where
        P: ParseType<PP>,
    {
        Stream {
            deserializer: self.deserializer,
            parse_type: self.parse_type.unwrap(),
            lifetime: std::marker::PhantomData::default(),
        }
    }
}

/// Root implementation of the `Stream` parser
impl<'de, R: Read<'de>> Stream<'de, R, Root> {
    /// Create a JSON stream deserializer from a Deserializer
    pub fn new(deserializer: Deserializer<R>) -> Self {
        Stream {
            deserializer,
            parse_type: Root::new(()),
            lifetime: std::marker::PhantomData::default(),
        }
    }

    /// Enter an array
    pub fn enter_array(mut self) -> Result<Stream<'de, R, Array<Root>>> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));
        tri!(self.parse_type.enter_array(&mut self.deserializer));
        Ok(Stream::inner_new(self))
    }

    /// Enter a map
    pub fn enter_map(mut self) -> Result<Stream<'de, R, Map<Root>>> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));
        tri!(self.parse_type.enter_map(&mut self.deserializer));
        Ok(Stream::inner_new(self))
    }

    /// Return the next value, the whole value will be parsed in memory, loosing the streaming feature
    pub fn next_value<K: Deserialize<'de>>(&mut self) -> Result<K> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));
        K::deserialize(&mut self.deserializer)
    }

    /// Create a single typed iterator
    pub fn iter<'a, T: Deserialize<'de>>(&'a mut self) -> StreamIterator<'de, 'a, R, Root, T> {
        StreamIterator {
            stream: self,
            element_type: std::marker::PhantomData::default(),
        }
    }

    /// Expect iterator to be at the end, should be called to avoid trailing characters
    pub fn end(mut self) -> Result<()> {
        self.deserializer.end()
    }
}

impl<'de, R: Read<'de>, P> Stream<'de, R, Array<P>> {
    /// Enter an array
    pub fn enter_array(mut self) -> Result<Stream<'de, R, Array<Array<P>>>> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));
        tri!(self.parse_type.enter_array(&mut self.deserializer));
        Ok(Stream::inner_new(self))
    }

    /// Enter an map
    pub fn enter_map(mut self) -> Result<Stream<'de, R, Map<Array<P>>>> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));
        tri!(self.parse_type.enter_map(&mut self.deserializer));
        Ok(Stream::inner_new(self))
    }

    /// Return the next value, the whole value will be parsed in memory, loosing the streaming feature
    pub fn next_value<K: Deserialize<'de>>(&mut self) -> Result<K> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));

        K::deserialize(&mut self.deserializer)
    }

    /// Test if we can end the current array
    pub fn can_end_array(&mut self) -> bool {
        self.deserializer.parse_whitespace()
            .unwrap_or_default() == Some(b']')
    }

    /// Leave the current array
    pub fn end_array(mut self) -> Result<Stream<'de, R, P>> {
        self.deserializer.end_seq()?;
        Ok(self.unwrap())
    }

    /// Create a single typed iterator
    pub fn iter<'a, T: Deserialize<'de>>(&'a mut self) -> StreamIterator<'de, 'a, R, Array<P>, T> {
        StreamIterator {
            stream: self,
            element_type: std::marker::PhantomData::default(),
        }
    }
}

impl<'de, R: Read<'de>, P> Stream<'de, R, Map<P>> {
    /// Enter an array
    pub fn enter_array(mut self) -> Result<(String, Stream<'de, R, Array<Map<P>>>)> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));

        let key = tri!(String::deserialize(&mut self.deserializer));

        tri!(self.deserializer.parse_object_colon());
        tri!(self.parse_type.enter_array(&mut self.deserializer));

        Ok((key, Stream::inner_new(self)))
    }

    /// Enter an map
    pub fn enter_map(mut self) -> Result<(String, Stream<'de, R, Map<Map<P>>>)> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));

        let key = tri!(String::deserialize(&mut self.deserializer));

        tri!(self.deserializer.parse_object_colon());
        tri!(self.parse_type.enter_map(&mut self.deserializer));

        Ok((key, Stream::inner_new(self)))
    }

    /// Return the next value, the whole value will be parsed in memory, loosing the streaming feature
    pub fn next_value<K: Deserialize<'de>>(&mut self) -> Result<(String, K)> {
        tri!(self.parse_type.parse_separator(&mut self.deserializer));

        let key = tri!(String::deserialize(&mut self.deserializer));

        tri!(self.deserializer.parse_object_colon());
        let val = tri!(K::deserialize(&mut self.deserializer));

        Ok((key, val))
    }

    /// Test if we can end the current map
    pub fn can_end_map(&mut self) -> bool {
        self.deserializer.parse_whitespace()
            .unwrap_or_default() == Some(b'}')
    }

    /// Leave the current map
    pub fn end_map(mut self) -> Result<Stream<'de, R, P>> {
        tri!(self.deserializer.end_map());
        Ok(self.unwrap())
    }

    /// Create a single typed iterator
    pub fn iter<'a, T: Deserialize<'de>>(&'a mut self) -> StreamIterator<'de, 'a, R, Map<P>, T> {
        StreamIterator {
            stream: self,
            element_type: std::marker::PhantomData::default(),
        }
    }
}

impl<'de, 'a, R: Read<'de>, T: Deserialize<'de>> Iterator for StreamIterator<'de, 'a, R, Root, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next_value() {
            Ok(v) => Some(Ok(v)),
            Err(err) if err.is_eof() => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'de, 'a, R: Read<'de>, P, T: Deserialize<'de>> Iterator
    for StreamIterator<'de, 'a, R, Array<P>, T>
{
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next_value() {
            Ok(v) => Some(Ok(v)),
            Err(_) if self.stream.can_end_array() => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'de, 'a, R: Read<'de>, P, T: Deserialize<'de>> Iterator
    for StreamIterator<'de, 'a, R, Map<P>, T>
{
    type Item = Result<(String, T)>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next_value() {
            Ok(v) => Some(Ok(v)),
            Err(_) if self.stream.can_end_map() => None,
            Err(err) => Some(Err(err)),
        }
    }
}
