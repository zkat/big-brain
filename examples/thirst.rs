use bevy::prelude::*;
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
    for mut thirst in thirsts.iter_mut() {
        thirst.thirst += thirst.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }
        println!("Thirst: {}", thirst.thirst);
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
// NOTE: In this example, we're not implementing ActionBuilder ourselves, but
// instead just relying on the blanket implementation for anything that
// implements Clone. So, the Clone derive matters here. This is enough in most
// cases.
#[derive(Clone, Component, Debug)]
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
    mut query: Query<(&Actor, &mut ActionState, &Drink)>,
) {
    for (Actor(actor), mut state, drink) in query.iter_mut() {
        // Use the drink_action's actor to look up the corresponding Thirst Component.
        if let Ok(mut thirst) = thirsts.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    println!("Time to drink some water!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    println!("drinking some water");
                    thirst.thirst -=
                        drink.per_second * (time.delta().as_micros() as f32 / 1_000_000.0);
                    if thirst.thirst <= drink.until {
                        // To "finish" an action, we set its state to Success or
                        // Failure.
                        *state = ActionState::Success;
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
}

// Then, we have something called "Scorers". These are special components that
// run in the background, calculating a "Score" value, which is what Big Brain
// will use to pick which Actions to execute.
//
// Just like with Actions, there's two pieces to this: the Scorer and the
// ScorerBuilder. And just like with Actions, there's a blanket implementation
// for Clone, so we only need the Component here.
#[derive(Clone, Component, Debug)]
pub struct Thirsty;

// Looks familiar? It's a lot like Actions!
pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    // Same dance with the Actor here, but now we use look up Score instead of ActionState.
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            // This is really what the job of a Scorer is. To calculate a
            // generic "Utility" score that the Big Brain engine will compare
            // against others, over time, and use to make decisions. This is
            // generally "the higher the better", and "first across the finish
            // line", but that's all configurable using Pickers!
            //
            // The score here must be between 0.0 and 1.0.
            score.set(thirst.thirst / 100.);
        }
    }
}

// Now that we have all that defined, it's time to add a Thinker to an entity!
// The Thinker is the actual "brain" behind all the AI. Every entity you want
// to have AI behavior should have one *or more* Thinkers attached to it.
pub fn init_entities(mut cmd: Commands) {
    // Create the entity and throw the Thirst component in there. Nothing special here.
    cmd.spawn().insert(Thirst::new(75.0, 2.0)).insert(
        Thinker::build()
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
    );
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .add_plugins(DefaultPlugins)
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
