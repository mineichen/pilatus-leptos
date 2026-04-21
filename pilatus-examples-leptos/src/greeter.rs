use futures_util::TryFutureExt;
use leptos::prelude::*;
use pilatus_leptos::{DeviceContext, PilatusWrapperSettings, VariableInput};
use pilatus_tick::GreeterParamsImpex;
use thaw::{Button, ButtonAppearance, Input};

#[component]
pub fn Greeter() -> impl IntoView {
    leptos::logging::log!("Create GreeterComponent");
    let device_context = expect_context::<DeviceContext>();
    let data = device_context.get::<GreeterParamsImpex<PilatusWrapperSettings>>();
    let lang = data.map_leaf(
        |x| x.lang.clone(),
        |target, prim_val| {
            target.lang = prim_val;
        },
    );

    let name = RwSignal::new(String::from(""));

    let action = Action::new_local(move |name: &String| {
        let name = name.clone();
        async move {
            if name.is_empty() {
                return Err("Name mustn't be empty".into());
            }
            gloo_net::http::Request::get(&format!("/api/pilatus-greeter/greet/{name}"))
                .send()
                .map_err(|e| e.to_string())
                .and_then(|r| async move {
                    if r.ok() {
                        r.text().await.map_err(|e| e.to_string())
                    } else {
                        Err(match r.text().await.as_deref() {
                            Ok("") | Err(_) => format!("Couldn't get Body: {}", r.status()),
                            Ok(body) => format!("Error: {body}"),
                        })
                    }
                })
                .await
        }
    });

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-white mb-1">"Greeter"</h1>
                <p class="text-slate-400">"Send a friendly greeting"</p>
            </div>

            <div class="bg-slate-800 rounded-xl border border-slate-700 p-6">
                <h2 class="text-lg font-semibold text-white mb-4">"Settings"</h2>

                <div class="mb-6">
                    <label class="text-slate-300 text-sm block mb-2">"Language"</label>
                    <VariableInput
                        value=lang
                        label="Language".to_string()
                    />
                </div>
            </div>

            <div class="bg-slate-800 rounded-xl border border-slate-700 p-6">
                <h2 class="text-lg font-semibold text-white mb-4">"Send Greeting"</h2>

                <div class="flex gap-3 mb-4">
                    <div class="flex-1">
                        <Input
                            value=name
                            placeholder="Enter your name"
                        />
                    </div>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on:click=move |_| {
                            action.dispatch(name.get_untracked());
                        }
                    >
                        "Say Hello"
                    </Button>
                </div>

                {move || action.pending().get().then(|| view! {
                    <div class="px-3 py-2 bg-amber-900/50 border border-amber-700 rounded-lg text-amber-300 text-sm">
                        "Sending greeting..."
                    </div>
                })}

                {move || match action.value().read().as_ref() {
                    Some(Err(e)) => view! {
                        <div class="px-3 py-2 bg-red-900/50 border border-red-700 rounded-lg text-red-300 text-sm">
                            "Error: " {e.clone()}
                        </div>
                    }.into_any(),
                    Some(Ok(greeting)) => view! {
                        <div class="px-4 py-3 bg-emerald-900/50 border border-emerald-700 rounded-lg">
                            <p class="text-emerald-300 text-lg font-medium">{greeting.clone()}</p>
                        </div>
                    }.into_any(),
                    None => "".into_any(),
                }}
            </div>
        </div>
    }
}
