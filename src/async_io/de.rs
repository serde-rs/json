use super::read::{read_to_end, AsyncRead};
use crate::error::Result;
use serde::de::DeserializeOwned;

/// Deserializes `T` from an async reader.
///
/// Buffers the full stream into memory, then delegates to [`crate::from_slice`].
/// This is the correct approach: serde_json's deserializer is synchronous by
/// design, and the buffering cost is bounded by the size of the input.
pub async fn from_async_reader<T: DeserializeOwned, R: AsyncRead + Unpin>(
    reader: R,
) -> Result<T> {
    let bytes = read_to_end(reader).await.map_err(crate::Error::io)?;
    crate::from_slice(&bytes)
}

/// Extension trait that adds [`from_async_reader`] to any deserializable type.
#[allow(async_fn_in_trait)]
pub trait AsyncDeserializer: Sized {
    /// Deserializes `Self` from an async reader.
    async fn from_async_reader<R: AsyncRead + Unpin>(reader: R) -> Result<Self>;
}

impl<T: DeserializeOwned> AsyncDeserializer for T {
    async fn from_async_reader<R: AsyncRead + Unpin>(reader: R) -> Result<Self> {
        self::from_async_reader(reader).await
    }
}
