use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    actions::{self, Action, ActionManager, ActionManagerWrapper, ActionState},
    choices::{Choice, ChoiceBuilder},
    scorers::Score,
    pickers::Picker,
};

#[derive(Debug, Clone, Copy)]
pub struct ActionEnt(pub Entity);

#[derive(Debug, Clone, Copy)]
pub struct ScorerEnt(pub Entity);

#[derive(Debug)]
pub struct Thinker {
    pub picker: Box<dyn Picker>,
    pub otherwise: Option<ActionEnt>,
    pub choices: Vec<Choice>,
    pub current_action: Option<ActionEnt>,
}

impl Thinker {
    pub fn load_from_str<S: AsRef<str>>(string: S) -> builder::Thinker {
        ron::de::from_str(string.as_ref()).expect("Failed to parse RON")
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> builder::Thinker {
        let f = File::open(&path).expect("Failed to open file");
        ron::de::from_reader(f).expect("Failed to read .ron file")
    }
}

mod builder {
    use super::*;
    #[derive(Debug, Deserialize)]
    pub struct Thinker {
        pub picker: Box<dyn Picker>,
        pub otherwise: Option<Box<dyn Action>>,
        pub choices: Vec<ChoiceBuilder>,
    }
}

impl builder::Thinker {
    pub fn build(self, actor: Entity, cmd: &mut Commands) -> ActionEnt {
        let action_ent = ActionState::build(Box::new(self), actor, cmd);
        cmd.entity(action_ent.0)
            .insert(ActiveThinker(true))
            .insert(ActionState::Requested);
        action_ent
    }
}

#[typetag::deserialize]
impl Action for builder::Thinker {
    fn build(
        self: Box<Self>,
        actor: Entity,
        action_ent: ActionEnt,
        cmd: &mut Commands,
    ) -> Box<dyn ActionManager> {
        let choices = self
            .choices
            .into_iter()
            .map(|choice| choice.build(actor, cmd))
            .collect();
        let otherwise = self
            .otherwise
            .map(|builder| ActionState::build(builder, actor, cmd));
        cmd.entity(action_ent.0).insert(Thinker {
            picker: self.picker,
            choices,
            otherwise,
            current_action: None,
        });
        cmd.entity(actor).push_children(&[action_ent.0]);
        Box::new(ThinkerManager)
    }
}

#[derive(Debug)]
pub struct ActiveThinker(bool);

#[derive(Debug)]
pub struct ThinkerManager;

impl ActionManager for ThinkerManager {
    fn activate(&self, _: Entity, action_ent: ActionEnt, cmd: &mut Commands) {
        cmd.entity(action_ent.0)
            .insert(ActiveThinker(false))
            .insert(ActionState::Requested);
    }
    fn deactivate(&self, action_ent: ActionEnt, cmd: &mut Commands) {
        cmd.entity(action_ent.0).remove::<ActiveThinker>();
    }
}

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
    builder_wrappers: Query<&ActionManagerWrapper>,
) {
    let start = Instant::now();
    for (thinker_ent, Parent(actor), mut thinker, active_thinker) in thinker_q.iter_mut().skip(iterations.index) {
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
                    let state = action_states.get_mut(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.").clone();
                    match state {
                        ActionState::Success | ActionState::Failure => {
                            let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                            *act_state = state.clone();
                            let mut state = action_states.get_mut(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                            *state = ActionState::Init;
                            thinker.current_action = None;
                        }
                        _ => {
                            let mut state = action_states.get_mut(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                            *state = ActionState::Cancelled;
                        }
                    }
                }
            }
            ActionState::Requested | ActionState::Executing => {
                if let Some(picked_action_ent) = thinker.picker.pick(&thinker.choices, &utilities) {
                    // Think about what action we're supposed to be taking. We do this
                    // every tick, because we might change our mind.
                    // ...and then execute it (details below).
                    exec_picked_action(
                        &mut cmd,
                        thinker_ent,
                        *actor,
                        &mut thinker,
                        &picked_action_ent,
                        &mut action_states,
                        &builder_wrappers,
                    );
                } else if let Some(default_action_ent) = &thinker.otherwise {
                    // Otherwise, let's just execute the default one! (if it's there)
                    let default_action_ent = *default_action_ent;
                    exec_picked_action(
                        &mut cmd,
                        thinker_ent,
                        *actor,
                        &mut thinker,
                        &default_action_ent,
                        &mut action_states,
                        &builder_wrappers,
                    );
                } else if let Some(current) = &mut thinker.current_action {
                    // If we didn't pick anything, and there's no default action,
                    // we need to see if there's any action currently executing,
                    // and cancel it. We also use this opportunity to clean up
                    // stale action components so they don't slow down joins.
                    let mut state = action_states.get_mut(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    let factory = builder_wrappers.get(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    match *state {
                        actions::ActionState::Init
                        | actions::ActionState::Success
                        | actions::ActionState::Failure => {
                            factory.0.deactivate(*current, &mut cmd);
                            *state = ActionState::Init;
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
    picked_action_ent: &ActionEnt,
    states: &mut Query<&mut ActionState>,
    builder_wrappers: &Query<&ActionManagerWrapper>,
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
    if let Some(current) = &mut thinker.current_action {
        if current.0 != picked_action_ent.0 {
            // So we've picked a different action than we were
            // currently executing. Just like before, we grab the
            // actual Action component (and we assume it exists).
            let mut curr_action_state = states.get_mut(current.0).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
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
                    let current_action_factory = builder_wrappers.get(current.0).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
                    current_action_factory
                        .0
                        .deactivate(*picked_action_ent, cmd);
                    let old_state = curr_action_state.clone();
                    *curr_action_state = ActionState::Init;
                    *current = *picked_action_ent;
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
            let mut picked_action_state = states.get_mut(picked_action_ent.0).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
            if *picked_action_state == ActionState::Init {
                let picked_action_factory = builder_wrappers.get(picked_action_ent.0).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
                picked_action_factory
                    .0
                    .activate(actor, *picked_action_ent, cmd);
                *picked_action_state = ActionState::Requested;
            }
        }
    } else {
        // This branch arm is called when there's no
        // current_action in the thinker. The logic here is pretty
        // straightforward -- we set the action, Request it, and
        // that's it.
        let picked_action_factory = builder_wrappers.get(picked_action_ent.0).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
        let mut picked_action_state = states.get_mut(picked_action_ent.0).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
        picked_action_factory
            .0
            .activate(actor, *picked_action_ent, cmd);
        thinker.current_action = Some(*picked_action_ent);
        *picked_action_state = ActionState::Requested;
    }
}
