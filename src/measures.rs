//! * A series of
//!   [Measures](https://en.wikipedia.org/wiki/Measure_(mathematics)) used to
//!  * weight score.

use bevy::prelude::*;

use crate::prelude::Score;

/// A Measure trait describes a way to combine scores together.
#[reflect_trait]
pub trait Measure: std::fmt::Debug + Sync + Send {
    /// Calculates a score from the child scores
    fn calculate(&self, inputs: Vec<(&Score, f32)>) -> f32;
}

/// A measure that adds all the elements together and multiplies them by the
/// weight.
#[derive(Debug, Clone, Reflect)]
pub struct WeightedSum;

impl Measure for WeightedSum {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |acc, (score, weight)| acc + score.0 * weight)
    }
}

/// A measure that multiplies all the elements together.
#[derive(Debug, Clone, Reflect)]
pub struct WeightedProduct;

impl Measure for WeightedProduct {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |acc, (score, weight)| acc * score.0 * weight)
    }
}

/// A measure that returns the max of the weighted child scares based on the
/// one-dimensional (Chebychev
/// Distance)[https://en.wikipedia.org/wiki/Chebyshev_distance].
#[derive(Debug, Clone, Reflect)]
pub struct ChebyshevDistance;

impl Measure for ChebyshevDistance {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        scores
            .iter()
            .fold(0f32, |best, (score, weight)| (score.0 * weight).max(best))
    }
}

/// The default measure which uses a weight to provide an intuitive curve.
#[derive(Debug, Clone, Default, Reflect)]
pub struct WeightedMeasure;

impl Measure for WeightedMeasure {
    fn calculate(&self, scores: Vec<(&Score, f32)>) -> f32 {
        let wsum: f32 = scores.iter().map(|(_score, weight)| weight).sum();

        if wsum == 0.0 {
            0.0
        } else {
            scores
                .iter()
                .map(|(score, weight)| weight / wsum * score.get().powf(2.0))
                .sum::<f32>()
                .powf(1.0 / 2.0)
        }
    }
}
