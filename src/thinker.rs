use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::prelude::*;

use crate::{
    actions::{self, ActionBuilder, ActionBuilderWrapper, ActionState},
    choices::{Choice, ChoiceBuilder},
    pickers::Picker,
    scorers::{Score, ScorerBuilder},
};

#[derive(Debug, Clone, Copy)]
pub struct ActionEnt(pub Entity);

#[derive(Debug, Clone, Copy)]
pub struct ScorerEnt(pub Entity);

#[derive(Debug)]
pub struct Thinker {
    picker: Arc<dyn Picker>,
    otherwise: Option<ActionBuilderWrapper>,
    choices: Vec<Choice>,
    current_action: Option<(ActionEnt, ActionBuilderWrapper)>,
}

impl Thinker {
    pub fn build() -> ThinkerBuilder {
        ThinkerBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct ThinkerBuilder {
    pub picker: Option<Arc<dyn Picker>>,
    pub otherwise: Option<ActionBuilderWrapper>, // Arc<dyn ActionBuilder>?
    pub choices: Vec<ChoiceBuilder>,
}

impl ThinkerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            picker: None,
            otherwise: None,
            choices: Vec::new(),
        }
    }

    pub fn picker(&mut self, picker: impl Picker + 'static) -> &mut Self {
        self.picker = Some(Arc::new(picker));
        self
    }

    pub fn when(
        &mut self,
        scorer: impl ScorerBuilder + 'static,
        action: impl ActionBuilder + 'static,
    ) -> &mut Self {
        self.choices.push(ChoiceBuilder::new(Arc::new(scorer), Arc::new(action)));
        self
    }

    pub fn otherwise(&mut self, otherwise: impl ActionBuilder + 'static) -> &mut Self {
        self.otherwise = Some(ActionBuilderWrapper::new(Arc::new(otherwise)));
        self
    }
}

impl ActionBuilder for ThinkerBuilder {
    fn build(&self, cmd: &mut Commands, action_ent: Entity, actor: Entity) {
        println!("building thinker");
        let choices = self
            .choices
            .iter()
            .map(|choice| choice.build(cmd, actor))
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
            .insert(ActiveThinker(false))
            .insert(ActionState::Requested);
    }
}

#[derive(Debug)]
pub struct ActiveThinker(bool);

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
    mut thinker_q: Query<(Entity, &Parent, &mut Thinker, &ActiveThinker)>,
    utilities: Query<&Score>,
    mut action_states: Query<&mut actions::ActionState>,
) {
    let start = Instant::now();
    for (thinker_ent, Parent(actor), mut thinker, active_thinker) in
        thinker_q.iter_mut().skip(iterations.index)
    {
        iterations.index += 1;

        let thinker_state = action_states
            .get_mut(thinker_ent)
            .expect("Where is it?")
            .clone();
        match thinker_state {
            ActionState::Init | ActionState::Success | ActionState::Failure => {
                if let ActiveThinker(true) = active_thinker {
                    let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                    *act_state = ActionState::Requested;
                }
            }
            ActionState::Cancelled => {
                if let Some(current) = &mut thinker.current_action {
                    let state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.").clone();
                    match state {
                        ActionState::Success | ActionState::Failure => {
                            let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                            *act_state = state.clone();
                            let mut state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                            *state = ActionState::Init;
                            thinker.current_action = None;
                        }
                        _ => {
                            let mut state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                            *state = ActionState::Cancelled;
                        }
                    }
                }
            }
            ActionState::Requested | ActionState::Executing => {
                if let Some(choice) = thinker.picker.pick(&thinker.choices, &utilities) {
                    // Think about what action we're supposed to be taking. We do this
                    // every tick, because we might change our mind.
                    // ...and then execute it (details below).
                    exec_picked_action(
                        &mut cmd,
                        thinker_ent,
                        *actor,
                        &mut thinker,
                        &choice.action,
                        &mut action_states,
                    );
                } else if let Some(default_action_ent) = &thinker.otherwise {
                    // Otherwise, let's just execute the default one! (if it's there)
                    let default_action_ent = default_action_ent.clone();
                    exec_picked_action(
                        &mut cmd,
                        thinker_ent,
                        *actor,
                        &mut thinker,
                        &default_action_ent,
                        &mut action_states,
                    );
                } else if let Some(current) = &mut thinker.current_action {
                    // If we didn't pick anything, and there's no default action,
                    // we need to see if there's any action currently executing,
                    // and cancel it. We also use this opportunity to clean up
                    // stale action components so they don't slow down joins.
                    let mut state = action_states.get_mut(current.0.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    match *state {
                        actions::ActionState::Init
                        | actions::ActionState::Success
                        | actions::ActionState::Failure => {
                            cmd.entity(current.0 .0).despawn_recursive();
                            thinker.current_action = None;
                        }
                        _ => {
                            *state = ActionState::Cancelled;
                        }
                    }
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
    thinker_ent: Entity,
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
        if *current_id != picked_action.0 {
            // So we've picked a different action than we were
            // currently executing. Just like before, we grab the
            // actual Action component (and we assume it exists).
            let mut curr_action_state = states.get_mut(action_ent.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
            // If the action is executing, or was requested, we
            // need to cancel it to make sure it stops. The Action
            // system will take care of resetting its state as
            // needed.
            match *curr_action_state {
                ActionState::Executing | ActionState::Requested => {
                    *curr_action_state = ActionState::Cancelled;
                    let mut thinker_state = states.get_mut(thinker_ent).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    *thinker_state = ActionState::Cancelled;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    let old_state = curr_action_state.clone();
                    thinker.current_action = Some((
                        ActionEnt(picked_action.1.attach(cmd, actor)),
                        picked_action.clone(),
                    ));
                    let mut thinker_state = states.get_mut(thinker_ent).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    *thinker_state = old_state;
                }
                ActionState::Cancelled => {}
            };
        } else {
            // Otherwise, it turns out we want to keep executing
            // the same action. Just in case, we go ahead and set
            // it as Requested if for some reason it had finished
            // but the Action System hasn't gotten around to
            // cleaning it up.
            let mut curr_action_state = states.get_mut(action_ent.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
            if *curr_action_state == ActionState::Init {
                *curr_action_state = ActionState::Requested;
            }
        }
    } else {
        // This branch arm is called when there's no
        // current_action in the thinker. The logic here is pretty
        // straightforward -- we set the action, Request it, and
        // that's it.
        let new_action = picked_action.1.attach(cmd, actor);
        thinker.current_action = Some((ActionEnt(new_action), picked_action.clone()));
    }
}
