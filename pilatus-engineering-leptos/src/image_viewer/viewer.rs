use std::time::Duration;

use anyhow::anyhow;
use futures_util::{FutureExt, TryStreamExt};
use imanot::Tools;
use imbuf::DynamicImage;
use leptos::html::Canvas;
use leptos::prelude::*;
use pilatus::device::DeviceId;
use pilatus_engineering::image::{ImageWithMeta, StreamImageError};
use wasm_bindgen::JsCast;

use crate::image_viewer::app::ChangeListener;

use super::{ImageProvider, OnFrameCallback, app::EframeImageViewer, app::ViewerHandle};

#[component]
pub fn ImageViewerComponent<T>(
    url: Signal<Option<String>>,
    #[allow(unused_variables)] provider: T,
    #[prop(optional)] list_all_but: Option<Signal<Option<DeviceId>>>,
    #[prop(optional)] set_url: Option<SignalSetter<String>>,
    #[prop(optional)] mut tools: Option<Tools>,
    #[prop(optional)] mut on_image: Option<
        Box<dyn FnMut(&mut Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>)>,
    >,
    #[prop(optional)] mut tool_change_listener: Option<ChangeListener>,
    #[prop(optional)] mut set_handle: Option<SignalSetter<Option<ViewerHandle>, LocalStorage>>,
    #[prop(optional)] active_layer: Option<SignalSetter<Option<usize>>>,
    #[prop(optional)] mut on_frame: Option<OnFrameCallback>,
) -> impl IntoView
where
    T: ImageProvider,
{
    let provider_error = provider.error();
    let provider = RwSignal::new_local(provider);
    let canvas_ref = NodeRef::<Canvas>::new();
    let on_image = RwSignal::new_local(on_image.take());
    leptos::logging::log!("Enter websocket viewer");
    let (viewer, set_viewer) = signal_local(None::<EframeImageViewer>);

    let await_viewer = move |v: ReadSignal<Option<EframeImageViewer>, LocalStorage>| async move {
        for _ in 0..10 {
            let Some(guard) = v.try_read() else {
                return Err(anyhow!("Can no longer wait for Viewer"));
            };
            match guard.as_ref() {
                Some(x) => return Ok(Some(x.handle().clone())),
                None => {
                    let retry_in = Duration::from_millis(100);
                    leptos::logging::log!("Not yet ready, waiting for {retry_in:?}");
                    drop(guard);
                    gloo_timers::future::sleep(retry_in).await;
                }
            }
        }
        Ok(None)
    };

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
            let on_frame = on_frame.take();
            leptos::reactive::spawn_local(async move {
                match EframeImageViewer::create(canvas, tools, listener, active_layer, on_frame)
                    .await
                {
                    Ok(viewer) => {
                        leptos::logging::log!("Setting viewer for this instance");
                        if let Some(set_handle) = set_handle {
                            set_handle.set(Some(viewer.handle().clone()));
                        }
                        if set_viewer.try_set(Some(viewer)).is_some() {
                            leptos::logging::error!("Setting viewer failed.");
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("eframe start() returned error: {e:?}");
                    }
                }
            });
        }
    });

    // Start the image stream processing when URL changes
    let image_acquisition = LocalResource::new(move || async move {
        let ws_url = url.get();
        let ws_url = match ws_url {
            Some(x) => x,
            None => match &*available.read() {
                Some(Ok(x)) if !x.is_empty() => x[0].clone(),
                _ => anyhow::bail!("No url provided and none available"),
            },
        };

        let (strategy, mut stream) = provider
            .try_update(|x| (x.history_strategy(), x.image_stream(ws_url)))
            .ok_or_else(|| anyhow::anyhow!("Can't extract image stream"))?;

        #[cfg(target_arch = "wasm32")]
        let mut last = js_sys::Date::now();

        while let Some(mut r) = stream.try_next().await? {
            #[cfg(target_arch = "wasm32")]
            let mut processing_start = js_sys::Date::now();
            on_image.update(|c| {
                c.as_mut().map(|x| (x)(&mut r));
            });
            let mut meta_image = super::super::decode::into_rgb(r)?.or_else(|e| match e {
                StreamImageError::ProcessingError { image, .. } => {
                    Ok(ImageWithMeta::with_hash(image, None))
                }
                e => Err(e),
            })?;
            let image = meta_image.image;
            let masks = super::super::decode::extract_from_extensions(
                &mut meta_image.extensions,
                [0, 0, 255, 128],
            );
            let stack = imanot::PixelAreaStack::from_iter(masks);

            let Some(viewer_ref) = await_viewer(viewer).await? else {
                leptos::logging::debug_log!("Viewer is not ready to display images");
                continue;
            };

            #[cfg(target_arch = "wasm32")]
            {
                let now = js_sys::Date::now();
                leptos::logging::log!(
                    "Forward next image to viewer {image:?} after {:?}ms (History: {strategy:?}, processing_time: {:?}ms)",
                    now - last,
                    now - processing_start
                );
                last = now;
            }

            viewer_ref.replace_image(image, stack, strategy).await;
        }

        anyhow::Ok(())
    });
    Effect::new(move || {
        let lock = image_acquisition.read();
        if let Some(Err(e)) = &*lock {
            leptos::logging::log!("Image stream error: {e:?}");
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
