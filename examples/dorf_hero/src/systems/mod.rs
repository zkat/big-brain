use std::{fmt::Debug, hash::Hash};

use bevy::{ecs::component::Component, prelude::*};
use bevy_tilemap::prelude::TilemapDefaultPlugins;

mod camera_follow_hero;

pub struct DorfHeroSystemsPlugin<T>(pub T)
where
    T: Component + Debug + Clone + Eq + Hash + Send + Sync;

impl<T> Plugin for DorfHeroSystemsPlugin<T>
where
    T: Component + Debug + Clone + Eq + Hash + Send + Sync,
{
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(TilemapDefaultPlugins).add_system_set(
            SystemSet::on_update(self.0.clone())
                .with_system(camera_follow_hero::camera_follow_hero.system()),
        );
    }
}
