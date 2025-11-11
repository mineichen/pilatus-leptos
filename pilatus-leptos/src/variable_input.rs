use leptos::prelude::*;
use thaw::{Button, Field, Input};

use crate::LeafRwSignal;

#[component]
pub fn VariableInput(
    /// The LeafRwSignal to bind to
    value: LeafRwSignal<String>,
    /// Label for the input field
    #[prop(optional)]
    label: Option<String>,
) -> impl IntoView {
    let (show_var_dialog, set_show_var_dialog) = signal(false);
    let new_var_name = RwSignal::new(String::new());

    // Reactive signals derived from the value
    let is_var = Signal::derive(move || value.is_variable());
    let var_name = Signal::derive(move || value.get_variable_name().unwrap_or_default());

    // Convert to variable
    let convert_to_var = move |_| {
        let var_name_val = new_var_name.get();
        if !var_name_val.is_empty() {
            match value.convert_to_variable(&var_name_val) {
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

    view! {
        <Field label=label.unwrap_or_default()>
            <div style="display: flex; gap: 8px; align-items: center;">
                <Input
                    value=value
                    disabled=is_var
                    attr:style="flex: 1;"
                />

                {move || {
                    if is_var.get() {
                        view! {
                            <div style="display: flex; gap: 4px; align-items: center;">
                                <span style="color: #0078ff; font-weight: bold;">
                                    "🔗 " {var_name.get()}
                                </span>
                                <Button on:click=convert_to_local size=thaw::ButtonSize::Small>
                                    "Use Local Value"
                                </Button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <Button
                                on:click=move |_| {
                                    set_show_var_dialog.set(true);
                                }
                                size=thaw::ButtonSize::Small
                            >
                                "🔗 Use Variable"
                            </Button>
                        }.into_any()
                    }
                }}
            </div>
        </Field>

        {move || {
            if show_var_dialog.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                        <div style="background: white; padding: 20px; border-radius: 8px; min-width: 300px;">
                            <h3>"Create Variable Reference"</h3>
                            <Field label="Variable Name">
                                <Input
                                    value=new_var_name
                                    placeholder="Enter variable name"
                                />
                            </Field>
                            <div style="margin-top: 16px; display: flex; gap: 8px; justify-content: flex-end;">
                                <Button on:click=move |_| set_show_var_dialog.set(false)>
                                    "Cancel"
                                </Button>
                                <Button on:click=convert_to_var>
                                    "Create Variable"
                                </Button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}
