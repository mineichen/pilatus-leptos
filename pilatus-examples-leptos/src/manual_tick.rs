use leptos::prelude::*;
use pilatus_leptos::{DeviceContext, FetchApi, FetchResult, PilatusWrapperSettings};
use pilatus_leptos_components::{Button, InputNumber};
use pilatus_tick::ManualTickParamsImpex;

#[component]
pub fn ManualTick() -> impl IntoView {
    leptos::logging::log!("Create ManualTickComponent");
    let fetch: FetchApi = expect_context();
    let increment = Action::new_local(move |_| async move {
        FetchResult::Ok(
            fetch
                .put_json_silent("/api/pilatus-manual-tick/increment", ())
                .await?
                .text()
                .await?,
        )
    });

    let device_context: DeviceContext = expect_context();
    let params = device_context.get::<ManualTickParamsImpex<PilatusWrapperSettings>>();

    let initial_count = params.map_leaf(
        |x| x.initial_count.clone(),
        |target, prim_val| target.initial_count = prim_val,
    );

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-foreground mb-1">"Manual Tick"</h1>
                <p class="text-muted-foreground">"Increment counter manually"</p>
            </div>

            <div class="bg-card rounded-xl border border-border p-6">
                <h2 class="text-lg font-semibold text-foreground mb-4">"Counter Settings"</h2>

                <div class="flex items-center gap-4 mb-6">
                    <label class="text-foreground text-sm w-28">"Initial Count"</label>
                    <InputNumber value=initial_count step=1/>
                </div>

                <div class="flex items-center gap-4 pt-4 border-t border-border">
                    <Button
                        on:click=move |_| {
                            increment.dispatch(());
                        }
                    >
                        "Increment"
                    </Button>

                    <div class="text-muted-foreground text-sm">
                        {move || match increment.value().get() {
                            Some(Ok(count)) => view! {
                                <span class="text-success">"Count: " {count}</span>
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <span class="text-red-400">"Error: " {e.to_string() }</span>
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
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
            Content<br/>
        </div>
    }
}
