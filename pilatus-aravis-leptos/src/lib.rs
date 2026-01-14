use std::ops::Deref;

use leptos::prelude::*;
use pilatus::Name;
use pilatus_engineering_leptos::ImageViewerComponent;
use pilatus_leptos::{DeviceContext, JsonDeviceView};

#[component]
pub fn AravisView() -> impl IntoView {
    let device_ctx = expect_context::<DeviceContext>();
    let device_id = Signal::derive(move || device_ctx.infos.read().device_id);

    let cameras = LocalResource::new(move || async move {
        let id = device_id.get();
        let url = format!("/api/pilatus-aravis/camera?device_id={id}");
        gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<pilatus_aravis::DeviceInfo>>()
            .await
            .map_err(|e| e.to_string())
    });

    let image_url = Signal::derive(move || {
        format!(
            "ws://localhost:4123/api/image/subscribe?format=Raw&device_id={}",
            device_id.read().deref()
        )
    });
    view! {
        <div>
            <h1>"Pilatus Engineering with canvas"</h1>
            <div>
                <ImageViewerComponent url=image_url/>
                <div>
                    <h3>"Camera"</h3>
                    <table style="width: 100%;">
                    {
                        move || { cameras.get().and_then(|x| x.ok()).map(|all| {
                            all.into_iter().map(|x| {
                            view! {
                                <tr>
                                    <td>{x.id}</td>
                                    <td>{x.physical_id}</td>
                                    <td>{x.vendor}</td>
                                    <td>{x.model}</td>
                                    <td>{x.protocol}</td>
                                    <td>{x.address}</td>
                                </tr>
                                }
                            }).collect::<Vec<_>>()
                        })}
                    }
                    </table>
                </div>
                <JsonDeviceView/>
            </div>
        </div>
    }
}
