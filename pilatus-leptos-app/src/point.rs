use leptos::prelude::*;
use pilatus_leptos_components::InputNumber;

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[component]
pub fn PointView(point: RwSignal<Point>) -> impl IntoView {
    let x = MappedSignal::new(point, |p: &Point| &p.x, |p: &mut Point| &mut p.x);
    let y = MappedSignal::new(point, |p: &Point| &p.y, |p: &mut Point| &mut p.y);

    view! {
        <div style="background-color: lightblue; padding: 20px;">
            <div>"X: " <InputNumber value=x step=1/></div>
            <div>"Y: " <InputNumber value=y step=1/></div>
            <div>"Point: (" {move || x.get()} ", " {move || y.get()} ")"</div>
        </div>
    }
}
