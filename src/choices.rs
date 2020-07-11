use serde::Deserialize;
use specs::{Entities, Entity, LazyUpdate, ReadStorage};

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
    pub fn calculate<'a>(&self, utilities: &ReadStorage<'a, Utility>) -> f32 {
        self.measure.calculate(
            self.utilities
                .iter()
                .map(|choice_cons| {
                    utilities
                        .get(choice_cons.0.clone())
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
    pub fn build(self, actor: Entity, ents: &Entities, lazy: &LazyUpdate) -> Choice {
        let action = self.then;
        Choice {
            measure: Box::new(WeightedMeasure),
            utilities: self
                .consider
                .iter()
                .map(|cons| cons.build(actor.clone(), ents, lazy))
                .collect(),
            action_state: ActionState::build(action, actor, ents, lazy),
        }
    }
}
