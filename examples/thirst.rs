use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::tracing::{debug, trace};
use big_brain::prelude::*;

// First, we define a "Thirst" component and associated system. This is NOT
// THE AI. It's a plain old system that just makes an entity "thirstier" over
// time. This is what the AI will later interact with.
//
// There's nothing special here. It's a plain old Bevy component.
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

pub fn thirst_system(time: Res<Time>, mut thirsts: Query<&mut Thirst>) {
    for mut thirst in &mut thirsts {
        thirst.thirst += thirst.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }
        trace!("Thirst: {}", thirst.thirst);
    }
}

// The second step is to define an action. What can the AI do, and how does it
// do it? This is the first bit involving Big Brain itself, and there's a few
// pieces you need:
//
// 1. An Action Component. This is just a plain Component we will query
//    against later.
// 2. An ActionBuilder. This is anything that implements the ActionBuilder
//    trait.
// 3. A System that will run Action code.
//
// These actions will be spawned and queued by the game engine when their
// conditions trigger (we'll configure what these are later).
//
// In most cases, the ActionBuilder just attaches the Action component to the
// actor entity. In this case, you can use the derive macro `ActionBuilder`
// to make your Action Component implement the ActionBuilder trait.
// You need your type to implement Clone and Debug (necessary for ActionBuilder)
#[derive(Clone, Component, Debug, ActionBuilder)]
#[action_label = "DrinkAction"]
pub struct Drink {
    until: f32,
    per_second: f32,
}

// Action systems execute according to a state machine, where the states are
// labeled by ActionState.
fn drink_action_system(
    time: Res<Time>,
    mut thirsts: Query<&mut Thirst>,
    // We execute actions by querying for their associated Action Component
    // (Drink in this case). You'll always need both Actor and ActionState.
    mut query: Query<(&Actor, &mut ActionState, &Drink, &ActionSpan)>,
) {
    for (Actor(actor), mut state, drink, span) in &mut query {
        // This sets up the tracing scope. Any `debug` calls here will be
        // spanned together in the output.
        let _guard = span.span().enter();

        // Use the drink_action's actor to look up the corresponding Thirst Component.
        if let Ok(mut thirst) = thirsts.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Time to drink some water!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    trace!("Drinking...");
                    thirst.thirst -=
                        drink.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
                    if thirst.thirst <= drink.until {
                        // To "finish" an action, we set its state to Success or
                        // Failure.
                        debug!("Done drinking water");
                        *state = ActionState::Success;
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    debug!("Action was cancelled. Considering this a failure.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

// Then, we have something called "Scorers". These are special components that
// run in the background, calculating a "Score" value, which is what Big Brain
// will use to pick which Actions to execute.
//
// Just like with Actions, there is a distinction between Scorer components
// and the ScorerBuilder which will attach those components to the Actor entity.
//
// Again, in most cases, you can use the `ScorerBuilder` derive macro to make your
// Scorer Component act as a ScorerBuilder. You need it to implement Clone and Debug.
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct Thirsty;

// Looks familiar? It's a lot like Actions!
pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    // Same dance with the Actor here, but now we use look up Score instead of ActionState.
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<Thirsty>>,
) {
    for (Actor(actor), mut score, span) in &mut query {
        if let Ok(thirst) = thirsts.get(*actor) {
            // This is really what the job of a Scorer is. To calculate a
            // generic "Utility" score that the Big Brain engine will compare
            // against others, over time, and use to make decisions. This is
            // generally "the higher the better", and "first across the finish
            // line", but that's all configurable using Pickers!
            //
            // The score here must be between 0.0 and 1.0.
            score.set(thirst.thirst / 100.0);
            if thirst.thirst >= 80.0 {
                span.span().in_scope(|| {
                    debug!("Thirst above threshold! Score: {}", thirst.thirst / 100.0)
                });
            }
        }
    }
}

// Now that we have all that defined, it's time to add a Thinker to an entity!
// The Thinker is the actual "brain" behind all the AI. Every entity you want
// to have AI behavior should have one *or more* Thinkers attached to it.
pub fn init_entities(mut cmd: Commands) {
    // Create the entity and throw the Thirst component in there. Nothing special here.
    cmd.spawn((
        Thirst::new(75.0, 2.0),
        Thinker::build()
            .label("My Thinker")
            .picker(FirstToScore { threshold: 0.8 })
            // Technically these are supposed to be ActionBuilders and
            // ScorerBuilders, but our Clone impls simplify our code here.
            .when(
                Thirsty,
                Drink {
                    until: 70.0,
                    per_second: 5.0,
                },
            ),
    ));
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            // Use `RUST_LOG=big_brain=trace,thirst=trace cargo run --example
            // thirst --features=trace` to see extra tracing output.
            filter: "big_brain=debug,thirst=debug".to_string(),
            ..default()
        }))
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities)
        .add_system(thirst_system)
        // Big Brain has specific stages for Scorers and Actions. If
        // determinism matters a lot to you, you should add your action and
        // scorer systems to these stages.
        .add_system_to_stage(BigBrainStage::Actions, drink_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, thirsty_scorer_system)
        .run();
}
