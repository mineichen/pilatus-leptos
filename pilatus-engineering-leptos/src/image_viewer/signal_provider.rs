use std::pin::Pin;

use futures_util::Stream;
use leptos::prelude::ReadSignal;
use leptos::prelude::*;

use crate::image_viewer::provider::ImageProviderStreamItem;

use super::provider::ImageProvider;

#[non_exhaustive]
pub struct SignalImageProvider {
    recv: Option<futures_channel::mpsc::Receiver<ImageProviderStreamItem>>,
    #[allow(dead_code, reason = "Keep effect alive")]
    effect: Effect<LocalStorage>,
}

impl SignalImageProvider {
    pub fn new(data: Signal<Option<ImageProviderStreamItem>, LocalStorage>) -> Self {
        let (mut send, recv) = futures_channel::mpsc::channel(2);
        let effect = Effect::new(move || {
            leptos::logging::log!("SignalImageProvider effect");
            if let Some(Ok(x)) = &*data.read() {
                if send.try_send(Ok(x.clone())).is_err() {
                    leptos::logging::warn!("Channel is full, cannot send more images");
                }
            }
        });
        leptos::logging::log!("SignalImageProvider::new_single");
        Self {
            recv: Some(recv),
            effect,
        }
    }
}

impl ImageProvider for SignalImageProvider {
    fn image_stream(
        &mut self,
        _url: String,
    ) -> Pin<Box<dyn Stream<Item = ImageProviderStreamItem> + 'static>> {
        Box::pin(self.recv.take().expect("Only called once")) as _
    }

    async fn list_sources(_ignore: Option<String>) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    fn error(&self) -> ReadSignal<Result<(), String>> {
        let (read, _) = signal(Ok(()));
        read
    }
}
