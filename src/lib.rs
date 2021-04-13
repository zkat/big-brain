pub mod evaluators;
pub mod pickers;

pub mod actions;
pub mod choices;
pub mod scorers;
pub mod thinker;

pub mod prelude {
    use super::*;

    pub use super::BigBrainPlugin;
    pub use actions::{ActionBuilder, ActionState};
    pub use pickers::{FirstToScore, Picker};
    pub use scorers::{AllOrNothing, FixedScore, Score, ScorerBuilder, SumOfScorers};
    pub use thinker::{Actor, Thinker};
}

use bevy::prelude::*;

pub struct BigBrainPlugin;

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(thinker::thinker_system.system());
        app.add_system(thinker::thinker_component_attach_system.system());
        app.add_system(thinker::thinker_component_detach_system.system());
        app.add_system(actions::steps_system.system());
        app.add_system(scorers::fixed_score_system.system());
        app.add_system(scorers::all_or_nothing_system.system());
        app.add_system(scorers::sum_of_scorers_system.system());
    }
}
