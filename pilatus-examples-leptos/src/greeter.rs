use std::{borrow::Cow, ops::Deref};

use futures_util::TryFutureExt;
use impex::{DefaultWrapperSettings, Impex};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use pilatus_leptos::{DeviceContext, DeviceParams, MapRwSignal};
use reactive_stores::Store;
use thaw::{Button, Field, Input};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Store, Impex)]
#[impex(derive(PartialEq, Eq, Clone))]
#[serde(default)]
pub(crate) struct ParamsCopyRemoveMe {
    lang: String,
    sub: SubItem,
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Store, Default, Impex)]
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
    let device_context = expect_context::<DeviceContext>();
    // let store = Store::new(ParamsCopyRemoveMe::default());
    // let store_field: Field<ParamsCopyRemoveMe> = store.into();
    // let sub = store.sub();
    // let sub: Field<SubItem> = sub.into();
    //

    // Effect::new(move || {
    //     leptos::logging::log!("Lang changed {}", store.lang().get());
    // });
    // Effect::new(move || {
    //     leptos::logging::log!("Sub changed {}", &sub.get().foo);
    // });
    let params = use_params::<DeviceParams>();
    let device_id = move || Some(params.read().as_ref().ok()?.device_id);
    let data =
        device_context.get::<ParamsCopyRemoveMeImpex>(Signal::derive(move || device_id().unwrap()));
    // let data = MapRwSignal::new(ParamsCopyRemoveMeImpex::<DefaultWrapperSettings>::default());
    let lang = data.map(
        |x| x.lang.deref().clone(),
        |target, value| target.lang.set_explicit(value),
    );

    // leptos::reactive::computed::create_slice(signal, getter, setter)
    let name = RwSignal::new(String::from(""));

    let action = Action::new_local(move |name: &String| {
        //sub.write().foo += 1;
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
            <p>"Language '" {move || lang.get().to_string()} "'" </p>
            <Field
                label = "Language"
            >
                <Input value=lang />
            </Field>

            <Input value=name placeholder="Enter your name"/>
            <Button on:click=move |_| { action.dispatch(name.get());}>"Say Hello"</Button>
            <hr/>

            { move|| if action.pending().get() { Cow::Borrowed( "Pending") } else { match  action.value().read().as_ref() {
                Some(Err(e)) => format!("Error: {e}").into(),
                Some(Ok(e)) => e.clone().into(),
                _ => "Never sent yet".into()
            } } }
        </div>
    }
}
