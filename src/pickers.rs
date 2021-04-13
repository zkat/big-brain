use bevy::prelude::*;

use crate::{choices::Choice, scorers::Score};

pub trait Picker: std::fmt::Debug + Sync + Send {
    fn pick(&self, _choices: &[Choice], _utilities: &Query<&Score>) -> Option<Choice>;
}

#[derive(Debug, Clone, Default)]
pub struct FirstToScore {
    pub threshold: f32,
}

impl Picker for FirstToScore {
    fn pick(&self, choices: &[Choice], utilities: &Query<&Score>) -> Option<Choice> {
        for choice in choices {
            let value = choice.calculate(utilities);
            if value >= self.threshold {
                return Some(choice.clone());
            }
        }
        None
    }
}
