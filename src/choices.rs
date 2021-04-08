use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    actions::{Action, ActionState},
    considerations::{Consideration, Utility},
    measures::{Measure, WeightedMeasure},
    thinker::{ActionEnt, ConsiderationEnt},
};

// Contains different types of Considerations and Actions
#[derive(Debug)]
pub struct Choice {
    pub measure: Box<dyn Measure>,
    pub utilities: Vec<ConsiderationEnt>,
    pub action_state: ActionEnt,
}
impl Choice {
    pub fn calculate(&self, utilities: &Query<&Utility>) -> f32 {
        self.measure.calculate(
            self.utilities
                .iter()
                .map(|choice_cons| {
                    utilities
                        .get(choice_cons.0)
                        .expect("Where did the utility go?")
                })
                .collect(),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ChoiceBuilder {
    pub consider: Vec<Box<dyn Consideration>>,
    pub then: Box<dyn Action>,
}
impl ChoiceBuilder {
    pub fn build(self, actor: Entity, cmd: &mut Commands) -> Choice {
        let action = self.then;
        Choice {
            measure: Box::new(WeightedMeasure),
            utilities: self
                .consider
                .iter()
                .map(|cons| cons.build(actor, cmd))
                .collect(),
            action_state: ActionState::build(action, actor, cmd),
        }
    }
}
