`big-brain` is a [Utility AI](https://en.wikipedia.org/wiki/Utility_system)
library for games, built for the [Bevy Game Engine](https://bevyengine.org/)

It lets you define complex, intricate AI behaviors for your entities based on
their perception of the world. Definitions are heavily data-driven, using
plain Rust, and you only need to program Scorers (entities that look at your
game world and come up with a Score), and Actions (entities that perform
actual behaviors upon the world). No other code is needed for actual AI
behavior.

See [the documentation](https://docs.rs/big-brain) for more details.

## Example

First, you define actions and considerations, which are just plain old `Bevy`
`Component`s and `System`s.

### Scorers

`Scorers`s are entities that look at the world and evaluate into `Score` values.

```rust
use bevy::prelude::*;
use big_brain::*;

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

pub fn thirsty_scorer_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Actor, &mut Score), With<Thirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            score.set(thirst.thirst);
            println!("Thirst: {}", thirst.thirst);
        }
    }
}
```

### Actions

`Action`s are the actual things your entities will _do_.

```rust
use bevy::prelude::*;
use big_brain::*;

#[derive(Debug, Clone)]
pub struct Drink;

impl Drink {
    pub fn build() -> DrinkBuilder {
        DrinkBuilder
    }
}

#[derive(Debug, Clone)]
pub struct DrinkBuilder;

impl ActionBuilder for DrinkBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Drink);
    }
}

fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    mut query: Query<(&Actor, &mut ActionState), With<Drink>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
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
```

### Thinker Definition

Finally, you can use it when define the `Thinker`:

```rust
let actor = cmd.spawn().insert(Thirst::new(70.0, 2.0)).id();

let thinker = Thinker::build()
    .picker(FirstToScore { threshold: 80.0 })
    .when(Thirsty::build(), Drink::build())
    .attach(&mut cmd, actor);
cmd.entity(actor).push_children(&[thinker]);
```

## License

This project is licensed under [the Parity License](LICENSE.md). Third-party contributions are licensed under Apache-2.0 and belong to their respective authors.

The Parity License is a copyleft license that, unlike the GPL family, allows you to license derivative and connected works under permissive licenses like MIT or Apache-2.0. It's free to use provided the work you do is freely available!

For proprietary use, please [contact me](mailto:kzm@zkat.tech?subject=big-brain%20license), or just [sponsor me on GitHub](https://github.com/users/zkat/sponsorship) under the appropriate tier to [acquire a proprietary-use license](LICENSE-PATRON.md)! This funding model helps me make my work sustainable and compensates me for the work it took to write this crate!
