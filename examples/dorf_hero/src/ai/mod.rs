use bevy::prelude::*;
use big_brain::prelude::*;

pub mod actions;
pub mod scorers;

pub struct DorfHeroAiPlugin;

impl Plugin for DorfHeroAiPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_plugin(BigBrainPlugin)
            .add_system(scorers::enemy_distance::enemy_distance.system())
            .add_system(scorers::fear_of_death::fear_of_death.system())
            .add_system(actions::meander::meander_action.system());
    }
}
