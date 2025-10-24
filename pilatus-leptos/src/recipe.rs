use crate::Point;
use leptos::prelude::*;
use thaw::{Button, SpinButton};

use crate::BusyButton;

#[component]
pub fn RecipeView() -> impl IntoView {
    view! {
        <Recipes let(x)>
            <PointView point=x/>
            <Button on:click=move |_| x.write().x += 1>"Increment"</Button>
            <BusyButton/>
        </Recipes>
    }
}
#[component]
pub fn PointView(point: RwSignal<Point>) -> impl IntoView {
    let x = leptos::slice!(point.x);
    let y = leptos::slice!(point.y);

    view! {
        <div style="background-color: lightblue; padding: 20px;">
            <div>"X: " <SpinButton<i32> value=x step_page=1/></div>
            <div>"Y: " <SpinButton<i32> value=y step_page=1/></div>
            <div>"Point: (" {move || x.0.get()} ", " {move || y.0.get()} ")"</div>
        </div>
    }
}

#[component]
pub fn Recipes<F, IV>(children: F) -> impl IntoView
where
    F: Fn(RwSignal<Point>) -> IV + 'static + Send,
    IV: IntoView + 'static,
{
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    leptos::logging::log!("READY COUNTER");

    let recipe = LocalResource::new(|| async {
        leptos::logging::log!("Start send request");
        gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
        let r = Result::<_, String>::Ok(
            gloo_net::http::Request::get("http://localhost:8080/api/recipe/get_all")
                .header("content-type", "application/json")
                .send()
                .await
                .map_err(|e| e.to_string())?
                .json::<pilatus::device::ActiveState>()
                .await
                .map(|r| r.recipes)
                .map_err(|e| e.to_string())?,
        );
        leptos::logging::log!("Result: {r:?}");
        r
    });
    let active = move || Some(recipe.get()?.ok()?.active().1.clone());
    let (res, _set_res) = signal(Result::<_, std::fmt::Error>::Ok(43));
    let scoped_value = RwSignal::new(Point { x: 0, y: 0 });
    Effect::new(move |prev| {
        let value = scoped_value.get();
        leptos::logging::log!("Value in Effect: {value:?}, prev: {prev:?}");
        value
    });
    view! {
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >
            {move|| {
                Some(format!("Foo: {}", recipe.get()?.ok()?.active().0))

            }}
            <Button on_click=on_click>"Number of Recipes?: " { count }</Button>
            <pre>
            { move || serde_json::to_string_pretty(&active()) }
            </pre>
            "After"
            <ErrorBoundary fallback = move|e| format!("Error: {e:?}")>
                <div>{res}</div>
            </ErrorBoundary>
            <pre>
            "Recipe: " { move|| Some(serde_json::to_string_pretty("foo")) }
            </pre>
            { move|| match recipe.get().as_ref() {
                Some(Ok(r)) => format!("{:?}", r.active().0),
                Some(Err(e)) => format!("Error: {e:?}").into(),
                None => "Not loaded".to_string()
            } }
            {children(scoped_value)}
        </Suspense>
    }
}
