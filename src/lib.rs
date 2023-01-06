/*!
`big-brain` is a [Utility AI](https://en.wikipedia.org/wiki/Utility_system)
library for games, built for the [Bevy Game Engine](https://bevyengine.org/)

It lets you define complex, intricate AI behaviors for your entities based on
their perception of the world. Definitions are heavily data-driven, using
plain Rust, and you only need to program Scorers (entities that look at your
game world and come up with a Score), and Actions (entities that perform
actual behaviors upon the world). No other code is needed for actual AI
behavior.

See [the documentation](https://docs.rs/big-brain) for more details.

### Features

* Highly concurrent/parallelizable evaluation.
* Integrates smoothly with Bevy.
* Proven game AI model.
* Highly composable and reusable.
* State machine-style continuous actions/behaviors.
* Action cancellation.

### Example

As a developer, you write application-dependent code to define
[`Scorers`](#scorers) and [`Actions`](#actions), and then put it all together
like building blocks, using [`Thinkers`](#thinkers) that will define the
actual behavior.

#### Scorers

`Scorer`s are entities that look at the world and evaluate into [`Score`](scorers::Score) values. You can think of them as the "eyes" of the AI system. They're a highly-parallel way of being able to look at the `World` and use it to make some decisions later.

```rust
use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Thirsty;

pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            score.set(thirst.thirst);
        }
    }
}
```

#### Actions

`Action`s are the actual things your entities will _do_. They are connected to
[`ActionState`](actions::ActionState)s that represent the current execution
state of the state machine.

```rust
use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Drink;

fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    mut query: Query<(&Actor, &mut ActionState), With<Drink>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok(mut thirst) = thirsts.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    thirst.thirst = 10.0;
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
```

#### Thinkers

Finally, you can use it when define the [`Thinker`](thinker::Thinker), which you can attach as a
regular Component:

```rust
cmd.spawn().insert(Thirst::new(70.0, 2.0)).insert(
    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
        .when(Thirsty, Drink),
);
```

#### App

Once all that's done, we just add our systems and off we go!

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(BigBrainPlugin)
    .add_startup_system(init_entities)
    .add_system(thirst_system)
    .add_system_to_stage(BigBrainStage::Actions, drink_action_system)
    .add_system_to_stage(BigBrainStage::Scorers, thirsty_scorer_system)
    .run();
```

### Contributing

1. Install the latest Rust toolchain (stable supported).
2. `cargo run --example thirst`
3. Happy hacking!

### License

This project is licensed under [the Apache-2.0 License](LICENSE.md).

*/

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
    pub use super::BigBrainStage;
    pub use actions::{ActionBuilder, ActionState, Concurrently, Steps};
    pub use big_brain_derive::{ActionBuilder, ScorerBuilder};
    pub use measures::{ChebyshevDistance, Measure, WeightedProduct, WeightedSum};
    pub use pickers::{FirstToScore, Highest, Picker};
    pub use scorers::{
        AllOrNothing, FixedScore, ProductOfScorers, Score, ScorerBuilder, SumOfScorers,
        WinningScorer,
    };
    pub use thinker::{
        Action, ActionSpan, Actor, HasThinker, Scorer, ScorerSpan, Thinker, ThinkerBuilder,
    };
}

use bevy::prelude::*;

/**
Core [`Plugin`] for Big Brain behavior. Required for any of the [`Thinker`](thinker::Thinker)-related magic to work.

### Example

```no_run
use bevy::prelude::*;
use big_brain::prelude::*;

App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(BigBrainPlugin)
    // ...insert entities and other systems.
    .run();
*/
pub struct BigBrainPlugin;

impl Plugin for BigBrainPlugin {
    fn build(&self, app: &mut App) {
        use CoreStage::*;

        app.add_stage_after(First, BigBrainStage::Scorers, SystemStage::parallel());
        app.add_system_set_to_stage(
            BigBrainStage::Scorers,
            SystemSet::new()
                .with_system(scorers::fixed_score_system)
                .with_system(scorers::measured_scorers_system)
                .with_system(scorers::all_or_nothing_system)
                .with_system(scorers::sum_of_scorers_system)
                .with_system(scorers::product_of_scorers_system)
                .with_system(scorers::winning_scorer_system)
                .with_system(scorers::evaluating_scorer_system),
        );

        app.add_stage_after(
            BigBrainStage::Scorers,
            BigBrainStage::Thinkers,
            SystemStage::parallel(),
        );
        app.add_system_to_stage(BigBrainStage::Thinkers, thinker::thinker_system);

        app.add_stage_after(PreUpdate, BigBrainStage::Actions, SystemStage::parallel());
        app.add_system_set_to_stage(
            BigBrainStage::Actions,
            SystemSet::new()
                .with_system(actions::steps_system)
                .with_system(actions::concurrent_system),
        );

        app.add_stage_after(Last, BigBrainStage::Cleanup, SystemStage::parallel());
        app.add_system_set_to_stage(
            BigBrainStage::Cleanup,
            SystemSet::new()
                .with_system(thinker::thinker_component_attach_system)
                .with_system(thinker::thinker_component_detach_system)
                .with_system(thinker::actor_gone_cleanup),
        );
    }
}

/**
BigBrainPlugin execution stages. Use these to schedule your own actions/scorers/etc.
*/
#[derive(Clone, Debug, Hash, Eq, PartialEq, StageLabel)]
pub enum BigBrainStage {
    /// Scorers are evaluated in this stage.
    Scorers,
    /// Actions are executed in this stage.
    Actions,
    /// Thinkers run their logic in this stage.
    Thinkers,
    /// Various internal cleanup items run in this final stage.
    Cleanup,
}
