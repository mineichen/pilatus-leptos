mod app;

pub use app::EframeImageViewer;

use futures::StreamExt;
use gloo_net::websocket::Message;
use leptos::html::Canvas;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ImageViewerComponent(url: Signal<String>) -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (viewer, set_viewer) = signal_local::<Option<EframeImageViewer>>(None);

    Effect::new(move |_| {
        leptos::logging::log!("Before read canvas");
        if let Some(canvas) = canvas_ref.get()
            && viewer.read_untracked().is_none()
        {
            leptos::reactive::spawn_local(async move {
                let canvas_element: web_sys::HtmlCanvasElement = canvas.into();
                match EframeImageViewer::create(canvas_element).await {
                    Ok(viewer) => {
                        set_viewer.set(Some(viewer));
                    }
                    Err(e) => {
                        leptos::logging::error!("eframe start() returned error: {e:?}");
                    }
                }
            });
        }
    });

    let stream = LocalResource::new({
        let viewer = viewer.clone();
        move || {
            let ws_url = url.get();
            let viewer = viewer.clone();
            async move {
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
        }
    });

    view! {
        <canvas
            node_ref=canvas_ref
            style="height: 500px; width: 100%; background-color: black;"
        />
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

fn now_millis() -> f64 {
    js_sys::Date::now()
}
