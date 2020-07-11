`big-brain` is a [Utility AI](https://en.wikipedia.org/wiki/Utility_system)
library for games, built on the [`specs` ECS](https://docs.rs/specs).

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

`Consideration`s are entities that look at the world and evaluate into `Utility`s.

```rust
use specs::{Component, Entity, ReadStorage, System, WriteStorage};
use big_brain::{Consideration, Utility};

use crate::components;

#[derive(Debug, Component, Consideration)]
pub struct Hunger {
    pub actor: Entity,
    #[consideration(default)]
    pub evaluator: PowerEvaluator,
    #[consideration(param)]
    pub weight: f32,
}

pub struct ConsiderHunger;
impl<'a> System<'a> for ConsiderHunger {
    type SystemData = (
        ReadStorage<'a, components::Hunger>,
        WriteStorage<'a, Hunger>,
        WriteStorage<'a, Utility>,
    );

    fn run(&mut self, (hungers, mut considerers, mut utilities): Self::SystemData) {
        for (conser, util) in (&mut considerers, &mut utilities).join() {
            if let Some(hunger) = hungers.get(conser.actor.clone()) {
                *util = Utility {
                    value: conser.evaluator.evaluate(hunger.hunger),
                    weight: conser.weight,
                };
            }
        }
    }
}
```

### Actions

`Action`s are the actual things your entities will _do_.

```rust
use specs::{Component, Entity, System, WriteStorage};
use big_brain::{Action, ActionState};

use crate::components;

#[derive(Debug, Clone, Component, Action)]
pub struct Eat {
    pub actor: Entity,
    #[action(default)]
    pub foo: f32,
    #[action(param)]
    pub reduce_by: f32,
}

pub struct EatSystem;
impl<'a> System<'a> for EatSystem {
    type SystemData = (
        WriteStorage<'a, components::Hunger>,
        WriteStorage<'a, Eat>,
        WriteStorage<'a, ActionState>,
    );
    fn run(&mut self, (mut hungers, mut eat_actions, mut states): Self::SystemData) {
        for (state, eat_action) in (&mut states, &mut eat_actions).join() {
            if let Some(hunger) = hungers.get_mut(eat_action.actor.clone()) {
                match state {
                    ActionState::Requested => {
                        hunger.hunger -= eat_action.reduce_by;
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
}
```

### Thinker Definition

Finally, you can define the `Thinker`:

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
