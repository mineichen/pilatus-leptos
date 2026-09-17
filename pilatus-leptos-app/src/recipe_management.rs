mod recipe_import;
mod recipe_row;
mod recipe_tags;

use leptos::prelude::*;
use pilatus_leptos_components::{Button, ButtonVariant};
use pilatus_leptos::RecipeContext;

use self::recipe_import::RecipeImport;
use self::recipe_row::{RecipeRow, RecipeRowProps};

#[component]
pub fn RecipeManagement() -> impl IntoView {
    let ctx: RecipeContext = expect_context();
    let ctx_create = ctx.clone();
    let ctx_commit = ctx.clone();

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
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-foreground mb-1">"Recipe Management"</h1>
                <p class="text-muted-foreground">"Create and manage recipes for your devices."</p>
            </div>

            <div class="bg-card rounded-xl border border-border">
                <div class="p-4 border-b border-border flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-foreground">"Recipes"</h2>
                    <div class="flex gap-2">
                        <RecipeImport />
                        <Button
                            on:click=move |_| {
                                create_action.dispatch(());
                            }
                        >
                            "Create Recipe"
                        </Button>
                        {move || {
                            if has_active_changes.get() {
                                view! {
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on:click=move |_| {
                                            commit_action.dispatch(());
                                        }
                                    >
                                        "Commit Changes"
                                    </Button>
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    </div>
                </div>

                {move || {
                    create_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                        view! {
                            <div class="mx-4 mt-4 px-4 py-2 rounded-lg bg-red-900/50 border border-red-700 text-red-200">
                                {format!("Error: {}", e)}
                            </div>
                        }
                    })
                }}

                {move || {
                    commit_action.value().read().as_ref().and_then(|result| result.as_ref().err()).map(|e| {
                        view! {
                            <div class="mx-4 mt-4 px-4 py-2 rounded-lg bg-red-900/50 border border-red-700 text-red-200">
                                {format!("Error: {}", e)}
                            </div>
                        }
                    })
                }}

                <div class="overflow-x-auto">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-border">
                                <th class="text-left px-4 py-3 text-sm font-medium text-muted-foreground">"Name"</th>
                                <th class="text-left px-4 py-3 text-sm font-medium text-muted-foreground">"Tags"</th>
                                <th class="text-left px-4 py-3 text-sm font-medium text-muted-foreground">"Created"</th>
                                <th class="text-left px-4 py-3 text-sm font-medium text-muted-foreground">"Status"</th>
                                <th class="text-left px-4 py-3 text-sm font-medium text-muted-foreground">"Actions"</th>
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
                </div>
            </div>
        </div>
    }
}
