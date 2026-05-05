use futures_util::{Stream, stream};
use leptos::prelude::{ReadSignal, RwSignal, use_context};
use pilatus_engineering::image::MetaImageDecoder;
use pilatus_leptos::FetchApi;
use std::{pin::Pin, sync::Arc};

use crate::{decode::parse, image_viewer::provider::ImageProviderStreamItem};

use super::provider::ImageProvider;

#[derive(Clone)]
#[non_exhaustive]
pub struct SingleImageProvider {
    error: RwSignal<Result<(), String>>,
    decoder: MetaImageDecoder,
}

impl Default for SingleImageProvider {
    fn default() -> Self {
        Self {
            error: RwSignal::new(Ok(())),
            decoder: use_context().unwrap_or(MetaImageDecoder::with_extensions(Arc::default())),
        }
    }
}

impl ImageProvider for SingleImageProvider {
    fn image_stream(
        &self,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        let decoder = self.decoder.clone();
        Box::pin(stream::once(async move {
            leptos::logging::log!("Fetching image from: {}", url);

            let fetch: FetchApi = leptos::prelude::expect_context();
            let response = fetch.get_silent(&url).await?;

            let bytes = response
                .binary()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

            leptos::logging::log!("Received {} bytes from HTTP", bytes.len());

            match parse(&bytes, &decoder)? {
                Ok(mut img) => Ok((
                    img.image,
                    super::super::decode::extract_from_extensions(
                        &mut img.extensions,
                        128,
                        [0, 0, 255],
                    ),
                )),
                Err(e) => Err(anyhow::anyhow!(
                    "HTTP-Response returned errornous frame: {e}"
                )),
            }
        }))
    }

    async fn list_sources(_ignore: Option<String>) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    fn error(&self) -> ReadSignal<Result<(), String>> {
        self.error.read_only()
    }
}
