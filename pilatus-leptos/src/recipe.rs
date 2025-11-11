use std::sync::Arc;

use crate::RecipeContext;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use pilatus::device::DeviceId;
use thaw::{Button, Textarea, TextareaSize};

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
    let ctx = expect_context::<RecipeContext>();
    let params = use_params::<DeviceParams>();
    view! {
        <h1>"Device"</h1>
        <Outlet/>

    }
}
