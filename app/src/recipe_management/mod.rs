mod recipe_row;
mod recipe_tags;

use leptos::prelude::*;
use pilatus_leptos::RecipeContext;

use self::recipe_row::RecipeRow;

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
                            let(recipe_id)>
                            >
                            <RecipeRow recipe=Memo::new({
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
                            })/>
                        </For>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
