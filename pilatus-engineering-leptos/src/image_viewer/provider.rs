use futures_util::Stream;
use imbuf::DynamicImage;
use leptos::prelude::ReadSignal;
use pilatus_engineering::image::{ImageWithMeta, StreamImageError};
use std::pin::Pin;

/// Trait for providing images to the ImageViewer
/// Implementations handle different image acquisition strategies
// pub type ImageProviderStreamItem =
//     anyhow::Result<(ImageWithMeta<Image<[u8; 3], 1>>, Vec<PixelArea>)>;
pub type ImageProviderStreamItem =
    anyhow::Result<Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>>;
pub trait ImageProvider: 'static {
    fn image_stream(
        &mut self,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>>;
    fn list_sources(ignore: Option<String>) -> impl Future<Output = anyhow::Result<Vec<String>>>;
    fn error(&self) -> ReadSignal<Result<(), String>>;
}
