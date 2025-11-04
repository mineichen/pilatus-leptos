use std::sync::Arc;

use crate::{DeviceContext, Point, point::PointView};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use pilatus::{Recipes, device::DeviceId};
use thaw::Button;

use crate::BusyButton;

#[component]
pub fn HomeView() -> impl IntoView {
    let point = RwSignal::new(Point { x: 0, y: 42 });
    view! {
        <h1>"Home"</h1>
        <PointView point=point />
        <Button on:click=move |_| point.write().x += 1>"Increment"</Button>
        <BusyButton/>
    }
}

use leptos_router::components::Outlet;

#[derive(PartialEq)]
pub struct DeviceParams {
    pub device_id: DeviceId,
}

impl leptos_router::params::Params for DeviceParams {
    fn from_map(
        map: &leptos_router::params::ParamsMap,
    ) -> Result<Self, leptos_router::params::ParamsError> {
        Ok(DeviceParams {
            device_id: map
                .get("device_id")
                .ok_or(leptos_router::params::ParamsError::MissingParam(
                    "device_id".to_string(),
                ))?
                .parse::<DeviceId>()
                .map_err(|x| leptos_router::params::ParamsError::Params(Arc::new(x)))?,
        })
    }
}
#[component]
pub fn DeviceView() -> impl IntoView {
    let params = use_params::<DeviceParams>();
    let device_id = Signal::derive(move || {
        params
            .read()
            .as_ref()
            .ok()
            .map(|p| p.device_id)
            .expect("Device ID must be present")
    });

    // Create a shared signal for child routes
    let device_context = expect_context::<DeviceContext>();
    let device_params = device_context.get_untyped(device_id);

    let device_params = device_params.map(
        |x| serde_json::to_string_pretty(&x).unwrap(),
        |target, value| *target = serde_json::from_str(&value).unwrap(),
    );
    Effect::new(move || {
        leptos::logging::log!("JsonChanged: {:?}", device_params.get());
    });

    view! {
        "Device: " { move || device_id.get().to_string() }<br/>
        <Outlet/>
        <pre>
            { move || device_params.get() }
        </pre>
    }
}
