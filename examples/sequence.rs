//! This example describes how to create an action that takes multiple steps.
//!
//! It is similar to the thirst example, but instead of just magically quenching a thirst,
//! the actor must be near a water source in order to drink.
//!
//! Note that it does not matter if the actor is already near a water source:
//! the MoveToWaterSource action will simply terminate immediately.

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

        // Thirst is capped at 100.0
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }

        println!("Thirst: {}", thirst.thirst);
    }
}

/// An action where the actor moves to the closest water source
#[derive(Clone, Component, Debug)]
pub struct MoveToWaterSource {
    // The movement speed of the actor.
    speed: f32,
}

/// Closest distance to a water source to be able to drink from it.
const MAX_DISTANCE: f32 = 0.1;

fn move_to_water_source_action_system(
    time: Res<Time>,
    // Find all water sources
    mut waters: Query<&Position, With<WaterSource>>,
    // We use Without to make disjoint queries.
    mut positions: Query<&mut Position, Without<WaterSource>>,
    // A query on all current MoveToWaterSource actions.
    mut action_query: Query<(&Actor, &mut ActionState, &MoveToWaterSource)>,
) {
    // Loop through all actions, just like you'd loop over all entities in any other query.
    for (actor, mut action_state, move_to) in action_query.iter_mut() {

        // Different behavior depending on action state.
        match *action_state {
            // Action was just requested; it hasn't been seen before.
            ActionState::Requested => {
                println!("Let's go find some water!");
                // We don't really need any initialization code here, since the queries are cheap enough.
                *action_state = ActionState::Executing;
            },
            ActionState::Executing => {

                // Look up the actor's position.
                let mut actor_position =
                    positions
                        .get_mut(actor.0)
                        .expect("actor has no position");

                println!("Actor position: {:?}", actor_position.position);

                // Look up the water source closest to them.
                let closest_water_source =
                    find_closest_water_source(&waters, &actor_position);

                // Find how far we are from it.
                let delta = closest_water_source.position - actor_position.position;

                let distance = delta.length();

                println!("Distance: {}", distance);

                if distance > MAX_DISTANCE {
                    // We're still too far, take a step toward it!

                    println!("Stepping closer.");

                    // How far can we travel during this frame?
                    let step_size = time.delta_seconds() * move_to.speed;

                    let step = if step_size > distance {
                        // We'll move by the full step
                        delta / distance * step_size
                    } else {
                        // Move only the remaining distance, or we might overshoot.
                        delta
                    };

                    // Move the actor.
                    actor_position.position += step;

                } else {
                    // We're within the required distance! We can declare success.

                    println!("We got there!");

                    // The action will be cleaned up automatically.
                    *action_state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                // Always treat cancellations, or we might keep doing this forever!
                // You don't need to terminate immediately, by the way, this is only a flag that
                // the cancellation has been requested. If the actor is balancing on a tightrope,
                // for instance, you may let them walk off before ending the action.
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }

}

/// A utility function that finds the closest water source to the actor.
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
    // Loop through all actions, just like you'd loop over all entities in any other query.
    for (Actor(actor), mut state, drink) in query.iter_mut() {

        // Look up the actor's position and thirst from the Actor component in the action entity.
        let (actor_position, mut thirst) = thirsts.get_mut(*actor).expect("actor has no thirst");

        match *state {
            ActionState::Requested => {
                // We'll start drinking as soon as we're requested to do so.
                *state = ActionState::Executing;
            }
            ActionState::Executing => {

                // Look up the closest water source.
                // Note that there is no explicit passing of a selected water source from the GoToWaterSource action,
                // so we look it up again. Note that this decouples the actions from each other,
                // so if the actor is already close to a water source, the GoToWaterSource action
                // will not be necessary (though it will not harm either).
                //
                // Essentially, being close to a water source would be a precondition for the Drink action.
                // How this precondition was fulfilled is not this code's concern.
                let closest_water_source =
                    find_closest_water_source(&waters, &*actor_position);

                // Find how far we are from it.
                let distance = (closest_water_source.position - actor_position.position).length();

                // Are we close enough?
                if distance < MAX_DISTANCE {
                    println!("Drinking!");

                    // Start reducing the thirst. Alternatively, you could send out some kind of
                    // DrinkFromSource event that indirectly decreases thirst.
                    thirst.thirst -= drink.per_second * time.delta_seconds();

                    // Once we hit 0 thirst, we stop drinking and report success.
                    if thirst.thirst <= 0.0 {
                        thirst.thirst = 0.0;
                        *state = ActionState::Success;
                    }
                } else {
                    // The actor was told to drink, but they can't drink when they're so far away!
                    // The action doesn't know how to deal with this case, it's the overarching system's
                    // to fulfill the precondition.
                    println!("We're too far away!");
                    *state = ActionState::Failure;
                }
            }
            // All Actions should make sure to handle cancellations!
            // Drinking is not a complicated action, so we can just interrupt it immediately.
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

    // We use the Steps struct to essentially build a "MoveAndDrink" action by composing
    // the MoveToWaterSource and Drink actions.
    //
    // If either of the steps fails, the whole action fails. That is: if the actor somehow fails
    // to move to the water source (which is not possible in our case) they will not attempt to
    // drink either. Getting them un-stuck from that situation is then up to other possible actions.
    //
    // We build up a list of steps that make it so that the actor will...
    let move_and_drink =
        Steps::build()
            // ...move to the water source...
            .step(MoveToWaterSource { speed: 1.0 })
            // ...and then drink.
            .step(Drink { per_second: 10.0 });

    // Build the thinker
    let thinker = Thinker::build()
        // We don't do anything unless we're thirsty enough.
        .picker(FirstToScore { threshold: 0.8 })
        .when(
            Thirsty,
            move_and_drink,
        );

    cmd.spawn()
        .insert(Thirst::new(75.0, 2.0))
        .insert(Position {
            position: Vec2::new(0.0, 0.0),
        })
        .insert(
            thinker,
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
