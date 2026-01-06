use app::*;

pub fn main() {
    use leptos::{logging, mount};

    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(App);
}
