/*!
 * A series of [Measures](https://en.wikipedia.org/wiki/Measure_(mathematics)) used to
 * weight score.
 */

use crate::prelude::Score;

/// A Measure trait describes a way to combine scores together
pub trait Measure: std::fmt::Debug + Sync + Send {
    /// Calculates a score from the child scores
    fn calculate(&self, scores: Vec<&Score>) -> f32;
}

/// A measure that adds all the elements together
#[derive(Debug, Clone)]
pub struct SumMeasure;

impl Measure for SumMeasure {
    fn calculate(&self, scores: Vec<&Score>) -> f32 {
        scores.iter().fold(0f32, |acc, item| acc + item.0)
    }
}

/// A measure that adds all the elements together and multiplies them by the weight
#[derive(Debug, Clone)]
pub struct WeightedSumMeasure(pub f32);

impl Measure for WeightedSumMeasure {
    fn calculate(&self, scores: Vec<&Score>) -> f32 {
        scores.iter().fold(0f32, |acc, item| acc + item.0) * self.0
    }
}

/// A measure that multiplies all the elements together
#[derive(Debug, Clone)]
pub struct ProductMeasure;

impl Measure for ProductMeasure {
    fn calculate(&self, scores: Vec<&Score>) -> f32 {
        scores.iter().fold(0f32, |acc, item| acc * item.0)
    }
}
