use std::ops::Deref;

use leptos::prelude::*;
use pilatus_engineering_leptos::{ImageViewerComponent, WebSocketImageProvider};
use pilatus_leptos::{DeviceContext, FetchApi, JsonDeviceView, ws_url_base};

#[component]
pub fn AravisView() -> impl IntoView {
    let device_ctx: DeviceContext = expect_context();
    let fetch: FetchApi = expect_context();
    let device_id = Signal::derive(move || device_ctx.infos.device_id);
    let cameras = LocalResource::new(move || async move {
        let id = device_id.get();
        let url = format!("/api/pilatus-aravis/camera?device_id={id}");
        fetch
            .get_json_silent::<Vec<pilatus_aravis::DeviceInfo>>(&url)
            .await
    });

    let image_url = Signal::derive(move || {
        Some(format!(
            "{}/api/image/subscribe?format=Raw&device_id={}",
            ws_url_base(),
            device_id.read().deref()
        ))
    });
    view! {
        <div>
            <h1>"Pilatus Engineering with canvas"</h1>
            <div>
                <ImageViewerComponent url=image_url provider=WebSocketImageProvider::default() />
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
