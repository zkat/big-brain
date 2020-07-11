use serde::{Deserialize, Serialize};
use typetag;

use crate::Utility;

#[typetag::serde]
pub trait Measure: std::fmt::Debug + Sync + Send {
    fn calculate(&self, utilities: Vec<&Utility>) -> f32;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeightedMeasure;

#[typetag::serde]
impl Measure for WeightedMeasure {
    fn calculate(&self, utilities: Vec<&Utility>) -> f32 {
        let wsum: f32 = utilities.iter().map(|el| el.weight).sum();
        if wsum == 0.0 {
            0.0
        } else {
            utilities
                .iter()
                .map(|el| el.weight / wsum * el.value.powf(2.0))
                .sum::<f32>()
                .powf(1.0 / 2.0)
        }
    }
}
