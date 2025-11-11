use std::str::FromStr;

use leptos::prelude::*;
use pilatus::Name;
use pilatus_leptos::{RecipeContext, RecipeInfo};
use thaw::{Button, ButtonSize, Input, Tag};

#[component]
pub fn RecipeTags(recipe_memo: Memo<RecipeInfo>) -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();
    let new_tag_input = RwSignal::new(String::new());
    let recipe_id = move || recipe_memo.read().id.clone();

    let add_tag_to_recipe = Action::new_local(move |_: &()| {
        let ctx_clone = ctx.clone();
        let recipe_id = recipe_id.clone();
        async move {
            let Ok(tag_name) = Name::from_str(&new_tag_input.get()) else {
                leptos::logging::error!("Invalid tag name: {}", new_tag_input.get());
                return Err(anyhow::anyhow!("Invalid tag name: {}", new_tag_input.get()));
            };
            let r = ctx_clone
                .add_tag_to_recipe(recipe_id(), tag_name.clone())
                .await;
            new_tag_input.set(String::new());
            r
        }
    });

    view! {
        <div style="display: flex; flex-direction: column; gap: 8px;">
            <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                {move || {
                    recipe_memo
                        .get()
                        .recipe
                        .tags
                        .iter()
                        .map(|tag| {
                            let tag = tag.to_string();
                            view! { <Tag>{tag}</Tag> }
                        })
                        .collect_view()
                }}

            </div>
            <div style="display: flex; gap: 4px; align-items: center;">
                <Input
                    value=thaw_utils::Model::from(new_tag_input)
                    placeholder="New tag..."
                    attr:style="max-width: 150px;"
                />
                <Button
                    size=ButtonSize::Small
                    on:click=move |_| {
                        add_tag_to_recipe.dispatch(());
                    }
                >
                    "+ Add Tag"
                </Button>
            </div>
            <div style="display: flex; gap: 4px; align-items: center;">
                {move || {
                    add_tag_to_recipe
                        .value()
                        .read()
                        .as_ref()
                        .and_then(|r| r.as_ref().err().map(|e| format!("Error: {e}")))
                }}

            </div>
        </div>
    }
}
