//! # Chapter 0 - Overview
//!
//! Big Brain is a highliy-concurrent [Utility
//! AI](https://en.wikipedia.org/wiki/Utility_system) library for games, built
//! for the [Bevy Game Engine](https://bevyengine.org/). The building blocks
//! for Big Brain can be highly-generic and highly-composable, letting you
//! reuse logic across your game by defining them in small pieces and then
//! just putting them all together in a declarative way.
//!
//! ### High-level Overview
//!
//! As a developer, you write application-dependent code to define [`Scorer`]s
//! and [`Action`]s, and then put it all together like building blocks, using
//! [`Thinker`]s that will define the actual behavior.
//!
//! We'll go over each of these at a high-level, and cover them in more
//! details later.
//!
//! #### Scorers
//!
//! [`Scorer`]s are entities that look at the world and evaluate into
//! [`Score`] values. You can think of them as the "eyes" of the AI system.
//! They're a highly-parallel way of being able to look at the `World` and use
//! it to make some decisions later.
//!
//! They are constructed using [`ScorerBuilder`]s, for which there's a handy
//! shortcut `derive` macro.
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
//! [`Action`]s are the actual things your entities will _do_. They are
//! connected to [`ActionState`]s that represent the current execution state
//! of the state machine. You can think of them as deconstructed async
//! functions.
//!
//! They are constructed using [`ActionBuilder`]s, for which there's a handy
//! shortcut `derive` macro.
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
//! Finally, you can use it when define the [`Thinker`], which you can attach
//! as a regular [`Component`]:
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
//!         .add_plugins(BigBrainPlugin::new(PreUpdate))
//!         .add_systems(Startup, init_entities)
//!         .add_systems(Update, thirst_system)
//!         .add_systems(PreUpdate, drink_action_system.in_set(BigBrainSet::Actions))
//!         .add_systems(PreUpdate, thirsty_scorer_system.in_set(BigBrainSet::Scorers))
//!         .run();
//! }
//! ```
//!
//! #### What's next?
//!
//! You can read the general [crate docs][crate], or continue with the [next
//! chapter of the tutorial][_tutorial::chapter_1] for an expanded guide on
//! how to build out and organize the AI for your game using Big Brain!.
#[allow(unused_imports)]
use bevy::prelude::*;

#[allow(unused_imports)]
use crate::prelude::{Action, ActionBuilder, ActionState, Score, Scorer, ScorerBuilder, Thinker};

pub use super::chapter_1 as next;
pub use crate::_tutorial as table_of_contents;
