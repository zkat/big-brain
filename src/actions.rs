use specs::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadStorage, WriteStorage};
use typetag;

use crate::ActionEnt;

#[derive(Debug, Component)]
pub struct ActionManagerWrapper(pub(crate) Box<dyn ActionManager>);

#[derive(Debug, Component, Clone, Eq, PartialEq)]
pub enum ActionState {
    Init,
    Requested,
    Executing,
    Cancelled,
    Success,
    Failure,
}

impl ActionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<'a>(state_ent: &ActionEnt, states: &'a ReadStorage<ActionState>) -> &'a Self {
        states
            .get(state_ent.0.clone())
            .expect("ActionState doesn't exist?")
    }

    pub fn get_mut<'a>(
        state_ent: &ActionEnt,
        states: &'a mut WriteStorage<ActionState>,
    ) -> &'a mut Self {
        states
            .get_mut(state_ent.0.clone())
            .expect("ActionState doesn't exist?")
    }

    pub(crate) fn build(
        builder: Box<dyn Action>,
        actor: Entity,
        ents: &Entities,
        lazy: &LazyUpdate,
    ) -> ActionEnt {
        let action_ent = ActionEnt(ents.create());
        let ent_clone = action_ent.clone();
        lazy.insert(ent_clone.0, ActionState::default());
        lazy.insert(
            ent_clone.0,
            ActionManagerWrapper(builder.build(actor, action_ent.clone(), ents, lazy)),
        );
        action_ent
    }
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Init
    }
}

/**
This trait defines new actions. In general, you should use the [derive macro](derive.Action.html) instead.
*/
#[typetag::deserialize]
pub trait Action: std::fmt::Debug + Send + Sync {
    fn build(
        self: Box<Self>,
        actor: Entity,
        action_ent: ActionEnt,
        ents: &Entities,
        lazy: &LazyUpdate,
    ) -> Box<dyn ActionManager>;
}

pub trait ActionManager: std::fmt::Debug + Send + Sync {
    fn activate(&self, actor: Entity, action: ActionEnt, lazy: &LazyUpdate);
    fn deactivate(&self, action: ActionEnt, lazy: &LazyUpdate);
}
