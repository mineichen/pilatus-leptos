use std::str::FromStr;

use leptos::prelude::*;
use pilatus_leptos_components::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Input};
use pilatus::Name;
use pilatus_leptos::{RecipeContext, RecipeInfo};

#[component]
pub fn RecipeTags(recipe_memo: Memo<RecipeInfo>) -> impl IntoView {
    let ctx: RecipeContext = expect_context();
    let ctx_remove = ctx.clone();
    let new_tag_input = RwSignal::new(String::new());
    let recipe_id = Signal::derive(move || recipe_memo.read().id.clone());
    let (tag_to_remove, set_tag_to_remove) = signal::<Option<Name>>(None);

    let add_tag_to_recipe = Action::new_local(move |_: &()| {
        let ctx = ctx.clone();
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
        <div class="flex flex-col gap-2">
            <div class="flex flex-wrap gap-1">
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
                                <Badge variant=BadgeVariant::Secondary>
                                    <span class="inline-flex items-center gap-1">
                                        <span>{tag_str.clone()}</span>
                                        <span title=format!("Remove tag {}", tag_str)>
                                            <Button
                                                variant=ButtonVariant::Destructive
                                                size=ButtonSize::IconSm
                                                on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    set_tag_to_remove.set(Some(tag_name.clone()));
                                                }
                                            >
                                                "×"
                                            </Button>
                                        </span>
                                    </span>
                                </Badge>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <div class="flex gap-2 items-center bg-card ">
                <Input
                    value=new_tag_input
                    placeholder="Enter new tag"
                />
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on:click=move |_| {
                        add_tag_to_recipe.dispatch(());
                    }
                >
                    "+ Add"
                </Button>
            </div>
            <div class="text-red-400 text-xs">
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
                        <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                            <div class="bg-card rounded-xl p-6 min-w-[300px] border border-border shadow-xl">
                                <h3 class="text-lg font-semibold text-foreground mt-0">"Remove Tag?"</h3>
                                <p class="text-muted-foreground my-4">{format!("Remove tag \"{}\"?", tag_str)}</p>
                                <div class="mt-4 flex gap-2 justify-end">
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on:click=move |_| set_tag_to_remove.set(None)
                                    >
                                        "Cancel"
                                    </Button>
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
                    }
                })
            }}
        </div>
    }
}
