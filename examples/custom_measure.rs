//! This example demonstrates how to build a custom measure and use that
//! in a Thinker.

use bevy::log::LogSettings;
use bevy::prelude::*;
use bevy::utils::tracing::{debug, trace};
use big_brain::prelude::*;
use big_brain::scorers::MeasuredScorer;

// Lets define a custom measure. There are quite a few built-in ones in big-brain,
// so we'll create a slightly useless Measure that sums together the weighted scores,
// but weights get divided by the Scorer's index in the Vec.
#[derive(Debug, Clone)]
pub struct SumWithDecreasingWeightMeasure;

impl Measure for SumWithDecreasingWeightMeasure {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .enumerate()
            .fold(0f32, |acc, (idx, (score, weight))| {
                acc + score.get() * weight / (1.0 + idx as f32)
            })
    }
}

// We'll keep this example fairly simple, let's have Waffles and Pancakes, and
// try to optimise our happiness based on keeping our waffle and pancake level high.
// Its kind of like the thirst example but sweeter.
#[derive(Component, Debug)]
pub struct Pancakes(pub f32);

#[derive(Component, Debug)]
pub struct Waffles(pub f32);

pub fn eat_dessert(time: Res<Time>, mut pancakes: Query<(&mut Pancakes, &mut Waffles)>) {
    let delta_t = time.delta_seconds();

    for (mut pancake, mut waffle) in pancakes.iter_mut() {
        pancake.0 = (pancake.0 - delta_t).max(0.0);
        waffle.0 = (waffle.0 - delta_t).max(0.0);

        info!("Pancake: {}, waffle: {}", pancake.0, waffle.0);
    }
}

// We have two actions, we can either eat pancakes or waffles, but not both, or....
// no no no, let's keep this sensible. Speaking of "sensible", as these actions are
// very similar we'll use generics to save writing them twice. We need a trait to
// update the pancake/waffle state
pub trait EatFood {
    fn get(&self) -> f32;
    fn eat(&mut self, amount: f32);
}

impl EatFood for Pancakes {
    fn get(&self) -> f32 {
        self.0
    }

    fn eat(&mut self, amount: f32) {
        self.0 = (self.0 + amount).clamp(0.0, 100.0)
    }
}
impl EatFood for Waffles {
    fn get(&self) -> f32 {
        self.0
    }

    fn eat(&mut self, amount: f32) {
        self.0 = (self.0 + amount).clamp(0.0, 100.0)
    }
}

// ok so now we can specify our actions
#[derive(Clone, Component, Debug)]
pub struct EatPancakes;

#[derive(Clone, Component, Debug)]
pub struct EatWaffles;

fn eat_thing_action<
    TActionMarker: std::fmt::Debug + Component,
    TActorMarker: Component + EatFood,
>(
    time: Res<Time>,
    mut items: Query<&mut TActorMarker>,
    // We execute actions by querying for their associated Action Component
    // (Drink in this case). You'll always need both Actor and ActionState.
    mut query: Query<(&Actor, &mut ActionState, &TActionMarker, &ActionSpan)>,
) {
    for (Actor(actor), mut state, action_marker, span) in query.iter_mut() {
        let _guard = span.span().enter();

        if let Ok(mut item) = items.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Time to eat something - {:?}", action_marker);
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    trace!("Eating {:?}", action_marker);

                    item.eat(time.delta_seconds() * 5.0);

                    if item.get() > 95.0 {
                        debug!("Done eating {:?}", action_marker);
                        *state = ActionState::Success;
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    debug!(
                        "Cancelled eating {:?}. Considering this a failure.",
                        action_marker
                    );
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

// Next we need to implement our Scorers, one for each of our Pancake and Waffle eating habits.
#[derive(Clone, Component, Debug)]
pub struct CravingPancakes;

#[derive(Clone, Component, Debug)]
pub struct CravingWaffles;

// We can make our Scorer generic as well I guess?
pub fn craving_food_scorer<
    TScoreMarker: std::fmt::Debug + Component,
    TActorMarker: Component + EatFood,
>(
    items: Query<&TActorMarker>,
    mut query: Query<(&Actor, &mut Score), With<TScoreMarker>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(item) = items.get(*actor) {
            score.set((1.0 - item.get() / 100.0).clamp(0.0, 1.0));
        }
    }
}

// Let's set up our world
pub fn init_entities(mut cmd: Commands) {
    cmd.spawn()
        .insert(Pancakes(50.0))
        .insert(Waffles(50.0))
        .insert(
            Thinker::build()
                .label("Hungry Thinker")
                .picker(Highest)
                // we use our custom measure here. The impact of the custom measure is that the
                // pancakes should be down-weighted. This means despite this being listed first,
                // all things being equal we should consume pancakes before waffles.
                .when(
                    MeasuredScorer::build(0.05)
                        .label("waffles")
                        .measure(SumWithDecreasingWeightMeasure)
                        .push(CravingWaffles, 0.5)
                        .push(CravingPancakes, 0.5),
                    EatPancakes,
                )
                // we use the default measure here
                .when(
                    MeasuredScorer::build(0.05)
                        .label("pancakes")
                        .push(CravingPancakes, 0.5)
                        .push(CravingWaffles, 0.5),
                    EatPancakes,
                ),
        );
}

fn main() {
    // Once all that's done, we just add our systems and off we go!
    App::new()
        .insert_resource(LogSettings {
            // Use `RUST_LOG=big_brain=trace,thirst=trace cargo run --example
            // custom_measure --features=trace` to see extra tracing output.
            filter: "big_brain=debug,thirst=debug".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BigBrainPlugin)
        .add_startup_system(init_entities)
        .add_system(eat_dessert)
        .add_system_to_stage(
            BigBrainStage::Actions,
            eat_thing_action::<EatPancakes, Pancakes>,
        )
        .add_system_to_stage(
            BigBrainStage::Actions,
            eat_thing_action::<EatWaffles, Waffles>,
        )
        .add_system_to_stage(
            BigBrainStage::Scorers,
            craving_food_scorer::<CravingPancakes, Pancakes>,
        )
        .add_system_to_stage(
            BigBrainStage::Scorers,
            craving_food_scorer::<CravingWaffles, Waffles>,
        )
        .run();
}
