/*!
 * A series of [Measures](https://en.wikipedia.org/wiki/Measure_(mathematics)) used to
 * weight score.
 */

use crate::prelude::Score;

/// A Measure trait describes a way to combine scores together
pub trait Measure: std::fmt::Debug + Sync + Send {
    /// Calculates a score from the child scores
    fn calculate(&self, inputs: Vec<(&Score, f32)>) -> f32;
}

/// A measure that adds all the elements together and multiplies them by the weight
#[derive(Debug, Clone)]
pub struct WeightedSumMeasure;

impl Measure for WeightedSumMeasure {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |acc, (score, weight)| acc + score.0 * weight)
    }
}

/// A measure that multiplies all the elements together
#[derive(Debug, Clone)]
pub struct WeightedProductMeasure;

impl Measure for WeightedProductMeasure {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |acc, (score, weight)| acc * score.0 * weight)
    }
}

/// A measure that returns the max of the weighted child scares based on the one-dimensional
/// (Chebychev Distance)[https://en.wikipedia.org/wiki/Chebyshev_distance]
#[derive(Debug, Clone)]
pub struct ChebyshevDistanceMeasure;

impl Measure for ChebyshevDistanceMeasure {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |best, (score, weight)| (score.0 * weight).max(best))
    }
}
