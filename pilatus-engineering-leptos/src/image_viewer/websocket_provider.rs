use std::{pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt, TryStreamExt, stream};
use gloo_net::websocket::Message;
use leptos::prelude::{ReadSignal, RwSignal, Set, use_context};
use pilatus::device::DeviceId;
use pilatus_engineering::image::{MetaImageDecoder, StreamImageError};
use pilatus_leptos::{FetchApi, ws_url_base};

use crate::{decode::parse, image_viewer::provider::ImageProviderStreamItem};

use super::provider::ImageProvider;

#[derive(Clone)]
#[non_exhaustive]
pub struct WebSocketImageProvider {
    pub error: RwSignal<Result<(), String>>,
    decoder: MetaImageDecoder,
}

impl Default for WebSocketImageProvider {
    fn default() -> Self {
        Self {
            error: RwSignal::new(Ok(())),
            decoder: use_context().unwrap_or(MetaImageDecoder::with_extensions(Arc::default())),
        }
    }
}

impl ImageProvider for WebSocketImageProvider {
    fn image_stream(
        &mut self,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        let state = crate::ws_suspend::SuspensibleWebSocket::new(url).map_err(Some);
        let error_signal = self.error;
        let decoder = self.decoder.clone();
        Box::pin(
            stream::unfold(state, move |ws_result| {
                let decoder = decoder.clone();
                async move {
                    let mut ws = match ws_result {
                        Ok(ws) => ws,
                        Err(e) => return e.map(|e| (Err(e), Err(None))),
                    };
                    loop {
                        match ws.next().await {
                            Some(Ok(Message::Bytes(bytes))) => {
                                return Some(match parse(&bytes, &decoder) {
                                    Ok(Ok(i)) => {
                                        error_signal.set(Ok(()));
                                        (Ok(Some(Ok(i))), Ok(ws))
                                    }
                                    #[expect(deprecated)]
                                    Ok(Err(StreamImageError::MissedItems(_))) => continue,
                                    Ok(Err(StreamImageError::ProcessingError { image, error })) => {
                                        leptos::logging::log!("Processing Error");
                                        error_signal.set(Err(error.to_string()));

                                        (
                                            Ok(Some(Err(StreamImageError::ProcessingError {
                                                image,
                                                error,
                                            }))),
                                            Ok(ws),
                                        )
                                    }
                                    Ok(Err(e)) => (Err(e.into()), Ok(ws)),
                                    Err(e) => (Err(e), Ok(ws)),
                                });
                            }
                            Some(Ok(_other)) => {
                                // Ignore unexpected message types, continue loop
                                continue;
                            }
                            Some(Err(crate::ws_suspend::SuspensibleError::Suspended)) => {
                                leptos::logging::log!(
                                    "Image WebSocket suspended; will reopen once resumed"
                                );
                                continue;
                            }
                            Some(Err(crate::ws_suspend::SuspensibleError::WebSocket(err))) => {
                                leptos::logging::error!("WebSocket error: {:?}", err);
                                return Some((Err(err), Err(None)));
                            }
                            None => {
                                leptos::logging::log!("WebSocket connection closed");
                                return None;
                            }
                        }
                    }
                }
            })
            // Remove MissingFrames error
            .try_filter_map(|x| std::future::ready(Ok(x))),
        )
    }

    async fn list_sources(ignored: Option<String>) -> anyhow::Result<Vec<String>> {
        let fetch: FetchApi = leptos::prelude::expect_context();
        let available = fetch
            .get_json_silent::<Vec<DeviceId>>("/api/image/list/subscribe")
            .await?
            .into_iter()
            .map(build_device_url);

        Ok(if let Some(ignore_device_id_str) = ignored {
            available
                .filter(|x| !x.contains(&ignore_device_id_str))
                .collect()
        } else {
            available.collect()
        })
    }
    fn error(&self) -> ReadSignal<Result<(), String>> {
        self.error.read_only()
    }
}

fn build_device_url(device_id: DeviceId) -> String {
    format!(
        "{}/api/image/subscribe?format=Raw&device_id={device_id}",
        ws_url_base()
    )
}
