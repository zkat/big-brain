pub use bevy;

pub use actions::*;
pub use choices::*;
pub use scorers::*;
pub use thinker::*;
pub use pickers::*;

pub mod evaluators;
mod pickers;

mod actions;
mod choices;
mod scorers;
mod thinker;

use bevy::prelude::*;

pub struct BigBrainPlugin;

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(thinker_system.system());
        app.add_system(thinker_component_attach_system.system());
        app.add_system(thinker_component_detach_system.system());
        app.add_system(steps_system.system());
        app.add_system(fixed_score_system.system());
        app.add_system(all_or_nothing_system.system());
        app.add_system(sum_of_scorers_system.system());
    }
}
