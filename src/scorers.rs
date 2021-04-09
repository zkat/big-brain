use bevy::prelude::*;

use crate::ScorerEnt;

#[derive(Debug, Clone, Default)]
pub struct Score(pub f32);

/**
This trait defines new Scorers. In general, you should use the [derive macro](derive.Scorer.html) instead.
*/
#[typetag::deserialize]
pub trait Scorer: std::fmt::Debug + Sync + Send {
    fn build(&self, entity: Entity, cmd: &mut Commands) -> ScorerEnt;
}
