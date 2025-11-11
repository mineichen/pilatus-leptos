mod recipe_tags;

use leptos::prelude::*;
use pilatus_leptos::RecipeContext;
use thaw::{Button, ButtonSize};

use self::recipe_tags::RecipeTags;

#[component]
pub fn RecipeManagement() -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();

    // Get real recipes from context - returns Memo<Vec<RecipeInfo>>
    let recipes = ctx.list_recipes();
    let recipe_ids = move || {
        recipes
            .get()
            .iter()
            .map(|info| info.id.clone())
            .collect::<Vec<_>>()
    };

    view! {
        <div style="padding: 20px;">
        <h1>"Recipe Management"</h1>
        <p style="color: #666; margin-bottom: 20px;">
            "Manage your recipes - activate, delete, and organize with tags"
        </p>

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
                        key=|recipe_id| recipe_id.clone()
                        children=move |recipe_id| {
                            // Create a stable Memo for this specific recipe
                            // This Memo will persist across For loop re-renders and track changes to the recipe
                            let recipe_memo = Memo::new({
                                let recipes = recipes.clone();
                                let recipe_id = recipe_id.clone();
                                move |_| {
                                    recipes
                                    .get()
                                    .iter()
                                    .find(|info| info.id == recipe_id)
                                    .unwrap()
                                    .clone()
                                }
                            });
                            let is_active = Signal::derive(move || recipe_memo.with(|x| x.is_active));
                            let recipe_id = move || recipe_memo.read().id.clone();

                            view! {
                                <tr style="border-bottom: 1px solid #e0e0e0;">
                                    <td style="padding: 12px;">
                                        <span style="font-weight: 500;">
                                            {move || recipe_memo.read().id.to_string()}
                                        </span>
                                    </td>
                                    <td style="padding: 12px;">
                                        <RecipeTags recipe_memo=recipe_memo />
                                    </td>
                                    <td style="padding: 12px;">
                                        <span style="color: #666;">
                                            {move || {
                                                recipe_memo
                                                    .read()
                                                    .recipe
                                                    .created
                                                    .format("%Y-%m-%d %H:%M")
                                                    .to_string()
                                            }}

                                        </span>
                                    </td>
                                    <td style="padding: 12px;">
                                        {is_active.get()
                                            .then_some(|| {
                                                view! {
                                                    <span style="color: #10b981; font-weight: 600;">"● Active"</span>
                                                }
                                            })}
                                        {(!is_active.get())
                                            .then_some(|| {
                                                view! {
                                                    <span style="color: #6b7280; font-weight: 500;">"○ Inactive"</span>
                                                }
                                            })}

                                    </td>
                                    <td style="padding: 12px;">
                                        <div style="display: flex; gap: 8px; justify-content: center;">
                                            <Button
                                                size=ButtonSize::Small
                                                disabled=is_active
                                                on:click=move |_| {
                                                    leptos::logging::log!(
                                                        "Activate recipe (id: {:?})", recipe_id(),

                                                    );
                                                }
                                            >
                                                "Activate"
                                            </Button>
                                            <Button
                                                size=ButtonSize::Small
                                                disabled=is_active
                                                on:click=move |_| {
                                                    leptos::logging::log!(
                                                        "Delete recipe: (id: {:?})", recipe_id(),
                                                    );
                                                }
                                            >
                                                "Delete"
                                            </Button>

                                        </div>
                                    </td>
                                </tr>
                            }
                    }
                />
                    </tbody>
                </table>
            </div>
        </div>
    }
}
