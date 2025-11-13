#![recursion_limit = "256"]

mod app;
mod busy_button;
mod home;
mod nav;
mod point;
mod recipe_management;

pub use app::*;
pub use recipe_management::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
