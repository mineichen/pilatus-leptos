use crate::MapRwSignal;
use leptos::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct DeviceContext {
    params: MapRwSignal<serde_json::Value>,
}

impl DeviceContext {
    /// Creates a new DeviceContext with the given initial JSON value
    pub fn new(initial: serde_json::Value) -> Self {
        Self {
            params: MapRwSignal::new(initial),
        }
    }

    /// Get a typed signal from the JSON params
    ///
    /// Todo: Remove default
    pub fn get<T: DeserializeOwned + Serialize + Send + Sync + PartialEq + Default + 'static>(
        &self,
    ) -> MapRwSignal<T> {
        self.params.map(
            |x| T::deserialize(x).unwrap_or_default(),
            |target, value| *target = serde_json::to_value(&value).unwrap(),
        )
    }
}

#[component]
pub fn ProvideDeviceContext(children: Children) -> impl IntoView {
    let device_context = DeviceContext::new(serde_json::Value::Null);
    provide_context(device_context);

    children()
}
