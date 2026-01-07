use leptos::prelude::*;
use pilatus_leptos::{RecipeContext, RecipeInfo};
use thaw::{Button, ButtonSize};

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
        <tr style="border-bottom: 1px solid #e0e0e0;">
            <td style="padding: 12px;">
                <span style="font-weight: 500;">{move || recipe.read().id.to_string()}</span>
            </td>
            <td style="padding: 12px;">
                <RecipeTags recipe_memo=recipe />
            </td>
            <td style="padding: 12px;">
                <span style="color: #666;">
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
            <td style="padding: 12px;">
                <span
                    style:color=move || if is_active.get() { "#10b981" } else { "#6b7280" }
                    style:font-weight="600"
                >
                    {move || if is_active.get() { "● Active" } else { "○ Inactive" }}
                </span>
            </td>
            <td style="padding: 12px;">
                <div style="display: flex; flex-direction: column; gap: 4px; align-items: center;">
                    <div style="display: flex; gap: 8px; justify-content: center; align-items: center;">
                        <Button
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
                                view! {
                                    <span style="color: #dc3545; font-size: 12px;">{format!("Error: {}", e)}</span>
                                }
                            })
                        }}
                    </div>
                    <div style="display: flex; gap: 8px; justify-content: center; align-items: center;">
                        <Button
                            size=ButtonSize::Small
                            on:click=move |_| {
                                duplicate_action.dispatch(());
                            }
                        >
                            "Duplicate"
                        </Button>
                        {move || {
                            duplicate_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                view! {
                                    <span style="color: #dc3545; font-size: 12px;">{format!("Error: {}", e)}</span>
                                }
                            })
                        }}
                    </div>
                    <div style="display: flex; gap: 8px; justify-content: center; align-items: center;">
                        <Button
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
                                view! {
                                    <span style="color: #dc3545; font-size: 12px;">{format!("Error: {}", e)}</span>
                                }
                            })
                        }}
                    </div>
                </div>
            </td>
        </tr>
    }
}
