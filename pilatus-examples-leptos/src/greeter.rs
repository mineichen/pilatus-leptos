use std::borrow::Cow;

use futures_util::TryFutureExt;
use leptos::prelude::*;
use pilatus_leptos::{DeviceContext, PilatusWrapperSettings, VariableInput};
use pilatus_tick::GreeterParamsImpex;
use thaw::{Button, Input};

#[component]
pub fn Greeter() -> impl IntoView {
    let device_context = expect_context::<DeviceContext>();
    let data = device_context.get::<GreeterParamsImpex<PilatusWrapperSettings>>();
    let lang = data.map_leaf(
        |x| x.lang.clone(),
        |target, prim_val| {
            target.lang = prim_val;
        },
    );

    // leptos::reactive::computed::create_slice(signal, getter, setter)
    let name = RwSignal::new(String::from(""));

    let action = Action::new_local(move |name: &String| {
        //sub.write().foo += 1;
        let name = name.clone();
        async move {
            if name.is_empty() {
                return Err("Name mustn't be empty".into());
            }
            gloo_net::http::Request::get(&format!("/api/greeter/greet/{name}"))
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
        <div style="background-color: lightgreen; padding: 20px;">
            <h1>"I'm the friendly greeter!"</h1>
            <p>"Language: " {move || lang.get_value().to_string()} </p>

            <VariableInput
                value=lang
                label="Language".to_string()
            />

            <Input value=name placeholder="Enter your name"/>
            <Button on:click=move |_| { action.dispatch(name.get_untracked());}>"Say Hello"</Button>
            <hr/>

            { move|| if action.pending().get() { Cow::Borrowed( "Pending") } else { match  action.value().read().as_ref() {
                Some(Err(e)) => format!("Error: {e}").into(),
                Some(Ok(e)) => e.clone().into(),
                _ => "Never sent yet".into()
            } } }
        </div>
    }
}
