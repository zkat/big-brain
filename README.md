`big-brain` is a [Utility AI](https://en.wikipedia.org/wiki/Utility_system)
library for games, built for the [Bevy Game Engine](https://bevyengine.org/)

It lets you define complex, intricate AI behaviors for your entities based on
their perception of the world. Definitions are almost entirely data-driven,
using plain `.ron` files, and you only need to program Scorers (entities that
look at your game world), and Actions (entities that perform actual behaviors
upon the world). No other code is needed for actual AI behavior.

See [the documentation](https://docs.rs/big-brain) for more details.

## Example

First, you define actions and considerations, which are just plain old `Bevy`
`Component`s and `System`s.

### Scorers

`Scorers`s are entities that look at the world and evaluate into `Score` values.

```rust
use bevy::prelude::*;
use big_brain::*;

#[derive(Debug, Scorer)]
pub struct Thirsty;

pub fn score_thirst_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Parent, &mut Score), With<Thirsty>>,
) {
    for (Parent(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            *score = Score(thirst.thirst);
        }
    }
}
```

### Actions

`Action`s are the actual things your entities will _do_.

```rust
use bevy::prelude::*;
use big_brain::*;

#[derive(Debug, Action)]
pub struct Drink;

fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    mut query: Query<(&Parent, &mut ActionState), With<Drink>>,
) {
    for (Parent(actor), mut state) in query.iter_mut() {
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

```ron
(
    picker: {"FirstToScore": (threshold: 80.0)},
    choices: [(
        when: {"Bladder": ()},
        then: {"Thinker": (
            picker: {"FirstToScore": (threshold: 80.0)},
            choices: [(
                when: [{"Bladder": ()}],
                then: {"Pee": ()}
            )]
        )}
    ), (
        // Here you go!
        when: {"Thirsty": ()},
        then: {"Drink": ()},
    ), (
        when: {"Hungry": ()},
        then: {"Eat": ()},
    )],
    otherwise: Some({"Meander": ()}),
)

```

## License

This project is licensed under [the Parity License](LICENSE.md). Third-party contributions are licensed under Apache-2.0 and belong to their respective authors.

The Parity License is a copyleft license that, unlike the GPL family, allows you to license derivative and connected works under permissive licenses like MIT or Apache-2.0. It's free to use provided the work you do is freely available!

For proprietary use, please [contact me](mailto:kzm@zkat.tech?subject=big-brain%20license), or just [sponsor me on GitHub](https://github.com/users/zkat/sponsorship) under the appropriate tier to [acquire a proprietary-use license](LICENSE-PATRON.md)! This funding model helps me make my work sustainable and compensates me for the work it took to write this crate!
