use app::App;
use leptos::prelude::*;
use leptos_router::{NestedRoute, StaticSegment};

pub fn main() {
    use leptos::{logging, mount};

    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(RootApp);
}

#[component]
fn RootApp() -> impl IntoView {
    #[cfg(feature = "pilatus-examples-leptos")]
    let extra_device_routes = (
        NestedRoute::new(StaticSegment("greeter"), pilatus_examples_leptos::Greeter),
        NestedRoute::new(
            StaticSegment("manual_tick"),
            pilatus_examples_leptos::ManualTick,
        ),
    );
    #[cfg(not(feature = "pilatus-examples-leptos"))]
    let extra_device_routes = (NestedRoute::new(
        StaticSegment("__inexistend_device_id"),
        || (),
    ),);
    view! { <App extra_device_routes=extra_device_routes/> }
}
