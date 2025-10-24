use std::borrow::Cow;

use futures_util::{FutureExt, TryFutureExt};
use leptos::prelude::*;
use thaw::{Button, Input};

#[component]
pub fn Greeter() -> impl IntoView {
    let device_message = expect_context::<RwSignal<String>>();
    let name = RwSignal::new(String::from(""));

    let action = Action::new_local(|name: &String| {
        let name = name.clone();
        async move {
            gloo_net::http::Request::get(&format!("/api/greeter/greet/{name}"))
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
        }
    });

    view! {
        <div style="background-color: lightgreen; padding: 20px;">
            <h1>"I'm the friendly greeter!"</h1>
            <Input value=name placeholder="Enter your name"/>
            <Button on:click=move |_| { action.dispatch(name.get());}>"Say Hello"</Button>
            <hr/>

            { move|| if action.pending().get() { Cow::Borrowed( "Pending") } else { match  action.value().read().as_ref() {
                Some(Err(e)) => format!("Error: {e}").into(),
                Some(Ok(e)) => e.clone().into(),
                _ => "Never sent yet".into()
            } } }
            <hr/>
            <Input value=device_message />
        </div>
    }
}
