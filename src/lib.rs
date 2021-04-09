pub use bevy;
pub use big_brain_derive::*;
pub use serde;
pub use typetag;

pub use actions::*;
pub use choices::*;
pub use scorers::*;
pub use thinker::*;

pub mod evaluators;
pub mod measures;
pub mod pickers;

mod actions;
mod choices;
mod scorers;
mod thinker;

use bevy::prelude::*;

pub struct BigBrainPlugin;

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(thinker_system.system());
    }
}
