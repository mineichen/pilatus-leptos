use std::str::FromStr;

use leptos::prelude::*;
use pilatus::Name;
use pilatus_leptos::{RecipeContext, RecipeInfo};
use thaw::{Button, ButtonSize, Input, Tag};

#[component]
pub fn RecipeTags(recipe_memo: Memo<RecipeInfo>) -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();
    let ctx_remove = ctx.clone();
    let new_tag_input = RwSignal::new(String::new());
    let recipe_id = Signal::derive(move || recipe_memo.read().id.clone());
    let (tag_to_remove, set_tag_to_remove) = signal::<Option<Name>>(None);

    let add_tag_to_recipe = Action::new_local(move |_: &()| {
        let ctx = ctx.clone();
        let recipe_id = recipe_id.clone();
        async move {
            let Ok(tag_name) = Name::from_str(&new_tag_input.get()) else {
                leptos::logging::error!("Invalid tag name: {}", new_tag_input.get());
                return Err(anyhow::anyhow!("Invalid tag name: {}", new_tag_input.get()));
            };
            let r = ctx
                .add_tag_to_recipe(recipe_id.get_untracked(), tag_name.clone())
                .await;
            new_tag_input.set(String::new());
            r
        }
    });

    let remove_tag_action = Action::new_local(move |tag_name: &Name| {
        let ctx = ctx_remove.clone();
        let recipe_id = recipe_id.get_untracked();
        let tag_name = tag_name.clone();
        async move {
            let r = ctx.remove_tag_from_recipe(recipe_id, tag_name).await;
            if r.is_ok() {
                set_tag_to_remove.set(None);
            }
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
                            let tag_name = tag.clone();
                            let tag_str = tag.to_string();
                            view! {
                                <Tag>
                                    <span style="display: inline-flex; align-items: center; gap: 4px;">
                                        <span>{tag_str.clone()}</span>
                                        <button
                                            style="background: none; border: none; color: #dc3545; cursor: pointer; font-size: 14px; padding: 0; margin-left: 4px; line-height: 1; display: inline-flex; align-items: center;"
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                set_tag_to_remove.set(Some(tag_name.clone()));
                                            }
                                        >
                                            "×"
                                        </button>
                                    </span>
                                </Tag>
                            }
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
                {move || {
                    remove_tag_action
                        .value()
                        .read()
                        .as_ref()
                        .and_then(|r| r.as_ref().err().map(|e| format!("Error: {e}")))
                }}

            </div>
            {move || {
                tag_to_remove.get().map(|tag_name| {
                    let tag_str = tag_name.to_string();
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; padding: 20px; border-radius: 8px; min-width: 300px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);">
                                <h3 style="margin-top: 0;">"Remove Tag?"</h3>
                                <p style="margin: 16px 0;">{format!("Are you sure you want to remove the tag \"{}\"?", tag_str)}</p>
                                <div style="margin-top: 20px; display: flex; gap: 8px; justify-content: flex-end;">
                                    <Button on:click=move |_| set_tag_to_remove.set(None)>
                                        "Cancel"
                                    </Button>
                                    <div style="background-color: #dc3545; border-color: #dc3545;">
                                        <Button
                                            on:click=move |_| {
                                                remove_tag_action.dispatch(tag_name.clone());
                                            }
                                        >
                                            "Remove"
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}
