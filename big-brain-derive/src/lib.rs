use darling::FromDeriveInput;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use action::Action;
use consideration::Consideration;

mod action;
mod consideration;

/**
`Action`s in `big-brain` are defined through this derive macro. Once defined,
they can be freely used in a .ron file. They define actual behaviors that an
`actor` will perform when the Thinker engine picks it as the active action,
based on [Considerations](derive.Consideration.html).

## Definition Example

```ignore
use specs::{Component, Entity, System, WriteStorage};
use big_brain::{Action, ActionState};

// These are your game's components.
use crate::components;

// This will be used to create `Action` components. They MUST implement the
// `specs::Component` trait.
#[derive(Debug, Clone, Component, Action)]
pub struct Eat {
    // All actions **must** have a public `actor` field. This will be populated
    // with the actual actor performing the Action. The `Entity` associated with
    // the `Action` itself is distinct from the actor.
    pub actor: Entity,

    // `default` fields will be populated using default::Default() when the
    // Action is instantiated. These cannot be used as params.
    #[action(default)]
    pub foo: f32,

    // `param` fields will be populated using the value passed in through the
    // `.ron` file.
    #[action(param)]
    pub reduce_by: f32,
}

// Once an Action component is defined, we define a System that can act on it.
pub struct EatSystem;

impl<'a> System<'a> for EatSystem {
    type SystemData = (
        WriteStorage<'a, components::Hunger>,
        // This is the actual Eat component.
        WriteStorage<'a, Eat>,
        // An ActionState component is attached to every Action Entity.
        // It contains the current running status of the Action, and will be
        // updated as needed by the actor's Thinker.
        WriteStorage<'a, ActionState>,
    );
    fn run(&mut self, (mut hungers, mut eat_actions, mut states): Self::SystemData) {
        // You can join the Eat and ActionState together. They're on the same component.
        for (state, eat_action) in (&mut states, &mut eat_actions).join() {
            // Any components attached to the actor must be fetched separately.
            if let Some(hunger) = hungers.get_mut(eat_action.actor.clone()) {
                match state {
                    // At the very least, every Action should handle the
                    // `Requested` state.
                    ActionState::Requested => {
                        hunger.hunger -= eat_action.reduce_by;
                        // Success tells the Thinker that this action succeeded!
                        *state = ActionState::Success;
                    }
                    // Make sure to handle Cancelled for long-running Actions.
                    // The Thinker will not continue until the state is either
                    // Success or Failure.
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

## Usage Example

```ignore
(
    picker: {"FirstToScore": ()},
    // Actions are defined using the `then` param to Choices
    choices: [(
        consider: [{"Hunger": ()}],
        // We can use the param defined in our derive definition here.
        // The `foo` field will be defaulted and cannot be defined here.
        then: {"Eat": (reduce_by: 80.0)},
    )]
)
```

*/
#[proc_macro_derive(Action, attributes(action))]
pub fn derive_action_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let action = Action::from_derive_input(&input).unwrap();
    quote!(#action).into()
}

/**
`Consideration`s in `big-brain` are defined through this derive macro. Once defined,
they can be freely used in a .ron file. While `Action`s define behaviors,
`Consideration`s are used to determine _whether_ to execute a certain action.

`Consideration`s are responsible for determining a specific `Utility`, or score,
in Utility AI terms. This score is what sets Utility AI apart from plain old
Behavior Trees.

Like anything else in an Entity system, considerations and their behaviors
consist of a `Component` and an associated `System`.

## Definition Example

```ignore
use specs::{Component, Entity, ReadStorage, System, WriteStorage};
use big_brain::{Consideration, Utility};

// These are your game's components.
use crate::components;

// `Consideration`s are defined by deriving them -- they MUST be Components.
#[derive(Debug, Component, Consideration)]
pub struct Hunger {
    // All considerations **must** have a public `actor` field. This will be populated
    // with the actual actor considering the world around it The `Entity` associated with
    // the `Consideration` itself is distinct from the actor.
    pub actor: Entity,

    // `default` fields will be populated using default::Default() when the
    // Consideration is instantiated. These cannot be used as params.
    #[consideration(default)]
    pub evaluator: PowerEvaluator,

    // `param` fields will be populated using the value passed in through the
    // `.ron` file.
    #[consideration(param)]
    pub weight: f32,
}

pub struct ConsiderHunger;

impl<'a> System<'a> for ConsiderHunger {
    type SystemData = (
        ReadStorage<'a, components::Hunger>,

        // This is the actual `Consideration` component.
        WriteStorage<'a, Hunger>,

        // The `Utility` component associated with this `Consideration` holds
        // the current calculated score for that consideration.
        WriteStorage<'a, Utility>,
    );

    fn run(&mut self, (hungers, mut considerers, mut utilities): Self::SystemData) {
        // Join the considerations with the utilities -- they share an `Entity`.
        for (conser, util) in (&mut considerers, &mut utilities).join() {
            // Any actor-related components must be fetched separately, based on
            // the consideration's `actor`.
            if let Some(hunger) = hungers.get(conser.actor.clone()) {
                *util = Utility {
                    // values and weights can be arbitrary numbers. The final
                    // score is based on combining these two values.
                    //
                    // Utilities with weight `0.0` are not used.
                    //
                    // For the formula, refer to the docs on `WeightedMeasure`.
                    value: conser.evaluator.evaluate(hunger.hunger),
                    weight: conser.weight,
                };
            }
        }
    }
}
```

## Usage Example

```ignore
(
    picker: {"FirstToScore": ()},
    choices: [(
        // Considerations to use are defined using the `consider` param in choices.
        // A choice can have zero or more considerations.
        consider: [{"Hunger": (weight: 1.0)}],

        // This is the action that will be executed if this choice "wins".
        then: {"Eat": ()},
    )]
)
```
*/
#[proc_macro_derive(Consideration, attributes(consideration))]
pub fn derive_consideration_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let consideration = Consideration::from_derive_input(&input).unwrap();
    (quote!(#consideration)).into()
}
