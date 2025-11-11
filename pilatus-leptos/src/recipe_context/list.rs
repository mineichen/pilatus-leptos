use crate::{DeviceInfos, RecipeContext};
use leptos::prelude::*;
use pilatus::{Name, Recipe, RecipeId, RecipeMetadataRaw};

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
    pub async fn add_tag_to_recipe(
        &self,
        recipe_id: pilatus::RecipeId,
        tag_name: Name,
    ) -> Result<(), anyhow::Error> {
        let root = self.root;
        let root_value = root.get_untracked();
        let Some(recipe) = root_value.get_with_id(&recipe_id) else {
            return Err(anyhow::anyhow!("Recipe {recipe_id} doesn't exist"));
        };
        let mut tags = recipe.tags.clone();
        tags.push(tag_name);
        let url = format!("/api/recipe/{}/meta", recipe_id);

        let metadata = RecipeMetadataRaw {
            new_id: recipe_id.clone(),
            tags,
        }
        .seal()?;

        let result = async {
            gloo_net::http::Request::put(&url)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&metadata)?)?
                .send()
                .await
        }
        .await;

        match result {
            Ok(response) if response.ok() => {
                let refresh_result = gloo_net::http::Request::get("/api/recipe/get_all")
                    .header("content-type", "application/json")
                    .send()
                    .await;

                if let Ok(refresh_response) = refresh_result
                    && let Ok(state) = refresh_response
                        .json::<pilatus::device::ActiveState>()
                        .await
                {
                    leptos::logging::log!(
                        "Tag added successfully {:?}",
                        state
                            .recipes
                            .get_with_id(recipe_id)
                            .map(|r| r.tags.clone())
                            .unwrap_or_default()
                    );
                    root.set(state.recipes);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Failed to refresh recipes"))
                }
            }
            Ok(response) => Err(anyhow::anyhow!(
                "Failed to add tag: HTTP {}",
                response.status()
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to add tag: {}", e)),
        }
    }
}
