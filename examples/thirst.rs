use bevy::prelude::*;
use big_brain::prelude::*;

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

// First, you need an Action and an ActionBuilder struct.
//
// These actions will be spawned and queued by the game engine when their
// conditions trigger (we'll configure what these are later).
#[derive(Debug, Clone)]
pub struct Drink;

// The convention is to attach a `::build()` function to the Action type.
impl Drink {
    pub fn build() -> DrinkBuilder {
        DrinkBuilder
    }
}

// Then we define an ActionBuilder, which is responsible for making new
// Action components for us.
#[derive(Debug, Clone)]
pub struct DrinkBuilder;

// All you need to implement heree is the `build()` method, which requires
// that you attach your actual component to the action Entity that was created
// and configured for you.
impl ActionBuilder for DrinkBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Drink);
    }
}

// Associated with that Drink Action, you then need to have a system that will
// actually execute those actions when they're "spawned" by the Big Brain
// engine. This is the actual "act" part of the Action.
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
    mut query: Query<(&Actor, &mut ActionState), With<Drink>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
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

#[derive(Debug, Clone)]
struct Idle;

impl Idle {
    fn build() -> IdleBuilder {
        IdleBuilder
    }
}

#[derive(Debug, Clone)]
struct IdleBuilder;

impl ActionBuilder for IdleBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Idle);
    }
}

fn idle_system(mut query: Query<&mut ActionState, With<Idle>>) {
    for mut state in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Cancelled => {
                *state = ActionState::Success;
            }
            ActionState::Executing => {}
            _ => {}
        }
    }
}
// Then, we have something called "Scorers". These are special components that
// run in the background, calculating a "Score" value, which is what Big Brain
// will use to pick which actions to execute.
//
// Just like with Actions, we use the convention of having separate
// ScorerBuilder and Scorer components. While it might seem like a lot of
// boilerplate, in a "real" application, you will almost certainly have data
// and configuration concerns. This pattern separates those nicely.
#[derive(Debug, Clone)]
pub struct Thirsty;

impl Thirsty {
    fn build() -> ThirstyBuilder {
        ThirstyBuilder
    }
}

#[derive(Debug, Clone)]
pub struct ThirstyBuilder;

impl ScorerBuilder for ThirstyBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(Thirsty);
    }
}

// Looks familiar? It's a lot likee Actions!
pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    // Same dance with the Parent here, but now Big Brain has added a Score component!
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            // This is really what the job of a Scorer is. To calculate a
            // generic Utility value that the Big Brain engine will compare
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
    cmd.spawn().insert(Thirst::new(70.0, 2.0)).insert(
        // Thinker::build().component() will return a regular component you
        // can attach normally!
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            // Note that what we pass in are _builders_, not components!
            .when(Thirsty::build(), Drink::build())
            .otherwise(Idle::build()),
    );
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
        .add_system(idle_system.system())
        .run();
}
