use futures::Stream;
use imbuf::Image;
use std::pin::Pin;

/// Trait for providing images to the ImageViewer
/// Implementations handle different image acquisition strategies
pub trait ImageProvider: 'static {
    fn image_stream(
        url: String,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<Image<[u8; 3], 1>>> + 'static>>;
    fn list_sources(ignore: Option<String>) -> impl Future<Output = anyhow::Result<Vec<String>>>;
}
