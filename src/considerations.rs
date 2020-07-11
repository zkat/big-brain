use specs::{Component, DenseVecStorage, Entities, Entity, LazyUpdate};
use typetag;

use crate::ConsiderationEnt;

#[derive(Debug, Clone, Default, Component)]
pub struct Utility {
    pub value: f32,
    pub weight: f32,
}


/**
This trait defines new considerations. In general, you should use the [derive macro](derive.Consideration.html) instead.
*/
#[typetag::deserialize]
pub trait Consideration: std::fmt::Debug + Sync + Send {
    fn build(&self, entity: Entity, ents: &Entities, lazy: &LazyUpdate) -> ConsiderationEnt;
}
