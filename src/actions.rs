use bevy::prelude::*;

use crate::ActionEnt;

#[derive(Debug)]
pub struct ActionManagerWrapper(pub(crate) Box<dyn ActionManager>);

#[derive(Debug, Clone, Eq, PartialEq)]
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

    pub(crate) fn build(builder: Box<dyn Action>, actor: Entity, cmd: &mut Commands) -> ActionEnt {
        let action_ent = ActionEnt(cmd.spawn().id());
        let manager_wrapper = ActionManagerWrapper(builder.build(actor, action_ent, cmd));
        cmd.entity(action_ent.0)
            .insert(ActionState::default())
            .insert(manager_wrapper);
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
        cmd: &mut Commands,
    ) -> Box<dyn ActionManager>;
}

pub trait ActionManager: std::fmt::Debug + Send + Sync {
    fn activate(&self, actor: Entity, action: ActionEnt, cmd: &mut Commands);
    fn deactivate(&self, action: ActionEnt, cmd: &mut Commands);
}
