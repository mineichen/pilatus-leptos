use futures_util::TryFutureExt;
use leptos::prelude::*;
use thaw::Button;

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

    view! {

        <h1>"ManualTick"</h1>
        <Button on:click=move |_| {
            increment.dispatch(());
        }>
            "Increment"
        </Button>
    }
}
