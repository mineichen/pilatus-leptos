use leptos::{either::Either, prelude::*};
use pilatus::Name;
use pilatus_leptos_components::{Button, ButtonSize, ButtonVariant, Input};
use thaw::Field;

use crate::LeafRwSignal;

#[component]
pub fn VariableInput<
    T: Into<String> + Clone + serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
>(
    /// The LeafRwSignal to bind to
    value: LeafRwSignal<T>,
    /// Label for the input field
    #[prop(optional)]
    label: Option<String>,
) -> impl IntoView {
    let (show_var_dialog, set_show_var_dialog) = signal(false);
    let new_var_name = RwSignal::new(String::new());

    // Reactive signals derived from the value
    let var_name = Signal::derive(move || value.get_variable_name());
    let is_var = Signal::derive(move || var_name.read().is_some());

    // Convert to variable
    let convert_to_var = move |_| {
        let Ok(var_name_val) = Name::new(new_var_name.get()) else {
            leptos::logging::log!("Invalid name");
            return;
        };
        if !var_name_val.is_empty() {
            match value.convert_to_variable(var_name_val) {
                Ok(()) => {
                    set_show_var_dialog.set(false);
                    new_var_name.set(String::new());
                }
                Err(e) => {
                    leptos::logging::error!("Failed to convert to variable: {}", e);
                }
            }
        }
    };

    // Convert to local value
    let convert_to_local = move |_| {
        let current_value = value.get_value();
        value.convert_to_local(current_value);
    };

    // Mapped string view of the leaf (variable references preserved).
    // `LeafRwSignal<String>` implements `Get` + `Set`, so it binds directly
    // with no extra `RwSignal`.
    let str_leaf = value.map(
        |t: T| t.into(),
        |s: String| {
            serde_json::from_value(serde_json::Value::String(s)).map_err(|e| {
                format!(
                    "Failed to deserialize {} from str: {}",
                    std::any::type_name::<T>(),
                    e
                )
            })
        },
    );

    view! {
        <Field label=label.unwrap_or_default()>
            <div style="display: flex; gap: 8px; align-items: center;">
                <Input
                    value=str_leaf
                    disabled= move|| {is_var.get()}
                    class="flex-1"
                />

                {move || {
                    if is_var.get() {
                        Either::Left(view! {
                            <div style="display: flex; gap: 4px; align-items: center;">
                                <span style="color: #0078ff; font-weight: bold;">
                                    "🔗 " {var_name}
                                </span>
                                <Button on:click=convert_to_local size=ButtonSize::Sm>
                                    "Use Local Value"
                                </Button>
                            </div>
                        })
                    } else {
                        Either::Right(view! {
                            <Button
                                on:click=move |_| {
                                    set_show_var_dialog.set(true);
                                }
                                size=ButtonSize::Sm
                            >
                                "🔗 Use Variable"
                            </Button>
                        })
                    }
                }}
            </div>
        </Field>

        {move || {
            show_var_dialog.get().then(move|| {
                view! {
                    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                        <div class="rounded-xl p-6 min-w-[320px] border border-border shadow-xl bg-card">
                            <h3 class="text-lg font-semibold text-foreground mt-0 mb-4">"Create Variable Reference"</h3>
                            <Field label="Variable Name">
                                <Input
                                    value=new_var_name
                                    placeholder="Enter variable name"
                                />
                            </Field>
                            <div class="flex gap-2 justify-end mt-4">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    on:click=move |_| set_show_var_dialog.set(false)
                                >
                                    "Cancel"
                                </Button>
                                <Button on:click=convert_to_var>
                                    "Create Variable"
                                </Button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
