use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    actions::{ActionBuilder, ActionBuilderWrapper},
    scorers::{Score, ScorerBuilder},
    thinker::ScorerEnt,
};

// Contains different types of Considerations and Actions
#[derive(Debug, Clone)]
pub struct Choice {
    pub(crate) scorer: ScorerEnt,
    pub(crate) action: ActionBuilderWrapper,
}
impl Choice {
    pub fn calculate(&self, scores: &Query<&Score>) -> f32 {
        scores
            .get(self.scorer.0)
            .expect("Where did the score go?")
            .0
    }
}

#[derive(Debug)]
pub struct ChoiceBuilder {
    pub when: Arc<dyn ScorerBuilder>,
    pub then: Arc<dyn ActionBuilder>,
}
impl ChoiceBuilder {
    pub fn new(scorer: Arc<dyn ScorerBuilder>, action: Arc<dyn ActionBuilder>) -> Self {
        Self {
            when: scorer,
            then: action,
        }
    }

    pub fn build(&self, cmd: &mut Commands, actor: Entity, parent: Entity) -> Choice {
        let scorer_ent = self.when.spawn_scorer(cmd, actor);
        cmd.entity(parent).push_children(&[scorer_ent]);
        Choice {
            scorer: ScorerEnt(scorer_ent),
            action: ActionBuilderWrapper::new(self.then.clone()),
        }
    }
}
