//! RustUI-based UI primitives for industrial HMI panels built with Leptos.
//!
//! This crate re-exports the RustUI (`leptos_ui` + `tw_merge`) components with minor modifications
//!
//! # Adding missing components
//!
//! Missing primitives (badge, card, dialog, …) can be downloaded from the
//! RustUI component registry:
//!
//! <https://github.com/rust-ui/leptos-ui/tree/main/app_crates/registry/src/ui>
//!
//! Each file there is self-contained: copy it byte-for-byte to
//! `src/<name>.rs`, then register it below (`mod <name>;` +
//! `pub use <name>::…;`). Components needing a hook take it from
//! `app_crates/registry/src/hooks` the same way (`theme_mode` was installed that way)
//!
//! the `icons` crate is not vendored, inline the needed SVG instead). Styling needs no extra wiring:
//! `pilatus-leptos-app/input.scss` already scans this crate via `@source`,
//! so the component's Tailwind classes are generated automatically. Only if
//! the component uses new semantic colors (beyond the existing
//! `background/foreground/primary/…` set), add the corresponding CSS
//! variables plus `@theme inline` mappings to `input.scss`, using the
//! values from
//! <https://github.com/rust-ui/leptos-ui/blob/main/style/tailwind.css>

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

mod alert;
mod button;
mod callout;
mod dialog;
mod input;
mod input_number;
mod textarea;
mod theme_mode;
mod theme_toggle;

pub use alert::{Alert, AlertDescription, AlertTitle};
pub use button::{Button, ButtonClass, ButtonSize, ButtonVariant};
pub use callout::{Callout, CalloutVariant};
pub use dialog::{
    Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
pub use input::{Input, InputType};
pub use input_number::{InputNumber, InputNumberValue};
pub use textarea::Textarea;
pub use theme_mode::{ThemeMode, use_theme_mode};
pub use theme_toggle::ThemeToggle;
pub use tw_merge::IntoTailwindClass;
