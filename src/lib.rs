//! `big-brain` is a [Utility
//! AI](https://en.wikipedia.org/wiki/Utility_system) library for implementing
//! rich, complex artificial intelligence mainly in video games using the
//! [`specs`](https://docs.rs/specs) ECS system.
//!
//! `big-brain` not only allows you to define these complex behaviors, but it
//! allows you to define them in a data-oriented format, such as
//! [`RON`](https://docs.rs/ron), potentially allowing non-programmers to
//! define many of these behaviors themselves.
//!
//! In general, the only things that need to be programmed are
//! [Actions](derive.Action.html) and
//! [Considerations](derive.Consideration.html). Everything else is included
//! with `big-brain`.
//!
//! For example, this is what a basic thinker might look like:
//!
//! ```ignore
//! // basic_needs.ron
//! (
//!   // The first Choice to score above the threshold will be executed.
//!   picker: {"FirstToScore": (threshold: 80.0)},
//!   // A list of choices, with their considerations evaluated in order,
//!   // and picked using the Picker.
//!   choices: [(
//!     consider: [{"Bladder": ()}],
//!     then: {"Pee": ()},
//!   ), (
//!     consider: [{"Hunger": ()}],
//!     // Thinkers are also actions, so you can nest them indefinitely.
//!     then: {"Thinker": (
//!         picker: {"FirstToScore": (threshold: 80.0)},
//!         choices: [(
//!             consider: [{"FoodInRange": (range: 10.0)}],
//!             then: {"EatFood": (range: 10.0)}
//!         ), (
//!             consider: [{"FoodNearby": (range: 1000.0)}],
//!             then: {"WalkToFood": (range: 1000.0)}
//!         )]
//!     )},
//!   )],
//!   // If no Choice goes over the threshold, we just... wander around
//!   otherwise: Some({"Meander": ()})
//! )
//! ```
//!
//! You would then load up the component into a `specs` entity like so:
//!
//! ```no_run
//! use big_brain::ThinkerBuilder;
//! use specs::{Entities, Entity, LazyUpdate, World, WorldExt};
//! let mut world = World::new();
//! let entity = world.create_entity().build();
//! world.exec(|(entities, lazy): (Entities, Read<LazyUpdate>)| {
//!     ThinkerBuilder::load_from("./path/to/basic.ron").build(entity, &entities, &lazy);
//! });
//! ```

pub use bevy;
pub use big_brain_derive::*;
pub use serde;
pub use typetag;

pub use actions::*;
pub use choices::*;
pub use considerations::*;
pub use thinker::*;

pub mod evaluators;
pub mod measures;
pub mod pickers;

mod actions;
mod choices;
mod considerations;
mod stopwatch;
mod thinker;
