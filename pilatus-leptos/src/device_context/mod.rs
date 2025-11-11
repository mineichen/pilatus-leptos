use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{DeviceParams, MapRwSignal};
use leptos::{either::Either, prelude::*};
use pilatus::{RecipeId, UntypedDeviceParamsWithVariables, device::DeviceId};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Clone)]
pub struct DeviceContext(Arc<Mutex<DeviceContextState>>);

// type DeviceTypeDeserializer =
//     Box<dyn Fn() -> Box<dyn std::any::Any + Send + Sync> + Send + Sync + 'static>;
// pub struct DeviceTypeFactory {
//     factory: DeviceTypeDeserializer,
//     device_type: &'static str,
// }

// impl DeviceTypeFactory {
//     pub fn new<T: std::any::Any + Send + Sync + DeserializeOwned + Default + Clone + 'static>(
//         device_type: &'static str,
//     ) -> Self {
//         Self {
//             factory: Box::new(move || Box::new(T::default())),
//             device_type,
//         }
//     }
// }
mod list;

pub struct DeviceContextState {
    //pub deserializers: HashMap<&'static str, DeviceTypeDeserializer>,
    // pub untyped: HashMap<DeviceId, Untyped>,
    // pub managed_signals: HashMap<(DeviceId, TypeId), Box<dyn std::any::Any + Send + Sync>>,
    root: MapRwSignal<Option<pilatus::Recipes>>,
    unsaved_changes_reader: ReadSignal<HashMap<DeviceId, (RecipeId, Value)>>,
    unsaved_changes_writer: WriteSignal<HashMap<DeviceId, (RecipeId, Value)>>,
    variables: RwSignal<HashMap<String, Value>>,
}

impl DeviceContext {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(DeviceContextState::new())))
    }
}

impl DeviceContextState {
    fn new() -> Self {
        let (unsaved_changes_reader, unsaved_changes_writer) = signal(Default::default());
        Self {
            // deserializers: device_types
            //     .into_iter()
            //     .map(|x| (x.device_type, x.factory))
            //     .collect(),
            //untyped: HashMap::new(),
            root: MapRwSignal::new(None),
            unsaved_changes_reader,
            unsaved_changes_writer,
            variables: RwSignal::new([("foo".to_string(), 42.into())].into_iter().collect()),
        }
    }
}

impl DeviceContext {
    pub fn get_active_router_device<T>(&self) -> MapRwSignal<T>
    where
        T: DeserializeOwned + Serialize + Send + Sync + PartialEq + Default + Clone + 'static,
    {
        let params = leptos_router::hooks::use_params::<DeviceParams>();
        let device_id = move || Some(params.read().as_ref().ok()?.device_id);
        self.get::<T>(Signal::derive(move || device_id().unwrap()))
    }

    /// Get a variable value by name, deserializing to the target type
    /// Panics if the variable is not found or cannot be deserialized
    pub fn get_variable<T: DeserializeOwned>(&self, name: &str) -> T {
        let lock = self.0.lock().unwrap();
        let variables = lock.variables;

        variables.with(|vars| {
            vars.get(name)
                .and_then(|val| T::deserialize(val).ok())
                .unwrap_or_else(|| {
                    panic!("Variable '{}' not found or cannot be deserialized", name)
                })
        })
    }

    /// Set a variable value by name
    pub fn set_variable<T: Serialize>(&self, name: &str, value: T) {
        let lock = self.0.lock().unwrap();
        let variables = lock.variables;

        variables.update(|vars| {
            if let Ok(json_val) = serde_json::to_value(&value) {
                vars.insert(name.to_string(), json_val);
            } else {
                leptos::logging::error!("Failed to serialize variable '{}' value", name);
            }
        });
    }

    /// Get a typed signal from the JSON params
    pub fn get_untyped(&self, device_id: Signal<DeviceId>) -> MapRwSignal<serde_json::Value> {
        let lock = self.0.lock().unwrap();
        let setter = lock.unsaved_changes_writer;
        lock.root.map(
            move |x| {
                //leptos::logging::log!("Create device JSON-Value");
                x.as_ref()
                    .expect("Recipes has to be downloaded at this point")
                    .active()
                    .1
                    .devices
                    .get(&device_id.get())
                    .expect("DeviceId must exits")
                    .params
                    .deref()
                    .clone()
            },
            move |target, x| {
                let recipes = target
                    .as_mut()
                    .expect("Recipes has to be downloaded at this point");
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
                //leptos::logging::log!("JSON-VALUE is set");
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
    let device_context = DeviceContext::new();
    let (root, ch_reader, ch_writer) = {
        let lock = device_context.0.lock().unwrap();
        (
            lock.root,
            lock.unsaved_changes_reader,
            lock.unsaved_changes_writer,
        )
    };
    provide_context(device_context);

    // Fetch recipes from the API
    let recipes_resource = LocalResource::new(|| async {
        //gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
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

    Effect::new(move || {
        if let Some(Ok(recipes)) = recipes_resource.get() {
            root.set(Some(recipes));
        }
    });

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
                    Ok(r) => {
                        leptos::logging::log!("Save was successful");
                    }
                    Err(e) => leptos::logging::error!("Store failed: {e:?}"),
                }
            }
        }
        anyhow::Ok("Foo")
    });
    let show_children = Memo::new(move |_| root.get().is_some());
    let mut children = Some(children);

    Effect::new(
        move |_| match (ch_reader.try_read(), action.pending().get_untracked()) {
            (Some(x), false) if x.len() > 0 => {
                leptos::logging::log!("Dispatch save from effect");
                action.dispatch(());
            }
            (_, true) => {
                leptos::logging::log!("Dispatch not needed: Running already",);
            }
            (Some(x), _) if x.len() == 0 => {
                leptos::logging::log!("Dispatch not needed: Empty list")
            }
            x => leptos::logging::log!("Dispatch not needed: {x:?}"),
        },
    );

    //  let manual_tick_device_id =   Signal::derive(move || DeviceId::from_str("e8e8eb2d-2325-4a40-aba7-7d223d39fe83").unwrap());
    leptos::task::spawn_local(async move {
        // for _ in 0..10 {
        //     gloo_timers::future::sleep(Duration::from_secs(1)).await;
        //     leptos::logging::log!("Heartbeat");
        //     if show_children.get_untracked() {
        //         let manual_tick = temp_effect_clone.get::<serde_json::Value>(manual_tick_device_id);
        //         let mut manual_tick_json = manual_tick.get_untracked();
        //         let count_field = &mut manual_tick_json["initial_count"];
        //         *count_field = (count_field.as_i64().unwrap() + 1).into();

        //         manual_tick.set(manual_tick_json);
        //         leptos::logging::log!("Showing children {:?}", manual_tick);
        //     }
        // }
    });
    view! {
        {move || {
            if *show_children.read() {
                Either::Left(children.take().expect("Only extracted max once")())
            } else {
                Either::Right(view! { "Loading..." })
            }
        }}
    }
}
