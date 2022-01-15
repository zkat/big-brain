/*!
Thinkers are the "brain" of an entity. You attach Scorers to it, and the Thinker picks the right Action to run based on the resulting Scores.
*/

use std::sync::Arc;

use bevy::{
    prelude::*,
    utils::{Duration, Instant},
};

use crate::{
    actions::{self, ActionBuilder, ActionBuilderWrapper, ActionState},
    choices::{Choice, ChoiceBuilder},
    pickers::Picker,
    scorers::{Score, ScorerBuilder},
};

/**
Wrapper for Actor entities. In terms of Scorers, Thinkers, and Actions, this is the [`Entity`] actually _performing_ the action, rather than the entity a Scorer/Thinker/Action is attached to. Generally, you will use this entity when writing Queries for Action and Scorer systems.
 */
#[derive(Debug, Clone, Component, Copy)]
pub struct Actor(pub Entity);

#[derive(Debug, Clone, Component, Copy)]
pub(crate) struct ActionEnt(pub Entity);

#[derive(Debug, Clone, Component, Copy)]
pub(crate) struct ScorerEnt(pub Entity);

/**
The "brains" behind this whole operation. A `Thinker` is what glues together `Actions` and `Scorers` and shapes larger, intelligent-seeming systems.

Note: Thinkers are also Actions, so anywhere you can pass in an Action (or [`ActionBuilder`]), you can pass in a Thinker (or [`ThinkerBuilder`]).

### Example

```no_run
pub fn init_entities(mut cmd: Commands) {
    cmd.spawn()
        .insert(Thirst::new(70.0, 2.0))
        .insert(Hunger::new(50.0, 3.0))
        .insert(
            Thinker::build()
                .picker(FirstToScore::new(80.0))
                .when(Thirsty::build(), Drink::build())
                .when(Hungry::build(), Eat::build())
                .otherwise(Meander::build()),
        );
}
```
 */
#[derive(Component, Debug)]
pub struct Thinker {
    picker: Arc<dyn Picker>,
    otherwise: Option<ActionBuilderWrapper>,
    choices: Vec<Choice>,
    current_action: Option<(ActionEnt, ActionBuilderWrapper)>,
}

impl Thinker {
    /**
    Make a new [`ThinkerBuilder`]. This is what you'll actually use to configure Thinker behavior.
     */
    pub fn build() -> ThinkerBuilder {
        ThinkerBuilder::new()
    }
}

/**
This is what you actually use to configure Thinker behavior. It's a plain old [`ActionBuilder`], as well.
 */
#[derive(Component, Debug, Default)]
pub struct ThinkerBuilder {
    picker: Option<Arc<dyn Picker>>,
    otherwise: Option<ActionBuilderWrapper>,
    choices: Vec<ChoiceBuilder>,
}

impl ThinkerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            picker: None,
            otherwise: None,
            choices: Vec::new(),
        }
    }

    /**
    Define a [`Picker`](crate::pickers::Picker) for this Thinker.
     */
    pub fn picker(mut self, picker: impl Picker + 'static) -> Self {
        self.picker = Some(Arc::new(picker));
        self
    }

    /**
    Define an [`ActionBuilder`](crate::actions::ActionBuilder) and [`ScorerBuilder`](crate::scorers::ScorerBuilder) pair.
     */
    pub fn when(
        mut self,
        scorer: impl ScorerBuilder + 'static,
        action: impl ActionBuilder + 'static,
    ) -> Self {
        self.choices
            .push(ChoiceBuilder::new(Arc::new(scorer), Arc::new(action)));
        self
    }

    /**
    Default `Action` to execute if the `Picker` did not pick any of the given choices.
     */
    pub fn otherwise(mut self, otherwise: impl ActionBuilder + 'static) -> Self {
        self.otherwise = Some(ActionBuilderWrapper::new(Arc::new(otherwise)));
        self
    }
}

impl ActionBuilder for ThinkerBuilder {
    fn build(&self, cmd: &mut Commands, action_ent: Entity, actor: Entity) {
        let choices = self
            .choices
            .iter()
            .map(|choice| choice.build(cmd, actor, action_ent))
            .collect();
        cmd.entity(action_ent)
            .insert(Thinker {
                // TODO: reasonable default?...
                picker: self
                    .picker
                    .clone()
                    .expect("ThinkerBuilder must have a Picker"),
                choices,
                otherwise: self.otherwise.clone(),
                current_action: None,
            })
            .insert(Name::new("Thinker"))
            .insert(ActionState::Requested);
    }
}

pub fn thinker_component_attach_system(
    mut cmd: Commands,
    q: Query<(Entity, &ThinkerBuilder), Without<HasThinker>>,
) {
    for (entity, thinker_builder) in q.iter() {
        let thinker = thinker_builder.spawn_action(&mut cmd, entity);
        cmd.entity(entity).insert(HasThinker(thinker));
    }
}

pub fn thinker_component_detach_system(
    mut cmd: Commands,
    q: Query<(Entity, &HasThinker), Without<ThinkerBuilder>>,
) {
    for (actor, HasThinker(thinker)) in q.iter() {
        cmd.entity(*thinker).despawn_recursive();
        cmd.entity(actor).remove::<HasThinker>();
    }
}

pub fn actor_gone_cleanup(
    mut cmd: Commands,
    actors: Query<&ThinkerBuilder>,
    q: Query<(Entity, &Actor)>,
) {
    for (child, Actor(actor)) in q.iter() {
        if actors.get(*actor).is_err() {
            // Actor is gone. Let's clean up.
            cmd.entity(child).despawn_recursive();
        }
    }
}

#[derive(Component, Debug)]
pub struct HasThinker(Entity);

pub struct ThinkerIterations {
    index: usize,
    max_duration: Duration,
}
impl ThinkerIterations {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            index: 0,
            max_duration,
        }
    }
}
impl Default for ThinkerIterations {
    fn default() -> Self {
        Self::new(Duration::from_millis(10))
    }
}

pub fn thinker_system(
    mut cmd: Commands,
    mut iterations: Local<ThinkerIterations>,
    mut thinker_q: Query<(Entity, &Actor, &mut Thinker)>,
    scores: Query<&Score>,
    mut action_states: Query<&mut actions::ActionState>,
) {
    let start = Instant::now();
    for (thinker_ent, Actor(actor), mut thinker) in thinker_q.iter_mut().skip(iterations.index) {
        iterations.index += 1;

        let thinker_state = action_states
            .get_mut(thinker_ent)
            .expect("Where is it?")
            .clone();
        match thinker_state {
            ActionState::Init => {
                let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                *act_state = ActionState::Requested;
            }
            ActionState::Requested => {
                let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                *act_state = ActionState::Executing;
            }
            ActionState::Success | ActionState::Failure => {}
            ActionState::Cancelled => {
                if let Some(current) = &mut thinker.current_action {
                    let state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.").clone();
                    match state {
                        ActionState::Success | ActionState::Failure => {
                            cmd.entity(current.0 .0).despawn_recursive();
                            thinker.current_action = None;
                        }
                        _ => {
                            let mut state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                            *state = ActionState::Cancelled;
                        }
                    }
                } else {
                    let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                    *act_state = ActionState::Success;
                }
            }
            ActionState::Executing => {
                if let Some(choice) = thinker.picker.pick(&thinker.choices, &scores) {
                    // Think about what action we're supposed to be taking. We do this
                    // every tick, because we might change our mind.
                    // ...and then execute it (details below).
                    let action = choice.action.clone();
                    exec_picked_action(&mut cmd, *actor, &mut thinker, &action, &mut action_states);
                } else if let Some(default_action_ent) = &thinker.otherwise {
                    // Otherwise, let's just execute the default one! (if it's there)
                    let default_action_ent = default_action_ent.clone();
                    exec_picked_action(
                        &mut cmd,
                        *actor,
                        &mut thinker,
                        &default_action_ent,
                        &mut action_states,
                    );
                }
            }
        }
        if iterations.index % 500 == 0 && start.elapsed() > iterations.max_duration {
            return;
        }
    }
    iterations.index = 0;
}

fn exec_picked_action(
    cmd: &mut Commands,
    actor: Entity,
    thinker: &mut Mut<Thinker>,
    picked_action: &ActionBuilderWrapper,
    states: &mut Query<&mut ActionState>,
) {
    // If we do find one, then we need to grab the corresponding
    // component for it. The "action" that `picker.pick()` returns
    // is just a newtype for an Entity.
    //

    // Now we check the current action. We need to check if we picked the same one as the previous tick.
    //
    // TODO: I don't know where the right place to put this is
    // (maybe not in this logic), but we do need some kind of
    // oscillation protection so we're not just bouncing back and
    // forth between the same couple of actions.
    if let Some((action_ent, ActionBuilderWrapper(current_id, _))) = &mut thinker.current_action {
        let mut curr_action_state = states.get_mut(action_ent.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
        if !Arc::ptr_eq(current_id, &picked_action.0)
            || matches!(*curr_action_state, ActionState::Success)
            || matches!(*curr_action_state, ActionState::Failure)
        {
            // So we've picked a different action than we were
            // currently executing. Just like before, we grab the
            // actual Action component (and we assume it exists).
            // If the action is executing, or was requested, we
            // need to cancel it to make sure it stops.
            match *curr_action_state {
                ActionState::Executing | ActionState::Requested => {
                    *curr_action_state = ActionState::Cancelled;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    // Despawn the action itself.
                    cmd.entity(action_ent.0).despawn_recursive();
                    thinker.current_action = Some((
                        ActionEnt(picked_action.1.spawn_action(cmd, actor)),
                        picked_action.clone(),
                    ));
                }
                ActionState::Cancelled => {}
            };
        } else {
            // Otherwise, it turns out we want to keep executing
            // the same action. Just in case, we go ahead and set
            // it as Requested if for some reason it had finished
            // but the Action System hasn't gotten around to
            // cleaning it up.
            if *curr_action_state == ActionState::Init {
                *curr_action_state = ActionState::Requested;
            }
        }
    } else {
        // This branch arm is called when there's no
        // current_action in the thinker. The logic here is pretty
        // straightforward -- we set the action, Request it, and
        // that's it.
        let new_action = picked_action.1.spawn_action(cmd, actor);
        thinker.current_action = Some((ActionEnt(new_action), picked_action.clone()));
    }
}
