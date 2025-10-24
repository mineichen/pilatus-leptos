use crate::{Point, point::PointView};
use leptos::{prelude::*, task::spawn_local};
use pilatus::Recipes;
use thaw::Button;

use crate::BusyButton;

#[derive(Copy, Clone)]
struct RecipeStore {
    recipes: LocalResource<Result<Recipes, String>>,
}

impl RecipeStore {
    pub fn new() -> Self {
        Self {
            recipes: LocalResource::new(|| async {
                Ok(gloo_net::http::Request::get("/api/recipe/get_all")
                    .header("content-type", "application/json")
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .json::<pilatus::device::ActiveState>()
                    .await
                    .map_err(|e| e.to_string())?
                    .recipes)
            }),
        }
    }
    pub fn active(&self) -> Result<pilatus::Recipe, String> {
        Ok(self
            .recipes
            .get()
            .ok_or_else(|| "No recipes".to_string())??
            .active()
            .1
            .clone())
    }
}

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

use leptos_router::{components::Outlet, hooks::use_params, params::Params};

#[derive(PartialEq)]
struct DeviceParams {
    device_id: usize,
}

impl leptos_router::params::Params for DeviceParams {
    fn from_map(
        map: &leptos_router::params::ParamsMap,
    ) -> Result<Self, leptos_router::params::ParamsError> {
        Ok(DeviceParams {
            device_id: map
                .get("device_id")
                .and_then(|id| id.parse::<usize>().ok())
                .ok_or(leptos_router::params::ParamsError::MissingParam(
                    "device_id".to_string(),
                ))?,
        })
    }
}
#[component]
pub fn DeviceView() -> impl IntoView {
    let params = use_params::<DeviceParams>();
    let device_id = move || Some(params.read().as_ref().ok()?.device_id);

    view! {
        "Device: " { device_id }<br/>
        <Recipes let(x)>
            <PointView point=x/>
            <Button on:click=move |_| x.write().x += 1>"Increment"</Button>
            <BusyButton/>
            <Outlet/>
        </Recipes>
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

    let recipe_store = RecipeStore::new();
    let active = move || recipe_store.active();
    let (res, _set_res) = signal(Result::<_, std::fmt::Error>::Ok(43));
    let scoped_value = RwSignal::new(Point { x: 0, y: 0 });
    Effect::new(move |prev| {
        let value = scoped_value.get();
        leptos::logging::log!("Value in Effect: {value:?}, prev: {prev:?}");
        value
    });
    spawn_local(async move {
        for _ in 0..20 {
            gloo_timers::future::sleep(std::time::Duration::from_millis(1000)).await;
            scoped_value.write().x += 1;
        }
    });
    view! {
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >
            // {move|| {
            //     Some(format!("Foo: {}", active.ok()?.created))
            // }}


            <Button on_click=on_click>"Number of Recipes?: " { count }</Button>
            <pre>
            { move || Some(serde_json::to_string_pretty( &active().ok()?)) }
            </pre>
            "After"
            <ErrorBoundary fallback = move|e| format!("Error: {e:?}")>
                <div>{res}</div>
            </ErrorBoundary>
            // { move|| match recipes.get().as_ref() {
            //     Some(Ok(r)) => format!("{:?}", r.active().0),
            //     Some(Err(e)) => format!("Error: {e:?}").into(),
            //     None => "Not loaded".to_string()
            // } }
            {children(scoped_value)}
        </Suspense>
    }
}
