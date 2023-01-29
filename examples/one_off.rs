use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::tracing::{debug, trace};
use big_brain::prelude::*;

#[derive(Clone, Component, Debug, ActionBuilder)]
struct OneOff;

fn one_off_action_system(mut query: Query<(&mut ActionState, &ActionSpan), With<OneOff>>) {
    for (mut state, span) in &mut query {
        let _guard = span.span().enter();
        match *state {
            ActionState::Requested => {
                debug!("One-off action!");
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                debug!("One-off action was cancelled. Considering this a failure.");
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

pub fn init_entities(mut cmd: Commands) {
    // You at least need to have a Thinker in order to schedule one-off
    // actions. It's not a general-purpose task scheduler.
    cmd.spawn((
        Thirst::new(75.0, 2.0),
        Thinker::build()
            .label("My Thinker")
            .picker(FirstToScore { threshold: 0.8 }),
    ));
}

#[derive(Component, Debug)]
pub struct Thirst {
    pub per_second: f32,
    pub thirst: f32,
}

impl Thirst {
    pub fn new(thirst: f32, per_second: f32) -> Self {
        Self { thirst, per_second }
    }
}

pub fn thirst_system(
    time: Res<Time>,
    mut thirsts: Query<(Entity, &mut Thirst)>,
    // We need to get to the Thinker. That takes a couple of steps.
    has_thinkers: Query<&HasThinker>,
    mut thinkers: Query<(&mut Thinker, &ActionSpan)>,
) {
    for (actor, mut thirst) in &mut thirsts {
        thirst.thirst += thirst.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
        if thirst.thirst >= 100.0 {
            let thinker_ent = has_thinkers.get(actor).unwrap().entity();
            let (mut thinker, span) = thinkers.get_mut(thinker_ent).unwrap();
            let _guard = span.span().enter();
            debug!("Scheduling one-off action");
            thinker.schedule_action(OneOff);
            thirst.thirst = 0.0;
        }
        trace!("Thirst: {}", thirst.thirst);
    }
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            // Use `RUST_LOG=big_brain=trace,thirst=trace cargo run --example
            // one_off --features=trace` to see extra tracing output.
            filter: "big_brain=debug,one_off=debug".to_string(),
            ..default()
        }))
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities)
        .add_system(thirst_system)
        // Big Brain has specific stages for Scorers and Actions. If
        // determinism matters a lot to you, you should add your action and
        // scorer systems to these stages.
        .add_system_to_stage(BigBrainStage::Actions, one_off_action_system)
        .run();
}
