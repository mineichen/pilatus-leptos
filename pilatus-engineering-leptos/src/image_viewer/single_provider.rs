use futures_util::{Stream, stream};
use leptos::prelude::{ReadSignal, RwSignal};
use std::pin::Pin;

use crate::{decode::parse, image_viewer::provider::ImageProviderStreamItem};

use super::provider::ImageProvider;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SingleImageProvider {
    error: RwSignal<Result<(), String>>,
}

impl Default for SingleImageProvider {
    fn default() -> Self {
        Self {
            error: RwSignal::new(Ok(())),
        }
    }
}

impl ImageProvider for SingleImageProvider {
    fn image_stream(
        &self,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        Box::pin(stream::once(async move {
            leptos::logging::log!("Fetching image from: {}", url);

            let response = gloo_net::http::Request::get(&url)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

            let bytes = response
                .binary()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

            leptos::logging::log!("Received {} bytes from HTTP", bytes.len());

            match parse(&bytes)? {
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
