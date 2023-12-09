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
//! See [the documentation](https://docs.rs/big-brain), alone with [the
//! tutorial][_tutorial::chapter_0] for more details.
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
//! ### bevy version and MSRV
//!
//! The current version of `big-brain` is compatible with `bevy` 0.12.1.
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

//#[cfg(feature = "documentation")]
pub mod _topic;
//#[cfg(feature = "documentation")]
pub mod _tutorial;

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

use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

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
///     .add_plugins((DefaultPlugins, BigBrainPlugin::new(PreUpdate)))
///     // ...insert entities and other systems.
///     .run();
#[derive(Debug, Clone, Reflect)]
#[reflect(from_reflect = false)]
pub struct BigBrainPlugin {
    #[reflect(ignore)]
    schedule: Interned<dyn ScheduleLabel>,
    #[reflect(ignore)]
    cleanup_schedule: Interned<dyn ScheduleLabel>,
}

impl BigBrainPlugin {
    /// Create the BigBrain plugin which runs the scorers, thinker and actions in the specified
    /// schedule
    pub fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
            cleanup_schedule: Last.intern(),
        }
    }

    /// Overwrite the Schedule that is used to run cleanup tasks. By default this happens in Last.
    pub fn set_cleanup_schedule(mut self, cleanup_schedule: impl ScheduleLabel) -> Self {
        self.cleanup_schedule = cleanup_schedule.intern();
        self
    }
}

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            self.schedule.intern(),
            (
                BigBrainSet::Scorers,
                BigBrainSet::Thinkers,
                BigBrainSet::Actions,
            )
                .chain(),
        )
        .configure_sets(self.cleanup_schedule.intern(), BigBrainSet::Cleanup)
        .add_systems(
            self.schedule.intern(),
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
        )
        .add_systems(
            self.schedule.intern(),
            thinker::thinker_system.in_set(BigBrainSet::Thinkers),
        )
        .add_systems(
            self.schedule.intern(),
            (actions::steps_system, actions::concurrent_system).in_set(BigBrainSet::Actions),
        )
        .add_systems(
            self.cleanup_schedule.intern(),
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
