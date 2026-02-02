use futures::Stream;
use std::pin::Pin;

use crate::{decode::parse, image_viewer::provider::ImageProviderStreamItem};

use super::provider::ImageProvider;

#[derive(Clone)]
pub struct SingleImageProvider;

impl ImageProvider for SingleImageProvider {
    fn image_stream(url: String) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        Box::pin(futures::stream::once(async move {
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
                Some(img) => Ok((img, Vec::new())),
                None => Err(anyhow::anyhow!(
                    "HTTP-Response doesn't return skipped frames"
                )),
            }
        }))
    }

    async fn list_sources(_ignore: Option<String>) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }
}
