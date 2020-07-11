use serde::{Deserialize, Serialize};
use specs::ReadStorage;
use typetag;

use crate::{choices::Choice, considerations::Utility, thinker::ActionEnt};

#[typetag::serde]
pub trait Picker: std::fmt::Debug + Sync + Send {
    fn pick<'a>(
        &mut self,
        _choices: &Vec<Choice>,
        _utilities: &ReadStorage<'a, Utility>,
    ) -> Option<ActionEnt>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirstToScore {
    pub threshold: f32,
}

#[typetag::serde]
impl Picker for FirstToScore {
    fn pick<'a>(
        &mut self,
        choices: &Vec<Choice>,
        utilities: &ReadStorage<'a, Utility>,
    ) -> Option<ActionEnt> {
        for choice in choices {
            let value = choice.calculate(utilities);
            if value >= self.threshold {
                return Some(choice.action_state.clone());
            }
        }
        None
    }
}
