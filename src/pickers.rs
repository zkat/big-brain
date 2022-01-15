/*!
Pickers are used by Thinkers to determine which of its Scorers will "win".
*/

use bevy::prelude::*;

use crate::{choices::Choice, scorers::Score};

/**
Required trait for Pickers. A Picker is given a slice of choices and a query that can be passed into `Choice::calculate`.

Implementations of `pick` must return `Some(Choice)` for the `Choice` that was picked, or `None`.
 */
pub trait Picker: std::fmt::Debug + Sync + Send {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice>;
}

/**
Picker that chooses the first `Choice` with a [`Score`] higher than its configured `threshold`.

### Example

```no_run
Thinker::build()
    .picker(FirstToScore::new(.8))
    // .when(...)
```
 */
#[derive(Debug, Clone, Default)]
pub struct FirstToScore {
    pub threshold: f32,
}

impl FirstToScore {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Picker for FirstToScore {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        for choice in choices {
            let value = choice.calculate(scores);
            if value >= self.threshold {
                return Some(choice);
            }
        }
        None
    }
}
