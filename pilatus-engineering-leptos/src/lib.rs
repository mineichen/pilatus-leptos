use leptos::html::Canvas;
use leptos::prelude::*;
use pilatus_leptos::JsonDeviceView;
use std::cell::Cell;

mod image_viewer;

#[component]
pub fn PilatusEngineeringView() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let started = Cell::new(false);
    // Create ImageViewer instance outside of Effect
    let image_viewer = std::rc::Rc::new(image_viewer::EframeImageViewer::new());

    // Use Effect::new to run once after mount
    // Use get_untracked() to avoid tracking canvas_ref reactively
    Effect::new({
        let image_viewer = image_viewer.clone();
        move |_| {
            if !started.get() {
                if let Some(canvas) = canvas_ref.get_untracked() {
                    started.set(true);
                    let canvas_element: web_sys::HtmlCanvasElement = canvas.into();
                    // Call start() with the canvas
                    let image_viewer = image_viewer.clone();
                    leptos::task::spawn_local(async move {
                        image_viewer.start(canvas_element).await;
                    });
                }
            }
        }
    });

    view! {
        <div>
            <h1>"Pilatus Engineering with canvas"</h1>
            <canvas
                node_ref=canvas_ref
                style="height: 500px;width: 100%; background-color: black;"
            />
            <JsonDeviceView/>
        </div>
    }
}
