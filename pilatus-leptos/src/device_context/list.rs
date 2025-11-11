use crate::DeviceContext;
use leptos::prelude::*;
use pilatus::device::DeviceId;

#[derive(Clone)]
pub struct DeviceListItem {
    pub device_id: DeviceId,
    pub name: pilatus::Name,
    pub device_type: String,
}

impl DeviceContext {
    pub fn list_devices(&self) -> Signal<Vec<DeviceListItem>> {
        let lock = self.0.lock().unwrap();
        let root = lock.root.read_only();
        Signal::derive(move || {
            root.with(|x| {
                x.as_ref()
                    .map(|r| {
                        let (_, active) = r.active();
                        active
                            .devices
                            .iter()
                            .map(|(id, device)| DeviceListItem {
                                name: device.device_name.clone(),
                                device_id: id.clone(),
                                device_type: device.device_type.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
        })
    }
}
