use std::ops::Deref;

use anyhow::{Context, anyhow};
use futures_channel::mpsc;
use futures_util::TryFutureExt;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use gloo_net::websocket::Message;
use gloo_net::websocket::futures::WebSocket;
use js_sys::wasm_bindgen::JsCast;
use leptos::prelude::*;
use pilatus::{Name, RecipeId};
use pilatus_leptos::ws_url_base;
use serde::Deserialize;
use thaw::{Button, ButtonAppearance};

#[derive(Deserialize)]
enum ImportServerMessage {
    Success,
    Error(String),
    Conflicts(ImportConflicts),
}

#[derive(Clone, PartialEq, Deserialize)]
struct ImportConflicts {
    recipe_ids: Vec<RecipeId>,
    variable_conflicts: Vec<VariableConflictInfo>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct VariableConflictInfo {
    name: Name,
    existing: serde_json::Value,
    imported: serde_json::Value,
}
#[derive(Clone)]
struct ConflictContext {
    conflicts: ImportConflicts,
    responder: mpsc::Sender<&'static str>,
}

#[component]
pub fn RecipeImport() -> impl IntoView {
    let (conflicts, set_conflicts) = signal(None::<ConflictContext>);

    let import_action = Action::new_local(move |file: &web_sys::File| {
        run_import(file.clone(), set_conflicts).map_err(|e| e.to_string())
    });

    let on_file_input = move |ev: leptos::ev::Event| {
        let target = ev.target().expect("event target");
        let input: web_sys::HtmlInputElement =
            target.dyn_into().expect("Is always a HtmlInputElement");
        let files = input.files().expect("files");
        if let Some(file) = files.get(0) {
            import_action.dispatch(file);
        }
        input.set_value("");
    };

    let dismiss = move || {
        import_action.clear();
    };

    view! {
        <div>
            <label class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-md bg-slate-700 text-slate-300 hover:bg-slate-600 hover:text-white transition-colors cursor-pointer">
                "Import Recipe"
                <input
                    type="file"
                    accept=".pilatusrecipe"
                    class="hidden"
                    disabled=move || import_action.pending().get()
                    on:change=on_file_input
                />
            </label>

            {move || {
                let action = import_action.value().read();
                match action.deref() {
                    Some(Ok(true)) => view! {
                        <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                            <div class="bg-slate-800 rounded-xl p-6 min-w-[300px] border border-slate-700 shadow-xl">
                                <h3 class="text-lg font-semibold text-emerald-400 mt-0">"Import Successful"</h3>
                                <p class="text-slate-400 my-4">"Recipes have been imported successfully."</p>
                                <div class="mt-4 flex gap-2 justify-end">
                                    <Button appearance=ButtonAppearance::Primary on:click=move |_| dismiss()>
                                        "Close"
                                    </Button>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    Some(Err(msg)) => {
                        let msg = msg.clone();
                        view! {
                            <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                                <div class="bg-slate-800 rounded-xl p-6 min-w-[300px] border border-red-700 shadow-xl">
                                    <h3 class="text-lg font-semibold text-red-400 mt-0">"Import Error"</h3>
                                    <p class="text-slate-400 my-4">{msg}</p>
                                    <div class="mt-4 flex gap-2 justify-end">
                                        <Button appearance=ButtonAppearance::Subtle on:click=move |_| dismiss()>
                                            "Close"
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    },

                    None if let Some(c) = conflicts.get() => {
                       view! { <ConflictModal conflicts=c /> }.into_any()
                    },

                    None if import_action.pending().get() => view! {
                        <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                            <div class="bg-slate-800 rounded-xl p-6 min-w-[300px] border border-slate-700 shadow-xl">
                                <h3 class="text-lg font-semibold text-white mt-0">"Importing..."</h3>
                                <p class="text-slate-400 my-4">"Uploading recipe file and processing on server."</p>
                            </div>
                        </div>
                    }.into_any(),
                    None | Some(Ok(false)) => ().into_any()
                }
            }}
        </div>
    }
}

async fn run_import(
    file: web_sys::File,
    set_conflicts: WriteSignal<Option<ConflictContext>>,
) -> anyhow::Result<bool> {
    let file_size = file.size() as u64;
    let mut wasm_stream = wasm_streams::ReadableStream::from_raw(file.stream()).into_stream();

    let ws_url = format!("{}/api/recipe/import", ws_url_base());
    let mut ws = WebSocket::open(&ws_url).context("Failed to connect")?;

    ws.send(Message::Bytes(file_size.to_le_bytes().into()))
        .await
        .context("Failed to send length")?;

    while let Some(x) = wasm_stream.next().await {
        let bytes = js_sys::Uint8Array::new(&x.map_err(|x| anyhow!("{x:?}"))?);
        let data = bytes.to_vec();
        ws.send(Message::Bytes(data)).await?;
    }

    loop {
        match recv_message(&mut ws).await? {
            ImportServerMessage::Success => return Ok(true),
            ImportServerMessage::Error(msg) => return Err(anyhow::anyhow!(msg)),
            ImportServerMessage::Conflicts(conflicts) => {
                let (tx, mut rx) = mpsc::channel(1);
                set_conflicts.set(Some(ConflictContext {
                    conflicts,
                    responder: tx,
                }));
                let resolution = rx.recv().await;
                set_conflicts.set(None);

                let Ok(strategy) = resolution else {
                    leptos::logging::log!("Import aborted");
                    return Ok(false);
                };

                let json_strategy = serde_json::to_string(&strategy)?;
                ws.send(Message::Text(json_strategy)).await?;
            }
        }
    }
}

async fn recv_message(ws: &mut WebSocket) -> anyhow::Result<ImportServerMessage> {
    let Some(msg) = ws.next().await.transpose()? else {
        anyhow::bail!("Connection closed unexpectedly")
    };
    let Message::Text(text) = msg else {
        anyhow::bail!("Expected Text, got: {msg:?}");
    };
    Ok(serde_json::from_str::<ImportServerMessage>(&text)?)
}

#[component]
fn ConflictModal(conflicts: ConflictContext) -> impl IntoView {
    let ConflictContext {
        conflicts,
        mut responder,
    } = conflicts;
    let mut replace_responder = responder.clone();
    let mut duplicate_responder = responder.clone();

    let has_recipe_conflicts = !conflicts.recipe_ids.is_empty();
    let has_variable_conflicts = !conflicts.variable_conflicts.is_empty();
    view! {
        <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
            <div class="bg-slate-800 rounded-xl p-6 min-w-[400px] max-w-[600px] border border-slate-700 shadow-xl">
                <h3 class="text-lg font-semibold text-amber-400 mt-0">"Import Conflicts"</h3>
                <p class="text-slate-400 my-4">"The imported recipes conflict with existing data. Choose how to resolve."</p>

                {move || {
                    if has_recipe_conflicts {
                        let ids = conflicts.recipe_ids.clone();
                        view! {
                            <div class="mb-4">
                                <h4 class="text-sm font-medium text-slate-300 mb-2">"Conflicting Recipe IDs"</h4>
                                <div class="bg-slate-900 rounded-lg p-3 max-h-[200px] overflow-y-auto">
                                    {ids.iter().map(|id| {
                                        view! {
                                            <div class="text-slate-400 text-sm py-1">{format!("{id}")}</div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                {move || {
                    if has_variable_conflicts {
                        let vars = conflicts.variable_conflicts.clone();
                        view! {
                            <div class="mb-4">
                                <h4 class="text-sm font-medium text-slate-300 mb-2">"Variable Conflicts"</h4>
                                <div class="bg-slate-900 rounded-lg p-3 max-h-[200px] overflow-y-auto space-y-2">
                                    {vars.iter().map(|v| {
                                        view! {
                                            <div class="text-sm">
                                                <span class="text-slate-300 font-medium">{format!("{}", v.name)}</span>
                                                <div class="flex gap-4 mt-1">
                                                    <span class="text-slate-500">"Existing:"</span>
                                                    <span class="text-red-400">{format!("{}", v.existing)}</span>
                                                </div>
                                                <div class="flex gap-4">
                                                    <span class="text-slate-500">"Imported:"</span>
                                                    <span class="text-emerald-400">{format!("{}", v.imported)}</span>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <div class="mt-4 flex gap-2 justify-end">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        on:click=move |_| responder.close_channel()
                    >
                        "Cancel"
                    </Button>
                    <Button
                        appearance=ButtonAppearance::Secondary
                        on:click=move |_| replace_responder.try_send("Replace").unwrap()
                    >
                        "Replace Existing"
                    </Button>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on:click=move |_| duplicate_responder.try_send("Duplicate").unwrap()
                    >
                        "Duplicate"
                    </Button>
                </div>
            </div>
        </div>
    }
}
