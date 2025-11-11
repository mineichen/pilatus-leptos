use crate::{DeviceInfos, RecipeContext};
use leptos::prelude::*;

impl RecipeContext {
    pub fn list_devices(&self) -> Signal<Vec<DeviceInfos>> {
        let root = self.root.read_only();
        Signal::derive(move || {
            root.with(|x| {
                x.as_ref()
                    .map(|r| {
                        let (_, active) = r.active();
                        active
                            .devices
                            .iter()
                            .map(|(&device_id, device)| DeviceInfos {
                                name: device.device_name.clone(),
                                device_id,
                                device_type: device.device_type.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
        })
    }
}
