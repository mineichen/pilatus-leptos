use leptos::prelude::*;
use pilatus::{PendingChangesStrategy, RecipeId};
use pilatus_leptos::RecipeContext;
use pilatus_leptos_components::{
    Button, ButtonVariant, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};

#[component]
pub fn SwitchDialogContent(
    target: RecipeId,
    show: RwSignal<bool>,
) -> impl IntoView {
    let ctx: RecipeContext = expect_context();
    let description = format!(
        "The active recipe has uncommitted changes. What should happen before switching to '{target}'?"
    );

    let switch_action = Action::new_local(move |strategy: &PendingChangesStrategy| {
        let ctx = ctx.clone();
        let target = target.clone();
        let strategy = *strategy;
        async move {
            ctx.start_recipe(target, strategy).await?;
            show.set(false);
            Ok::<_, anyhow::Error>(())
        }
    });

    view! {
        <DialogHeader>
            <DialogTitle>"Uncommitted changes"</DialogTitle>
            <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        {move || {
            switch_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                view! {
                    <div class="px-4 py-2 rounded-lg bg-red-900/50 border border-red-700 text-red-200">
                        {format!("Error: {}", e)}
                    </div>
                }
            })
        }}
        <DialogFooter>
            <Button
                variant=ButtonVariant::Outline
                attr:disabled=move || switch_action.pending().get()
                on:click=move |_| show.set(false)
            >
                "Don't switch"
            </Button>
            <Button
                variant=ButtonVariant::Destructive
                attr:disabled=move || switch_action.pending().get()
                on:click=move |_| {
                    switch_action.dispatch(PendingChangesStrategy::Discard);
                }
            >
                "Discard changes"
            </Button>
            <Button
                attr:disabled=move || switch_action.pending().get()
                on:click=move |_| {
                    switch_action.dispatch(PendingChangesStrategy::Commit);
                }
            >
                "Commit changes"
            </Button>
        </DialogFooter>
    }
}
