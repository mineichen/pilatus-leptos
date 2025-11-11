use futures_util::TryFutureExt;
use impex::Impex;
use leptos::prelude::*;
use pilatus_leptos::{PilatusWrapperSettings, RecipeContext};
use thaw::{Button, SpinButton};

#[derive(Impex)]
#[impex(derive(PartialEq, Eq, Clone))]
struct ManualTickParams {
    initial_count: i64,
}
impl Default for ManualTickParams {
    fn default() -> Self {
        Self { initial_count: 0 }
    }
}

#[component]
pub fn ManualTick() -> impl IntoView {
    let increment = Action::new_local(|_| async {
        gloo_net::http::Request::put("/api/manual/increment")
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

    let device_context = expect_context::<RecipeContext>();
    let params =
        device_context.get_active_router_device::<ManualTickParamsImpex<PilatusWrapperSettings>>();

    let initial_count = params.map_leaf(
        |x| x.initial_count.clone(),
        |target, prim_val| target.initial_count = prim_val,
    );

    view! {

        <div>"Initial count: " {move || initial_count.get()}</div>
        <SpinButton<i64> value=initial_count step_page=1/>
        <Button on:click=move |_| {
            increment.dispatch(());
        }>
            "Increment"
        </Button>
    }
}
