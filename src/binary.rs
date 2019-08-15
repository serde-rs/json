/// Specifies the way to encode serde "bytes" type into JSON.
pub enum BinaryMode {
    /// Encodes serde "bytes" type as an array of numbers.
    /// This is the original behavior of `serde_json`, and it is not recommended if you have binary
    /// data: you could encode, but then you may not decode if the data type does not explicitly support
    /// deserializing from a "sequence of numbers".
    Array,

    /// Encodes serde "bytes" type as hex-encoded string.
    #[cfg(feature = "binary_hex")]
    Hex,
}
