use super::write::{write_all, AsyncWrite};
use crate::error::Result;
use serde::Serialize;

/// Serializes `value` as JSON into an async writer.
///
/// Serializes to an in-memory [`Vec<u8>`] via [`crate::to_vec`], then drains
/// it to `writer`. This bounds memory usage to the size of one serialized value.
pub async fn to_async_writer<T: ?Sized + Serialize, W: AsyncWrite + Unpin>(
    writer: W,
    value: &T,
) -> Result<()> {
    let bytes = crate::to_vec(value)?;
    write_all(writer, &bytes).await.map_err(crate::Error::io)
}

/// Extension trait that adds [`to_async_writer`] to any serializable type.
#[allow(async_fn_in_trait)]
pub trait AsyncSerializer {
    /// Serializes `self` as JSON into an async writer.
    async fn to_async_writer<W: AsyncWrite + Unpin>(&self, writer: W) -> Result<()>;
}

impl<T: Serialize + ?Sized> AsyncSerializer for T {
    async fn to_async_writer<W: AsyncWrite + Unpin>(&self, writer: W) -> Result<()> {
        self::to_async_writer(writer, self).await
    }
}
