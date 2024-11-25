use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    actions::{ActionBuilder, ActionBuilderWrapper},
    scorers::{self, Score, ScorerBuilder},
    thinker::Scorer,
};

/// Contains different types of Considerations and Actions
#[derive(Debug, Clone, Reflect)]
#[reflect(from_reflect = false)]
pub struct Choice {
    pub(crate) scorer: Scorer,
    #[reflect(ignore)]
    pub(crate) action: ActionBuilderWrapper,
    pub(crate) action_label: Option<String>,
}

impl Choice {
    pub fn calculate(&self, scores: &Query<&Score>) -> f32 {
        scores
            .get(self.scorer.0)
            .expect("Where did the score go?")
            .0
    }
}

/// Builds a new [`Choice`].
#[derive(Clone, Debug, Reflect)]
#[reflect(from_reflect = false)]
pub struct ChoiceBuilder {
    when_label: Option<String>,
    #[reflect(ignore)]
    pub when: Arc<dyn ScorerBuilder>,
    then_label: Option<String>,
    #[reflect(ignore)]
    pub then: Arc<dyn ActionBuilder>,
}
impl ChoiceBuilder {
    pub fn new(scorer: Arc<dyn ScorerBuilder>, action: Arc<dyn ActionBuilder>) -> Self {
        Self {
            when_label: scorer.label().map(|s| s.into()),
            when: scorer,
            then_label: action.label().map(|s| s.into()),
            then: action,
        }
    }

    pub fn build(&self, cmd: &mut Commands, actor: Entity, parent: Entity) -> Choice {
        let scorer_ent = scorers::spawn_scorer(&*self.when, cmd, actor);
        cmd.entity(parent).add_children(&[scorer_ent]);
        Choice {
            scorer: Scorer(scorer_ent),
            action_label: self.then.label().map(|s| s.into()),
            action: ActionBuilderWrapper::new(self.then.clone()),
        }
    }
}
