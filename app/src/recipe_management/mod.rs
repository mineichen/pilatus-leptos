mod recipe_row;
mod recipe_tags;

use leptos::prelude::*;
use pilatus_leptos::RecipeContext;
use thaw::Button;

use self::recipe_row::{RecipeRow, RecipeRowProps};

#[component]
pub fn RecipeManagement() -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();
    let ctx_create = ctx.clone();
    let ctx_commit = ctx.clone();

    // Get real recipes from context - returns Memo<Vec<RecipeInfo>>
    let recipes = ctx.list_recipes();
    let recipe_ids =
        move || recipes.with(|x| x.iter().map(|info| info.id.clone()).collect::<Vec<_>>());
    let has_active_changes = ctx.has_active_changes();

    let create_action = Action::new_local(move |_: &()| {
        let ctx = ctx_create.clone();
        async move { ctx.create_new_default_recipe().await }
    });

    let commit_action = Action::new_local(move |_: &()| {
        let ctx = ctx_commit.clone();
        async move { ctx.commit_changes().await }
    });

    view! {
        <div style="padding: 20px;">
        <h1 style="padding-bottom: 20px;">"Recipe Management"</h1>

        <div style="background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
            <table style="width: 100%; border-collapse: collapse;">
                <thead>
                    <tr style="border-bottom: 2px solid #e0e0e0;">
                        <th style="text-align: left; padding: 12px; font-weight: 600;">"Recipe Name"</th>
                        <th style="text-align: left; padding: 12px; font-weight: 600;">"Tags"</th>
                        <th style="text-align: left; padding: 12px; font-weight: 600;">"Created Date"</th>
                        <th style="text-align: left; padding: 12px; font-weight: 600;">"Status"</th>
                        <th style="text-align: center; padding: 12px; font-weight: 600;">"Actions"</th>
                    </tr>
                </thead>
                    <tbody>
                        <For
                            each=recipe_ids
                            key=|id| id.clone()
                            let(id)>
                            {
                                let recipe = Memo::new(move |_| {
                                    recipes.with(|x| x.iter().find(|x|x.id == id).cloned()).unwrap()
                                });

                                move || {
                                    RecipeRow(RecipeRowProps {
                                        recipe
                                    })

                                }
                            }
                        </For>
                    </tbody>
                </table>
                <div style="display: flex; gap: 8px; align-items: center; margin-top: 16px;">
                    <Button class="mt-4" on:click=move |_| {
                        create_action.dispatch(());
                    }>
                        "Create Recipe"
                    </Button>
                    {move || {
                        create_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                            view! {
                                <span style="color: #dc3545; font-size: 12px;">{format!("Error: {}", e)}</span>
                            }
                        })
                    }}
                    {move || {
                        if has_active_changes.get() {
                            view! {
                                <>
                                    <Button class="mt-4" on:click=move |_| {
                                        commit_action.dispatch(());
                                    }>
                                        "Commit changes"
                                    </Button>
                                    {move || {
                                        commit_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                                            view! {
                                                <span style="color: #dc3545; font-size: 12px;">{format!("Error: {}", e)}</span>
                                            }
                                        })
                                    }}
                                </>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
