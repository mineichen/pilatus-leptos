use std::str::FromStr;

use crate::{DeviceInfos, JsonDeviceView, MapRwSignal, RecipeContext, VariableChangeCtx};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_params_map;
use pilatus::device::DeviceId;
use pilatus_leptos_components::{
    Button, ButtonSize, ButtonVariant, Dialog, DialogBody, DialogContent, DialogHeader, DialogTitle,
};

use leptos_router::components::Outlet;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct DeviceContext {
    pub infos: DeviceInfos,
}

impl DeviceContext {
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
    ) -> MapRwSignal<T> {
        let recipe_context: RecipeContext = expect_context();
        let device_id = self.infos.device_id;
        recipe_context.get(Signal::derive(move || device_id))
    }

    pub fn get_untyped(&self) -> MapRwSignal<serde_json::Value> {
        let recipe_context: RecipeContext = expect_context();
        let device_id = self.infos.device_id;
        recipe_context.get_untyped(Signal::derive(move || device_id))
    }
}
#[component]
pub fn DeviceView() -> impl IntoView {
    let ctx: RecipeContext = expect_context();
    let params = use_params_map();

    let device_id_str = move || params.read().get("device_id");
    let device_id = Signal::derive(move || DeviceId::from_str(&device_id_str()?).ok());
    let device_infos = ctx.get_active_device_infos(device_id);

    // Effect is required to avoid, that the old DeviceView is loaded with data from new Device
    let (delayed_infos, set_delayed) = signal(None);
    Effect::new(move || {
        if let Some(infos) = device_infos.get() {
            set_delayed.set(Some(infos));
        }
    });
    let show_settings = RwSignal::new(false);
    view! {
        { move|| {

            if let Some(infos) = delayed_infos.get() {
                leptos::logging::log!("DeviceInfos changed {infos:?}");
                let name = infos.name.to_string();
                provide_context(DeviceContext { infos });
                Either::Left(view! {
                    <div class="space-y-6">
                        <div class="flex items-start justify-between gap-4">
                            <h1 class="text-2xl font-bold text-foreground">{name}</h1>
                            <span title="Device settings">
                                <Button variant=ButtonVariant::Ghost size=ButtonSize::IconSm on:click=move |_| show_settings.set(true)>"⚙️"</Button>
                            </span>
                        </div>

                        <Outlet/>

                        <DeviceSettingsModal show=show_settings/>
                    </div>
                })
            } else {
                Either::Right(view! {
                {move|| if let Some(parsed) = device_id.get().as_ref() {
                        format!("No device with ID {parsed} found in active recipe")
                    } else if let Some(unknown_id) = device_id_str() {
                        format!("Invalid device ID: {unknown_id:?}")
                    } else {
                        "No device ID provided".to_string()
                    }
                }})
            }
        }}
    }
}

#[component]
fn DeviceSettingsModal(show: RwSignal<bool>) -> impl IntoView {
    view! {
        <Dialog show=show>
            <DialogContent class="sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>"Device Settings"</DialogTitle>
                </DialogHeader>
                <DialogBody class="min-h-0 flex-1 overflow-y-auto">
                    <JsonDeviceView on_close=Callback::new(move |()| show.set(false)) />
                </DialogBody>
            </DialogContent>
        </Dialog>
    }
}
