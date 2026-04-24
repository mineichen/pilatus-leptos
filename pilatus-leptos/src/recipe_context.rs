use std::{collections::HashMap, ops::Deref, sync::Arc, time::Duration};

use futures_util::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use leptos::prelude::*;
use pilatus::{
    Name, RecipeId, Recipes, UntypedDeviceParamsWithVariables, VariablesPatch, device::DeviceId,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

use crate::{FetchApi, MapRwSignal, VariableChangeCtx};

pub mod list;

pub use list::RecipeInfo;

#[derive(Clone, PartialEq, Debug)]
pub struct DeviceInfos {
    pub device_id: DeviceId,
    pub name: pilatus::Name,
    pub device_type: String,
}

#[derive(Clone, PartialEq, Debug)]
struct UnsavedDeviceChange {
    recipe_id: RecipeId,
    params: Value,
    var_changes: VariablesPatch,
}

#[derive(Clone)]
pub struct RecipeContext(Arc<RecipeContextState>);

impl std::ops::Deref for RecipeContext {
    type Target = RecipeContextState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct RecipeContextState {
    client_id: Uuid,
    root: MapRwSignal<pilatus::Recipes>,
    // The Recipes might become invalid between updates. This value is only written with values approved by the server
    valid_root: RwSignal<pilatus::Recipes>,
    unsaved_changes_reader: ReadSignal<HashMap<DeviceId, UnsavedDeviceChange>>,
    unsaved_changes_writer: WriteSignal<HashMap<DeviceId, UnsavedDeviceChange>>,
    variables: RwSignal<HashMap<Name, Value>>,
}

impl RecipeContext {
    pub(crate) fn new(recipes: pilatus::Recipes, client_id: Uuid) -> Self {
        Self(Arc::new(RecipeContextState::new(recipes, client_id)))
    }
}

impl RecipeContextState {
    fn new(recipes: pilatus::Recipes, client_id: Uuid) -> Self {
        let (unsaved_changes_reader, unsaved_changes_writer) = signal(Default::default());
        Self {
            client_id,
            root: MapRwSignal::new(recipes.clone()),
            valid_root: RwSignal::new(recipes),
            unsaved_changes_reader,
            unsaved_changes_writer,
            variables: RwSignal::new(
                [(Name::new("foo").unwrap(), 42.into())]
                    .into_iter()
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetVariableError {
    #[error("Variable not found")]
    NotFound,
    #[error("Failed to deserialize variable: {0}")]
    DeserializationError(#[from] serde_json::Error),
}

impl RecipeContext {
    /// Get a variable value by name, deserializing to the target type
    /// Panics if the variable is not found or cannot be deserialized
    pub fn expect_variable<T: DeserializeOwned>(&self, name: &Name) -> T {
        self.get_variable(name)
            .unwrap_or_else(|e| panic!("Cannot get Variable '{name}': {e}"))
    }

    pub fn get_variable<T: DeserializeOwned>(&self, name: &Name) -> Result<T, GetVariableError> {
        self.variables.with(|vars| {
            T::deserialize(vars.get(name).ok_or(GetVariableError::NotFound)?)
                .map_err(GetVariableError::from)
        })
    }

    /// Set a variable value by name
    pub fn set_variable<T: Serialize>(&self, name: Name, value: T) {
        self.variables.update(|vars| {
            if let Ok(json_val) = serde_json::to_value(&value) {
                vars.insert(name, json_val);
            } else {
                leptos::logging::error!("Failed to serialize variable '{}' value", name);
            }
        });
    }

    pub fn get_active_device_infos(
        &self,
        device_id: Signal<Option<DeviceId>>,
    ) -> Memo<Option<DeviceInfos>> {
        let root = self.root.read_only();
        Memo::new(move |_old| {
            let device_id = device_id.get()?;
            let recipes = root.read();
            recipes
                .active()
                .1
                .devices
                .get(&device_id)
                .map(|device| DeviceInfos {
                    name: device.device_name.clone(),
                    device_id,
                    device_type: device.device_type.clone(),
                })
        })
    }

    /// Get a typed signal from the JSON params
    /// Panics, if the device_id doesn't exist in active. Usually, DeviceContext is responsible not to render children if this is the case
    pub fn get_untyped(&self, device_id: Signal<DeviceId>) -> MapRwSignal<serde_json::Value> {
        let setter = self.unsaved_changes_writer;
        let getter = build_getter(device_id);
        self.root.map(
            move |x| getter(x).clone(),
            move |recipes, x| {
                let device_id = device_id.get_untracked();
                let (active_id, active) = recipes.get_active();

                let device = active
                    .devices
                    .get_mut(&device_id)
                    .unwrap_or_else(|| panic!("Unknown DeviceId {device_id} in active recipe"));
                device.params = UntypedDeviceParamsWithVariables::from_serializable(&x)
                    .expect("Expect serialize to work");

                setter.update(move |u| {
                    u.insert(
                        device_id,
                        UnsavedDeviceChange {
                            recipe_id: active_id,
                            params: x,
                            var_changes: HashMap::new(),
                        },
                    );
                });
            },
        )
    }
    pub fn get<
        T: DeserializeOwned
            + Serialize
            + Send
            + Sync
            + PartialEq
            + Default
            + Clone
            + 'static
            + impex::Visitor<VariableChangeCtx>,
    >(
        &self,
        device_id: Signal<DeviceId>,
    ) -> MapRwSignal<T> {
        let valid_root = self.valid_root;
        let getter = build_getter(device_id);
        let setter = self.unsaved_changes_writer;
        let recipe_context = self.clone();
        self.root.map(
            move |x| {
                T::deserialize(getter(x)).unwrap_or_else(|_| {
                    let valid = valid_root.read();
                    let x = getter(&valid);
                    T::deserialize(x).unwrap_or_else(|e| {
                        panic!(
                            "Cannot extract {:?} from {:?}: {e}",
                            std::any::type_name::<T>(),
                            x
                        )
                    })
                })
            },
            move |recipes, mut x| {
                let device_id = device_id.get_untracked();
                let (active_id, active) = recipes.get_active();

                let device = active
                    .devices
                    .get_mut(&device_id)
                    .unwrap_or_else(|| panic!("Unknown DeviceId {device_id} in active recipe"));
                device.params = UntypedDeviceParamsWithVariables::from_serializable(&x)
                    .expect("Expect serialize to work");

                let mut visitor = VariableChangeCtx::new(recipe_context.clone());

                x.visit(&mut visitor);

                setter.update(move |u| {
                    u.insert(
                        device_id,
                        UnsavedDeviceChange {
                            recipe_id: active_id,
                            params: serde_json::to_value(&x).expect("Serialization always works"),
                            var_changes: visitor.var_changes,
                        },
                    );
                });
            },
        )
    }
}

fn build_getter(device_id: Signal<DeviceId>) -> impl Fn(&Recipes) -> &Value {
    move |recipes| {
        recipes
            .active()
            .1
            .devices
            .get(&device_id.get())
            .expect("DeviceId must exist")
            .params
            .deref()
    }
}

#[component]
pub fn ProvideDeviceContext(children: Children) -> impl IntoView {
    let (error_signal, set_error_signal) = signal(None);
    let recipes_resource =
        LocalResource::new(
            move || async move { load_recipes_until_success(set_error_signal).await },
        );

    let mut children = Some(children);
    let my_id = uuid::Uuid::new_v4();

    view! {
        {move || {
            error_signal.get().map(|error| {
                view! {
                    <div
                        style="
                            position: fixed;
                            top: 0;
                            left: 0;
                            width: 100vw;
                            height: 100vh;
                            background-color: rgba(0, 0, 0, 0.8);
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            z-index: 9999;
                            pointer-events: all;
                        "
                    >
                        <div
                            style="
                                background-color: white;
                                padding: 2rem;
                                border-radius: 8px;
                                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
                                max-width: 500px;
                                text-align: center;
                                border: 2px solid #ef4444;
                            "
                        >
                            <div
                                style="
                                    color: #ef4444;
                                    font-size: 3rem;
                                    margin-bottom: 2rem;
                                "
                            >
                                "⚠️"
                            </div>
                            <h2
                                style="
                                    color: #1f2937;
                                    margin: 0 0 1rem 0;
                                    font-size: 1.5rem;
                                    font-weight: bold;
                                "
                            >
                                "Failed to Load Recipes"
                            </h2>
                            <p
                                style="
                                    color: #6b7280;
                                    margin: 0 0 1.5rem 0;
                                    font-size: 1rem;
                                    line-height: 1.5;
                                "
                            >
                                "Unable to connect to the recipe service. The application will retry automatically."
                            </p>
                            <div
                                style="
                                    background-color: #fef2f2;
                                    border: 1px solid #fecaca;
                                    border-radius: 4px;
                                    padding: 1rem;
                                    margin-bottom: 1rem;
                                    font-family: monospace;
                                    font-size: 0.875rem;
                                    color: #991b1b;
                                    text-align: left;
                                    overflow-x: auto;
                                "
                            >
                                {error}
                            </div>
                        </div>
                    </div>
                }
            })

        }}
        {move || {
            recipes_resource.get().map(|recipes| {
                let fetch: FetchApi = expect_context();
                let device_context = RecipeContext::new(recipes, my_id);
                let (ch_reader, ch_writer) = {
                    (
                        device_context.0.unsaved_changes_reader,
                        device_context.0.unsaved_changes_writer,
                    )
                };
                provide_context(device_context.clone());

                leptos::task::spawn_local(async move {
                    start_recipe_stream_listener(device_context, set_error_signal).await;
                });

                const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

                let action = Action::new_local(move |_| async move {
                    loop {
                        let mut next = None;
                        ch_writer.update(|x| {
                            next = x.extract_if(|_, _| true).next();
                        });
                        let Some((device_id, change)) = next else {
                            break;
                        };

                        gloo_timers::future::sleep(DEBOUNCE_DURATION).await;
                        if ch_reader.read_untracked().get(&device_id).is_some() {
                            leptos::logging::debug_log!("Device : {device_id:?} has newer pending changes");
                        } else {
                            leptos::logging::debug_log!("Sending update now ({device_id:?}): {:?}", change.params);
                            let url = format!("/api/recipe/{}/device/{device_id}/params?key={}", change.recipe_id, my_id);
                            let body = &serde_json::json!( {
                                "parameters": change.params,
                                "variables": {}
                            });
                            fetch.put_json(&url, body).await.ok();
                        }
                    }
                    anyhow::Ok(())
                });

                Effect::new(
                    move |_| match (ch_reader.try_read(), action.pending().get_untracked()) {
                        (Some(x), false) if !x.is_empty() => {
                            leptos::logging::debug_log!("Dispatch save from effect");
                            action.dispatch(());
                        }
                        (_, true) => {
                            leptos::logging::debug_log!("Dispatch not needed: Running already",);
                        }
                        (Some(x), _) if x.is_empty() => {
                            leptos::logging::debug_log!("Dispatch not needed: Empty list")
                        }
                        x => leptos::logging::log!("Dispatch not needed: {x:?}"),
                    },
                );

                children.take().expect("extracted max once")()
            })
        }}

    }
}

async fn start_recipe_stream_listener(
    ctx: RecipeContext,
    set_error_signal: WriteSignal<Option<String>>,
) {
    let ws_url = {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let protocol = if location.protocol().unwrap_or_default() == "https:" {
            "wss:"
        } else {
            "ws:"
        };
        let host = location
            .host()
            .expect("Cannot listen to recipes without location");
        format!("{}//{}/api/recipe/stream", protocol, host)
    };
    let my_client_id_str = ctx.client_id.to_string();
    loop {
        match WebSocket::open(&ws_url) {
            Ok(mut ws) => {
                leptos::logging::debug_log!("Connecting to WebSocket '{}' was successful", ws_url);

                // Read messages from the server
                loop {
                    match ws.next().await {
                        Some(Ok(Message::Text(client))) => {
                            if client != my_client_id_str {
                                leptos::logging::debug_log!("Recipe change UUID: {}", client);
                                if let Err(e) = ctx.refresh_recipes().await {
                                    leptos::logging::error!(
                                        "Cannot update recipes after receiving change from client {client:?}: {e:?}"
                                    )
                                }
                            } else {
                                leptos::logging::debug_log!(
                                    "Got a change, which was produced by myself"
                                )
                            }
                        }
                        Some(Ok(Message::Bytes(_))) => {
                            leptos::logging::warn!("Received unexpected binary message");
                        }
                        Some(Err(e)) => {
                            leptos::logging::error!("WebSocket error: {:?}", e);
                            break;
                        }
                        None => {
                            leptos::logging::log!("WebSocket stream ended");
                            break;
                        }
                    }
                }

                leptos::logging::log!("WebSocket connection closed, will reload recipes and retry");
            }
            Err(e) => {
                leptos::logging::error!("Failed to create WebSocket connection: {:?}", e);
            }
        }

        ctx.set_root(load_recipes_until_success(set_error_signal).await);
    }
}

async fn load_recipes_until_success(error_signal: WriteSignal<Option<String>>) -> Recipes {
    loop {
        match RecipeContext::load_recipes().await {
            Ok(recipes) => {
                error_signal.set(None);
                return recipes;
            }
            Err(e) => {
                error_signal.set(Some(e.to_string()));
                gloo_timers::future::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
