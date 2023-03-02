//! [![crates.io](https://img.shields.io/crates/v/big-brain.svg)](https://crates.io/crates/big-brain)
//! [![docs.rs](https://docs.rs/big-brain/badge.svg)](https://docs.rs/big-brain)
//! [![Apache
//! 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE.md)
//!
//! `big-brain` is a [Utility
//! AI](https://en.wikipedia.org/wiki/Utility_system) library for games, built
//! for the [Bevy Game Engine](https://bevyengine.org/)
//!
//! It lets you define complex, intricate AI behaviors for your entities based
//! on their perception of the world. Definitions are heavily data-driven,
//! using plain Rust, and you only need to program Scorers (entities that look
//! at your game world and come up with a Score), and Actions (entities that
//! perform actual behaviors upon the world). No other code is needed for
//! actual AI behavior.
//!
//! See [the documentation](https://docs.rs/big-brain) for more details.
//!
//! ### Features
//!
//! * Highly concurrent/parallelizable evaluation.
//! * Integrates smoothly with Bevy.
//! * Proven game AI model.
//! * Highly composable and reusable.
//! * State machine-style continuous actions/behaviors.
//! * Action cancellation.
//!
//! ### Example
//!
//! As a developer, you write application-dependent code to define
//! [`Scorers`](#scorers) and [`Actions`](#actions), and then put it all
//! together like building blocks, using [`Thinkers`](#thinkers) that will
//! define the actual behavior.
//!
//! #### Scorers
//!
//! `Scorer`s are entities that look at the world and evaluate into `Score`
//! values. You can think of them as the "eyes" of the AI system. They're a
//! highly-parallel way of being able to look at the `World` and use it to
//! make some decisions later.
//!
//! ```rust
//! use bevy::prelude::*;
//! use big_brain::prelude::*;
//! # #[derive(Component, Debug)]
//! # struct Thirst { thirst: f32 }
//!
//! #[derive(Debug, Clone, Component, ScorerBuilder)]
//! pub struct Thirsty;
//!
//! pub fn thirsty_scorer_system(
//!     thirsts: Query<&Thirst>,
//!     mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
//! ) {
//!     for (Actor(actor), mut score) in query.iter_mut() {
//!         if let Ok(thirst) = thirsts.get(*actor) {
//!             score.set(thirst.thirst);
//!         }
//!     }
//! }
//! ```
//!
//! #### Actions
//!
//! `Action`s are the actual things your entities will _do_. They are
//! connected to `ActionState`s that represent the current execution state of
//! the state machine.
//!
//! ```rust
//! use bevy::prelude::*;
//! use big_brain::prelude::*;
//! # #[derive(Component, Debug)]
//! # struct Thirst { thirst: f32 }
//!
//! #[derive(Debug, Clone, Component, ActionBuilder)]
//! pub struct Drink;
//!
//! fn drink_action_system(
//!     mut thirsts: Query<&mut Thirst>,
//!     mut query: Query<(&Actor, &mut ActionState), With<Drink>>,
//! ) {
//!     for (Actor(actor), mut state) in query.iter_mut() {
//!         if let Ok(mut thirst) = thirsts.get_mut(*actor) {
//!             match *state {
//!                 ActionState::Requested => {
//!                     thirst.thirst = 10.0;
//!                     *state = ActionState::Success;
//!                 }
//!                 ActionState::Cancelled => {
//!                     *state = ActionState::Failure;
//!                 }
//!                 _ => {}
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! #### Thinkers
//!
//! Finally, you can use it when define the `Thinker`, which you can attach as
//! a regular Component:
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use big_brain::prelude::*;
//! # #[derive(Debug, Component)]
//! # struct Thirst(f32, f32);
//! # #[derive(Debug, Clone, Component, ScorerBuilder)]
//! # struct Thirsty;
//! # #[derive(Debug, Clone, Component, ActionBuilder)]
//! # struct Drink;
//! fn spawn_entity(cmd: &mut Commands) {
//!     cmd.spawn((
//!         Thirst(70.0, 2.0),
//!         Thinker::build()
//!             .picker(FirstToScore { threshold: 0.8 })
//!             .when(Thirsty, Drink),
//!     ));
//! }
//! ```
//!
//! #### App
//!
//! Once all that's done, we just add our systems and off we go!
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use big_brain::prelude::*;
//! # fn init_entities() {}
//! # fn thirst_system() {}
//! # fn drink_action_system() {}
//! # fn thirsty_scorer_system() {}
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(BigBrainPlugin)
//!         .add_startup_system(init_entities)
//!         .add_system(thirst_system)
//!         .add_system(drink_action_system.in_set(BigBrainSet::Actions))
//!         .add_system(thirsty_scorer_system.in_set(BigBrainSet::Scorers))
//!         .run();
//! }
//! ```
//!
//! ### bevy version and MSRV
//!
//! The current version of `big-brain` is compatible with `bevy` 0.9.0.
//!
//! The Minimum Supported Rust Version for `big-brain` should be considered to
//! be the same as `bevy`'s, which as of the time of this writing was "the
//! latest stable release".
//!
//! ### Reflection
//!
//! All relevant `big-brain` types implement the bevy `Reflect` trait, so you
//! should be able to get some useful display info while using things like
//! [`bevy_inspector_egui`](https://crates.io/crates/bevy_inspector_egui).
//!
//! This implementation should **not** be considered stable, and individual
//! fields made visible may change at **any time** and not be considered
//! towards semver. Please use this feature **only for debugging**.
//!
//! ### Contributing
//!
//! 1. Install the latest Rust toolchain (stable supported).
//! 2. `cargo run --example thirst`
//! 3. Happy hacking!
//!
//! ### License
//!
//! This project is licensed under [the Apache-2.0 License](LICENSE.md).

pub mod evaluators;
pub mod pickers;

pub mod actions;
pub mod choices;
pub mod measures;
pub mod scorers;
pub mod thinker;

pub mod prelude {
    /*!
    Convenience module with the core types you're most likely to use when working with Big Brain. Mean to be used like `use big_brain::prelude::*;`
    */
    use super::*;

    pub use super::BigBrainPlugin;
    pub use super::BigBrainSet;
    pub use actions::{ActionBuilder, ActionState, ConcurrentMode, Concurrently, Steps};
    pub use big_brain_derive::{ActionBuilder, ScorerBuilder};
    pub use evaluators::{Evaluator, LinearEvaluator, PowerEvaluator, SigmoidEvaluator};
    pub use measures::{ChebyshevDistance, Measure, WeightedProduct, WeightedSum};
    pub use pickers::{FirstToScore, Highest, Picker};
    pub use scorers::{
        AllOrNothing, EvaluatingScorer, FixedScore, MeasuredScorer, ProductOfScorers, Score,
        ScorerBuilder, SumOfScorers, WinningScorer,
    };
    pub use thinker::{
        Action, ActionSpan, Actor, HasThinker, Scorer, ScorerSpan, Thinker, ThinkerBuilder,
    };
}

use bevy::prelude::*;

/// Core [`Plugin`] for Big Brain behavior. Required for any of the
/// [`Thinker`](thinker::Thinker)-related magic to work.
///
/// ### Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use big_brain::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugin(BigBrainPlugin)
///     // ...insert entities and other systems.
///     .run();
#[derive(Debug, Clone, Reflect)]
pub struct BigBrainPlugin;

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets((
            BigBrainSet::Scorers.in_base_set(CoreSet::First),
            BigBrainSet::Thinkers
                .in_base_set(CoreSet::First)
                .after(BigBrainSet::Scorers),
            BigBrainSet::Actions.in_base_set(CoreSet::PreUpdate),
            BigBrainSet::Cleanup.in_base_set(CoreSet::Last),
        ));

        app.add_systems(
            (
                scorers::fixed_score_system,
                scorers::measured_scorers_system,
                scorers::all_or_nothing_system,
                scorers::sum_of_scorers_system,
                scorers::product_of_scorers_system,
                scorers::winning_scorer_system,
                scorers::evaluating_scorer_system,
            )
                .in_set(BigBrainSet::Scorers),
        );

        app.add_system(thinker::thinker_system.in_set(BigBrainSet::Thinkers));

        app.add_systems(
            (actions::steps_system, actions::concurrent_system).in_set(BigBrainSet::Actions),
        );

        app.add_systems(
            (
                thinker::thinker_component_attach_system,
                thinker::thinker_component_detach_system,
                thinker::actor_gone_cleanup,
            )
                .in_set(BigBrainSet::Cleanup),
        );
    }
}

/// [`BigBrainPlugin`] system sets. Use these to schedule your own
/// actions/scorers/etc.
#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet, Reflect)]
pub enum BigBrainSet {
    /// Scorers are evaluated in this set.
    Scorers,
    /// Actions are executed in this set.
    Actions,
    /// Thinkers run their logic in this set.
    Thinkers,
    /// Various internal cleanup items run in this final set.
    Cleanup,
}
