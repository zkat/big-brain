//! This example describes how to create a composite action that executes multiple sub-actions
//! concurrently.
//!
//! `Race` succeeds when any of the sub-actions succeed.
//! `Join` succeeds if all the sub-actions succeed.
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::tracing::debug;
use big_brain::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

/// An action where the actor has to guess a given number
#[derive(Clone, Component, Debug)]
pub struct GuessNumber {
    // Number to guess (between 0 and 10 included)
    to_guess: u8,
    // Rng to perform guesses
    rng: SmallRng,
}


fn guess_number_action(
    // A query on all current MoveToWaterSource actions.
    mut action_query: Query<(&Actor, &mut ActionState, &mut GuessNumber, &ActionSpan)>,
) {
    // Loop through all actions, just like you'd loop over all entities in any other query.
    for (_actor, mut action_state, mut guess_number, span) in &mut action_query {
        let _guard = span.span().enter();

        // Different behavior depending on action state.
        match *action_state {
            // Action was just requested; it hasn't been seen before.
            ActionState::Requested => {
                debug!("Let's try to guess the secret number: {:?}", guess_number.to_guess);
                // We don't really need any initialization code here, since the queries are cheap enough.
                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                // Guess a number. If we guessed right, succeed; else keep trying.
                let guess: u8 = guess_number.rng.gen_range(0..=10);
                debug!("Guessed: {:?}", guess);
                if guess == guess_number.to_guess {
                    debug!("Guessed the secret number: {:?}! Action succeeded.", guess_number.to_guess);
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

// We will use a dummy scorer that always returns 1.0
#[derive(Clone, Component, Debug)]
pub struct DummyScorer;

pub fn dummy_scorer_system(
    mut query: Query<(&Actor, &mut Score), With<DummyScorer>>,
) {
    for (Actor(_actor), mut score) in &mut query {
        score.set(1.0);
    }
}

pub fn init_entities(mut cmd: Commands) {

    let number_to_guess: u8 = 5;
    // We use the Race struct to build a composite action that will try to guess
    // multiple numbers. If any of the guesses are right, the whole `Race` action succeeds.
    let race_guess_numbers = Race::build()
        .label("RaceToGuessNumbers")
        // ...try to guess a first number
        .push(GuessNumber { to_guess: number_to_guess, rng: SmallRng::from_entropy() })
        // ...try to guess a second number
        .push(GuessNumber { to_guess: number_to_guess, rng: SmallRng::from_entropy() });

    // We use the Join struct to build a composite action that will try to guess
    // multiple numbers. If all of the guesses are right, the whole `Race` action succeeds.
    let join_guess_numbers = Join::build()
        .label("JoinToGuessNumbers")
        // ...try to guess a first number
        .push(GuessNumber { to_guess: number_to_guess, rng: SmallRng::from_entropy() })
        // ...try to guess a second number
        .push(GuessNumber { to_guess: number_to_guess, rng: SmallRng::from_entropy() });

    // We'll use `Steps` to execute a sequence of actions.
    // First, we'll guess the numbers with 'Race', and then we'll guess the numbers with 'Join'
    // See the `sequence.rs` example for more details.
    let guess_numbers = Steps::build()
        .label("RaceAndThenJoin")
        .step(race_guess_numbers)
        .step(join_guess_numbers);

    // Build the thinker
    let thinker = Thinker::build()
        .label("GuesserThinker")
        // always select the action with the highest score
        .picker(Highest)
        .when(DummyScorer, guess_numbers);

    cmd.spawn( thinker);
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            // Use `RUST_LOG=big_brain=trace,thirst=trace cargo run --example thirst --features=trace` to see extra tracing output.
            filter: "big_brain=debug,race=debug".to_string(),
            ..default()
        }))
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities)
        .add_system_to_stage(BigBrainStage::Actions, guess_number_action)
        .add_system_to_stage(BigBrainStage::Scorers, dummy_scorer_system)
        .run();
}
