use leptos::prelude::*;
use thaw::{Button, Textarea, TextareaSize};

use crate::DeviceContext;

#[component]
pub fn JsonDeviceView() -> impl IntoView {
    let device_context: DeviceContext = expect_context();
    let device_params = device_context.get_untyped();

    let device_params = device_params.map(
        |x| serde_json::to_string_pretty(&x).unwrap(),
        |target, value| *target = serde_json::from_str(&value).unwrap(),
    );

    // Initialize local editable state once with current value
    let edited_json = RwSignal::new(device_params.get_untracked());

    // Track the last known saved value to detect external changes
    let (last_saved_value, set_last_saved_value) = signal(device_params.get_untracked());

    // Track validation errors
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Detect external changes (not from our edits)
    let has_external_update = Memo::new(move |_| {
        let current_server = device_params.get();
        let last_saved = last_saved_value.get();
        current_server != last_saved
    });

    // Save handler
    let on_save = move |_| match serde_json::from_str::<serde_json::Value>(&edited_json.get()) {
        Ok(parsed) => {
            let formatted = serde_json::to_string_pretty(&parsed).unwrap();
            device_params.set(formatted.clone());
            set_last_saved_value.set(formatted.clone());
            edited_json.set(formatted);
            set_error_message.set(None);
        }
        Err(e) => {
            let error_text = e.to_string();
            let clean_error = error_text.split(" at ").next().unwrap_or(&error_text);
            set_error_message.set(Some(format!("Invalid JSON: {}", clean_error)));
        }
    };

    // Adopt external changes
    let on_adopt = move |_| {
        let current = device_params.get();
        edited_json.set(current.clone());
        set_last_saved_value.set(current);
    };

    view! {

        {move || {
            has_external_update.get().then(move|| {
                view! {
                    <div style="background-color: #fff3cd; border: 1px solid #ffc107; padding: 15px; margin: 10px 0; border-radius: 4px;">
                        <strong>"⚠ Update Available"</strong>
                        <p>"The device configuration has been updated externally."</p>
                        <Button on:click=on_adopt>"Adopt Changes"</Button>
                    </div>
                }
            })
        }}

        {move || {
            error_message.get().map(move|error| {
                view! {
                    <div style="background-color: #f8d7da; border: 1px solid #dc3545; color: #721c24; padding: 15px; margin: 10px 0; border-radius: 4px;">
                        <strong>"❌ Error"</strong>
                        <p>{error}</p>
                    </div>
                }
            })
        }}

        <div style="margin-top: 20px;">
            <Textarea
                value=edited_json
                size=TextareaSize::Large
                attr:style="width: 100%; height: 300px;"
            />
            <div style="margin-top: 10px;">
                <Button on:click=on_save>"Save"</Button>
            </div>
        </div>
    }
}
