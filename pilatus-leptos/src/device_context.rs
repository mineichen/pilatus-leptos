use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::MapRwSignal;
use leptos::{either::Either, prelude::*};
use pilatus::{UntypedDeviceParamsWithVariables, device::DeviceId};
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

pub struct DeviceContextState {
    //pub deserializers: HashMap<&'static str, DeviceTypeDeserializer>,
    // pub untyped: HashMap<DeviceId, Untyped>,
    // pub managed_signals: HashMap<(DeviceId, TypeId), Box<dyn std::any::Any + Send + Sync>>,
    root: MapRwSignal<Option<pilatus::Recipes>>,
    unsaved_changes_reader: ReadSignal<HashMap<DeviceId, Value>>,
    unsaved_changes_writer: WriteSignal<HashMap<DeviceId, Value>>,
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
        }
    }
}

impl DeviceContext {
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
                let (_active_id, active) = recipes.get_active();

                let device = active
                    .devices
                    .get_mut(&device_id)
                    .unwrap_or_else(|| panic!("Unknown DeviceId {device_id} in active recipe"));
                device.params = UntypedDeviceParamsWithVariables::from_serializable(&x)
                    .expect("Expect serialize to work");
                setter.update(move |u| {
                    u.insert(device_id, x);
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
                        std::any::TypeId::of::<T>(),
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

    Effect::new(move || {
        if let Some(Ok(recipes)) = recipes_resource.get() {
            root.set(Some(recipes));
        }
    });

    let action = Action::new_local(move |_| async move {
        loop {
            let mut next = None;
            ch_writer.update(|x| {
                next = x.extract_if(|_, _| true).next();
            });
            let Some((device_id, value)) = next else {
                break;
            };
            leptos::logging::log!("Sending update now ({device_id:?}): {value:?}");
            gloo_timers::future::sleep(Duration::from_secs(1)).await;
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

    view! {
        {move || {
            if show_children.get() {
                Either::Left(children.take().expect("Only extracted max once")())
            } else {
                Either::Right(view! { "Loading..." })
            }
        }}
    }
}
