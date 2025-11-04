use crate::BusyButton;
use crate::point::Point;
use crate::point::PointView;
use leptos::prelude::*;
use thaw::Button;

#[component]
pub fn HomeView() -> impl IntoView {
    let point = RwSignal::new(Point { x: 0, y: 42 });
    view! {
        <h1>"Home"</h1>
        <PointView point=point />
        <Button on:click=move |_| point.write().x += 1>"Increment"</Button>
        <BusyButton/>
    }
}
