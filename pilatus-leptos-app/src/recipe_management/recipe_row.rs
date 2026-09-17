use leptos::prelude::*;
use pilatus_leptos_components::{Button, ButtonClass, ButtonSize, ButtonVariant, IntoTailwindClass};
use pilatus_leptos::{RecipeContext, RecipeInfo};

use super::recipe_tags::RecipeTags;

#[component]
pub fn RecipeRow(recipe: Memo<RecipeInfo>) -> impl IntoView {
    let ctx: RecipeContext = expect_context();
    let ctx_activate = ctx.clone();
    let ctx_duplicate = ctx.clone();
    let ctx_delete = ctx.clone();
    let is_active = Signal::derive(move || recipe.read().is_active);
    let recipe_id_signal = move || recipe.read().id.clone();

    let activate_action = Action::new_local(move |_: &()| {
        let ctx = ctx_activate.clone();
        let recipe_id = recipe_id_signal();
        async move { ctx.activate_recipe(recipe_id).await }
    });

    let duplicate_action = Action::new_local(move |_: &()| {
        let ctx = ctx_duplicate.clone();
        let recipe_id = recipe_id_signal();
        async move { ctx.duplicate_recipe(recipe_id).await }
    });

    let delete_action = Action::new_local(move |_: &()| {
        let ctx = ctx_delete.clone();
        let recipe_id = recipe_id_signal();
        async move { ctx.delete_recipe(recipe_id).await }
    });

    view! {
        <tr class="border-b border-border/50 hover:bg-muted/30 transition-colors">
            <td class="px-4 py-3">
                <span class="font-medium text-foreground">{move || recipe.read().id.to_string()}</span>
            </td>
            <td class="px-4 py-3">
                <RecipeTags recipe_memo=recipe />
            </td>
            <td class="px-4 py-3">
                <span class="text-muted-foreground">
                    {move || {
                        recipe
                            .read()
                            .recipe
                            .created
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    }}
                </span>
            </td>
            <td class="px-4 py-3">
                <span class=move || {
                    if is_active.get() { "text-success font-medium" } else { "text-muted-foreground" }
                }>
                    {move || if is_active.get() { "● Active" } else { "○ Inactive" }}
                </span>
            </td>
            <td class="px-4 py-3">
                <div class="flex flex-col gap-2">
                    <div class="flex gap-2 items-center">
                        <span class=move || {
                            if is_active.get() { "pointer-events-none opacity-50" } else { "" }
                        }>
                            <Button
                                size=ButtonSize::Sm
                                on:click=move |_| {
                                    activate_action.dispatch(());
                                }
                            >
                                "Activate"
                            </Button>
                        </span>
                        {move || {
                            activate_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                view! { <span class="text-red-400 text-xs">{format!("Error: {}", e)}</span> }
                            })
                        }}
                    </div>
                    <div class="flex gap-2 items-center">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on:click=move |_| {
                                duplicate_action.dispatch(());
                            }
                        >
                            "Duplicate"
                        </Button>
                        {move || {
                            duplicate_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                view! { <span class="text-red-400 text-xs">{format!("Error: {}", e)}</span> }
                            })
                        }}
                    </div>
                    <div class="flex gap-2 items-center">
                        <span class=move || {
                            if is_active.get() { "pointer-events-none opacity-50" } else { "" }
                        }>
                            <Button
                                variant=ButtonVariant::Destructive
                                size=ButtonSize::Sm
                                on:click=move |_| {
                                    delete_action.dispatch(());
                                }
                            >
                                "Delete"
                            </Button>
                        </span>
                        {move || {
                            delete_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                view! { <span class="text-red-400 text-xs">{format!("Error: {}", e)}</span> }
                            })
                        }}
                    </div>
                    <div class="flex gap-2 items-center">
                        <a
                            href=move || format!("/api/recipe/{}/export", recipe.read().id)
                            target="_blank"
                            class=move || {
                                ButtonClass {
                                    variant: ButtonVariant::Secondary,
                                    size: ButtonSize::Sm,
                                }
                                    .with_class("")
                            }
                        >
                            "Export"
                        </a>
                    </div>
                </div>
            </td>
        </tr>
    }
}
