use std::ops::Deref;

use futures_util::TryFutureExt;
use leptos::prelude::*;
use leptos::{either::Either, logging::debug_error};
use pilatus::Name;
use pilatus_emulation_camera::ActiveRecipeImpex;
use pilatus_engineering_leptos::{ImageViewerComponent, WebSocketImageProvider};
use pilatus_leptos::{
    DeviceContext, FetchApi, FetchError, JsonDeviceView, PilatusWrapperSettings, ws_url_base,
};
use thaw::{Button, ButtonAppearance, Input};
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn EmulationCameraView() -> impl IntoView {
    leptos::logging::log!("Create EmulationCameraView");
    let device_ctx: DeviceContext = expect_context();
    let fetch: FetchApi = expect_context();
    let device_id = device_ctx.infos.device_id;

    let collections = LocalResource::new(move || async move {
        leptos::logging::log!("ChangeDeviceId: {device_id}");
        let url = format!("/api/pilatus-emulation-camera/collection?device_id={device_id}");
        fetch.get_json_silent::<Vec<Name>>(&url).await
    });

    let (show_new_collection_dialog, set_show_new_collection_dialog) = signal(false);
    let (pending_file, set_pending_file) = signal_local::<Option<web_sys::File>>(None);
    let new_collection_name = RwSignal::new(String::new());
    let (_is_creating_new_collection, set_is_creating_new_collection) = signal(false);

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
            _ => debug_error!("Setting expected to work here"),
        },
    );

    let active_images = LocalResource::new(move || {
        let collection = active_collection.get();
        async move {
            let Some(collection) = collection else {
                return Ok::<_, FetchError>(Vec::new());
            };
            let url = format!("/api/recipe/file/list/{device_id}/{collection}");
            let files = fetch.get_json_silent::<Vec<String>>(&url).await?;
            let prefix = format!("{collection}/");
            let mut images: Vec<Name> = files
                .iter()
                .filter_map(|path| {
                    let name = path.strip_prefix(&prefix)?.strip_suffix(".png")?;
                    Name::new(name).ok()
                })
                .collect();
            images.sort();
            Ok(images)
        }
    });

    let upload_action =
        Action::new_local(move |(collection_name, file): &(Name, web_sys::File)| {
            let collection_name = collection_name.clone();
            let file = web_sys::File::clone(file);
            async move {
                let file_name = file.name();

                let image_name = file_name
                    .rsplit_once('.')
                    .map(|(name, _)| name)
                    .unwrap_or(&file_name);

                let url = format!(
                    "/api/recipe/file/{device_id}/{collection_name}/{image_name}.png"
                );
                let array_buffer = JsFuture::from(file.array_buffer())
                    .await
                    .map_err(|e| FetchError::Other(format!("Failed to read file: {:?}", e)))?;

                fetch
                    .put_body(&url, array_buffer)
                    .await
                    .map(|_| file_name.clone())
            }
        });

    let delete_action = Action::new_local(move |collection_name: &Name| {
        let url = format!(
            "/api/pilatus-emulation-camera/collection/{}?device_id={}",
            collection_name, device_id
        );
        fetch.delete(&url).map_ok(|_| ())
    });

    let delete_image_action =
        Action::new_local(move |(collection_name, image_name): &(Name, Name)| {
            let url = format!("/api/recipe/file/{device_id}/{collection_name}/{image_name}.png");
            fetch.delete(&url).map_ok(|_| ())
        });

    Effect::new(move |_| {
        if upload_action.value().get().is_some_and(|r| r.is_ok()) {
            collections.refetch();
            active_images.refetch();
        }
    });

    Effect::new(move |_| {
        if delete_action.value().get().is_some_and(|r| r.is_ok()) {
            collections.refetch();
            active_images.refetch();
        }
    });

    Effect::new(move |_| {
        if delete_image_action.value().get().is_some_and(|r| r.is_ok()) {
            active_images.refetch();
        }
    });

    let confirm_new_collection = move || {
        let name_str = new_collection_name.get_untracked();
        if !name_str.is_empty()
            && let Ok(collection_name) = name_str.parse::<Name>()
            && let Some(file) = pending_file.get_untracked()
        {
            set_is_creating_new_collection.set(true);
            upload_action.dispatch_local((collection_name, file));
        }
        set_show_new_collection_dialog.set(false);
        new_collection_name.set(String::new());
        set_pending_file.set(None);
    };

    let cancel_new_collection = move || {
        set_show_new_collection_dialog.set(false);
        new_collection_name.set(String::new());
        set_pending_file.set(None);
    };

    let image_url = Signal::derive(move || {
        Some(format!(
            "{}/api/image/subscribe?format=Raw&device_id={}",
            ws_url_base(),
            device_id
        ))
    });

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-white mb-1">"Emulation Camera"</h1>
                <p class="text-slate-400">"Camera emulation with image collections"</p>
            </div>

            <div class="flex gap-6">
                <div class="flex-[2] min-w-0 bg-slate-800 rounded-xl border border-slate-700 overflow-hidden">
                    <ImageViewerComponent url=image_url provider=WebSocketImageProvider::default()/>
                </div>

                <div class="flex-1 min-w-0 flex flex-col">
                    <div class="bg-slate-800 rounded-xl border border-slate-700 p-4 flex-1 overflow-auto">
                        <h2 class="text-lg font-semibold text-white mb-1">"Collections"</h2>
                        <p class="text-xs text-slate-500 mb-4">"Drag images onto a collection or the drop zone below"</p>

                        {move || {
                            upload_action.value().get().map(|file_name| view! {
                                <ErrorBoundary fallback =|errors| {
                                    errors.get()
                                        .into_iter()
                                        .map(|(_, e)| {
                                            dbg!(&e);

                                            view! {
                                            <div class="px-3 py-2 mb-3 bg-red-900/50 border border-red-700 rounded-lg text-red-300 text-sm">
                                                "✗ " { e.to_string()}
                                            </div>
                                        }})
                                        .collect::<Vec<_>>()
                                }>
                                    <div class="px-3 py-2 mb-3 bg-emerald-900/50 border border-emerald-700 rounded-lg text-emerald-300 text-sm">
                                        "✓ Uploaded " {file_name}
                                    </div>
                                </ErrorBoundary>
                            })
                        }}

                        {move || upload_action.pending().get().then(|| view! {
                            <div class="px-3 py-2 mb-3 bg-amber-900/50 border border-amber-700 rounded-lg text-amber-300 text-sm">
                                "Uploading..."
                            </div>
                        })}

                        <Suspense fallback=move || view! { <p class="text-slate-400">"Loading..."</p> }>
                            {move || {
                                collections.get().map(|result| {
                                    match result {
                                        Err(e) => Either::Left(view! {
                                            <p class="text-red-400">"Error: " {e.to_string()}</p>
                                        }),
                                        Ok(names) => Either::Right(view! {
                                            <div class="space-y-2">
                                                <For
                                                    each=move || names.clone()
                                                    key=|name| name.clone()
                                                    let(name)
                                                >
                                                    {
                                                        let (name, _set_name) = signal(name.clone());

                                                        let is_active = Signal::derive(move || {
                                                            active_collection.read().as_deref() == Some(name.read().deref())
                                                        });

                                                        view! {
                                                            <div
                                                                class=move || {
                                                                    let active = is_active.get();
                                                                    if active {
                                                                        "bg-emerald-900/30 border-2 border-emerald-600 rounded-lg p-3 cursor-pointer transition-colors"
                                                                    } else {
                                                                        "bg-slate-700/50 border-2 border-slate-600 border-dashed rounded-lg p-3 cursor-pointer transition-colors hover:border-slate-500"
                                                                    }
                                                                }
                                                                on:dragover=move |ev| {
                                                                    ev.prevent_default();
                                                                }
                                                                on:drop=move |ev| {
                                                                    ev.prevent_default();
                                                                    if let Some(dt) = ev.data_transfer()
                                                                        && let Some(files) = dt.files()
                                                                        && let Some(file) = files.get(0) {
                                                                        upload_action.dispatch_local((name.get(), file));
                                                                    }
                                                                }
                                                            >
                                                                <div class="flex items-center justify-between">
                                                                    <div class="flex items-center gap-2">
                                                                        {move || {
                                                                            if is_active.get() {
                                                                                view! { <span class="text-emerald-400">"✓"</span> }.into_any()
                                                                            } else {
                                                                                view! { <span class="text-slate-500">"📁"</span> }.into_any()
                                                                            }
                                                                        }}
                                                                        <span class="text-white font-medium">{move || name.read().to_string()}</span>
                                                                    </div>
                                                                    <div class="flex items-center gap-1">
                                                                        {move || {
                                                                            (!is_active.get()).then(|| view! {
                                                                                <Button
                                                                                    appearance=ButtonAppearance::Primary
                                                                                    size=thaw::ButtonSize::Small
                                                                                    on:click=move |ev| {
                                                                                        ev.stop_propagation();
                                                                                        active_collection.set(Some(name.get()));
                                                                                    }
                                                                                >
                                                                                    "Activate"
                                                                                </Button>
                                                                            })
                                                                        }}
                                                                        <button
                                                                            class="text-slate-500 hover:text-red-400 transition-colors p-1"
                                                                            title="Delete"
                                                                            on:click=move |ev| {
                                                                                ev.stop_propagation();
                                                                                delete_action.dispatch_local(name.get());
                                                                            }
                                                                        >
                                                                            "🗑️"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    }
                                                </For>

                                                <div
                                                    class="bg-slate-700/30 border-2 border-dashed border-slate-500 rounded-lg p-4 cursor-pointer transition-colors hover:border-slate-400 hover:bg-slate-700/50 text-center"
                                                    on:dragover=move |ev| {
                                                        ev.prevent_default();
                                                    }
                                                    on:drop=move |ev| {
                                                        ev.prevent_default();
                                                        if let Some(dt) = ev.data_transfer()
                                                            && let Some(files) = dt.files()
                                                            && let Some(file) = files.get(0) {
                                                            set_pending_file.set(Some(file));
                                                            set_show_new_collection_dialog.set(true);
                                                        }
                                                    }
                                                >
                                                    <div class="text-slate-400 text-2xl mb-2">"📥"</div>
                                                    <div class="text-slate-300 font-medium">"Drop image to create collection"</div>
                                                    <div class="text-slate-500 text-xs mt-1">"Drag & drop an image file here"</div>
                                                </div>
                                            </div>
                                        }),
                                    }
                                })
                            }}
                        </Suspense>

                        <hr class="border-slate-700 my-4"/>

                        <div class="flex items-center justify-between gap-2 mb-3">
                            <div class="flex items-center gap-2 min-w-0">
                                <h2 class="text-sm font-semibold text-white uppercase tracking-wider shrink-0">
                                    "Images"
                                </h2>
                                {move || {
                                    active_collection.get().map(|collection| {
                                        let name = collection.to_string();
                                        let tooltip = name.clone();
                                        view! {
                                            <span
                                                class="truncate max-w-[130px] text-[11px] px-2 py-0.5 rounded-full bg-emerald-900/40 border border-emerald-700/50 text-emerald-300"
                                                title=tooltip
                                            >
                                                {name}
                                            </span>
                                        }
                                    })
                                }}
                            </div>
                            {move || {
                                active_images.get().and_then(|_| {
                                    active_collection.get().map(|_| {
                                        let count = active_images
                                            .get()
                                            .and_then(|result| result.ok())
                                            .map_or(0, |images| images.len());
                                        view! {
                                            <span class="text-[11px] text-slate-400 bg-slate-700/50 px-2 py-0.5 rounded-full shrink-0">
                                                {count}
                                            </span>
                                        }
                                    })
                                })
                            }}
                        </div>

                        <Suspense
                            fallback=move || {
                                view! {
                                    <div class="grid grid-cols-4 gap-2">
                                        {(0..8)
                                            .map(|_| {
                                                view! {
                                                    <div class="rounded-lg bg-slate-700/30 border border-slate-700 h-24 animate-pulse"></div>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                }
                            }
                        >
                            {move || {
                                active_images.get().map(|result| {
                                    match result {
                                        Err(e) => Either::Left(view! {
                                            <p class="text-red-400 text-sm">"Error: " {e.to_string()}</p>
                                        }),
                                        Ok(images) => Either::Right({
                                            let active = active_collection.get();
                                            let cards = active.as_ref().map(|active| {
                                                images
                                                    .iter()
                                                    .map(|image| {
                                                        let image_name = image.to_string();
                                                        let alt_text = image_name.clone();
                                                        let tooltip = image_name.clone();
                                                        let thumb_url = format!(
                                                            "/api/recipe/file/{device_id}/{active}/{image}.png"
                                                        );
                                                        let delete_image =
                                                            (active.clone(), image.clone());
                                                        view! {
                                                            <div class="group relative rounded-lg overflow-hidden bg-slate-900/60 border border-slate-700 hover:border-emerald-600/60 transition-colors">
                                                                <img
                                                                    src=thumb_url
                                                                    alt=alt_text
                                                                    loading="lazy"
                                                                    class="w-full h-20 object-cover"
                                                                />
                                                                <div class="flex items-center justify-between gap-1 px-2 py-1.5">
                                                                    <span
                                                                        class="text-xs text-slate-300 truncate"
                                                                        title=tooltip
                                                                    >
                                                                        {image_name}
                                                                    </span>
                                                                    <button
                                                                        class="text-slate-500 hover:text-red-400 opacity-0 group-hover:opacity-100 focus:opacity-100 transition-opacity p-0.5 shrink-0"
                                                                        title="Delete image"
                                                                        on:click=move |_| {
                                                                            delete_image_action.dispatch_local(delete_image.clone());
                                                                        }
                                                                    >
                                                                        "🗑️"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            });

                                            match cards {
                                                Some(cards) if !cards.is_empty() => view! {
                                                    <div class="grid grid-cols-4 gap-2">{cards}</div>
                                                }
                                                .into_any(),
                                                None => view! {
                                                    <div class="border-2 border-dashed border-slate-700 rounded-lg p-4 text-center">
                                                        <div class="text-slate-600 text-xl mb-1">"📁"</div>
                                                        <p class="text-slate-500 text-xs">"Activate a collection to browse its images"</p>
                                                    </div>
                                                }
                                                .into_any(),
                                                Some(_) => view! {
                                                    <div class="border-2 border-dashed border-slate-700 rounded-lg p-4 text-center">
                                                        <div class="text-slate-600 text-xl mb-1">"🖼️"</div>
                                                        <p class="text-slate-500 text-xs">"No images in this collection yet"</p>
                                                    </div>
                                                }
                                                .into_any(),
                                            }
                                        }),
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            <div class="bg-slate-800 rounded-xl border border-slate-700 p-4">
                <h2 class="text-lg font-semibold text-white mb-4">"Device Settings"</h2>
                <JsonDeviceView/>
            </div>

            {move || show_new_collection_dialog.get().then(|| view! {
                <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                    <div class="bg-slate-800 rounded-xl p-6 min-w-[320px] border border-slate-700 shadow-xl">
                        <h3 class="text-lg font-semibold text-white mt-0 mb-4">"Create New Collection"</h3>
                        <Input
                            value=new_collection_name
                            placeholder="Enter collection name"
                        />
                        <div class="flex gap-2 justify-end mt-4">
                            <Button
                                appearance=ButtonAppearance::Subtle
                                on:click=move |_| cancel_new_collection()
                            >
                                "Cancel"
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Primary
                                on:click=move |_| confirm_new_collection()
                            >
                                "Create"
                            </Button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}
