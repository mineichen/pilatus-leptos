use std::rc::Rc;

use futures_util::{FutureExt, StreamExt};
use imanot::Tools;
use imbuf::DynamicImage;
use leptos::html::Canvas;
use leptos::prelude::*;
use pilatus::device::DeviceId;
use pilatus_engineering::image::{ImageWithMeta, StreamImageError};
use wasm_bindgen::JsCast;

use crate::image_viewer::app::ChangeListener;

use super::{ImageProvider, app::EframeImageViewer, app::ViewerHandle};

#[component]
pub fn ImageViewerComponent<T>(
    url: Signal<Option<String>>,
    #[allow(unused_variables)] provider: T,
    #[prop(optional)] list_all_but: Option<Signal<Option<DeviceId>>>,
    #[prop(optional)] set_url: Option<SignalSetter<String>>,
    #[prop(optional)] mut tools: Option<Tools>,
    #[prop(optional)] primary: Option<Signal<String>>,
    #[prop(optional)] mut on_image: Option<
        Box<dyn FnMut(&mut Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>)>,
    >,
    #[prop(optional)] mut tool_change_listener: Option<ChangeListener>,
    #[prop(optional)] mut set_handle: Option<SignalSetter<Option<ViewerHandle>>>,
    #[prop(optional)] active_layer: Option<SignalSetter<Option<usize>>>,
) -> impl IntoView
where
    T: ImageProvider,
{
    let provider_error = provider.error();
    let provider = Rc::new(provider);
    let canvas_ref = NodeRef::<Canvas>::new();
    let on_image = RwSignal::new_local(on_image.take());
    leptos::logging::log!("Enter websocket viewer");
    let (viewer, set_viewer) = signal_local(None::<EframeImageViewer>);

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
            let listener = tool_change_listener.take().unwrap_or_else(|| {
                Box::new(|_masks, layer| {
                    leptos::logging::log!("Change tool stuff on layer {layer:?}");
                    std::future::ready(Ok(())).boxed()
                })
            });
            let set_handle = set_handle.take();
            leptos::reactive::spawn_local(async move {
                match EframeImageViewer::create(canvas, tools, listener, active_layer).await {
                    Ok(viewer) => {
                        leptos::logging::log!("Setting viewer for this instance");
                        if let Some(set_handle) = set_handle {
                            set_handle.set(Some(viewer.handle().clone()));
                        }
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
                viewer_ref.handle().set_primary(primary.get());
            }
        });
    }
    // Start the image stream processing when URL changes
    let _image_acquisition = LocalResource::new(move || {
        let provider = provider.clone();
        async move {
            let ws_url = url.get();
            let ws_url = match ws_url {
                Some(x) => x,
                None => match &*available.read() {
                    Some(Ok(x)) if !x.is_empty() => x[0].clone(),
                    _ => return,
                },
            };

            let mut stream = provider.image_stream(ws_url);

            #[cfg(target_arch = "wasm32")]
            let mut last = js_sys::Date::now();

            while let Some(result) = stream.next().await {
                let mut meta_image = match result {
                    Ok(mut r) => {
                        {
                            on_image.update(|c| {
                                if let Some(c) = c {
                                    (c)(&mut r);
                                }
                            });
                        };
                        match super::super::decode::into_rgb(r) {
                            Ok(Ok(image)) => image,

                            Ok(Err(e)) => {
                                leptos::logging::warn!("Backend error: {e}");
                                break;
                            }
                            Err(e) => {
                                leptos::logging::warn!("Invalid protocol: {e}");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("Error receiving image: {}", e);
                        break;
                    }
                };
                let image = meta_image.image;
                let masks = super::super::decode::extract_from_extensions(
                    &mut meta_image.extensions,
                    128,
                    [0, 0, 255],
                );

                let Some(guard) = viewer.try_read() else {
                    break;
                };
                let Some(viewer_ref) = guard.as_ref() else {
                    leptos::logging::debug_log!("Viewer is not ready to display images");
                    continue;
                };

                #[cfg(target_arch = "wasm32")]
                {
                    let now = js_sys::Date::now();
                    leptos::logging::log!(
                        "Forward image to viewer {image:?} at {:?}ms",
                        now - last
                    );
                    last = now;
                }

                viewer_ref.handle().replace_image(image, masks).await;
            }

            leptos::logging::log!("Image stream closed");
        }
    });

    let (is_fullscreen, set_is_fullscreen) = signal(false);

    Effect::new(move |_| {
        if is_fullscreen.get()
            && let Some(parent) = canvas_ref
                .get()
                .and_then(|c| c.parent_element()?.dyn_into::<web_sys::HtmlElement>().ok())
        {
            parent
                .focus()
                .inspect_err(|_| leptos::logging::error!("Cannot set focus"))
                .ok();
        }
    });

    view! {
        <div
            class=move || if is_fullscreen.get() { "fixed inset-0 z-50 bg-black" } else { "relative" }
            tabindex=0
            on:keydown=move |e: web_sys::KeyboardEvent| {
                if e.key() == "Escape" && is_fullscreen.get_untracked() {
                    set_is_fullscreen.set(false);
                }
            }
        >
            <canvas
                node_ref=canvas_ref
                style=move || if is_fullscreen.get() { "height: 100%; width: 100%; background-color: black;".to_string() } else { "height: 100%; min-height: 500px; width: 100%; background-color: black;".to_string() }
            />
            {move || provider_error.read().as_ref().err().map(|e| {
                view! {
                    <div style="position: absolute; top: 8px; left: 8px; right: 8px; background-color: #dc2626; border: 1px solid #ef4444; border-radius: 0.375rem; padding: 0.5rem 0.75rem; color: white; font-size: 0.875rem;">
                        {format!("Error: {e}")}
                    </div>
                }
            })}
            {move || (!is_fullscreen.get()).then(|| view! {
                <button
                    class="absolute bottom-2 left-2 p-2 text-white/70 hover:text-white bg-black/30 hover:bg-black/50 rounded-lg transition-colors"
                    title="Fullscreen"
                    on:click=move |_| set_is_fullscreen.set(true)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M8 3H5a2 2 0 0 0-2 2v3"/>
                        <path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
                        <path d="M3 16v3a2 2 0 0 0 2 2h3"/>
                        <path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
                    </svg>
                </button>
            })}
            {move || is_fullscreen.get().then(|| view! {
                <button
                    class="absolute top-4 right-4 p-2 text-white/70 hover:text-white bg-black/30 hover:bg-black/50 rounded-lg transition-colors"
                    title="Close"
                    on:click=move |_| set_is_fullscreen.set(false)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M18 6L6 18"/>
                        <path d="M6 6l12 12"/>
                    </svg>
                </button>
            })}
        </div>
    }
}
