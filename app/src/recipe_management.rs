use std::str::FromStr;

use leptos::prelude::*;
use pilatus::Name;
use pilatus_leptos::RecipeContext;
use thaw::{Button, ButtonSize, Input, Tag};

#[component]
pub fn RecipeManagement() -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();

    // Get real recipes from context
    let recipes = ctx.list_recipes();

    view! {
        <div style="padding: 20px;">
            <h1>"Recipe Management"</h1>
            <p style="color: #666; margin-bottom: 20px;">"Manage your recipes - activate, delete, and organize with tags"</p>

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
                            each=move || recipes.get()
                            key=|recipe_info| recipe_info.id.clone()
                            let:recipe_info
                        >
                            <tr style="border-bottom: 1px solid #e0e0e0;">
                                <td style="padding: 12px;">
                                    <span style="font-weight: 500;">{recipe_info.id.to_string()}</span>
                                </td>
                                <td style="padding: 12px;">
                                    {
                                        let recipe_id = recipe_info.id.clone();
                                        let new_tag_input = RwSignal::new(String::new());
                                        let ctx_clone = ctx.clone();

                                        let add_tag_to_recipe =
                                        Action::new_local(move |_: &()| {
                                            let ctx_clone = ctx_clone.clone();
                                            let recipe_id = recipe_id.clone();
                                            async move {
                                                let Ok(tag_name) = Name::from_str(&new_tag_input.get()) else {
                                                    leptos::logging::error!("Invalid tag name: {}", new_tag_input.get());
                                                    return Err(anyhow::anyhow!("Invalid tag name: {}", new_tag_input.get()));
                                                };
                                                let r = ctx_clone.add_tag_to_recipe(recipe_id.clone(), tag_name.clone()).await;
                                                new_tag_input.set(String::new());
                                                r
                                            }
                                        });

                                        view! {
                                            <div style="display: flex; flex-direction: column; gap: 8px;">
                                                <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                                                    <For
                                                        each=move || recipe_info.recipe.tags.clone()
                                                        key=|tag| tag.to_string()
                                                        let:tag
                                                    >
                                                        <Tag>
                                                            {tag.to_string()}
                                                        </Tag>
                                                    </For>
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
                                                        {move || add_tag_to_recipe.value().read().as_ref().and_then(|r| r.as_ref().err().map(|e| format!("Error: {e}")))}
                                                </div>
                                            </div>
                                        }
                                    }
                                </td>
                                <td style="padding: 12px;">
                                    <span style="color: #666;">
                                        {recipe_info.recipe.created.format("%Y-%m-%d %H:%M").to_string()}
                                    </span>
                                </td>
                                <td style="padding: 12px;">
                                    {if recipe_info.is_active {
                                        view! {
                                            <span style="color: #10b981; font-weight: 600;">"● Active"</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span style="color: #6b7280; font-weight: 500;">"○ Inactive"</span>
                                        }.into_any()
                                    }}
                                </td>
                                <td style="padding: 12px;">
                                    <div style="display: flex; gap: 8px; justify-content: center;">
                                        {
                                            let recipe_id_str = recipe_info.id.to_string();
                                            let recipe_id_activate = recipe_info.id.clone();
                                            let recipe_id_delete = recipe_info.id.clone();
                                            let is_active = recipe_info.is_active;

                                            view! {
                                                <>
                                                    {if is_active {
                                                        view! {
                                                            <Button
                                                                size=ButtonSize::Small
                                                                disabled=true
                                                            >
                                                                "Activate"
                                                            </Button>
                                                        }.into_any()
                                                    } else {
                                                        let id_str_for_activate = recipe_id_str.clone();
                                                        view! {
                                                            <Button
                                                                size=ButtonSize::Small
                                                                on:click=move |_| {
                                                                    leptos::logging::log!("Activate recipe: {} (id: {:?})", id_str_for_activate, recipe_id_activate);
                                                                    // Logic will be added later

                                                                }
                                                            >
                                                                "Activate"
                                                            </Button>
                                                        }.into_any()
                                                    }}

                                                    <Button
                                                        size=ButtonSize::Small
                                                        disabled=is_active
                                                        on:click=move |_| {
                                                            leptos::logging::log!("Delete recipe: {} (id: {:?})", recipe_id_str, recipe_id_delete);
                                                            // Logic will be added later
                                                        }
                                                    >
                                                        "Delete"
                                                    </Button>
                                                </>
                                            }
                                        }
                                    </div>
                                </td>
                            </tr>
                        </For>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
