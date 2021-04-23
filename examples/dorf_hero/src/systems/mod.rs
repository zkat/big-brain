use bevy::prelude::{AppBuilder, IntoSystem, Plugin};

mod camera_follow_hero;

pub struct DorfHeroSystemsPlugin;

impl Plugin for DorfHeroSystemsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(camera_follow_hero::camera_follow_hero.system());
    }
}
