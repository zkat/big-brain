use bevy::prelude::*;

use crate::ConsiderationEnt;

#[derive(Debug, Clone, Default)]
pub struct Utility {
    pub value: f32,
    pub weight: f32,
}

/**
This trait defines new considerations. In general, you should use the [derive macro](derive.Consideration.html) instead.
*/
#[typetag::deserialize]
pub trait Consideration: std::fmt::Debug + Sync + Send {
    fn build(&self, entity: Entity, cmd: &mut Commands) -> ConsiderationEnt;
}
