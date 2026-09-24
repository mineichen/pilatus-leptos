use leptos::prelude::*;
use tw_merge::tw_merge;

/// A RustUI-styled checkbox bound directly to a `bool` signal.
///
/// Based on the RustUI registry `Checkbox`
/// (<https://github.com/rust-ui/leptos-ui/blob/main/app_crates/registry/src/ui/checkbox.rs>),
/// with three modifications: the `icons` crate dependency is replaced by an
/// inline SVG, instead of `checked: Signal<bool>` plus an
/// `on_checked_change` callback it takes anything readable and writable
/// (`RwSignal`, `MapRwSignal`, …), mirroring [`crate::Input`], and it takes
/// an `id` so callers can associate an external `<label r#for=...>`.
///
/// Labels are deliberately *not* a prop and must never wrap the checkbox:
/// a `<button role="checkbox">` nested inside a `<label>` double-toggles in
/// some browsers (the label re-dispatches the click to the button), which
/// made identically-looking checkboxes behave differently depending on
/// where you clicked. Instead, pass a `<label r#for=...>` as a child next
/// to the box, linked via the `id` prop:
///
/// ```ignore
/// <Checkbox checked=my_bool id="my-box">
///     <label r#for="my-box">"My option"</label>
/// </Checkbox>
/// ```
#[component]
pub fn Checkbox<V>(
    // Controlled checked state.
    checked: V,

    // Styling
    #[prop(into, optional)] class: String,

    // Associates an external `<label r#for=...>` child with the box.
    #[prop(into, optional)] id: Option<String>,

    #[prop(into, optional)] disabled: Signal<bool>,

    // Usually a `<label r#for=...>`; rendered next to the box.
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
where
    V: Get<Value = bool> + Set<Value = bool> + Clone + Send + Sync + 'static,
{
    // Touch-first sizing: the box is size-8, comfortably below the
    // NumberInput stepper buttons (40/38px) and the input row (36px).
    // A checked box is a solid primary fill, so at equal size it reads
    // heavier than the steppers; slightly smaller keeps the row balanced.
    // The tick is size-5 (20px), leaving 6px inset so it doesn't crowd the
    // border.
    let button_class = tw_merge!(
        "peer flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md border border-input shadow-xs outline-none transition-shadow dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50",
        class
    );

    let checked_for_state = checked.clone();
    let checked_for_aria = checked.clone();
    let checked_for_icon = checked.clone();
    let checked_for_toggle = checked;

    view! {
        <span class="inline-flex items-center gap-2" data-name="CheckboxField">
            <button
                data-name="Checkbox"
                type="button"
                role="checkbox"
                id=id
                class=button_class
            data-state=move || {
                if checked_for_state.get() { "checked" } else { "unchecked" }
            }
            aria-checked=move || checked_for_aria.get().to_string()
            disabled=disabled
            on:click=move |_| {
                if !disabled.get_untracked() {
                    checked_for_toggle.set(!checked_for_toggle.get());
                }
            }
        >
            <span
                data-name="CheckboxIndicator"
                class="flex justify-center items-center text-current transition-none"
            >
                {move || {
                    checked_for_icon.get().then(|| view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="20"
                                height="20"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="size-5"
                            >
                            <path d="M20 6 9 17l-5-5" stroke-width="2.5" />
                        </svg>
                    })
                }}
            </span>
            </button>
            {children.map(|c| c())}
        </span>
    }
}
