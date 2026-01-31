use futures::Stream;
use std::pin::Pin;

/// Trait for providing images to the ImageViewer
/// Implementations handle different image acquisition strategies
pub trait ImageProvider: 'static {
    /// Returns a stream of image bytes
    /// Each item in the stream is a Result containing the raw image bytes
    fn image_stream(url: String) -> Pin<Box<dyn Stream<Item = anyhow::Result<Vec<u8>>> + 'static>>;
    fn list_sources(ignore: Option<String>) -> impl Future<Output = anyhow::Result<Vec<String>>>;
}
