use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Deserialize;
use specs::{
    Component, DenseVecStorage, Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System,
    WriteStorage,
};

use crate::{
    actions::{Action, ActionManager, ActionManagerWrapper, ActionState},
    choices::{Choice, ChoiceBuilder},
    considerations::Utility,
    pickers::Picker,
    stopwatch::Stopwatch,
};

#[derive(Debug, Clone)]
pub struct ActionEnt(pub Entity);

#[derive(Debug, Clone)]
pub struct ConsiderationEnt(pub Entity);

#[derive(Component, Debug)]
pub struct Thinker {
    pub picker: Box<dyn Picker>,
    pub otherwise: Option<ActionEnt>,
    pub choices: Vec<Choice>,
    pub actor: Entity,
    pub current_action: Option<ActionEnt>,
    pub timer: Stopwatch,
}

impl Thinker {
    pub fn load_from<P: AsRef<Path>>(path: P) -> builder::Thinker {
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
    pub fn build(self, actor: Entity, ents: &Entities, lazy: &LazyUpdate) -> ActionEnt {
        let action_ent = ActionState::build(Box::new(self), actor, ents, lazy);
        lazy.insert(action_ent.0.clone(), ActiveThinker(true));
        lazy.insert(action_ent.0.clone(), ActionState::Requested);
        action_ent
    }
}

#[typetag::deserialize]
impl Action for builder::Thinker {
    fn build(
        self: Box<Self>,
        actor: Entity,
        action_ent: ActionEnt,
        ents: &Entities,
        lazy: &LazyUpdate,
    ) -> Box<dyn ActionManager> {
        lazy.insert(
            action_ent.0.clone(),
            Thinker {
                picker: self.picker,
                actor,
                choices: self
                    .choices
                    .into_iter()
                    .map(|choice| choice.build(actor.clone(), &ents, &lazy))
                    .collect(),
                otherwise: self
                    .otherwise
                    .map(|builder| ActionState::build(builder, actor.clone(), ents, lazy)),
                current_action: None,
                timer: Stopwatch::new(),
            },
        );
        Box::new(ThinkerManager)
    }
}

#[derive(Debug, Component)]
pub struct ActiveThinker(bool);

#[derive(Debug)]
pub struct ThinkerManager;

impl ActionManager for ThinkerManager {
    fn activate(&self, _: Entity, action_ent: ActionEnt, lazy: &LazyUpdate) {
        lazy.insert(action_ent.0, ActiveThinker(false));
        lazy.insert(action_ent.0, ActionState::Requested);
    }
    fn deactivate(&self, action_ent: ActionEnt, lazy: &LazyUpdate) {
        lazy.remove::<ActiveThinker>(action_ent.0);
    }
}

pub struct ThinkerSystem {
    index: usize,
    max_duration: Duration,
}
impl ThinkerSystem {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            index: 0,
            max_duration,
        }
    }
}

impl<'a> System<'a> for ThinkerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Thinker>,
        ReadStorage<'a, ActiveThinker>,
        ReadStorage<'a, Utility>,
        WriteStorage<'a, ActionState>,
        ReadStorage<'a, ActionManagerWrapper>,
        Read<'a, LazyUpdate>,
    );
    fn run(
        &mut self,
        (ents, mut thinkers, active_thinkers, utilities, mut action_states, builder_wrappers, lazy): Self::SystemData,
    ) {
        let start = Instant::now();
        for (thinker, thinker_ent, active_thinker) in (&mut thinkers, &ents, &active_thinkers)
            .join()
            .skip(self.index)
        {
            self.index += 1;

            let thinker_state = action_states
                .get(thinker_ent.clone())
                .expect("Where is it?")
                .clone();
            match thinker_state {
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    if let ActiveThinker(true) = active_thinker {
                        let act_state = action_states.get_mut(thinker_ent.clone()).expect("???");
                        *act_state = ActionState::Requested;
                    }
                }
                ActionState::Cancelled => {
                    if let Some(current) = &mut thinker.current_action {
                        let state = action_states.get(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.").clone();
                        match state {
                            ActionState::Success | ActionState::Failure => {
                                let act_state =
                                    action_states.get_mut(thinker_ent.clone()).expect("???");
                                *act_state = state.clone();
                                let state = action_states.get_mut(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                                *state = ActionState::Init;
                                thinker.current_action = None;
                            }
                            _ => {
                                let state = action_states.get_mut(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                                *state = ActionState::Cancelled;
                            }
                        }
                    }
                }
                ActionState::Requested | ActionState::Executing => {
                    if let Some(picked_action_ent) =
                        thinker.picker.pick(&thinker.choices, &utilities)
                    {
                        // Think about what action we're supposed to be taking. We do this
                        // every tick, because we might change our mind.
                        // ...and then execute it (details below).
                        exec_picked_action(
                            thinker_ent,
                            thinker,
                            &picked_action_ent,
                            &mut action_states,
                            &builder_wrappers,
                            &lazy,
                        );
                    } else if let Some(default_action_ent) = &thinker.otherwise {
                        // Otherwise, let's just execute the default one! (if it's there)
                        let default_action_ent = default_action_ent.clone();
                        exec_picked_action(
                            thinker_ent,
                            thinker,
                            &default_action_ent,
                            &mut action_states,
                            &builder_wrappers,
                            &lazy,
                        );
                    } else if let Some(current) = &mut thinker.current_action {
                        // If we didn't pick anything, and there's no default action,
                        // we need to see if there's any action currently executing,
                        // and cancel it. We also use this opportunity to clean up
                        // stale action components so they don't slow down joins.
                        let state = action_states.get_mut(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                        let factory = builder_wrappers.get(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                        match state {
                            ActionState::Init | ActionState::Success | ActionState::Failure => {
                                factory.0.deactivate(current.clone(), &lazy);
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
            thinker.timer.reset();
            thinker.timer.start();
            if self.index % 500 == 0 && start.elapsed() > self.max_duration {
                return;
            }
        }
        self.index = 0;
    }
}

fn exec_picked_action(
    thinker_ent: Entity,
    thinker: &mut Thinker,
    picked_action_ent: &ActionEnt,
    states: &mut WriteStorage<ActionState>,
    builder_wrappers: &ReadStorage<ActionManagerWrapper>,
    lazy: &Read<LazyUpdate>,
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
            let curr_action_state = states.get_mut(current.0.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
            // If the action is executing, or was requested, we
            // need to cancel it to make sure it stops. The Action
            // system will take care of resetting its state as
            // needed.
            match curr_action_state {
                ActionState::Executing | ActionState::Requested => {
                    *curr_action_state = ActionState::Cancelled;
                    let thinker_state = states.get_mut(thinker_ent.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
                    *thinker_state = ActionState::Cancelled;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    let current_action_factory = builder_wrappers.get(current.0.clone()).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
                    current_action_factory
                        .0
                        .deactivate(picked_action_ent.clone(), &lazy);
                    let old_state = curr_action_state.clone();
                    *curr_action_state = ActionState::Init;
                    *current = picked_action_ent.clone();
                    let thinker_state = states.get_mut(thinker_ent.clone()).expect("Couldn't find a component corresponding to the current action. This is definitely a bug.");
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
            let picked_action_state = states.get_mut(picked_action_ent.0.clone()).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
            match picked_action_state {
                ActionState::Init => {
                    let picked_action_factory = builder_wrappers.get(picked_action_ent.0.clone()).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
                    picked_action_factory.0.activate(
                        thinker.actor.clone(),
                        picked_action_ent.clone(),
                        lazy,
                    );
                    *picked_action_state = ActionState::Requested;
                }
                _ => {}
            }
        }
    } else {
        // This branch arm is called when there's no
        // current_action in the thinker. The logic here is pretty
        // straightforward -- we set the action, Request it, and
        // that's it.
        let picked_action_factory = builder_wrappers.get(picked_action_ent.0.clone()).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
        let picked_action_state = states.get_mut(picked_action_ent.0.clone()).expect("Couldn't find an Action component corresponding to an Action entity. This is definitely a bug.");
        picked_action_factory
            .0
            .activate(thinker.actor.clone(), picked_action_ent.clone(), lazy);
        thinker.current_action = Some(picked_action_ent.clone());
        *picked_action_state = ActionState::Requested;
    }
}
