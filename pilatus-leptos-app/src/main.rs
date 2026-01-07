use leptos::prelude::*;
use leptos_router::{NestedRoute, StaticSegment};
use pilatus_leptos_app::App;

pub fn main() {
    use leptos::{logging, mount};

    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(RootApp);
}

#[component]
fn RootApp() -> impl IntoView {
    // Empty tuple doesn't work, as it matches and doesn't forward to JsonDeviceView, but the Page-Default
    let extra_device_routes = (NestedRoute::new(
        StaticSegment("__inexistend_device_id"),
        || (),
    ),);
    view! { <App extra_device_routes=extra_device_routes/> }
}
