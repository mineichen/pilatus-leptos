mod device_context;
mod impex_strategy;
mod json_device;
mod leaf_rw_signal;
mod map_rw_signal;
mod recipe_context;
mod variable_input;

use std::fmt::Display;

pub use device_context::*;
pub use impex_strategy::*;
pub use json_device::*;
pub use leaf_rw_signal::*;
pub use map_rw_signal::*;
pub use recipe_context::*;
pub use variable_input::*;

pub fn ws_url_base() -> impl Display {
    struct UrlBase {
        host: String,
    }

    impl Display for UrlBase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("ws://{}:4123", self.host))
        }
    }

    let host = web_sys::window()
        .and_then(|x| x.document())
        .and_then(|x| x.location())
        .and_then(|x| x.hostname().ok())
        .expect("Unable to get Location");

    UrlBase { host }
}
