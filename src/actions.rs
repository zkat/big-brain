/*!
Defines Action-related functionality. This module includes the ActionBuilder trait and some Composite Actions for utility.
*/
use std::sync::Arc;

use bevy::prelude::*;
#[cfg(feature = "trace")]
use bevy::utils::tracing::trace;

use crate::thinker::{Action, ActionSpan, Actor};

/**
The current state for an Action.
*/
#[derive(Debug, Clone, Component, Eq, PartialEq)]
#[component(storage = "SparseSet")]
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
pub(crate) struct ActionBuilderWrapper(pub Arc<ActionBuilderId>, pub Arc<dyn ActionBuilder>);

impl ActionBuilderWrapper {
    pub fn new(builder: Arc<dyn ActionBuilder>) -> Self {
        ActionBuilderWrapper(Arc::new(ActionBuilderId), builder)
    }
}

/**
Trait that must be defined by types in order to be `ActionBuilder`s. `ActionBuilder`s' job is to spawn new `Action` entities on demand. In general, most of this is already done for you, and the only method you really have to implement is `.build()`.

The `build()` method MUST be implemented for any `ActionBuilder`s you want to define.
*/
pub trait ActionBuilder: std::fmt::Debug + Send + Sync {
    /**

    MUST insert your concrete Action component into the Scorer [`Entity`], using
     `cmd`. You _may_ use `actor`, but it's perfectly normal to just ignore it.

    Note that this method is automatically implemented for any Components that
    implement Clone, so you don't need to define it yourself unless you want
    more complex parameterization of your Actions.

    ### Example

    Using `Clone` (the easy way):

    ```no_run
    #[derive(Debug, Clone, Component)]
    struct MyAction;
    ```

    Implementing it manually:

    ```no_run
    struct MyBuilder;
    #[derive(Debug, Component)]
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
     * A label to display when logging using the Action's tracing span.
     */
    fn label(&self) -> Option<&str> {
        None
    }
}

impl<T> ActionBuilder for T
where
    T: Component + Clone + std::fmt::Debug + Send + Sync,
{
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(T::clone(self));
    }
}

/**
 * Spawns a new Action Component, using the given ActionBuilder. This is useful when you're doing things like writing composite Actions.
 */
pub fn spawn_action<T: ActionBuilder + ?Sized>(
    builder: &T,
    cmd: &mut Commands,
    actor: Entity,
) -> Entity {
    let action_ent = Action(cmd.spawn().id());
    let span = ActionSpan::new(action_ent.entity(), ActionBuilder::label(builder));
    let _guard = span.span().enter();
    debug!("New Action spawned.");
    cmd.entity(action_ent.entity())
        .insert(Name::new("Action"))
        .insert(ActionState::new())
        .insert(Actor(actor));
    builder.build(cmd, action_ent.entity(), actor);
    std::mem::drop(_guard);
    cmd.entity(action_ent.entity()).insert(span);
    action_ent.entity()
}

/**
[`ActionBuilder`] for the [`Steps`] component. Constructed through `Steps::build()`.
*/
#[derive(Debug)]
pub struct StepsBuilder {
    label: Option<String>,
    steps: Vec<Arc<dyn ActionBuilder>>,
}

impl StepsBuilder {
    /**
     * Sets the logging label for the Action
     */
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /**
    Adds an action step. Order matters.
    */
    pub fn step(mut self, action_builder: impl ActionBuilder + 'static) -> Self {
        self.steps.push(Arc::new(action_builder));
        self
    }
}

impl ActionBuilder for StepsBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity) {
        if let Some(step) = self.steps.get(0) {
            let child_action = spawn_action(step.as_ref(), cmd, actor);
            cmd.entity(action)
                .insert(Name::new("Steps Action"))
                .insert(Steps {
                    active_step: 0,
                    active_ent: Action(child_action),
                    steps: self.steps.clone(),
                })
                .push_children(&[child_action]);
        }
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
            .step(MyAction)
            .step(MyNextAction)
        )
```
*/
#[derive(Component, Debug)]
pub struct Steps {
    steps: Vec<Arc<dyn ActionBuilder>>,
    active_step: usize,
    active_ent: Action,
}

impl Steps {
    /**
    Construct a new [`StepsBuilder`] to define the steps to take.
    */
    pub fn build() -> StepsBuilder {
        StepsBuilder {
            steps: Vec::new(),
            label: None,
        }
    }
}

/**
System that takes care of executing any existing [`Steps`] Actions.
*/
pub fn steps_system(
    mut cmd: Commands,
    mut steps_q: Query<(Entity, &Actor, &mut Steps, &ActionSpan)>,
    mut states: Query<&mut ActionState>,
) {
    use ActionState::*;
    for (seq_ent, Actor(actor), mut steps_action, _span) in steps_q.iter_mut() {
        let active_ent = steps_action.active_ent.entity();
        let current_state = states.get_mut(seq_ent).unwrap().clone();
        #[cfg(feature = "trace")]
        let _guard = _span.span().enter();
        match current_state {
            Requested => {
                // Begin at the beginning
                #[cfg(feature = "trace")]
                trace!(
                    "Initializing StepsAction and requesting first step: {:?}",
                    active_ent
                );
                *states.get_mut(active_ent).unwrap() = Requested;
                *states.get_mut(seq_ent).unwrap() = Executing;
            }
            Executing => {
                let mut step_state = states.get_mut(active_ent).unwrap();
                match *step_state {
                    Init => {
                        // Request it! This... should not really happen? But just in case I'm missing something... :)
                        *step_state = Requested;
                    }
                    Executing | Requested => {
                        // do nothing. Everything's running as it should.
                    }
                    Cancelled => {
                        // Wait for the step to wrap itself up, and we'll decide what to do at that point.
                    }
                    Failure => {
                        // Fail ourselves
                        #[cfg(feature = "trace")]
                        trace!("Step {:?} failed. Failing entire StepsAction.", active_ent);
                        let step_state = step_state.clone();
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                        cmd.entity(steps_action.active_ent.entity())
                            .despawn_recursive();
                    }
                    Success if steps_action.active_step == steps_action.steps.len() - 1 => {
                        // We're done! Let's just be successful
                        #[cfg(feature = "trace")]
                        trace!("StepsAction completed all steps successfully.");
                        let step_state = step_state.clone();
                        let mut seq_state = states.get_mut(seq_ent).expect("idk");
                        *seq_state = step_state;
                        cmd.entity(steps_action.active_ent.entity())
                            .despawn_recursive();
                    }
                    Success => {
                        #[cfg(feature = "trace")]
                        trace!("Step succeeded, but there's more steps. Spawning next action.");
                        // Deactivate current step and go to the next step
                        cmd.entity(steps_action.active_ent.entity())
                            .despawn_recursive();

                        steps_action.active_step += 1;
                        let step_builder = steps_action.steps[steps_action.active_step].clone();
                        let step_ent = spawn_action(step_builder.as_ref(), &mut cmd, *actor);
                        #[cfg(feature = "trace")]
                        trace!("Spawned next step: {:?}", step_ent);
                        cmd.entity(seq_ent).push_children(&[step_ent]);
                        steps_action.active_ent = Action(step_ent);
                    }
                }
            }
            Cancelled => {
                // Cancel current action
                #[cfg(feature = "trace")]
                trace!("StepsAction has been cancelled. Cancelling current step {:?} before finalizing.", active_ent);
                let mut step_state = states.get_mut(active_ent).expect("oops");
                if *step_state == Requested || *step_state == Executing {
                    *step_state = Cancelled;
                } else if *step_state == Failure || *step_state == Success {
                    *states.get_mut(seq_ent).unwrap() = step_state.clone();
                }
            }
            Init | Success | Failure => {
                // Do nothing.
            }
        }
    }
}

/**
[`ActionBuilder`] for the [`Concurrently`] component. Constructed through `Concurrently::build()`.
*/
#[derive(Debug)]
pub struct ConcurrentlyBuilder {
    actions: Vec<Arc<dyn ActionBuilder>>,
    label: Option<String>,
}

impl ConcurrentlyBuilder {
    /**
     * Sets the logging label for the Action
     */
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /**
    Add an action to execute. Order does not matter.
    */
    pub fn push(mut self, action_builder: impl ActionBuilder + 'static) -> Self {
        self.actions.push(Arc::new(action_builder));
        self
    }
}

impl ActionBuilder for ConcurrentlyBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn build(&self, cmd: &mut Commands, action: Entity, actor: Entity) {
        let children: Vec<Entity> = self
            .actions
            .iter()
            .map(|action| spawn_action(action.as_ref(), cmd, actor))
            .collect();
        cmd.entity(action)
            .insert(Name::new("Concurrent Action"))
            .push_children(&children[..])
            .insert(Concurrently {
                actions: children.into_iter().map(Action).collect(),
            });
    }
}

/**
Composite Action that executes a number of Actions concurrently, as long as they all result in a `Success`ful [`ActionState`].

### Example

```ignore
Thinker::build()
    .when(
        MyScorer,
        Concurrent::build()
            .push(MyAction)
            .push(MyOtherAction)
        )
```
*/
#[derive(Component, Debug)]
pub struct Concurrently {
    actions: Vec<Action>,
}

impl Concurrently {
    /**
    Construct a new [`ConcurrentlyBuilder`] to define the actions to take.
    */
    pub fn build() -> ConcurrentlyBuilder {
        ConcurrentlyBuilder {
            actions: Vec::new(),
            label: None,
        }
    }
}

/**
System that takes care of executing any existing [`Concurrently`] Actions.
*/
pub fn concurrent_system(
    concurrent_q: Query<(Entity, &Concurrently, &ActionSpan)>,
    mut states_q: Query<&mut ActionState>,
) {
    use ActionState::*;
    for (seq_ent, concurrent_action, _span) in concurrent_q.iter() {
        let current_state = states_q.get_mut(seq_ent).expect("uh oh").clone();
        #[cfg(feature = "trace")]
        let _guard = _span.span.enter();
        match current_state {
            Requested => {
                #[cfg(feature = "trace")]
                trace!(
                    "Initializing Concurrently action with {} children.",
                    concurrent_action.actions.len()
                );
                // Begin at the beginning
                let mut current_state = states_q.get_mut(seq_ent).expect("uh oh");
                *current_state = Executing;
                for action in concurrent_action.actions.iter() {
                    let child_ent = action.entity();
                    let mut child_state = states_q.get_mut(child_ent).expect("uh oh");
                    *child_state = Requested;
                }
            }
            Executing => {
                let mut all_success = true;
                let mut failed_idx = None;
                for (idx, action) in concurrent_action.actions.iter().enumerate() {
                    let child_ent = action.entity();
                    let mut child_state = states_q.get_mut(child_ent).expect("uh oh");
                    match *child_state {
                        Failure => {
                            failed_idx = Some(idx);
                            all_success = false;
                            #[cfg(feature = "trace")]
                            trace!("Concurrently action has failed. Cancelling all other actions that haven't completed yet.");
                        }
                        Success => {}
                        _ => {
                            all_success = false;
                            if failed_idx.is_some() {
                                *child_state = Cancelled;
                            }
                        }
                    }
                }
                if all_success {
                    let mut state_var = states_q.get_mut(seq_ent).expect("uh oh");
                    *state_var = Success;
                } else if let Some(idx) = failed_idx {
                    for action in concurrent_action.actions.iter().take(idx) {
                        let child_ent = action.entity();
                        let mut child_state = states_q.get_mut(child_ent).expect("uh oh");
                        match *child_state {
                            Failure | Success => {}
                            _ => {
                                *child_state = Cancelled;
                            }
                        }
                    }
                    let mut state_var = states_q.get_mut(seq_ent).expect("uh oh");
                    *state_var = Cancelled;
                }
            }
            Cancelled => {
                // Cancel all actions
                let mut all_done = true;
                let mut any_failed = false;
                for action in concurrent_action.actions.iter() {
                    let child_ent = action.entity();
                    let mut child_state = states_q.get_mut(child_ent).expect("uh oh");
                    match *child_state {
                        Init | Success => {
                            // Do nothing
                        }
                        Failure => {
                            any_failed = true;
                        }
                        _ => {
                            all_done = false;
                            *child_state = Cancelled;
                        }
                    }
                }
                if all_done {
                    let mut state_var = states_q.get_mut(seq_ent).expect("uh oh");
                    if any_failed {
                        #[cfg(feature = "trace")]
                        trace!("Concurrently action has failed due to failed children.");
                        *state_var = Failure;
                    } else {
                        #[cfg(feature = "trace")]
                        trace!("All Concurrently children have completed Successfully.");
                        *state_var = Success;
                    }
                }
            }
            Init | Success | Failure => {
                // Do nothing.
            }
        }
    }
}
