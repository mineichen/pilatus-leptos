use futures::{Stream, StreamExt, TryStreamExt};
use gloo_net::websocket::Message;
use imbuf::Image;
use pilatus::device::DeviceId;
use std::pin::Pin;

use crate::decode::parse;

use super::provider::ImageProvider;

#[derive(Clone)]
pub struct WebSocketImageProvider;

impl ImageProvider for WebSocketImageProvider {
    fn image_stream(
        url: String,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<Image<[u8; 3], 1>>> + 'static>> {
        let state = crate::ws_suspend::SuspensibleWebSocket::new(url).map_err(Some);

        Box::pin(
            futures::stream::unfold(state, |ws_result| async move {
                let mut ws = match ws_result {
                    Ok(ws) => ws,
                    Err(e) => return e.map(|e| (Err(e), Err(None))),
                };
                loop {
                    match ws.next().await {
                        Some(Ok(Message::Bytes(bytes))) => {
                            return Some((parse(&bytes), Ok(ws)));
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
            .try_filter_map(|x| async move { Ok(x) }),
        )
    }

    async fn list_sources(ignored: Option<String>) -> anyhow::Result<Vec<String>> {
        let available = gloo_net::http::Request::get("/api/image/list/subscribe")
            .send()
            .await?
            .json::<Vec<DeviceId>>()
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
}

fn build_device_url(device_id: DeviceId) -> String {
    format!("ws://localhost:4122/api/image/subscribe?format=Raw&device_id={device_id}")
}
