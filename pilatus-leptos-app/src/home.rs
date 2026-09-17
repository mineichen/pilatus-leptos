use crate::busy_button::BusyButton;
use crate::point::Point;
use crate::point::PointView;
use leptos::prelude::*;
use pilatus_leptos_components::Button;

#[component]
pub fn HomeView() -> impl IntoView {
    let point = RwSignal::new(Point { x: 0, y: 42 });

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-foreground mb-1">"Dashboard"</h1>
                <p class="text-muted-foreground">"Select a device from the sidebar to begin."</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <div class="bg-card rounded-xl p-6 border border-border">
                    <h3 class="text-lg font-semibold text-foreground mb-4">"Point Coordinates"</h3>
                    <PointView point=point />
                    <div class="mt-4">
                        <Button on:click=move |_| point.write().x += 1>"Increment X"</Button>
                    </div>
                </div>

                <div class="bg-card rounded-xl p-6 border border-border">
                    <h3 class="text-lg font-semibold text-foreground mb-4">"Async Demo"</h3>
                    <BusyButton />
                </div>
            </div>
        </div>
    }
}
