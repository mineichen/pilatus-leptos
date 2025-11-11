use crate::{DeviceInfos, RecipeContext};
use leptos::prelude::*;
use pilatus::{Recipe, RecipeId};

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
            root.with(|x| {
                x.as_ref()
                    .map(|r| {
                        let (_, active) = r.active();
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
                    .unwrap_or_default()
            })
        })
    }

    pub fn list_recipes(&self) -> Signal<Vec<RecipeInfo>> {
        let root = self.root.read_only();
        Signal::derive(move || {
            root.with(|x| {
                x.as_ref()
                    .map(|recipes| {
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
                    .unwrap_or_default()
            })
        })
    }
}
