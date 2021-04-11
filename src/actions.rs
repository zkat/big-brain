use bevy::prelude::*;

use crate::ActionEnt;

#[derive(Debug)]
pub struct ActionRunnerWrapper(pub(crate) Box<dyn ActionRunner>);

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

    pub(crate) fn attach(builder: Box<dyn Action>, actor: Entity, cmd: &mut Commands) -> ActionEnt {
        let action_ent = ActionEnt(cmd.spawn().id());
        let manager_wrapper = ActionRunnerWrapper(builder.attach(actor, action_ent, cmd));
        cmd.entity(action_ent.0)
            .insert(ActionState::default())
            .insert(manager_wrapper);
        cmd.entity(actor).push_children(&[action_ent.0]);
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
    fn attach(
        self: Box<Self>,
        actor: Entity,
        action_ent: ActionEnt,
        cmd: &mut Commands,
    ) -> Box<dyn ActionRunner>;
}

pub trait ActionRunner: std::fmt::Debug + Send + Sync {
    fn activate(&self, actor: Entity, action: ActionEnt, cmd: &mut Commands);
    fn deactivate(&self, action: ActionEnt, cmd: &mut Commands);
}

#[derive(Debug)]
pub struct Steps {
    steps: Vec<ActionEnt>,
    active_step: usize,
}

pub fn steps_system(
    mut cmd: Commands,
    mut steps_q: Query<(Entity, &Parent, &mut Steps)>,
    mut states: Query<&mut ActionState>,
    runners: Query<&ActionRunnerWrapper>,
) {
    use ActionState::*;
    for (seq_ent, Parent(actor), mut steps_action) in steps_q.iter_mut() {
        let current_state = states.get_mut(seq_ent).expect("uh oh").clone();
        match current_state {
            Requested => {
                // Begin at the beginning
                let step_ent = steps_action.steps[steps_action.active_step];
                let step_runner = runners.get(step_ent.0).expect("oops");
                let mut step_state = states.get_mut(step_ent.0).expect("oops");
                step_runner.0.activate(*actor, step_ent, &mut cmd);
                *step_state = Requested;
                let mut current_state = states.get_mut(seq_ent).expect("uh oh");
                *current_state = Executing;
            }
            Executing => {
                let mut step_state = states
                    .get_mut(steps_action.steps[steps_action.active_step].0)
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
                        let step_ent = steps_action.steps[steps_action.active_step];
                        let step_state = step_state.clone();
                        let step_runner = runners.get(step_ent.0).expect("oops");
                        step_runner.0.deactivate(step_ent, &mut cmd);
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                    }
                    Success if steps_action.active_step == steps_action.steps.len() - 1 => {
                        // We're done! Let's just be successful
                        let step_ent = steps_action.steps[steps_action.active_step];
                        let step_state = step_state.clone();
                        let step_runner = runners.get(step_ent.0).expect("oops");
                        step_runner.0.deactivate(step_ent, &mut cmd);
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                    }
                    Success => {
                        // Deactivate current step and go to the next step
                        let step_ent = steps_action.steps[steps_action.active_step];
                        let step_runner = runners.get(step_ent.0).expect("oops");
                        step_runner.0.deactivate(step_ent, &mut cmd);

                        steps_action.active_step += 1;
                        let step_ent = steps_action.steps[steps_action.active_step];
                        let step_runner = runners.get(step_ent.0).expect("oops");
                        let mut step_state = states.get_mut(step_ent.0).expect("oops");
                        step_runner.0.activate(*actor, step_ent, &mut cmd);
                        *step_state = ActionState::Requested;
                    }
                }
            }
            Cancelled => {
                // Cancel current action
                let step_ent = steps_action.steps[steps_action.active_step];
                let step_runner = runners.get(step_ent.0).expect("oops");
                let mut step_state = states.get_mut(step_ent.0).expect("oops");
                step_runner.0.activate(*actor, step_ent, &mut cmd);
                *step_state = ActionState::Cancelled;
            }
            Init | Success | Failure => {
                // Do nothing.
            }
        }
    }
}

mod seq_action {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Steps {
        steps: Vec<Box<dyn Action>>,
    }

    #[typetag::deserialize]
    impl Action for Steps {
        fn attach(
            self: Box<Self>,
            actor: Entity,
            _action_ent: ActionEnt,
            cmd: &mut Commands,
        ) -> Box<dyn ActionRunner> {
            let runner = StepsRunner {
                steps: self
                    .steps
                    .into_iter()
                    .map(|builder| ActionState::attach(builder, actor, cmd))
                    .collect(),
            };
            let children: Vec<_> = runner.steps.iter().map(|x| x.0).collect();
            cmd.entity(actor).push_children(&children[..]);
            Box::new(runner)
        }
    }

    #[derive(Debug)]
    struct StepsRunner {
        steps: Vec<ActionEnt>,
    }

    impl ActionRunner for StepsRunner {
        fn activate(&self, _actor: Entity, action_ent: ActionEnt, cmd: &mut Commands) {
            cmd.entity(action_ent.0).insert(super::Steps {
                active_step: 0,
                steps: self.steps.clone(),
            });
        }
        fn deactivate(&self, action_ent: ActionEnt, cmd: &mut Commands) {
            cmd.entity(action_ent.0).remove::<super::Steps>();
        }
    }
}
