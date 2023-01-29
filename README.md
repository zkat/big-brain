# big-brain

[![crates.io](https://img.shields.io/crates/v/big-brain.svg)](https://crates.io/crates/big-brain)
[![docs.rs](https://docs.rs/big-brain/badge.svg)](https://docs.rs/big-brain)
[![Apache 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE.md)

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

#[derive(Debug, Clone, Component, ScorerBuilder)]
#[scorer_label = "Thirsty"]
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

#[derive(Debug, Clone, Component, ActionBuilder)]
#[action_label = "Drink"]
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

### Examples

The full source code of the above Thirst/Drink action example can be found in the [Thirst example](examples/thirst.rs).

Also, the [Sequence Example](examples/sequence.rs) example describes how to use `Steps` to compose several actions
together sequentially.

### Contributing

1. Install the latest Rust toolchain (stable supported).
2. `cargo run --example thirst`
3. Happy hacking!

### License

This project is licensed under [the Apache-2.0 License](LICENSE.md).
