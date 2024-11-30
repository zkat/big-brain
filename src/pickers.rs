//! Pickers are used by Thinkers to determine which of its Scorers will "win".

use bevy::prelude::*;

use crate::{choices::Choice, scorers::Score};

/// Required trait for Pickers. A Picker is given a slice of choices and a
/// query that can be passed into `Choice::calculate`.
///
/// Implementations of `pick` must return `Some(Choice)` for the `Choice` that
/// was picked, or `None`.
#[reflect_trait]
pub trait Picker: std::fmt::Debug + Sync + Send {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice>;
}

/// Picker that chooses the first `Choice` with a [`Score`] higher than its
/// configured `threshold`.
///
/// ### Example
///
/// ```
/// # use big_brain::prelude::*;
/// # fn main() {
/// Thinker::build()
///     .picker(FirstToScore::new(0.8))
///     // .when(...)
/// # ;
/// # }
/// ```
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

/// Picker that chooses the `Choice` with the highest non-zero [`Score`], and the first highest in case of a tie.
///
/// ### Example
///
/// ```
/// # use big_brain::prelude::*;
/// # fn main() {
/// Thinker::build()
///     .picker(Highest)
///     // .when(...)
/// # ;
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Highest;

impl Picker for Highest {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        let mut max_score = 0f32;

        choices.iter().fold(None, |acc, choice| {
            let score = choice.calculate(scores);

            if score <= max_score || score <= 0.0 {
                return acc;
            }

            max_score = score;
            Some(choice)
        })
    }
}

/// Picker that chooses the highest `Choice` with a [`Score`] higher than its
/// configured `threshold`.
///
/// ### Example
///
/// ```
/// # use big_brain::prelude::*;
/// # fn main() {
/// Thinker::build()
///     .picker(HighestToScore::new(0.8))
///     // .when(...)
/// # ;
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct HighestToScore {
    pub threshold: f32,
}

impl HighestToScore {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Picker for HighestToScore {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        let mut highest_score = 0f32;

        choices.iter().fold(None, |acc, choice| {
            let score = choice.calculate(scores);

            if score <= self.threshold || score <= highest_score {
                return acc;
            }

            highest_score = score;
            Some(choice)
        })
    }
}
