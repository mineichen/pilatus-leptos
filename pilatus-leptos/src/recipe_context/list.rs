use crate::{DeviceInfos, FetchApi, RecipeContext};
use leptos::prelude::*;
use pilatus::{Name, Recipe, RecipeId, RecipeMetadataRaw, Recipes};

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

    pub fn has_active_changes(&self) -> Signal<bool> {
        let root = self.root.read_only();
        Signal::derive(move || root.with(|recipes| recipes.has_active_changes()))
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

        self.fetch.put_json(&url, &metadata).await?;

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

    /// Remove a tag from a specific recipe
    /// This sends an API request to update the recipe on the server
    pub async fn remove_tag_from_recipe(
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
        tags.retain(|tag| tag != &tag_name);
        let url = format!("/api/recipe/{}/meta", recipe_id);

        let metadata = RecipeMetadataRaw {
            new_id: recipe_id.clone(),
            tags: tags.clone(),
        }
        .seal()?;

        self.fetch.put_json_silent(&url, &metadata).await?;

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

    /// Delete a recipe
    /// This sends a DELETE request to the server and refreshes the recipe list
    pub async fn delete_recipe(&self, recipe_id: RecipeId) -> Result<(), anyhow::Error> {
        let url = format!("/api/recipe/{}", recipe_id);
        self.fetch.delete(&url).await?;
        leptos::logging::log!("Recipe deleted successfully: {:?}", recipe_id);
        Ok(())
    }

    /// Duplicate a recipe
    /// This sends a PUT request to clone the recipe and refreshes the recipe list
    pub async fn duplicate_recipe(&self, recipe_id: RecipeId) -> Result<(), anyhow::Error> {
        let url = format!("/api/recipe/{}/clone", recipe_id);
        self.fetch.put(&url).await?;
        leptos::logging::log!("Recipe cloned successfully: {:?}", recipe_id);
        Ok(())
    }

    /// Create a new default recipe
    /// This sends a PUT request to create a new default recipe and refreshes the recipe list
    pub async fn create_new_default_recipe(&self) -> Result<(), anyhow::Error> {
        let url = "/api/recipe/new_default";
        self.fetch.put(url).await?;
        leptos::logging::log!("Recipe created successfully");
        Ok(())
    }

    /// Activate a recipe
    /// This sends a PUT request to start the recipe and refreshes the recipe list
    pub async fn activate_recipe(&self, recipe_id: RecipeId) -> Result<(), anyhow::Error> {
        let url = format!("/api/recipe/start/{}", recipe_id);
        self.fetch.put(&url).await?;
        leptos::logging::log!("Recipe activated successfully: {:?}", recipe_id);
        Ok(())
    }

    /// Commit changes
    /// This sends a PUT request to commit changes and refreshes the recipe list
    pub async fn commit_changes(&self) -> Result<(), anyhow::Error> {
        let url = "/api/recipe/commit";
        self.fetch.put(url).await?;
        Ok(())
    }

    /// Refresh the recipe list from the server
    pub(crate) async fn refresh_recipes(&self) -> Result<(), anyhow::Error> {
        let recipes = Self::load_recipes(self.fetch).await?;

        leptos::logging::debug_log!("Refresh recipes");
        self.set_root(recipes);
        Ok(())
    }

    pub(super) fn set_root(&self, recipes: Recipes) {
        self.root.set(recipes.clone());
        self.valid_root.set(recipes);
    }

    pub(super) async fn load_recipes(fetch: FetchApi) -> Result<Recipes, anyhow::Error> {
        let active_state: pilatus::device::ActiveState =
            fetch.get_json_silent("/api/recipe/get_all").await?;
        Ok(active_state.recipes)
    }
}
