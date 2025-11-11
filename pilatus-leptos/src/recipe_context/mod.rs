use std::{collections::HashMap, ops::Deref, sync::Arc, time::Duration};

use crate::MapRwSignal;
use leptos::{either::Either, prelude::*};
use pilatus::{RecipeId, UntypedDeviceParamsWithVariables, device::DeviceId};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub mod list;

pub use list::RecipeInfo;

#[derive(Clone, PartialEq)]
pub struct DeviceInfos {
    pub device_id: DeviceId,
    pub name: pilatus::Name,
    pub device_type: String,
}

#[derive(Clone)]
pub struct RecipeContext(Arc<DeviceContextState>);

impl std::ops::Deref for RecipeContext {
    type Target = DeviceContextState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct DeviceContextState {
    root: MapRwSignal<pilatus::Recipes>,
    unsaved_changes_reader: ReadSignal<HashMap<DeviceId, (RecipeId, Value)>>,
    unsaved_changes_writer: WriteSignal<HashMap<DeviceId, (RecipeId, Value)>>,
    variables: RwSignal<HashMap<String, Value>>,
}

impl RecipeContext {
    fn new(recipes: pilatus::Recipes) -> Self {
        Self(Arc::new(DeviceContextState::new(recipes)))
    }
}

impl DeviceContextState {
    fn new(recipes: pilatus::Recipes) -> Self {
        let (unsaved_changes_reader, unsaved_changes_writer) = signal(Default::default());
        Self {
            root: MapRwSignal::new(recipes),
            unsaved_changes_reader,
            unsaved_changes_writer,
            variables: RwSignal::new([("foo".to_string(), 42.into())].into_iter().collect()),
        }
    }
}

impl RecipeContext {
    /// Get a variable value by name, deserializing to the target type
    /// Panics if the variable is not found or cannot be deserialized
    pub fn get_variable<T: DeserializeOwned>(&self, name: &str) -> T {
        self.variables.with(|vars| {
            vars.get(name)
                .and_then(|val| T::deserialize(val).ok())
                .unwrap_or_else(|| {
                    panic!("Variable '{}' not found or cannot be deserialized", name)
                })
        })
    }

    /// Set a variable value by name
    pub fn set_variable<T: Serialize>(&self, name: &str, value: T) {
        self.variables.update(|vars| {
            if let Ok(json_val) = serde_json::to_value(&value) {
                vars.insert(name.to_string(), json_val);
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
    pub fn get_untyped(&self, device_id: Signal<DeviceId>) -> MapRwSignal<serde_json::Value> {
        let setter = self.unsaved_changes_writer;
        self.root.map(
            move |recipes| {
                recipes
                    .active()
                    .1
                    .devices
                    .get(&device_id.get())
                    .expect("DeviceId must exits")
                    .params
                    .deref()
                    .clone()
            },
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
                    u.insert(device_id, (active_id, x));
                });
            },
        )
    }
    pub fn get<
        T: DeserializeOwned + Serialize + Send + Sync + PartialEq + Default + Clone + 'static,
    >(
        &self,
        device_id: Signal<DeviceId>,
    ) -> MapRwSignal<T> {
        self.get_untyped(device_id).map(
            |x| {
                T::deserialize(x).unwrap_or_else(|e| {
                    panic!(
                        "Cannot extract {:?} from {:?}: {e}",
                        std::any::type_name::<T>(),
                        x
                    )
                })
            },
            |target, x| {
                *target = serde_json::to_value(&x).expect("Serialization always works");
            },
        )
    }
}

#[component]
pub fn ProvideDeviceContext(children: Children) -> impl IntoView {
    // Fetch recipes from the API
    let recipes_resource = LocalResource::new(|| async {
        gloo_net::http::Request::get("/api/recipe/get_all")
            .header("content-type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<pilatus::device::ActiveState>()
            .await
            .map_err(|e| e.to_string())
            .map(|state| state.recipes)
    });

    let mut children = Some(children);

    view! {
        {move || {
            if let Some(Ok(recipes)) = recipes_resource.get() {
                let device_context = RecipeContext::new(recipes.clone());
                let (ch_reader, ch_writer) = {
                    (
                        device_context.0.unsaved_changes_reader,
                        device_context.0.unsaved_changes_writer,
                    )
                };
                provide_context(device_context);

                const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

                let action = Action::new_local(move |_| async move {
                    loop {
                        let mut next = None;
                        ch_writer.update(|x| {
                            next = x.extract_if(|_, _| true).next();
                        });
                        let Some((device_id, (recipe_id, value))) = next else {
                            break;
                        };

                        gloo_timers::future::sleep(DEBOUNCE_DURATION).await;
                        if ch_reader.read_untracked().get(&device_id).is_some() {
                            leptos::logging::log!("Device : {device_id:?} has newer pending changes");
                        } else {
                            leptos::logging::log!("Sending update now ({device_id:?}): {value:?}");
                            let url = format!("/api/recipe/{recipe_id}/device/{device_id}/params");

                            let r = async move {
                                gloo_net::http::Request::put(&url)
                                    .header("content-type", "application/json")
                                    .body(
                                        serde_json::json!( {
                                            "parameters": value,
                                            "variables": {}
                                        })
                                        .to_string(),
                                    )?
                                    .send()
                                    .await
                            }
                            .await;
                            match r {
                                Ok(_r) => {
                                    leptos::logging::log!("Save was successful");
                                }
                                Err(e) => leptos::logging::error!("Store failed: {e:?}"),
                            }
                        }
                    }
                    anyhow::Ok("Foo")
                });

                Effect::new(
                    move |_| match (ch_reader.try_read(), action.pending().get_untracked()) {
                        (Some(x), false) if !x.is_empty() => {
                            leptos::logging::log!("Dispatch save from effect");
                            action.dispatch(());
                        }
                        (_, true) => {
                            leptos::logging::log!("Dispatch not needed: Running already",);
                        }
                        (Some(x), _) if x.is_empty() => {
                            leptos::logging::log!("Dispatch not needed: Empty list")
                        }
                        x => leptos::logging::log!("Dispatch not needed: {x:?}"),
                    },
                );

                Either::Left(children.take().expect("Only extracted max once")())
            } else {
                Either::Right(view! { "Loading..." })
            }
        }}
    }
}
