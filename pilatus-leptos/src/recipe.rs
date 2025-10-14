use leptos::prelude::*;
use thaw::Button;

#[component]
pub fn RecipeView() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    leptos::logging::log!("READY COUNTER");

    view! {
        <Button on_click=on_click>"Number of Recipes?: " {count}</Button>
    }
}
