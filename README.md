`big-brain` is a [Utility AI](https://en.wikipedia.org/wiki/Utility_system)
library for games, built for the [Bevy Game Engine](https://bevyengine.org/)

It lets you define complex, intricate AI behaviors for your entities based on
their perception of the world. Definitions are almost entirely data-driven,
using plain `.ron` files, and you only need to program considerations
(entities that look at your game world), and actions (entities that perform
actual behaviors upon the world). No other code is needed for actual AI
behavior.

See [the documentation](https://docs.rs/big-brain) for more details.

## Example

First, you define actions and considerations, which are just plain old `specs`
`Component`s and `System`s.

### Considerations

`Consideration`s are entities that look at the world and evaluate into `Utility` values.

```rust
use bevy::prelude::*;
use big_brain::*;

#[derive(Debug, Consideration)]
pub struct ThirstConsideration {
    #[consideration(default)]
    pub evaluator: PowerEvaluator,
    #[consideration(param)]
    pub weight: f32,
}

pub fn thirst_consideration_system(
    thirsts: Query<&Thirst>,
    mut query: Query<(&Parent, &ThirstConsideration, &mut Utility)>,
) {
    for (Parent(actor), conser, mut util) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            *util = Utility {
                value: conser.evaluator.evaluate(thirst.thirst),
                weight: conser.weight,
            };
        }
    }
}
```

### Actions

`Action`s are the actual things your entities will _do_.

```rust
#[derive(Debug, Action)]
pub struct DrinkAction {}

fn drink_action_system(
    mut thirsts: Query<&mut Thirst>,
    mut query: Query<(&Parent, &DrinkAction, &mut ActionState)>,
) {
    for (Parent(actor), _drink_action, mut state) in query.iter_mut() {
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
// behavior.ron
(
    picker: {"FirstToScore": (threshold: 80.0)},
    otherwise: Some({"Meander": ()}),
    choices: [(
        consider: [{"Bladder": (weight: 3.0)}],
        // Thinkers are infinitely nestable! They're actually Actions themselves.
        then: {"Thinker": (
            picker: {"FirstToScore": (threshold: 80.0)},
            choices: [(
                consider: [{"Bladder": (weight: 3.0)}],
                then: {"Pee": ()}
            )]
        )}
    ), (
        consider: [{"Thirst": (weight: 2.0)}],
        then: {"Drink": ()},
    ), (
        consider: [{"Hunger": (weight: 2.0)}],
        then: {"Eat": ()},
    )],
)
```

## License

This project is licensed under [the Parity License](LICENSE.md). Third-party contributions are licensed under Apache-2.0 and belong to their respective authors.

The Parity License is a copyleft license that, unlike the GPL family, allows you to license derivative and connected works under permissive licenses like MIT or Apache-2.0. It's free to use provided the work you do is freely available!

For proprietary use, please [contact me](mailto:kzm@zkat.tech?subject=big-brain%20license), or just [sponsor me on GitHub](https://github.com/users/zkat/sponsorship) under the appropriate tier to [acquire a proprietary-use license](LICENSE-PATRON.md)! This funding model helps me make my work sustainable and compensates me for the work it took to write this crate!
