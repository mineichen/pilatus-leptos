use std::borrow::Cow;

use futures_util::TryFutureExt;
use impex::Impex;
use leptos::prelude::*;
use pilatus_leptos::{RecipeContext, PilatusWrapperSettings, VariableInput};
use serde::{Deserialize, Serialize};
use thaw::{Button, Input};

#[derive(Deserialize, Serialize, PartialEq, Clone, Impex)]
#[impex(derive(PartialEq, Eq, Clone))]
#[serde(default)]
pub(crate) struct ParamsCopyRemoveMe {
    lang: String,
    sub: SubItem,
}
#[derive(Deserialize, Serialize, PartialEq, Clone, Default, Impex)]
#[impex(derive(PartialEq, Eq, Clone))]
struct SubItem {
    foo: i32,
}

impl Default for ParamsCopyRemoveMe {
    fn default() -> Self {
        Self {
            lang: "FakeItTillYouMakeIt".into(),
            sub: Default::default(),
        }
    }
}

#[component]
pub fn Greeter() -> impl IntoView {
    let device_context = expect_context::<RecipeContext>();
    let data = device_context
        .get_active_router_device::<ParamsCopyRemoveMeImpex<PilatusWrapperSettings>>();
    let lang = data.map_leaf(
        |x| x.lang.clone(),
        |target, prim_val| target.lang = prim_val,
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
            <p>"Language: " {move || lang.get_value()} </p>

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
