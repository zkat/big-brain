//!
//! This example describes how to create an action that takes multiple steps.
//!
//! It is similar to the thirst example, but instead of just magically quenching a thirst,
//! the actor must be near a water source in order to drink.

use bevy::prelude::*;
use big_brain::actions::StepsBuilder;
use big_brain::prelude::*;

/// First, we make a simple Position component.
#[derive(Component, Debug, Copy, Clone)]
pub struct Position {
    pub position: Vec2,
}

/// A marker component for an entity that describes a water source.
#[derive(Component, Debug)]
pub struct WaterSource;

/// We steal the Thirst component from the thirst example.
#[derive(Component, Debug)]
pub struct Thirst {
    /// How much thirstier the entity gets over time.
    pub per_second: f32,
    /// How much thirst the entity currently has.
    pub thirst: f32,
}

impl Thirst {
    pub fn new(thirst: f32, per_second: f32) -> Self {
        Self { thirst, per_second }
    }
}

/// A simple system that just pushes the thirst value up over time.
/// Just a plain old Bevy system, big-brain is not involved yet.
pub fn thirst_system(time: Res<Time>, mut thirsts: Query<&mut Thirst>) {
    for mut thirst in thirsts.iter_mut() {
        thirst.thirst += thirst.per_second * time.delta_seconds();
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }
        println!("Thirst: {}", thirst.thirst);
    }
}

/// An action where the actor moves to the closest water source
#[derive(Clone, Component, Debug)]
pub struct MoveToWaterSource {
    speed: f32,
}

/// Closest distance to a water source to be able to drink from it.
const MAX_DISTANCE: f32 = 0.1;

fn move_to_water_source_action_system(
    time: Res<Time>,
    mut waters: Query<&Position, With<WaterSource>>,
    mut positions: Query<&mut Position, Without<WaterSource>>,
    mut action_query: Query<(&Actor, &mut ActionState, &MoveToWaterSource)>,
) {
    for (actor, mut action_state, move_to) in action_query.iter_mut() {

        match *action_state {
            ActionState::Requested => {
                println!("Let's go find some water!");
                *action_state = ActionState::Executing;
            },
            ActionState::Executing => {

                let mut actor_position =
                    positions
                        .get_mut(actor.0)
                        .expect("actor has no position");

                println!("Actor position: {:?}", actor_position.position);

                let closest_water_source =
                    find_closest_water_source(&waters, &actor_position);

                let delta = closest_water_source.position - actor_position.position;

                let distance = delta.length();

                println!("Distance: {}", distance);

                if distance > MAX_DISTANCE {

                    println!("Stepping closer.");

                    let step_size = time.delta_seconds() * move_to.speed;

                    let step = if step_size > distance {
                        delta
                    } else {
                        delta / distance * step_size
                    };

                    actor_position.position += step;
                } else {

                    println!("We got there!");

                    *action_state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }

}

fn find_closest_water_source(waters: &Query<&Position, With<WaterSource>>, actor_position: &Position) -> Position {
    waters
        .iter()
        .min_by(|a, b| {
            let da = (a.position - actor_position.position).length_squared();
            let db = (b.position - actor_position.position).length_squared();
            da.partial_cmp(&db).unwrap()
        })
        .expect("no water sources")
        .clone()
}

/// A simple action: the actor's thirst shall decrease, but only if they are near a water source.
#[derive(Clone, Component, Debug)]
pub struct Drink {
    per_second: f32,
}

fn drink_action_system(
    time: Res<Time>,
    mut thirsts: Query<(&Position, &mut Thirst), Without<WaterSource>>,
    mut waters: Query<&Position, With<WaterSource>>,
    mut query: Query<(&Actor, &mut ActionState, &Drink)>,
) {
    for (Actor(actor), mut state, drink) in query.iter_mut() {

        let (actor_position, mut thirst) = thirsts.get_mut(*actor).expect("actor has no thirst");

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {

                let closest_water_source =
                    find_closest_water_source(&waters, &*actor_position);

                let distance = (closest_water_source.position - actor_position.position).length();

                if distance < MAX_DISTANCE {
                    println!("Drinking!");
                    thirst.thirst -= drink.per_second * time.delta_seconds();
                    if thirst.thirst <= 0.0 {
                        thirst.thirst = 0.0;
                        *state = ActionState::Success;
                    }
                } else {
                    println!("We're too far away!");
                    *state = ActionState::Failure;
                }
            }
            // All Actions should make sure to handle cancellations!
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

// Scorers are the same as in the thirst example.
#[derive(Clone, Component, Debug)]
pub struct Thirsty;

pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            score.set(thirst.thirst / 100.);
        }
    }
}

pub fn init_entities(mut cmd: Commands) {

    // Spawn two water sources.
    cmd.spawn()
        .insert(WaterSource)
        .insert(Position {
            position: Vec2::new(10.0, 10.0),
        });

    cmd.spawn()
        .insert(WaterSource)
        .insert(Position {
            position: Vec2::new(-10.0, 0.0),
        });

    let move_and_drink =
        Steps::build()
            .step(MoveToWaterSource { speed: 1.0 })
            .step(Drink { per_second: 10.0 });

    cmd.spawn()
        .insert(Thirst::new(75.0, 2.0))
        .insert(Position {
            position: Vec2::new(0.0, 0.0),
        })
        .insert(
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(
                Thirsty,
                move_and_drink,
            ),
    );
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities)
        .add_system(thirst_system)
        .add_system_to_stage(BigBrainStage::Actions, drink_action_system)
        .add_system_to_stage(BigBrainStage::Actions, move_to_water_source_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, thirsty_scorer_system)
        .run();
}
