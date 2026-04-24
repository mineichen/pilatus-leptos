use std::pin::Pin;

use futures_util::{Stream, StreamExt, TryStreamExt, stream};
use gloo_net::websocket::Message;
use leptos::prelude::{ReadSignal, RwSignal, Set};
use pilatus::device::DeviceId;
use pilatus_engineering::image::StreamImageError;
use pilatus_leptos::{FetchApi, ws_url_base};

use crate::{decode::parse, image_viewer::provider::ImageProviderStreamItem};

use super::provider::ImageProvider;

#[derive(Clone)]
#[non_exhaustive]
pub struct WebSocketImageProvider {
    pub error: RwSignal<Result<(), String>>,
}

impl Default for WebSocketImageProvider {
    fn default() -> Self {
        Self {
            error: RwSignal::new(Ok(())),
        }
    }
}

impl ImageProvider for WebSocketImageProvider {
    fn image_stream(
        &self,
        url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        let state = crate::ws_suspend::SuspensibleWebSocket::new(url).map_err(Some);
        let error_signal = self.error.clone();
        Box::pin(
            stream::unfold(state, move |ws_result| async move {
                let mut ws = match ws_result {
                    Ok(ws) => ws,
                    Err(e) => return e.map(|e| (Err(e), Err(None))),
                };
                loop {
                    match ws.next().await {
                        Some(Ok(Message::Bytes(bytes))) => {
                            return Some(match parse(&bytes) {
                                Ok(Ok(mut i)) => {
                                    let areas = super::super::decode::extract_from_extensions(
                                        &mut i.extensions,
                                        128,
                                        [0, 0, 255],
                                    );
                                    error_signal.set(Ok(()));

                                    (Ok(Some((i.image, areas))), Ok(ws))
                                }
                                #[expect(deprecated)]
                                Ok(Err(StreamImageError::MissedItems(_))) => continue,
                                Ok(Err(StreamImageError::ProcessingError { image, error })) => {
                                    leptos::logging::log!("Processing Error");
                                    error_signal.set(Err(error.to_string()));

                                    (Ok(Some((image, Vec::new()))), Ok(ws))
                                }
                                Ok(Err(e)) => (Err(e.into()), Ok(ws)),
                                Err(e) => (Err(e.into()), Ok(ws)),
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
