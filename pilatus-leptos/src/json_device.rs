use leptos::{either::Either, prelude::*};
use pilatus_leptos_components::{
    Alert, AlertDescription, AlertTitle, Button, ButtonVariant, Callout, DialogFooter, Textarea,
};

use crate::DeviceContext;

#[component]
pub fn JsonDeviceView(#[prop(optional)] on_close: Option<Callback<()>>) -> impl IntoView {
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
        let current_server = device_params.read();
        let last_saved = last_saved_value.read();
        &*current_server != &*last_saved
    });

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
            set_error_message.set(Some(clean_error.to_string()));
        }
    };

    let on_reset = move |_| {
        set_last_saved_value.set(device_params.get_untracked());
    };
    let on_adopt = move |_| {
        let current = device_params.get();
        edited_json.set(current.clone());
        set_last_saved_value.set(current);
    };

    view! {

        {move || {
            if has_external_update.get() {
                Either::Left(view! {
                    <Alert class="border-warning border-l-4 bg-warning-light dark:border-warning/50 dark:bg-warning-dark/20">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="size-4 text-warning"
                        >
                            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
                            <path d="M12 9v4" />
                            <path d="M12 17h.01" />
                        </svg>
                            <AlertTitle class="mb-2">"Update Avcurrent_serverailable"</AlertTitle>
                        <AlertDescription>
                            "The device configuration has been updated externally."
                        </AlertDescription>
                        <div class="mt-3 flex justify-end gap-2">
                            <Button variant=ButtonVariant::Ghost on:click=on_reset>"Ignore"</Button>
                            <Button on:click=on_adopt>"Reload"</Button>
                        </div>
                    </Alert>
                })
            } else {
                Either::Right(view! {
                    {move || {
                        error_message.get().map(move |error| {
                            view! {
                                <Callout title="Error" class="border-destructive/50 bg-destructive/5 dark:bg-destructive/10">
                                    {error}
                                </Callout>
                            }
                        })
                    }}
                    <Textarea
                        value=edited_json
                        rows=14
                        class="font-mono text-xs leading-relaxed"
                    />
                    <DialogFooter>
                        {on_close.map(|on_close| view! {
                            <Button variant=ButtonVariant::Ghost on:click=move |_| on_close.run(())>"Cancel"</Button>
                        })}
                        <Button on:click=on_save>"Save"</Button>
                    </DialogFooter>
                })
            }
        }}
    }
}
