use crate::{DeviceInfos, RecipeContext};
use leptos::prelude::*;
use pilatus::{Name, Recipe, RecipeId};

#[derive(Clone, Debug)]
pub struct RecipeInfo {
    pub id: RecipeId,
    pub recipe: Recipe,
    pub is_active: bool,
}

impl RecipeContext {
    pub fn list_devices(&self) -> Signal<Vec<DeviceInfos>> {
        let root = self.root.read_only();
        Signal::derive(move || {
            root.with(|recipes| {
                let (_, active) = recipes.active();
                active
                    .devices
                    .iter()
                    .map(|(&device_id, device)| DeviceInfos {
                        name: device.device_name.clone(),
                        device_id,
                        device_type: device.device_type.clone(),
                    })
                    .collect()
            })
        })
    }

    pub fn list_recipes(&self) -> Signal<Vec<RecipeInfo>> {
        let root = self.root.read_only();
        Signal::derive(move || {
            root.with(|recipes| {
                let (active_id, _) = recipes.active();
                recipes
                    .iter_without_backup()
                    .map(|(id, recipe)| RecipeInfo {
                        id: id.clone(),
                        recipe: recipe.clone(),
                        is_active: id == &active_id,
                    })
                    .collect()
            })
        })
    }

    /// Add a tag to a specific recipe
    /// This sends an API request to update the recipe on the server
    pub fn add_tag_to_recipe(&self, recipe_id: pilatus::RecipeId, tag_name: Name) {
        let root = self.root;

        leptos::task::spawn_local(async move {
            let url = format!("/api/recipe/{}/meta", recipe_id);

            let result = async {
                gloo_net::http::Request::put(&url)
                    .header("content-type", "application/json")
                    .body(serde_json::json!({ "tag": tag_name }).to_string())?
                    .send()
                    .await
            }
            .await;

            match result {
                Ok(response) if response.ok() => {
                    leptos::logging::log!(
                        "Successfully added tag '{}' to recipe {:?}",
                        tag_name,
                        recipe_id
                    );

                    // Refresh the recipes from the server
                    let refresh_result = gloo_net::http::Request::get("/api/recipe/get_all")
                        .header("content-type", "application/json")
                        .send()
                        .await;

                    if let Ok(refresh_response) = refresh_result {
                        if let Ok(state) = refresh_response
                            .json::<pilatus::device::ActiveState>()
                            .await
                        {
                            root.set(state.recipes);
                        }
                    }
                }
                Ok(response) => {
                    leptos::logging::error!("Failed to add tag: HTTP {}", response.status());
                }
                Err(e) => {
                    leptos::logging::error!("Failed to add tag: {}", e);
                }
            }
        });
    }
}
