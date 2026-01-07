use leptos::prelude::*;
use thaw::SpinButton;

#[derive(Clone, Copy, Debug, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[component]
pub fn PointView(point: RwSignal<Point>) -> impl IntoView {
    //let x = leptos::reactive::computed::create_slice(point, move |x| x.x, move |x, n| x.x = n);
    let x = leptos::slice!(point.x);
    let y = leptos::slice!(point.y);

    Effect::new(move |_| {
        leptos::logging::log!("Point in Effect: {:?}", x.0.get());
        x.0.get()
    });

    view! {
        <div style="background-color: lightblue; padding: 20px;">
            <div>"X: " <SpinButton<i32> value=x step_page=1/></div>
            <div>"Y: " <SpinButton<i32> value=y step_page=1/></div>
            <div>"Point: (" {move || x.0.get()} ", " {move || y.0.get()} ")"</div>
        </div>
    }
}
