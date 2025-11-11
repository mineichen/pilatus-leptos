use leptos::prelude::*;
use pilatus_leptos::DeviceContext;
use thaw::{Anchor, AnchorLink};

#[component]
pub fn Nav() -> impl IntoView {
    let ctx = use_context::<DeviceContext>().expect("Requires DeviceContext to render nav");
    view! {
        <Anchor>
            <AnchorLink title="Home" href="/" />
            <For
                each=move || ctx.list_devices().get()
                key=|x| x.device_id
                let(x)>
                <AnchorLink title = x.name.to_string() href = format!("/device/{}/{}", x.device_id, x.device_type)/>
            </For>
        </Anchor>
    }
}
