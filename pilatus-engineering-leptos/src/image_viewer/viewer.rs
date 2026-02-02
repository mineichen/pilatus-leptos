use egui_pixels::Tools;
use futures::StreamExt;
use leptos::html::Canvas;
use leptos::prelude::*;
use pilatus::device::DeviceId;
use thaw::Button;

use super::{ImageProvider, app::EframeImageViewer};

#[component]
pub fn ImageViewerComponent<T>(
    url: Signal<Option<String>>,
    #[allow(unused_variables)] provider: T,
    #[prop(optional)] list_all_but: Option<Signal<Option<DeviceId>>>,
    #[prop(optional)] set_url: Option<SignalSetter<String>>,
    #[prop(optional)] mut tools: Option<Tools>,
    #[prop(optional)] primary: Option<Signal<String>>,
) -> impl IntoView
where
    T: ImageProvider,
{
    let canvas_ref = NodeRef::<Canvas>::new();
    leptos::logging::log!("Enter websocket viewer");
    let (viewer, set_viewer) = signal_local(None::<EframeImageViewer>);
    let (canvas_class, set_canvas_class) = signal(None as Option<&'static str>);

    let available = LocalResource::new(move || async move {
        let Some(maybe_ignored_device_id) = list_all_but else {
            return Ok(Vec::new());
        };
        let maybe_ignore_device_id = maybe_ignored_device_id
            .try_get()
            .and_then(|x| Some(x?.to_string()));

        let available = T::list_sources(maybe_ignore_device_id).await?;
        leptos::logging::log!(
            "Possibilities: {available:?}, ignore: {:?}",
            maybe_ignored_device_id
        );
        if let Some(set_url) = set_url
            && url.read_untracked().is_none()
            && !available.is_empty()
        {
            set_url.set(available[0].clone());
        }
        anyhow::Ok(available)
    });

    // This effect is expected to run only once to create the viewer
    Effect::new(move |_| {
        leptos::logging::log!("Before read canvas");
        if let Some(canvas) = canvas_ref.get()
            && viewer.read_untracked().is_none()
        {
            let tools = tools.take().unwrap_or_default();
            leptos::reactive::spawn_local(async move {
                match EframeImageViewer::create(canvas, tools).await {
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
    if let Some(primary) = primary {
        Effect::new(move || {
            if let Some(viewer_ref) = viewer.read().as_ref() {
                viewer_ref.set_primary(primary.get());
            }
        });
    }
    // Start the image stream processing when URL changes
    let _image_acquisition = LocalResource::new(move || async move {
        let ws_url = url.get();
        let ws_url = match ws_url {
            Some(x) => x,
            None => match &*available.read() {
                Some(Ok(x)) if !x.is_empty() => x[0].clone(),
                _ => return,
            },
        };

        let mut stream = T::image_stream(ws_url);

        #[cfg(target_arch = "wasm32")]
        let mut last = js_sys::Date::now();

        while let Some(result) = stream.next().await {
            let (image, masks) = match result {
                Ok(image) => image,
                Err(e) => {
                    leptos::logging::error!("Error receiving image: {}", e);
                    break;
                }
            };

            let guard = viewer.try_read();
            let Some(viewer_ref) = guard.as_deref().and_then(|v| v.as_ref()) else {
                leptos::logging::debug_log!("Viewer is not ready to display images");

                continue;
            };

            #[cfg(target_arch = "wasm32")]
            {
                let now = js_sys::Date::now();
                leptos::logging::log!("Forward image to viewer {image:?} at {:?}ms", now - last);
                last = now;
            }

            viewer_ref.replace_image(image, masks).await;
        }

        leptos::logging::log!("Image stream closed");
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
    }
}
