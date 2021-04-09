use bevy::prelude::*;

use crate::ScorerEnt;

#[derive(Debug, Clone, Default)]
pub struct Score(pub(crate) f32);

impl Score {
    pub fn set(&mut self, value: f32) {
        if !(0.0..=100.0).contains(&value) {
            panic!("Score value must be between 0.0 and 100.0");
        }
        self.0 = value;
    }
}

/**
This trait defines new Scorers. In general, you should use the [derive macro](derive.Scorer.html) instead.
*/
#[typetag::deserialize]
pub trait Scorer: std::fmt::Debug + Sync + Send {
    fn build(&self, entity: Entity, cmd: &mut Commands) -> ScorerEnt;
}
