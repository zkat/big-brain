/*!
Defines Action-related functionality. This module includes the ActionBuilder trait and some Composite Actions for utility.
*/
use std::sync::Arc;

use bevy::prelude::*;

use crate::thinker::{ActionEnt, Actor};

/**
The current state for an Action.
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActionState {
    /**
    Initial state. No action should be performed.
    */
    Init,
    /**
    Action requested. The Action-handling system should start executing this Action ASAP and change the status to the next state.
    */
    Requested,
    /**
    The action has ongoing execution. The associated Thinker will try to keep executing this Action as-is until it changes state or it gets Cancelled.
    */
    Executing,
    /**
    An ongoing Action has been cancelled. The Thinker might set this action for you, so for Actions that execute for longer than a single tick, **you must check whether the Cancelled state was set** and change do either Success or Failure. Thinkers will wait on Cancelled actions to do any necessary cleanup work, so this can hang your AI if you don't look for it.
    */
    Cancelled,
    /**
    The Action was a success. This is used by Composite Actions to determine whether to continue execution.
    */
    Success,
    /**
    The Action failed. This is used by Composite Actions to determine whether to halt execution.
    */
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ActionBuilderId;

#[derive(Debug, Clone)]
pub(crate) struct ActionBuilderWrapper(pub ActionBuilderId, pub Arc<dyn ActionBuilder>);

impl ActionBuilderWrapper {
    pub fn new(builder: Arc<dyn ActionBuilder>) -> Self {
        ActionBuilderWrapper(ActionBuilderId, builder)
    }
}

/**
Trait that must be defined by types in order to be `ActionBuilder`s. `ActionBuilder`s' job is to spawn new `Action` entities. In general, most of this is already done for you, and the only method you really have to implement is `.build()`.

The `build()` method MUST be implemented for any `ActionBuilder`s you want to define.
*/
pub trait ActionBuilder: std::fmt::Debug + Send + Sync {
    /**
    MUST insert your concrete Action component into the `action` [`Entity`], using `cmd`. You _may_ use `actor`, but it's perfectly normal to just ignore it.

    ### Example

    ```no_run
    struct MyBuilder;
    struct MyAction;

    impl ActionBuilder for MyBuilder {
        fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity) {
            cmd.entity(action).insert(MyAction);
        }
    }
    ```
    */
    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity);

    /**
    Don't implement this yourself unless you know what you're doing.
     */
    fn attach(&self, cmd: &mut Commands, actor: Entity) -> Entity {
        let action_ent = ActionEnt(cmd.spawn().id());
        cmd.entity(action_ent.0)
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(ActionState::new())
            .insert(Actor(actor));
        self.build(cmd, action_ent.0, actor);
        action_ent.0
    }
}

/**
[`ActionBuilder`] for the [`Steps`] component. Constructed through `Steps::build()`.
*/
#[derive(Debug)]
pub struct StepsBuilder {
    steps: Vec<Arc<dyn ActionBuilder>>,
}

impl StepsBuilder {
    /**
    Adds an action step. Order matters.
    */
    pub fn step(&mut self, action_builder: impl ActionBuilder + 'static) -> &mut Self {
        self.steps.push(Arc::new(action_builder));
        self
    }
}

impl ActionBuilder for StepsBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity) {
        let child_action = self.steps[0].attach(cmd, actor);
        cmd.entity(action)
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(Steps {
                active_step: 0,
                active_ent: ActionEnt(child_action),
                steps: self.steps.clone(),
            })
            .push_children(&[child_action]);
    }
}

/**
Composite Action that executes a series of steps in sequential order, as long as each step results in a `Success`ful [`ActionState`].

### Example

```ignore
Thinker::build()
    .when(
        MyScorer,
        Steps::build()
            .step(MyAction::build())
            .step(MyNextAction::build())
        )
```
*/
#[derive(Debug)]
pub struct Steps {
    steps: Vec<Arc<dyn ActionBuilder>>,
    active_step: usize,
    active_ent: ActionEnt,
}

impl Steps {
    /**
    Construct a new [`StepsBuilder`] to define the steps to take.
    */
    pub fn build() -> StepsBuilder {
        StepsBuilder { steps: Vec::new() }
    }
}

/**
System that takes care of executing any existing [`Steps`] Actions.
*/
pub fn steps_system(
    mut cmd: Commands,
    mut steps_q: Query<(Entity, &Actor, &mut Steps)>,
    mut states: Query<&mut ActionState>,
) {
    use ActionState::*;
    for (seq_ent, Actor(actor), mut steps_action) in steps_q.iter_mut() {
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
                let mut step_state = states.get_mut(steps_action.active_ent.0).expect("bug");
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
                        let step_builder = steps_action.steps[steps_action.active_step].clone();
                        let step_ent = step_builder.attach(&mut cmd, *actor);
                        cmd.entity(seq_ent).push_children(&[step_ent]);
                        let mut step_state = states.get_mut(step_ent).expect("oops");
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
