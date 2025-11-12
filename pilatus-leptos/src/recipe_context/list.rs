use crate::{DeviceInfos, RecipeContext};
use leptos::prelude::*;
use pilatus::{Name, Recipe, RecipeId, RecipeMetadataRaw};

#[derive(Clone, Debug, PartialEq)]
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

    pub fn list_recipes(&self) -> Memo<Vec<RecipeInfo>> {
        let root = self.root.read_only();

        Memo::new(move |_| {
            root.with(|recipes| {
                let (active_id, _) = recipes.active();
                recipes
                    .iter_without_backup()
                    .map(|(id, recipe)| {
                        let is_active = &active_id == id;

                        RecipeInfo {
                            id: id.clone(),
                            recipe: recipe.clone(),
                            is_active,
                        }
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
            tags: tags.clone(),
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
                // Update in place instead of replacing the entire root
                root.update(|recipes| {
                    if let Some(recipe_mut) = recipes.get_with_id_mut(&recipe_id) {
                        leptos::logging::log!(
                            "Updating tags in place: {:?} -> {:?}",
                            recipe_mut.tags,
                            tags
                        );
                        recipe_mut.tags = tags;
                    }
                });
                Ok(())
            }
            Ok(response) => Err(anyhow::anyhow!(
                "Failed to add tag: HTTP {}",
                response.status()
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to add tag: {}", e)),
        }
    }

    /// Delete a recipe
    /// This sends a DELETE request to the server and refreshes the recipe list
    pub async fn delete_recipe(&self, recipe_id: RecipeId) -> Result<(), anyhow::Error> {
        let url = format!("/api/recipe/{}", recipe_id);
        let response = gloo_net::http::Request::delete(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete recipe: {}", e))?;

        if !response.ok() {
            return Err(anyhow::anyhow!(
                "Failed to delete recipe: HTTP {}",
                response.status()
            ));
        }

        // Refresh the recipe list from the server
        self.refresh_recipes().await?;
        leptos::logging::log!("Recipe deleted successfully: {:?}", recipe_id);
        Ok(())
    }

    /// Duplicate a recipe
    /// This sends a PUT request to clone the recipe and refreshes the recipe list
    pub async fn duplicate_recipe(&self, recipe_id: RecipeId) -> Result<(), anyhow::Error> {
        let url = format!("/api/recipe/{}/clone", recipe_id);
        let response = gloo_net::http::Request::put(&url)
            .header("content-type", "application/json")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clone recipe: {}", e))?;

        if !response.ok() {
            return Err(anyhow::anyhow!(
                "Failed to clone recipe: HTTP {}",
                response.status()
            ));
        }

        // Refresh the recipe list from the server
        self.refresh_recipes().await?;
        leptos::logging::log!("Recipe cloned successfully: {:?}", recipe_id);
        Ok(())
    }

    /// Refresh the recipe list from the server
    async fn refresh_recipes(&self) -> Result<(), anyhow::Error> {
        let response = gloo_net::http::Request::get("/api/recipe/get_all")
            .header("content-type", "application/json")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch recipes: {}", e))?;

        let active_state: pilatus::device::ActiveState = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse recipes: {}", e))?;

        // Update the root signal with the new recipes
        self.root.set(active_state.recipes);
        Ok(())
    }
}
