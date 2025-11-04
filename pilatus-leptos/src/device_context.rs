use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::MapRwSignal;
use leptos::{either::Either, prelude::*};
use pilatus::{UntypedDeviceParamsWithVariables, device::DeviceId};
use serde::{Serialize, de::DeserializeOwned};

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

pub struct DeviceContextState {
    //pub deserializers: HashMap<&'static str, DeviceTypeDeserializer>,
    // pub untyped: HashMap<DeviceId, Untyped>,
    // pub managed_signals: HashMap<(DeviceId, TypeId), Box<dyn std::any::Any + Send + Sync>>,
    root: MapRwSignal<Option<pilatus::Recipes>>,
}

impl DeviceContext {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(DeviceContextState::new())))
    }

    pub fn update_root(&self, recipes: pilatus::Recipes) {
        self.0.lock().unwrap().update_root(recipes);
    }
}

impl DeviceContextState {
    fn new() -> Self {
        Self {
            // deserializers: device_types
            //     .into_iter()
            //     .map(|x| (x.device_type, x.factory))
            //     .collect(),
            //untyped: HashMap::new(),
            root: MapRwSignal::new(None),
        }
    }

    pub fn update_root(&self, recipes: pilatus::Recipes) {
        self.root.set(Some(recipes));
    }
}

struct Untyped {
    value: serde_json::Value,
    device_type: String,
}

impl DeviceContext {
    /// Get a typed signal from the JSON params
    pub fn get_untyped(&self, device_id: Signal<DeviceId>) -> MapRwSignal<serde_json::Value> {
        self.0.lock().unwrap().root.map(
            move |x| {
                leptos::logging::log!("Create device JSON-Value");
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
                leptos::logging::log!("Sett device JSON-VALUE");
                let recipes = target
                    .as_mut()
                    .expect("Recipes has to be downloaded at this point");
                let device_id = device_id.get();
                let (_active_id, active) = recipes.get_active();

                let device = active
                    .devices
                    .get_mut(&device_id)
                    .unwrap_or_else(|| panic!("Unknown DeviceId {device_id} in active recipe"));
                device.params = UntypedDeviceParamsWithVariables::from_serializable(&x)
                    .expect("Expect serialize to work");
                leptos::logging::log!("JSON-VALUE is set");
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
                        std::any::TypeId::of::<T>(),
                        x
                    )
                })
            },
            |target, x| {
                *target = serde_json::to_value(&x).expect("Serialization always works");
            },
        )

        // let updater = |value: serde_json::Value| {
        // let signal = self.typed_signals.get(&device_id).map(|x| x.signal.clone()).unwrap_or_else(|| {
        // let device = self
        //     .untyped
        //     .get(&device_id)
        //     .map(|x| T::deserialize(x).unwrap_or_default())
        //     .unwrap_or_default();
        // self.params.map(
        //     |x| T::deserialize(x).unwrap_or_default(),
        //     |target, value| *target = serde_json::to_value(&value).unwrap(),
        // )
    }
}

#[component]
pub fn ProvideDeviceContext<F, V>(children: F) -> impl IntoView
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    let device_context = DeviceContext::new();
    let root = device_context.0.lock().unwrap().root;
    provide_context(device_context.clone());

    // Fetch recipes from the API
    let recipes_resource = LocalResource::new(|| async {
        gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
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

    // Update root when recipes are loaded
    let device_context_for_effect = device_context.clone();
    Effect::new(move || {
        if let Some(Ok(recipes)) = recipes_resource.get() {
            device_context_for_effect.update_root(recipes);
        }
    });

    view! {
        {move || {
            if root.get().is_some() {
                Either::Left(children())
            } else {
                Either::Right(view! { "Loading..." })
            }
        }}
    }
}
