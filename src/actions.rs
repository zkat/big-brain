use std::sync::Arc;

use bevy::prelude::*;

use crate::ActionEnt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActionState {
    Init,
    Requested,
    Executing,
    Cancelled,
    Success,
    Failure,
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Init
    }
}

impl ActionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn spawn(
        cmd: &mut Commands,
        builder: &Arc<dyn ActionBuilder>,
        actor: Entity,
    ) -> ActionEnt {
        let action_ent = ActionEnt(cmd.spawn().id());
        cmd.entity(action_ent.0).insert(ActionState::new());
        cmd.entity(actor).push_children(&[action_ent.0]);
        builder.build(cmd, actor, action_ent.0);
        action_ent
    }
}

pub trait ActionBuilder: std::fmt::Debug + Send + Sync {
    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity);
}

#[derive(Debug)]
struct StepsBuilder {
    steps: Vec<Arc<dyn ActionBuilder>>,
}

impl StepsBuilder {
    pub fn step(&mut self, action_builder: impl ActionBuilder + 'static) -> &mut Self {
        self.steps.push(Arc::new(action_builder));
        self
    }
}

impl ActionBuilder for StepsBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity) {
        let child_action = ActionState::spawn(cmd, &self.steps[0], actor);
        cmd.entity(action).insert(Steps {
            active_step: 0,
            active_ent: child_action,
            steps: self.steps.clone(),
        });
    }
}
#[derive(Debug)]
pub struct Steps {
    steps: Vec<Arc<dyn ActionBuilder>>,
    active_step: usize,
    active_ent: ActionEnt,
}

impl Steps {
    pub fn build() -> StepsBuilder {
        StepsBuilder { steps: Vec::new() }
    }
}

pub fn steps_system(
    mut cmd: Commands,
    mut steps_q: Query<(Entity, &Parent, &mut Steps)>,
    mut states: Query<&mut ActionState>,
) {
    use ActionState::*;
    for (seq_ent, Parent(actor), mut steps_action) in steps_q.iter_mut() {
        let current_state = states.get_mut(seq_ent).expect("uh oh").clone();
        match current_state {
            Requested => {
                // Begin at the beginning
                let mut step_state = states.get_mut(steps_action.active_ent.0).expect("oops");
                *step_state = Requested;
                let mut current_state = states.get_mut(seq_ent).expect("uh oh");
                *current_state = Executing;
            }
            Executing => {
                let mut step_state = states
                    .get_mut(steps_action.active_ent.0)
                    .expect("bug");
                match *step_state {
                    Init => {
                        // Request it! This... should not really happen? But just in case I'm missing something... :)
                        *step_state = Requested;
                    }
                    Executing | Requested => {
                        // do nothing. Everything's running as it should.
                    }
                    Cancelled | Failure => {
                        // Cancel ourselves
                        let step_state = step_state.clone();
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                        cmd.entity(steps_action.active_ent.0).despawn_recursive();
                    }
                    Success if steps_action.active_step == steps_action.steps.len() - 1 => {
                        // We're done! Let's just be successful
                        let step_state = step_state.clone();
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                        cmd.entity(steps_action.active_ent.0).despawn_recursive();
                    }
                    Success => {
                        // Deactivate current step and go to the next step
                        cmd.entity(steps_action.active_ent.0).despawn_recursive();

                        steps_action.active_step += 1;
                        let step_builder = steps_action.steps[steps_action.active_step];
                        let step_ent = ActionState::spawn(&mut cmd, &step_builder, *actor);
                        let mut step_state = states.get_mut(step_ent.0).expect("oops");
                        *step_state = ActionState::Requested;
                    }
                }
            }
            Cancelled => {
                // Cancel current action
                let mut step_state = states.get_mut(steps_action.active_ent.0).expect("oops");
                *step_state = ActionState::Cancelled;
            }
            Init | Success | Failure => {
                // Do nothing.
            }
        }
    }
}
