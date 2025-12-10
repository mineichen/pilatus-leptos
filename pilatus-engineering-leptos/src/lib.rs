use std::ops::Deref;

use futures::StreamExt;
use gloo_net::websocket::Message;
use leptos::prelude::*;
use leptos::{html::Canvas, logging};
use pilatus::Name;
use pilatus_emulation_camera::ActiveRecipeImpex;
use pilatus_leptos::{DeviceContext, JsonDeviceView, PilatusWrapperSettings};
use wasm_bindgen::JsCast;

mod decode;
mod image_viewer;
mod ws_suspend;

#[component]
pub fn PilatusEngineeringView() -> impl IntoView {
    let device_ctx = expect_context::<DeviceContext>();
    let device_id = Signal::derive(move || device_ctx.infos.read().device_id);

    let collections = LocalResource::new(move || async move {
        let id = device_id.get();
        let url = format!("/api/engineering/emulation-camera/collection?device_id={id}");
        gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Name>>()
            .await
            .map_err(|e| e.to_string())
    });

    let (show_new_collection_dialog, set_show_new_collection_dialog) = signal(false);
    let (pending_file, set_pending_file) = signal_local::<Option<web_sys::File>>(None);
    let (new_collection_name, set_new_collection_name) = signal(String::new());
    let (is_creating_new_collection, set_is_creating_new_collection) = signal(false);

    let params_impex =
        device_ctx.get::<pilatus_emulation_camera::ParamsImpex<PilatusWrapperSettings>>();

    let file = params_impex.map(|x| x.file.clone(), |x, file| x.file = file);
    let file_active = file.map(|x| x.active.clone(), |x, active| x.active = active);
    let active_collection = file_active.map(
        |x| {
            if let ActiveRecipeImpex::Named(n) = x {
                Some(n.to_value())
            } else {
                None
            }
        },
        |x, name| match (x, name) {
            (ActiveRecipeImpex::Named(cur), Some(new_x)) => {
                cur.set_explicit(new_x);
            }
            _ => logging::debug_error!("Setting expected to work here"),
        },
    );

    let upload_action =
        Action::new_local(move |(collection_name, file): &(Name, web_sys::File)| {
            let collection_name = collection_name.clone();
            let file = file.clone();
            async move {
                let file_name = file.name();

                // Remove file extension for the endpoint
                let image_name = file_name
                    .rsplit_once('.')
                    .map(|(name, _)| name)
                    .unwrap_or(&file_name);

                let id = device_id.get_untracked();
                let url = format!(
                    "/api/engineering/emulation-camera/collection/{}/{}?device_id={}",
                    collection_name, image_name, id
                );

                upload_file(&url, file).await.map(|_| file_name.clone())
            }
        });

    // Effect to refresh collections after successful upload to a new collection
    Effect::new(move |_| {
        if let Some(Ok(_)) = upload_action.value().get() {
            if is_creating_new_collection.get() {
                collections.refetch();
                set_is_creating_new_collection.set(false);
            }
        }
    });

    let delete_action = Action::new_local(move |collection_name: &Name| {
        let collection_name = collection_name.clone();
        async move {
            let id = device_id.get_untracked();
            let url = format!(
                "/api/engineering/emulation-camera/collection/{}?device_id={}",
                collection_name, id
            );

            gloo_net::http::Request::delete(&url)
                .send()
                .await
                .map_err(|e| format!("Delete failed: {:?}", e))?;

            Ok::<(), String>(())
        }
    });

    // Effect to refresh collections after successful deletion
    Effect::new(move |_| {
        if let Some(Ok(_)) = delete_action.value().get() {
            collections.refetch();
        }
    });

    let confirm_new_collection = move || {
        let name_str = new_collection_name.get_untracked();
        if !name_str.is_empty() {
            if let Ok(collection_name) = name_str.parse::<Name>() {
                if let Some(file) = pending_file.get_untracked() {
                    set_is_creating_new_collection.set(true);
                    upload_action.dispatch_local((collection_name, file));
                }
            }
        }
        set_show_new_collection_dialog.set(false);
        set_new_collection_name.set(String::new());
        set_pending_file.set(None);
    };

    let cancel_new_collection = move || {
        set_show_new_collection_dialog.set(false);
        set_new_collection_name.set(String::new());
        set_pending_file.set(None);
    };

    let canvas_ref = NodeRef::<Canvas>::new();
    let (viewer, set_viewer) = signal_local::<Option<image_viewer::EframeImageViewer>>(None);
    Effect::new(move |_| {
        leptos::logging::log!("Before read canvas");
        if let Some(canvas) = canvas_ref.get()
            && viewer.read_untracked().is_none()
        {
            leptos::reactive::spawn_local(async move {
                let canvas_element: web_sys::HtmlCanvasElement = canvas.into();
                match image_viewer::EframeImageViewer::create(canvas_element).await {
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

    let stream = LocalResource::new(move || async move {
        let ws_url = format!("ws://localhost:4123/api/image/subscribe?format=Raw");
        // let ws_url = format!("ws://localhost:8080/api/image/subscribe?device_id={id}&format=Raw");

        let mut ws = ws_suspend::SuspensibleWebSocket::new(ws_url)?;
        leptos::logging::debug_log!("Suspensible WebSocket created");

        let mut last = now_millis();
        while let Some(message_result) = ws.next().await {
            let bytes = match message_result {
                Ok(Message::Bytes(bytes)) => bytes,
                Ok(_other) => {
                    // Ignore unexpected message types for this viewer.
                    continue;
                }
                Err(ws_suspend::SuspensibleError::Suspended) => {
                    leptos::logging::log!("Image WebSocket suspended; will reopen once resumed");
                    continue;
                }
                Err(ws_suspend::SuspensibleError::WebSocket(err)) => {
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

            if let Some(image) = decode::parse(&bytes)? {
                viewer.replace_image(image).await;
            } else {
                leptos::logging::log!("No image in this frame");
            }
        }

        leptos::logging::log!("WebSocket connection closed");
        anyhow::Ok(())
    });
    view! {
        <div>
            <h1>"Pilatus Engineering with canvas"</h1>
            <div style="display: flex; gap: 20px;">
                <div style="flex: 1;">
                    <canvas
                        node_ref=canvas_ref
                        style="height: 500px; width: 100%; background-color: black;"
                    />
                </div>
                <div style="width: 200px;">
                    <h3>"Collections"</h3>
                    <p style="font-size: 11px; color: #666; margin: 4px 0 8px 0; font-style: italic;">
                        "Drag & drop images here"
                    </p>
                    {move || {
                        upload_action.value().get().map(|result| {
                            match result {
                                Ok(file_name) => view! {
                                    <div style="padding: 8px; margin-bottom: 8px; background: #e8f4f8; color: #333; border-radius: 4px; font-size: 12px;">
                                        "✓ Uploaded " {file_name}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div style="padding: 8px; margin-bottom: 8px; background: #ffe0e0; color: #cc0000; border-radius: 4px; font-size: 12px;">
                                        "✗ Error: " {e}
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                    {move || upload_action.pending().get().then(|| view! {
                        <div style="padding: 8px; margin-bottom: 8px; background: #fff8dc; color: #666; border-radius: 4px; font-size: 12px;">
                            "Uploading..."
                        </div>
                    })}
                    <Suspense fallback=move || view! { <p>"Loading collections..."</p> }>
                        {move || {
                            collections.get().map(|result| {
                                match result {
                                    Ok(names) => view! {
                                        <ul style="list-style: none; padding: 0; margin: 0;">
                                            <For
                                                each=move || names.clone()
                                                key=|name| name.clone()
                                                let(name)
                                            >
                                                {
                                                    let (name, _set_name) = signal(name.clone());

                                                    let is_active = Signal::derive(move || {
                                                        active_collection.get().as_ref() == Some(&name.read().deref())
                                                    });


                                                    view! {
                                                        <li
                                                            style=move || {
                                                                let active = is_active.get();
                                                                format!(
                                                                    "padding: 10px; margin: 4px 0; background: {}; border: 2px {} {}; border-radius: 6px; cursor: pointer; transition: all 0.2s; position: relative;",
                                                                    if active { "#e8f5e9" } else { "#f0f0f0" },
                                                                    if active { "solid" } else { "dashed" },
                                                                    if active { "#4caf50" } else { "#ccc" }
                                                                )
                                                            }
                                                            on:dragover=move |ev| {
                                                                ev.prevent_default();
                                                                if let Some(target) = ev.target() {
                                                                    if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                                        let _ = elem.style().set_property("background", "#d0e8f0");
                                                                        let _ = elem.style().set_property("border-color", "#4a90a4");
                                                                    }
                                                                }
                                                            }
                                                            on:dragleave=move |ev| {
                                                                if let Some(target) = ev.target() {
                                                                    if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                                        let active = is_active.get_untracked();
                                                                        let bg = if active { "#e8f5e9" } else { "#f0f0f0" };
                                                                        let border = if active { "#4caf50" } else { "#ccc" };
                                                                        let _ = elem.style().set_property("background", bg);
                                                                        let _ = elem.style().set_property("border-color", border);
                                                                    }
                                                                }
                                                            }
                                                            on:drop=move |ev| {
                                                                ev.prevent_default();
                                                                if let Some(target) = ev.target() {
                                                                    if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                                        let active = is_active.get_untracked();
                                                                        let bg = if active { "#e8f5e9" } else { "#f0f0f0" };
                                                                        let border = if active { "#4caf50" } else { "#ccc" };
                                                                        let _ = elem.style().set_property("background", bg);
                                                                        let _ = elem.style().set_property("border-color", border);
                                                                    }
                                                                }
                                                                if let Some(dt) = ev.data_transfer() {
                                                                    if let Some(files) = dt.files() {
                                                                        if let Some(file) = files.get(0) {
                                                                            upload_action.dispatch_local((name.get(), file));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        >
                                                            <div style="display: flex; align-items: center; justify-content: space-between;">
                                                                <div style="display: flex; align-items: center; gap: 8px;">
                                                                    {move || {
                                                                        if is_active.get() {
                                                                            view! { <span style="font-size: 18px; color: #4caf50;">"✓"</span> }.into_any()
                                                                        } else {
                                                                            view! { <span style="font-size: 18px; opacity: 0.4;">"📁"</span> }.into_any()
                                                                        }
                                                                    }}
                                                                    <span style="font-weight: 500;">{move|| name.read().to_string() }</span>
                                                                </div>
                                                                <div style="display: flex; align-items: center; gap: 4px;">
                                                                    {move || {
                                                                        (!is_active.get()).then(|| view! {
                                                                            <button
                                                                                on:click=move |ev| {
                                                                                    ev.stop_propagation();
                                                                                    active_collection.set(Some(name.get()));
                                                                                }
                                                                                style="background: #4caf50; border: none; cursor: pointer; padding: 4px 8px; font-size: 11px; color: white; border-radius: 4px; transition: opacity 0.2s;"
                                                                                title="Activate this collection"
                                                                            >
                                                                                "Activate"
                                                                            </button>
                                                                        })
                                                                    }}
                                                                    <button
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            delete_action.dispatch_local(name.get());
                                                                        }
                                                                        style="background: transparent; border: none; cursor: pointer; padding: 4px; font-size: 16px; opacity: 0.5; transition: opacity 0.2s;"
                                                                        title="Delete collection"
                                                                    >
                                                                        "🗑️"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </li>
                                                    }
                                                }
                                            </For>
                                            <li
                                                style="padding: 10px; margin: 4px 0; background: #fff8e1; border: 2px dashed #ffa726; border-radius: 6px; cursor: pointer; transition: all 0.2s; position: relative;"
                                                on:dragover=move |ev| {
                                                    ev.prevent_default();
                                                    if let Some(target) = ev.target() {
                                                        if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                            let _ = elem.style().set_property("background", "#ffe0b2");
                                                            let _ = elem.style().set_property("border-color", "#ff9800");
                                                        }
                                                    }
                                                }
                                                on:dragleave=move |ev| {
                                                    if let Some(target) = ev.target() {
                                                        if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                            let _ = elem.style().set_property("background", "#fff8e1");
                                                            let _ = elem.style().set_property("border-color", "#ffa726");
                                                        }
                                                    }
                                                }
                                                on:drop=move |ev| {
                                                    ev.prevent_default();
                                                    if let Some(target) = ev.target() {
                                                        if let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>() {
                                                            let _ = elem.style().set_property("background", "#fff8e1");
                                                            let _ = elem.style().set_property("border-color", "#ffa726");
                                                        }
                                                    }
                                                    if let Some(dt) = ev.data_transfer() {
                                                        if let Some(files) = dt.files() {
                                                            if let Some(file) = files.get(0) {
                                                                set_pending_file.set(Some(file));
                                                                set_show_new_collection_dialog.set(true);
                                                            }
                                                        }
                                                    }
                                                }
                                            >
                                                <div style="display: flex; align-items: center; justify-content: space-between;">
                                                    <span style="font-weight: 500; color: #f57c00;">"+ Create New Collection"</span>
                                                    <span style="font-size: 18px; opacity: 0.6;">"✨"</span>
                                                </div>
                                            </li>
                                        </ul>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <p style="color: red;">"Error: " {e.to_string()}</p>
                                    }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
            { move || {
                stream.read().as_ref()?.as_ref().err().map(move|e| view! {
                    <div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
                        <p class="font-medium">Error occurred</p>
                        <p class="text-sm mt-1">{e.to_string()}</p>
                    </div>
                })
            } }
            <JsonDeviceView/>
            {move || show_new_collection_dialog.get().then(|| view! {
                <div style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                    <div style="background: white; padding: 24px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); min-width: 300px;">
                        <h3 style="margin-top: 0;">"Create New Collection"</h3>
                        <p style="color: #666; font-size: 14px; margin-bottom: 16px;">"Enter a name for the new collection:"</p>
                        <input
                            type="text"
                            placeholder="Collection name"
                            prop:value=move || new_collection_name.get()
                            on:input=move |ev| set_new_collection_name.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    confirm_new_collection();
                                } else if ev.key() == "Escape" {
                                    cancel_new_collection();
                                }
                            }
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; margin-bottom: 16px; box-sizing: border-box;"
                        />
                        <div style="display: flex; gap: 8px; justify-content: flex-end;">
                            <button
                                on:click=move |_| cancel_new_collection()
                                style="padding: 8px 16px; border: 1px solid #ccc; background: white; border-radius: 4px; cursor: pointer;"
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=move |_| confirm_new_collection()
                                style="padding: 8px 16px; border: none; background: #4a90a4; color: white; border-radius: 4px; cursor: pointer;"
                            >
                                "Create"
                            </button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

fn now_millis() -> f64 {
    js_sys::Date::now()
}

async fn upload_file(url: &str, file: web_sys::File) -> Result<(), String> {
    use js_sys::Uint8Array;
    use wasm_bindgen_futures::JsFuture;

    // Read file as ArrayBuffer
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    let uint8_array = Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    let response = gloo_net::http::Request::post(url)
        .body(bytes)
        .map_err(|e| format!("Request build error: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Upload failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("HTTP {}: {}", status, error_text));
    }

    Ok(())
}
