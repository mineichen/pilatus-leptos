use leptos::prelude::*;
use pilatus_leptos::{RecipeContext, RecipeInfo};
use thaw::{Button, ButtonAppearance, ButtonSize};

use super::recipe_tags::RecipeTags;

#[component]
pub fn RecipeRow(recipe: Memo<RecipeInfo>) -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();
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
        <tr class="border-b border-slate-700/50 hover:bg-slate-700/30 transition-colors">
            <td class="px-4 py-3">
                <span class="font-medium text-white">{move || recipe.read().id.to_string()}</span>
            </td>
            <td class="px-4 py-3">
                <RecipeTags recipe_memo=recipe />
            </td>
            <td class="px-4 py-3">
                <span class="text-slate-400">
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
                    if is_active.get() { "text-emerald-400 font-medium" } else { "text-slate-500" }
                }>
                    {move || if is_active.get() { "● Active" } else { "○ Inactive" }}
                </span>
            </td>
            <td class="px-4 py-3">
                <div class="flex flex-col gap-2">
                    <div class="flex gap-2 items-center">
                        <Button
                            appearance=ButtonAppearance::Primary
                            size=ButtonSize::Small
                            disabled=is_active
                            on:click=move |_| {
                                activate_action.dispatch(());
                            }
                        >
                            "Activate"
                        </Button>
                        {move || {
                            activate_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                view! { <span class="text-red-400 text-xs">{format!("Error: {}", e)}</span> }
                            })
                        }}
                    </div>
                    <div class="flex gap-2 items-center">
                        <Button
                            appearance=ButtonAppearance::Secondary
                            size=ButtonSize::Small
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
                        <Button
                            appearance=ButtonAppearance::Subtle
                            size=ButtonSize::Small
                            disabled=is_active
                            on:click=move |_| {
                                delete_action.dispatch(());
                            }
                        >
                            "Delete"
                        </Button>
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
                            class="inline-flex items-center justify-center px-3 py-1.5 text-sm rounded-md bg-slate-700 text-slate-300 hover:bg-slate-600 hover:text-white transition-colors"
                        >
                            "Export"
                        </a>
                    </div>
                </div>
            </td>
        </tr>
    }
}
