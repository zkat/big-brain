use bevy::prelude::*;
use big_brain::*;

// First, we define a "Thirst" component and associated system. This is NOT
// THE AI. It's a plain old system that just makes an entity "thirstier" over
// time. This is what the AI will later interact with.
//
// There's nothing special here. It's a plain old Bevy component.
#[derive(Debug)]
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
        thirst.thirst += thirst.per_second * (time.delta().as_micros() as f32 / 1000000.0);
        if thirst.thirst >= 100.0 {
            thirst.thirst = 100.0;
        }
    }
}

// The second step is to define an action. What can the AI do, and how does it
// do it? This is the first bit involving Big Brain itself, and there's a few
// pieces you need:

// First, you need an Action struct, and derive Action.
//
// These actions will be spawned and queued by the game engine when their
// conditions trigger (we'll configure what these are later).
#[derive(Debug, Action)]
pub struct Drink;

// Associated with that Drink Action, you then need to have a system that will
// actually execute those actions when they're "spawned" by the Big Brain
// engine.
//
// In our case, we want the Thirst components, since we'll be changing those.
// Additionally, we want to pick up the DrinkAction components, as well as
// their associated ActionState. Note that the Drink Action belongs to a
// *separate entity* from the owner of the Thirst component!
fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    // We grab the Parent here, because individual Actions are parented to the
    // entity "doing" the action.
    //
    // ActionState is an enum that described the specific run-state the action
    // is in. You can think of Actions as state machines. They get requested,
    // they can be cancelled, they can run to completion, etc. Cancellations
    // usually happen because the target action changed (due to a different
    // Scorer winning). But you can also cancel the actions yourself by
    // setting the state in the Action system.
    mut query: Query<(&Parent, &mut ActionState), With<Drink>>,
) {
    for (Parent(actor), mut state) in query.iter_mut() {
        // Use the drink_action's actor to look up the corresponding Thirst.
        if let Ok(mut thirst) = thirsts.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    thirst.thirst = 10.0;
                    println!("drank some water");
                    *state = ActionState::Success;
                }
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
// will use to pick which actions to execute.
//
// Additionally, though, we pull in an evaluator and define a weight. Which is
// just mathy stuff you can tweak to get the behavior you want. More on this
// in the docs (later), but for now, just put them in there and trust the
// system. :)
#[derive(Debug, Scorer)]
pub struct Thirsty;

// Look familiar? Similar dance to Actions here.
pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    // Same dance with the Parent here, but now we've added a Score!
    mut query: Query<(&Parent, &mut Score), With<Thirsty>>,
) {
    for (Parent(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            // This is really what the job of a Scorer is. To calculate a
            // generic Utility value that the Big Brain engine will compare
            // against others, over time, and use to make decisions. This is
            // generally "the higher the better", and "first across the finish
            // line", but that's all configurable using Pickers!
            //
            // The score here must be between 0.0 and 100.0.
            score.set(thirst.thirst);
            println!("Thirst: {}", thirst.thirst);
        }
    }
}

// Now that we hav eall that defined, it's time to add a Thinker to an entity!
// The Thinker is the actual "brain" behind all the AI. Every entity you want
// to have AI behavior should have one *or more* Thinkers attached to it.
// Thinkers are configured using RON right now, with a DSL that makes it easy
// to define, in data, the actual behavior you want.
pub fn init_entities(mut cmd: Commands) {
    let actor = cmd.spawn().insert(Thirst::new(70.0, 2.0)).id();

    // Here's a very simple one that only has one scorer and one
    // associated action. But you can have more of them, and even nest them by
    // using more Thinkers (which are actually themselves Actions). See
    // basic.ron in examples/ for a more involved Thinker definition.
    //
    // Ultimately, these Thinkers are meant to be usable by non-programmers:
    // You, the developer, create Actions and Scorers, and someone else
    // is then able to put them all together like LEGOs into all sorts of
    // intricate logic.
    Thinker::load_from_str(
        r#"
(
    picker: {"FirstToScore": (threshold: 80.0)},
    choices: [(
        when: {"Thirsty": ()},
        // This action will fire when (and as long as) Thirsty scores >=80.0.
        then: {"Drink": ()},
    )],
)
"#,
    )
    .build(actor, &mut cmd);
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities.system())
        .add_system(thirst_system.system())
        .add_system(drink_action_system.system())
        .add_system(thirsty_scorer_system.system())
        .run();
}
