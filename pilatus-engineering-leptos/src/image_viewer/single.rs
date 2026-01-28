use std::ops::Deref;

use leptos::html::Canvas;
use leptos::prelude::*;

pub use crate::image_viewer::app::EframeImageViewer;
use thaw::Button;

#[component]
pub fn SingleImageViewerComponent(url: Signal<Option<String>>) -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    leptos::logging::log!("Enter single image viewer");
    let (viewer, set_viewer) = signal_local::<Option<EframeImageViewer>>(None);
    let (canvas_class, set_canvas_class) = signal(None as Option<&'static str>);

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

    let image_fetch = LocalResource::new(move || async move {
        let Some(fetch_url) = url.get() else {
            leptos::logging::debug_warn!("Should remove image here");
            return Ok(());
        };

        leptos::logging::log!("Fetching image from: {}", fetch_url);

        let response = gloo_net::http::Request::get(&fetch_url).send().await?;

        let bytes = response.binary().await?;

        leptos::logging::log!("Received {} bytes from HTTP", bytes.len());

        let viewer = viewer.read();
        let Some(viewer) = viewer.deref() else {
            leptos::logging::debug_log!("Viewer is not ready to display images");
            return Err(anyhow::anyhow!("Viewer not initialized"));
        };

        if let Some(image) = crate::decode::parse(&bytes)? {
            viewer.replace_image(image).await;
        } else {
            leptos::logging::log!("No valid image data received");
            return Err(anyhow::anyhow!("No valid image data"));
        }

        leptos::logging::log!("Image loaded successfully");
        anyhow::Ok(())
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
            image_fetch.read().as_ref()?.as_ref().err().map(move|e| view! {
                <div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
                    <p class="font-medium">Error occurred</p>
                    <p class="text-sm mt-1">{e.to_string()}</p>
                </div>
            })
        } }
    }
}
