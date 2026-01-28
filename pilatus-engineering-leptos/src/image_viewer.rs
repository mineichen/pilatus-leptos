use futures::StreamExt;
use gloo_net::websocket::Message;
use leptos::html::Canvas;
use leptos::prelude::*;
use pilatus::device::DeviceId;

mod app;
mod single;

pub use app::EframeImageViewer;
pub use single::SingleImageViewerComponent;
use thaw::Button;

#[component]
pub fn ImageViewerComponent(
    url: Signal<Option<String>>,
    #[prop(optional)] list_all_but: Option<Signal<Option<DeviceId>>>,
    #[prop(optional)] set_url: Option<SignalSetter<String>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    leptos::logging::log!("Enter viewer");
    let (viewer, set_viewer) = signal_local::<Option<EframeImageViewer>>(None);
    let (canvas_class, set_canvas_class) = signal(None as Option<&'static str>);

    let available = LocalResource::new(move || async move {
        let Some(maybe_ignored_device_id) = list_all_but else {
            return Ok(Vec::new());
        };
        let mut device_ids: Vec<DeviceId> =
            gloo_net::http::Request::get("/api/image/list/subscribe")
                .send()
                .await?
                .json()
                .await?;
        let maybe_ignore_device_id = maybe_ignored_device_id.try_get().and_then(|x| x);
        leptos::logging::log!(
            "Possibilities: {device_ids:?}, ignore: {:?}",
            maybe_ignored_device_id
        );

        if let Some(ignore_device_id) = maybe_ignore_device_id {
            device_ids.retain(|x| x != &ignore_device_id);
        }
        let available = device_ids.iter().map(build_device_url).collect::<Vec<_>>();
        if let Some(set_url) = set_url
            && url.read_untracked().is_none()
            && !available.is_empty()
        {
            set_url.set(available[0].clone());
        }
        anyhow::Ok(available)
    });
    Effect::new(move |_| {
        leptos::logging::log!("Before read canvas");
        if let Some(canvas) = canvas_ref.get()
            && viewer.read_untracked().is_none()
        {
            leptos::reactive::spawn_local(async move {
                match EframeImageViewer::create(canvas).await {
                    Ok(viewer) => {
                        leptos::logging::log!("Setting viewer for this instance");
                        set_viewer.set(Some(viewer));
                    }
                    Err(e) => {
                        leptos::logging::error!("eframe start() returned error: {e:?}");
                    }
                }
            });
        }
    });

    let stream = LocalResource::new(move || {
        async move {
            let ws_url = url.get();
            let ws_url = match ws_url {
                Some(x) => x,
                None => match &*available.read() {
                    Some(Ok(x)) if !x.is_empty() => x[0].clone(),
                    Some(Err(e)) => return Err(anyhow::anyhow!("{e}")),
                    _ => return Ok(()),
                },
            };

            let mut ws = crate::ws_suspend::SuspensibleWebSocket::new(ws_url)?;
            leptos::logging::debug_log!("Suspensible WebSocket created");

            let mut last = now_millis();
            while let Some(message_result) = ws.next().await {
                let bytes = match message_result {
                    Ok(Message::Bytes(bytes)) => bytes,
                    Ok(_other) => {
                        // Ignore unexpected message types for this viewer.
                        continue;
                    }
                    Err(crate::ws_suspend::SuspensibleError::Suspended) => {
                        leptos::logging::log!(
                            "Image WebSocket suspended; will reopen once resumed"
                        );
                        continue;
                    }
                    Err(crate::ws_suspend::SuspensibleError::WebSocket(err)) => {
                        leptos::logging::error!("WebSocket error: {:?}", err);
                        return Err(err);
                    }
                };

                let viewer_opt = viewer.try_read();
                let Some(active) = viewer_opt.as_deref() else {
                    leptos::logging::debug_log!("Viewer state not accessible yet");
                    continue;
                };
                let Some(viewer) = active else {
                    leptos::logging::debug_log!("Viewer is not ready to display images");
                    continue;
                };

                let now = now_millis();
                leptos::logging::log!(
                    "Forward image to viewer {} at {:?}ms",
                    bytes.len(),
                    now - last
                );
                last = now;

                if let Some(image) = crate::decode::parse(&bytes)? {
                    viewer.replace_image(image).await;
                } else {
                    leptos::logging::log!("No image in this frame");
                }
            }

            leptos::logging::log!("WebSocket connection closed");
            anyhow::Ok(())
        }
    });

    view! {
        <canvas
            node_ref=canvas_ref
            style="height: 500px; width: 100%; background-color: black;"
            class=canvas_class
        />
        <Button on_click= move|_| set_canvas_class.set(Some("fullscreen"))>
            "Fullscreen"
        </Button>
        { move || {
            stream.read().as_ref()?.as_ref().err().map(move|e| view! {
                <div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
                    <p class="font-medium">Error occurred</p>
                    <p class="text-sm mt-1">{e.to_string()}</p>
                </div>
            })
        } }
    }
}

fn build_device_url(device_id: &DeviceId) -> String {
    format!("ws://localhost:4123/api/image/subscribe?format=Raw&device_id={device_id}")
}

fn now_millis() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        panic!("now_millis() is only supported on wasm32")
    }
}
