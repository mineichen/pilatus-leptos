use leptos::prelude::*;
use pilatus_leptos::RecipeInfo;
use thaw::{Button, ButtonSize};

use super::recipe_tags::RecipeTags;

#[component]
pub fn RecipeRow(recipe: Memo<RecipeInfo>) -> impl IntoView {
    let is_active = Signal::derive(move || recipe.read().is_active);
    let recipe_id_signal = move || recipe.read().id.clone();

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
                <div style="display: flex; gap: 8px; justify-content: center;">
                    <Button
                        size=ButtonSize::Small
                        disabled=is_active
                        on:click=move |_| {
                            leptos::logging::log!("Activate recipe (id: {:?})", recipe_id_signal());
                        }
                    >
                        "Activate"
                    </Button>
                    <Button
                        size=ButtonSize::Small
                        disabled=is_active
                        on:click=move |_| {
                            leptos::logging::log!("Delete recipe: (id: {:?})", recipe_id_signal());
                        }
                    >
                        "Delete"
                    </Button>

                </div>
            </td>
        </tr>
    }
}
