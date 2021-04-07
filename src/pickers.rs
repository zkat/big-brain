use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{choices::Choice, considerations::Utility, thinker::ActionEnt};

#[typetag::serde]
pub trait Picker: std::fmt::Debug + Sync + Send {
    fn pick<'a>(&mut self, _choices: &[Choice], _utilities: &Query<&Utility>) -> Option<ActionEnt>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirstToScore {
    pub threshold: f32,
}

#[typetag::serde]
impl Picker for FirstToScore {
    fn pick<'a>(&mut self, choices: &[Choice], utilities: &Query<&Utility>) -> Option<ActionEnt> {
        for choice in choices {
            let value = choice.calculate(utilities);
            if value >= self.threshold {
                return Some(choice.action_state.clone());
            }
        }
        None
    }
}
