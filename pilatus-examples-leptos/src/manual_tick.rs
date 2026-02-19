use futures_util::TryFutureExt;
use leptos::prelude::*;
use pilatus_leptos::{DeviceContext, PilatusWrapperSettings};
use pilatus_tick::ManualTickParamsImpex;
use thaw::{Button, ButtonAppearance, SpinButton};

#[component]
pub fn ManualTick() -> impl IntoView {
    leptos::logging::log!("Create ManualTickComponent");
    let increment = Action::new_local(|_| async {
        gloo_net::http::Request::put("/api/pilatus-manual-tick/increment")
            .send()
            .map_err(|e| e.to_string())
            .and_then(|r| async move {
                if r.status() == 200 {
                    r.text().await.map_err(|e| e.to_string())
                } else {
                    Err(match r.text().await.as_deref() {
                        Ok("") | Err(_) => format!("Couldn't get Body: {}", r.status()),
                        Ok(body) => format!("Error: {body}"),
                    })
                }
            })
            .await
    });

    let device_context = expect_context::<DeviceContext>();
    let params = device_context.get::<ManualTickParamsImpex<PilatusWrapperSettings>>();

    let initial_count = params.map_leaf(
        |x| x.initial_count.clone(),
        |target, prim_val| target.initial_count = prim_val,
    );

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-white mb-1">"Manual Tick"</h1>
                <p class="text-slate-400">"Increment counter manually"</p>
            </div>

            <div class="bg-slate-800 rounded-xl border border-slate-700 p-6">
                <h2 class="text-lg font-semibold text-white mb-4">"Counter Settings"</h2>
                
                <div class="flex items-center gap-4 mb-6">
                    <label class="text-slate-300 text-sm w-28">"Initial Count"</label>
                    <SpinButton<u32> value=initial_count step_page=1/>
                </div>

                <div class="flex items-center gap-4 pt-4 border-t border-slate-700">
                    <Button
                        appearance=ButtonAppearance::Primary
                        on:click=move |_| {
                            increment.dispatch(());
                        }
                    >
                        "Increment"
                    </Button>
                    
                    <div class="text-slate-400 text-sm">
                        {move || match increment.value().get() {
                            Some(Ok(count)) => view! {
                                <span class="text-emerald-400">"Count: " {count}</span>
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <span class="text-red-400">"Error: " {e}</span>
                            }.into_any(),
                            None => "".into_any(),
                        }}
                    </div>
                </div>

                {move || increment.pending().get().then(|| view! {
                    <div class="mt-4 px-3 py-2 bg-amber-900/50 border border-amber-700 rounded-lg text-amber-300 text-sm">
                        "Processing..."
                    </div>
                })}
            </div>
        </div>
    }
}
